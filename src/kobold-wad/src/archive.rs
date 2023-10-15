use std::{collections::BTreeMap, fs, io, mem, path::Path};

use kobold_utils::{
    binrw,
    libdeflater::DecompressionError,
    thiserror::{self, Error},
};
use memmap2::{Mmap, MmapOptions};

use crate::types as wad_types;

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
    pub fn heap<P: AsRef<Path>>(path: P, verify_crc: bool) -> Result<Self, ArchiveError> {
        HeapArchive::open(path, verify_crc).map(|a| Self(ArchiveInner::Heap(a)))
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
    pub fn mmap<P: AsRef<Path>>(path: P, verify_crc: bool) -> Result<Self, ArchiveError> {
        MemoryMappedArchive::open(path, verify_crc).map(|a| Self(ArchiveInner::MemoryMapped(a)))
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

    /// Gets the raw contents of an archived file by its string name.
    pub fn file_raw(&self, name: &str) -> Option<&wad_types::File> {
        self.journal().find(name)
    }

    /// Extracts the raw file contents out of the archive.
    pub fn file_contents(&self, file: &wad_types::File) -> &[u8] {
        file.extract(self.raw_archive())
    }
}

pub(crate) struct Journal {
    inner: BTreeMap<String, wad_types::File>,
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
    fn open<P: AsRef<Path>>(path: P, verify_crc: bool) -> Result<Self, ArchiveError> {
        // Attempt to open the file at the given path.
        let file = fs::File::open(path)?;
        let mut this = Self {
            // SAFETY: We own the file and keep it around until the mapping
            // is closed; see comments in `MemoryMappedArchive` above.
            //
            // Since archive files are generally treated as read-only by us
            // and most other applications, we likely won't run into any
            // synchronization conflicts we need to account for.
            mapping: unsafe { MmapOptions::new().populate().map(&file)? },
            file,
            journal: Journal {
                inner: BTreeMap::new(),
            },
        };

        // Parse the archive and build the file journal.
        let archive = wad_types::Archive::parse(&mut io::Cursor::new(&this.mapping))?;
        if verify_crc {
            archive.verify_crcs(&this.mapping)?;
        }
        this.journal.build_from(archive);

        Ok(this)
    }
}

struct HeapArchive {
    // The journal of files in the archive.
    journal: Journal,

    // The raw archive data, allocated on the heap.
    data: Box<[u8]>,
}

impl HeapArchive {
    fn open<P: AsRef<Path>>(path: P, verify_crc: bool) -> Result<Self, ArchiveError> {
        // Attempt to read the given file into a byte vector.
        let mut this = Self {
            journal: Journal {
                inner: BTreeMap::new(),
            },
            data: fs::read(path)?.into_boxed_slice(),
        };

        // Parse the archive and build the file journal.
        let archive = wad_types::Archive::parse(&mut io::Cursor::new(&this.data))?;
        if verify_crc {
            archive.verify_crcs(&this.data)?;
        }
        this.journal.build_from(archive);

        Ok(this)
    }
}
