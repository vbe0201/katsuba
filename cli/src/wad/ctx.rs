use std::{
    collections::BTreeMap,
    fs::{self, File},
    io,
    marker::PhantomData,
    mem,
    path::PathBuf,
};

use anyhow::{bail, Result};
use kobold::formats::wad;
use memmap2::{Mmap, MmapOptions};

use super::{crc, inflater::Inflater};
use crate::progress_bar::ProgressBar;

/// Central processing context for WAD archives.
pub struct WadContext<'a> {
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

    _lt: PhantomData<&'a File>,
}

impl<'a> WadContext<'a> {
    /// Creates a WAD context for unpacking an archive
    /// with [`WadContext::extract_all`].
    pub fn map_for_unpack(file: &'a File, out: PathBuf, crc: bool) -> Result<Self> {
        debug_assert!(out.is_dir());

        // Create the context and map the archive file into memory.
        let mut this = Self {
            // SAFETY: `file` lives for 'a, so it won't be dropped
            // before the mapping we're creating here.
            mapping: unsafe { MmapOptions::new().populate().map(file)? },
            out,
            journal: BTreeMap::new(),
            crc,
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
        let file_count = self.journal.len() as u32;

        let mut progress = ProgressBar::<20>::new("Extracting KIWAD archive...", file_count)?;
        let mut inflater = Inflater::new();

        for (idx, (path, file)) in self.journal.iter().enumerate() {
            // Extract the file range we care about.
            let contents = Self::file_contents(&self.mapping, file);

            // Verify CRC if we're supposed to.
            if self.crc && crc::hash(contents) != file.crc {
                bail!("CRC mismatch -- encoded file hash does not match actual data hash");
            }

            let decompressed = if file.compressed {
                inflater.decompress(contents, file.size_uncompressed as _)?
            } else {
                contents
            };

            let out = self.out.join(path);

            // Make sure the directory for the file exists.
            if let Some(dir) = out.parent() {
                if !dir.exists() {
                    fs::create_dir_all(dir)?;
                }
            }

            // Write the file itself.
            fs::write(&out, decompressed)?;

            // Update the progress bar after every file.
            progress.update(idx as u32 + 1)?;
        }

        // Update the progress one last time to display 100%.
        progress.update(file_count)?;

        Ok(())
    }
}
