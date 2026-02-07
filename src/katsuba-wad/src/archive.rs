use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    mem,
    path::Path,
};

use libdeflater::DecompressionError;
use memmap2::{Mmap, MmapOptions};
use thiserror::Error;

use crate::{glob, types as wad_types};

/// Errors that may occur when working with KIWAD archives.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// I/O error from reading or mapping a file.
    #[error("failed to open archive: {0}")]
    Io(#[from] io::Error),

    /// Decompression of a file in the archive failed.
    #[error("failed to decompress archive file: {0}")]
    Zlib(#[from] DecompressionError),

    /// Failed to verify the archive integrity.
    #[error("{0}")]
    Verification(#[from] wad_types::VerificationError),

    /// File data ranges overlap, indicating a potential zip bomb.
    #[error("overlapping data ranges detected between '{file1}' and '{file2}'")]
    OverlappingRanges {
        /// The first file with overlapping data.
        file1: String,
        /// The second file with overlapping data.
        file2: String,
    },
}

/// Representation of a KIWAD archive loaded into memory.
///
/// This type is designed for reading existing archives
/// and querying file information from them.
///
/// It supports two modes of interacting with an underlying
/// archive file: read or mmap.
#[derive(Debug)]
pub struct Archive(ArchiveInner);

#[derive(Debug)]
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

    /// Gets an immutable reference to the header of this archive.
    #[inline]
    pub fn header(&self) -> &wad_types::Header {
        &self.journal().header
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

#[derive(Debug)]
pub(crate) struct Journal {
    // A mapping of file names to their journal entry.
    pub inner: BTreeMap<String, wad_types::File>,
    /// The header of a KIWAD archive.
    pub header: wad_types::Header,
    // The file permissions on UNIX systems.
    mode: u32,
}

impl Journal {
    pub fn new(mode: u32) -> Self {
        Self {
            inner: BTreeMap::new(),
            header: wad_types::Header {
                version: 0,
                file_count: 0,
                flags: None,
            },
            mode,
        }
    }

    pub fn insert(&mut self, mut file: wad_types::File) {
        let name = mem::take(&mut file.name);
        self.inner.insert(name, file);
    }

    fn build_from(&mut self, archive: wad_types::Archive) -> Result<(), ArchiveError> {
        let wad_types::Archive { header, files } = archive;

        self.header = header;
        files.into_iter().for_each(|f| self.insert(f));
        self.validate_no_overlaps()
    }

    fn find(&self, file: &str) -> Option<&wad_types::File> {
        self.inner.get(file)
    }

    fn validate_no_overlaps(&self) -> Result<(), ArchiveError> {
        // Collect byte ranges as (start, end, name) for non-empty files.
        let mut ranges: Vec<_> = self
            .inner
            .iter()
            .filter(|(_, f)| !f.is_unpatched && f.size() > 0)
            .map(|(name, f)| {
                let start = f.offset as u64;
                let end = start + f.size() as u64;
                (start, end, name.as_str())
            })
            .collect();

        // Sort by start offset so overlapping ranges become adjacent.
        ranges.sort_unstable_by_key(|&(start, _, _)| start);

        // Two sorted ranges [a, b) and [c, d) overlap if c < b.
        for pair in ranges.windows(2) {
            let (_, end, name1) = pair[0];
            let (start, _, name2) = pair[1];

            if start < end {
                return Err(ArchiveError::OverlappingRanges {
                    file1: name1.to_string(),
                    file2: name2.to_string(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct MemoryMappedArchive {
    // The journal of files in the archive.
    journal: Journal,

    // Internally kept memory mapping of the archive file contents.
    //
    // By guaranteed drop order, this will be unmapped before the
    // file below is closed.
    mapping: Mmap,
}

impl MemoryMappedArchive {
    fn new(file: fs::File) -> Result<Self, ArchiveError> {
        let mut this = Self {
            // SAFETY: Since archive files are generally treated as read-only
            // by us and most other applications, we likely won't run into any
            // synchronization conflicts we need to account for.
            mapping: unsafe { MmapOptions::new().populate().map(&file)? },
            journal: Journal::new(file_mode(&file)),
        };

        // Parse the archive and build the file journal.
        let mut archive = wad_types::Archive::parse(&mut io::Cursor::new(&this.mapping))?;
        archive.verify(&this.mapping)?;
        this.journal.build_from(archive)?;

        Ok(this)
    }

    fn open<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        // Attempt to open the file at the given path.
        let file = fs::File::open(path)?;
        Self::new(file)
    }
}

#[derive(Debug)]
struct HeapArchive {
    // The journal of files in the archive.
    journal: Journal,

    // The raw archive data, allocated on the heap.
    data: Box<[u8]>,
}

impl HeapArchive {
    fn new(mut file: fs::File) -> Result<Self, ArchiveError> {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        Self::from_vec(buf, file_mode(&file))
    }

    fn from_vec(buf: Vec<u8>, mode: u32) -> Result<Self, ArchiveError> {
        let mut this = Self {
            journal: Journal::new(mode),
            data: buf.into_boxed_slice(),
        };

        // Parse the archive and build the file journal.
        let mut archive = wad_types::Archive::parse(&mut &*this.data)?;
        archive.verify(&this.data)?;
        this.journal.build_from(archive)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::File;

    fn make_file(name: &str, offset: u32, size: u32) -> (String, File) {
        (
            name.to_string(),
            File {
                offset,
                uncompressed_size: size,
                compressed_size: 0,
                compressed: false,
                crc: 0,
                is_unpatched: false,
                name: String::new(),
            },
        )
    }

    fn journal_with_files(files: Vec<(String, File)>) -> Journal {
        let mut journal = Journal::new(0o644);
        journal.inner = files.into_iter().collect();
        journal
    }

    #[test]
    fn no_overlap_adjacent_ranges() {
        let journal = journal_with_files(vec![make_file("a", 0, 100), make_file("b", 100, 100)]);
        assert!(journal.validate_no_overlaps().is_ok());
    }

    #[test]
    fn no_overlap_gap_between() {
        let journal = journal_with_files(vec![make_file("a", 0, 100), make_file("b", 200, 100)]);
        assert!(journal.validate_no_overlaps().is_ok());
    }

    #[test]
    fn overlap_partial() {
        let journal = journal_with_files(vec![make_file("a", 0, 100), make_file("b", 50, 100)]);
        let err = journal.validate_no_overlaps().unwrap_err();
        assert!(matches!(err, ArchiveError::OverlappingRanges { .. }));
    }

    #[test]
    fn overlap_complete() {
        let journal = journal_with_files(vec![make_file("a", 0, 100), make_file("b", 0, 100)]);
        let err = journal.validate_no_overlaps().unwrap_err();
        assert!(matches!(err, ArchiveError::OverlappingRanges { .. }));
    }

    #[test]
    fn overlap_contained() {
        let journal = journal_with_files(vec![make_file("a", 0, 200), make_file("b", 50, 50)]);
        let err = journal.validate_no_overlaps().unwrap_err();
        assert!(matches!(err, ArchiveError::OverlappingRanges { .. }));
    }

    #[test]
    fn overlap_multiple_files() {
        let journal = journal_with_files(vec![
            make_file("a", 0, 100),
            make_file("b", 50, 100),
            make_file("c", 200, 100),
        ]);
        let err = journal.validate_no_overlaps().unwrap_err();
        assert!(matches!(err, ArchiveError::OverlappingRanges { .. }));
    }

    #[test]
    fn empty_files_ignored() {
        let journal = journal_with_files(vec![make_file("a", 0, 0), make_file("b", 0, 0)]);
        assert!(journal.validate_no_overlaps().is_ok());
    }

    #[test]
    fn unpatched_files_ignored() {
        let mut files = vec![make_file("a", 0, 100), make_file("b", 50, 100)];
        files[1].1.is_unpatched = true;
        let journal = journal_with_files(files);
        assert!(journal.validate_no_overlaps().is_ok());
    }

    #[test]
    fn single_file_ok() {
        let journal = journal_with_files(vec![make_file("a", 0, 100)]);
        assert!(journal.validate_no_overlaps().is_ok());
    }

    #[test]
    fn empty_journal_ok() {
        let journal = journal_with_files(vec![]);
        assert!(journal.validate_no_overlaps().is_ok());
    }
}
