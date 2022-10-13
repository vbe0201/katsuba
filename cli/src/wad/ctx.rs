use std::{collections::BTreeMap, fs::File, io, marker::PhantomData, mem, path::PathBuf};

use anyhow::Result;
use flate2::{Decompress, FlushDecompress};
use kobold::formats::wad;
use memmap2::{Mmap, MmapOptions};

use super::driver::Driver;

/// Central processing context for WAD archives.
pub struct WadContext<'a, D> {
    /// The archive file mapped to memory.
    mapping: Mmap,

    /// Output directory for the archive files after
    /// pack/unpack operations.
    out: PathBuf,

    /// The journal of files in the WAD archive.
    journal: BTreeMap<PathBuf, wad::File>,

    /// Whether checksums should be verified during
    /// extraction.
    crc: bool,

    /// The [`Driver`] for processing I/O requests.
    driver: D,

    _lt: PhantomData<&'a File>,
}

impl<'a, D: Driver> WadContext<'a, D> {
    /// Creates a WAD context for unpacking an archive
    /// with [`WadContext::extract_all`].
    pub fn map_for_unpack(file: &'a File, out: PathBuf, crc: bool) -> Result<Self> {
        debug_assert!(out.is_dir());

        // Create the context and map the archive file into memory.
        // XXX: Profile populate() performance on Linux.
        let mut this = Self {
            // SAFETY: `file` lives for 'a, so it won't be dropped
            // before the mapping we're creating here.
            mapping: unsafe { MmapOptions::new().populate().map(file)? },
            out,
            journal: BTreeMap::new(),
            crc,
            driver: D::default(),
            _lt: PhantomData,
        };

        // Parse the archive journal from memory and insert into the B-tree.
        let archive = wad::Archive::parse(&mut io::Cursor::new(&this.mapping[..]))?;
        archive
            .files
            .into_iter()
            .for_each(|file| this.insert_file(file));

        Ok(this)
    }

    fn insert_file(&mut self, mut file: wad::File) {
        let file_path = PathBuf::from(mem::take(&mut file.name));
        self.journal.insert(file_path, file);
    }

    fn file_contents<'b>(mmap: &'b Mmap, file: &wad::File) -> &'b [u8] {
        let offset = file.start_offset as usize;
        let size = if file.compressed {
            file.size_compressed
        } else {
            file.size_uncompressed
        };

        &mmap[offset..offset + size as usize]
    }

    /// Extracts all files in the archive to disk.
    pub fn extract_all(&mut self) -> Result<()> {
        let mut inflater = Decompress::new(true);
        let mut scratch = Vec::new();

        for (path, file) in &self.journal {
            // Verify CRC if we're supposed to.
            if self.crc {
                todo!();
            }

            // Get the uncompressed file contents, if necessary.
            let contents = Self::file_contents(&self.mapping, file);
            let contents = if file.compressed {
                scratch.reserve(file.size_uncompressed as _);
                inflater.decompress_vec(contents, &mut scratch, FlushDecompress::Finish)?;

                &scratch[..]
            } else {
                contents
            };

            self.driver.extract_file(&self.out.join(path), contents)?;
        }

        self.driver.wait()?;

        Ok(())
    }
}
