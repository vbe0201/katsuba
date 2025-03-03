use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufWriter, Seek, Write},
    path::Path,
};

use libdeflater::CompressionError;
use tempfile::tempfile_in;
use thiserror::Error;

use crate::{crc, deflater::Deflater, types as wad_types};

const ALWAYS_UNCOMPRESSED: &[&str] = &["mp3", "ogg"];

/// Errors that may occur when assembling KIWAD archives.
#[derive(Debug, Error)]
pub enum BuilderError {
    /// An I/O error occurred while working with files.
    #[error("{0}")]
    Io(#[from] io::Error),

    /// More files were added to the archive than allowed.
    #[error("archive too large to represent")]
    TooLarge,

    /// Compression of a file's contents failed.
    #[error("failed to compress archive file: {0}")]
    Zlib(#[from] CompressionError),

    /// The archive could not be serialized to the output file.
    #[error("failed to serialize archive: {0}")]
    Serialize(binrw::Error),

    /// Received an invalid path for the output archive file.
    #[error("path to output archive file must have a parent component")]
    Path,
}

impl From<binrw::Error> for BuilderError {
    fn from(value: binrw::Error) -> Self {
        match value {
            binrw::Error::Io(e) => Self::Io(e),
            e => Self::Serialize(e),
        }
    }
}

#[inline(always)]
fn checked_u32(x: usize) -> Result<u32, BuilderError> {
    u32::try_from(x).or(Err(BuilderError::TooLarge))
}

struct BuilderState {
    // The raw archive structure we're building. This is what we will
    // serialize in the end, sans the actual file contents.
    archive: wad_types::Archive,

    // The byte size of the journal we are building. We progressively
    // update it because individual journal entries are dynamic size.
    journal_size: usize,

    // The offset of the next file's data in the archive. This does
    // not respect the size of the journal yet.
    next_file_offset: u32,
}

impl BuilderState {
    fn new(version: u32, flags: u8) -> Self {
        let archive = wad_types::Archive {
            header: wad_types::Header {
                version,
                file_count: 0,
                flags: (version >= 2).then_some(flags),
            },
            files: Vec::new(),
        };

        Self {
            journal_size: archive.binary_size(),
            archive,
            next_file_offset: 0,
        }
    }

    fn intern_file(&mut self, record: wad_types::File, data: &[u8]) -> Result<(), BuilderError> {
        let record_size = record.binary_size();

        // Add the file record to the archive journal.
        self.archive.files.push(record);
        self.archive.header.file_count += 1;

        // Update positional offsets for the next file.
        self.journal_size += record_size;
        self.next_file_offset = self
            .next_file_offset
            .checked_add(checked_u32(data.len())?)
            .ok_or(BuilderError::TooLarge)?;

        Ok(())
    }

    fn patch_file_offsets(&mut self) -> Result<(), BuilderError> {
        let journal_size = checked_u32(self.journal_size)?;
        for file in &mut self.archive.files {
            file.offset = file
                .offset
                .checked_add(journal_size)
                .ok_or(BuilderError::TooLarge)?;
        }

        Ok(())
    }

    fn sort_journal(&mut self) {
        self.archive.files.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// A builder for programatically creating KIWAD archives.
///
/// To avoid out-of-memory errors when trying to build very large
/// archives, the builder keeps a temporary blob cache file in the
/// same directory as the output file.
///
/// Thus, consumers of the API only need to keep one archive file
/// at a time in memory.
pub struct ArchiveBuilder {
    // The progressive archive state.
    state: BuilderState,

    // The zlib deflater to handle file compression, one at a time.
    deflater: Deflater,

    // The output archive file we are writing to.
    outfile: BufWriter<File>,

    // A temporary file we use as a blob cache for compressed data.
    // This allows us to buffer big amounts of data without having
    // to keep them in memory. The file will be appended to `outfile`
    // before it is deleted.
    blob_cache: BufWriter<File>,
}

impl ArchiveBuilder {
    /// Creates a new archive builder from the archive version, its flags,
    /// and the output path to the final archive file.
    ///
    /// This will fail if either the output file or the temporary blob cache
    /// file in the same directory fail to be created.
    ///
    /// `flags` will be ignored on `version < 2`.
    pub fn new<P: AsRef<Path>>(version: u32, flags: u8, out: P) -> Result<Self, BuilderError> {
        let out = out.as_ref();
        let parent = out.parent().ok_or(BuilderError::Path)?;

        let outfile = File::create(out).map(BufWriter::new)?;
        let blob_cache = tempfile_in(parent).map(BufWriter::new)?;

        Ok(Self {
            state: BuilderState::new(version, flags),
            deflater: Deflater::new(),
            outfile,
            blob_cache,
        })
    }

    /// Adds an uncompressed file to the archive.
    ///
    /// `name` is a relative path to the start of the archive where the
    /// file will be located.
    pub fn add_file(
        &mut self,
        name: impl AsRef<Path>,
        contents: &[u8],
    ) -> Result<(), BuilderError> {
        let record = wad_types::File {
            offset: self.state.next_file_offset,
            uncompressed_size: checked_u32(contents.len())?,
            compressed_size: u32::MAX,
            compressed: false,
            crc: crc::hash(contents),
            is_unpatched: false,
            name: name.as_ref().to_string_lossy().to_string(),
        };

        self.state.intern_file(record, contents)?;
        self.blob_cache.write_all(contents)?;

        Ok(())
    }

    /// Adds a compressed file to the archive.
    ///
    /// `name` is a relative path to the start of the archive where the
    /// file will be located.
    ///
    /// `contents` is the file data which will be compressed internally.
    pub fn add_file_compressed(
        &mut self,
        name: impl AsRef<Path>,
        contents: &[u8],
    ) -> Result<(), BuilderError> {
        let path = name.as_ref();

        // Check if the given file path ends with a file that is conditionally
        // uncompressed. In that case, we just delegate to `add_file`.
        if path
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ALWAYS_UNCOMPRESSED.contains(&ext))
            .unwrap_or(false)
        {
            return self.add_file(name, contents);
        }

        let compressed = self.deflater.compress(contents)?;
        let record = wad_types::File {
            offset: self.state.next_file_offset,
            uncompressed_size: checked_u32(contents.len())?,
            compressed_size: checked_u32(compressed.len())?,
            compressed: true,
            crc: crc::hash(compressed),
            is_unpatched: false,
            name: path.to_string_lossy().to_string(),
        };

        self.state.intern_file(record, compressed)?;
        self.blob_cache.write_all(compressed)?;

        Ok(())
    }

    /// Finalizes the archive building and writes all data to the
    /// output file.
    ///
    /// The temporary blob cache will be deleted by the OS after this.
    pub fn finish(mut self) -> Result<(), BuilderError> {
        self.state.patch_file_offsets()?;

        // Sort files in ascending path order to maintain compatibility
        // with KingsIsle's official sorting order.
        self.state.sort_journal();

        // Serialize the KIWAD header and file journal, then merge
        // the blob cache to the end of the output file.
        self.state.archive.write(&mut self.outfile)?;
        {
            let mut blob_cache = match self.blob_cache.into_inner() {
                Ok(f) => f,
                Err(e) => return Err(BuilderError::Io(e.into_error())),
            };
            blob_cache.seek(io::SeekFrom::Start(0))?;

            io::copy(&mut blob_cache, &mut self.outfile)?;
        }

        Ok(())
    }
}
