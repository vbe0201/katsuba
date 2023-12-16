use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    mem,
    path::Path,
};

use katsuba_utils::{
    binrw,
    libdeflater::DecompressionError,
    thiserror::{self, Error},
};
use memmap2::{Mmap, MmapOptions};

use crate::{glob, types as wad_types};

/// Errors that may occur when working with KIWAD archives.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// An I/O operation when reading or mapping a file failed.
    #[error("failed to open archive: {0}")]
    Io(#[from] io::Error),

    /// Decompression of a file in the archive failed.
    #[error("failed to decompress archive file: {0}")]
    Zlib(#[from] DecompressionError),

    /// Failed to parse the archive file.
    #[error("failed to parse archive: {0}")]
    Parse(#[from] binrw::Error),

    /// CRC validation of an archive file failed.
    #[error("{0}")]
    Crc(#[from] wad_types::CrcMismatch),
}

/// Representation of a KIWAD archive loaded into memory.
///
/// This type is designed for reading existing archives
/// and querying file information from them.
///
/// It supports two modes of interacting with an underlying
/// archive file: read or mmap.
pub struct Archive(ArchiveInner);

enum ArchiveInner {
    MemoryMapped(MemoryMappedArchive),
    Heap(HeapArchive),
}

impl Archive {
    /// Creates an archive from an open file in heap-allocated memory.
    ///
    /// See [`Archive::open_heap`] for further details.
    pub fn heap(file: fs::File) -> Result<Self, ArchiveError> {
        HeapArchive::new(file).map(|a| Self(ArchiveInner::Heap(a)))
    }

    /// Creates an archive on the heap from a pre-allocated buffer holding
    /// the archive contents.
    ///
    /// See [`Archive::open_heap`] for further details.
    pub fn from_vec(buf: Vec<u8>) -> Result<Self, ArchiveError> {
        HeapArchive::from_vec(buf, 0o666).map(|a| Self(ArchiveInner::Heap(a)))
    }

    /// Opens a file at the given `path` and operates on it from
    /// heap-allocated memory.
    ///
    /// The file handle will be closed immediately after reading.
    ///
    /// `verify_crc` will optionally run validation of all encoded
    /// CRCs in the archive files when `true`.
    ///
    /// This is the preferred option of working with relatively small
    /// files but it's always best to profile.
    pub fn open_heap<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        HeapArchive::open(path).map(|a| Self(ArchiveInner::Heap(a)))
    }

    /// Creates an archive by mapping the open file into memory.
    ///
    /// See [`Archive::open_mmap`] for further details.
    pub fn mmap(file: fs::File) -> Result<Self, ArchiveError> {
        MemoryMappedArchive::new(file).map(|a| Self(ArchiveInner::MemoryMapped(a)))
    }

    /// Opens a file at the given `path` and operates on it from
    /// a memory mapping.
    ///
    /// The file handle will be kept open for the entire lifetime
    /// of the [`Archive`] object.
    ///
    /// `verify_crc` will optionally run validation of all encoded
    /// CRCs in the archive files when `true`.
    ///
    /// This is the preferred option of working with relatively large
    /// files but it's always best to profile.
    pub fn open_mmap<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        MemoryMappedArchive::open(path).map(|a| Self(ArchiveInner::MemoryMapped(a)))
    }

    /// Returns the UNIX permissions of the archive file.
    ///
    /// On other platforms, this value may be ignored.
    #[inline]
    pub fn mode(&self) -> u32 {
        self.journal().mode
    }

    #[inline]
    pub(crate) fn journal(&self) -> &Journal {
        match &self.0 {
            ArchiveInner::MemoryMapped(a) => &a.journal,
            ArchiveInner::Heap(a) => &a.journal,
        }
    }

    #[inline]
    pub(crate) fn raw_archive(&self) -> &[u8] {
        match &self.0 {
            ArchiveInner::MemoryMapped(a) => &a.mapping,
            ArchiveInner::Heap(a) => &a.data,
        }
    }

    /// Gets the number of files in the archive.
    #[inline]
    pub fn len(&self) -> usize {
        self.journal().inner.len()
    }

    /// Whether the archive is empty, i.e. does not contain any files.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets a raw mapping of archive files from path to file metadata.
    /// in the archive.
    ///
    /// Note that the [`wad_types::File::name`] fields are empty strings,
    /// use the map key for an entry to obtain this information.
    #[inline]
    pub fn files(&self) -> &BTreeMap<String, wad_types::File> {
        &self.journal().inner
    }

    /// Builds an iterator over `(path, file)` pairs in the archive where
    /// the path satifies the given UNIX glob pattern.
    #[inline]
    pub fn iter_glob(&self, pattern: &str) -> Result<glob::GlobIter<'_>, glob::GlobError> {
        glob::GlobIter::new(self, pattern)
    }

    /// Gets the raw contents of an archived file by its string name.
    pub fn file_raw(&self, name: &str) -> Option<&wad_types::File> {
        self.journal().find(name)
    }

    /// Extracts the raw file contents out of the archive.
    pub fn file_contents(&self, file: &wad_types::File) -> Option<&[u8]> {
        if file.is_unpatched {
            return None;
        }

        file.extract(self.raw_archive())
    }
}

pub(crate) struct Journal {
    // A mapping of file names to their journal entry.
    inner: BTreeMap<String, wad_types::File>,

    // The file permissions on UNIX systems.
    mode: u32,
}

impl Journal {
    fn insert(&mut self, mut file: wad_types::File) {
        let name = mem::take(&mut file.name);
        self.inner.insert(name, file);
    }

    fn build_from(&mut self, archive: wad_types::Archive) {
        archive.files.into_iter().for_each(|f| self.insert(f));
    }

    fn find(&self, file: &str) -> Option<&wad_types::File> {
        self.inner.get(file)
    }
}

struct MemoryMappedArchive {
    // The journal of files in the archive.
    journal: Journal,

    // Internally kept memory mapping of the archive file contents.
    //
    // By guaranteed drop order, this will be unmapped before the
    // file below is closed.
    mapping: Mmap,

    // The backing file of the above mapping.
    //
    // Owned by this structure so the mapping never becomes invalid.
    // Closed when this structure is dropped.
    #[allow(unused)]
    file: fs::File,
}

impl MemoryMappedArchive {
    fn new(file: fs::File) -> Result<Self, ArchiveError> {
        let mut this = Self {
            // SAFETY: We own the file and keep it around until the mapping
            // is closed; see comments in `MemoryMappedArchive` above.
            //
            // Since archive files are generally treated as read-only by us
            // and most other applications, we likely won't run into any
            // synchronization conflicts we need to account for.
            mapping: unsafe { MmapOptions::new().populate().map(&file)? },
            journal: Journal {
                inner: BTreeMap::new(),
                mode: file_mode(&file),
            },
            file,
        };

        // Parse the archive and build the file journal.
        let mut archive = wad_types::Archive::parse(io::Cursor::new(&this.mapping))?;
        archive.verify_crcs(&this.mapping)?;
        this.journal.build_from(archive);

        Ok(this)
    }

    fn open<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        // Attempt to open the file at the given path.
        let file = fs::File::open(path)?;
        Self::new(file)
    }
}

struct HeapArchive {
    // The journal of files in the archive.
    journal: Journal,

    // The raw archive data, allocated on the heap.
    data: Box<[u8]>,
}

impl HeapArchive {
    fn new(mut file: fs::File) -> Result<Self, ArchiveError> {
        let mut buf = {
            let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
            Vec::with_capacity(size)
        };
        file.read_to_end(&mut buf)?;

        Self::from_vec(buf, file_mode(&file))
    }

    fn from_vec(buf: Vec<u8>, mode: u32) -> Result<Self, ArchiveError> {
        let mut this = Self {
            journal: Journal {
                inner: BTreeMap::new(),
                mode,
            },
            data: buf.into_boxed_slice(),
        };

        // Parse the archive and build the file journal.
        let mut archive = wad_types::Archive::parse(io::Cursor::new(&this.data))?;
        archive.verify_crcs(&this.data)?;
        this.journal.build_from(archive);

        Ok(this)
    }

    fn open<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        let file = fs::File::open(path)?;
        Self::new(file)
    }
}

fn file_mode(_f: &fs::File) -> u32 {
    match () {
        #[cfg(unix)]
        () => {
            use std::os::unix::fs::MetadataExt;
            _f.metadata().map(|m| m.mode()).unwrap_or(0o666)
        }

        #[cfg(not(unix))]
        () => 0,
    }
}
