//! Common types and structures in the KIWAD format.

use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinResult, BinWriterExt,
};
use katsuba_utils::binrw_ext::{read_prefixed_string, write_prefixed_string};
use thiserror::Error;

use crate::crc;

pub(crate) fn is_unpatched_file(data: &[u8]) -> bool {
    // SAFETY: Transmuting bytes to larger integer types is legal.
    let (prefix, aligned, suffix) = unsafe { data.align_to::<u128>() };

    // Speed up the checks by comparing most data in chunks of 16
    // bytes. For smaller-than-16-bytes leading and trailing chunks
    // we do byte comparisons.
    prefix.iter().all(|&x| x == 0)
        && aligned.iter().all(|&x| x == 0)
        && suffix.iter().all(|&x| x == 0)
}

/// Error type produced by [`Archive::verify_crcs`].
#[derive(Clone, Copy, Debug, PartialEq, Error)]
#[error("CRC mismatch -- expected {expected}, got {actual}")]
pub struct CrcMismatch {
    /// The expected CRC value.
    pub expected: u32,
    /// The actual computed CRC value.
    pub actual: u32,
}

/// The header of a KIWAD archive.
#[binrw]
#[derive(Clone, Copy, Debug)]
pub struct Header {
    /// The format version in use.
    pub version: u32,
    /// The total number of files in the archive.
    pub file_count: u32,
    /// Implementation-defined config flags associated with
    /// the archive.
    #[br(if(version >= 2))]
    pub flags: Option<u8>,
}

impl Header {
    #[cfg(feature = "builder")]
    fn binary_size(&self) -> usize {
        8 + if self.version >= 2 { 1 } else { 0 }
    }
}

/// Metadata for a file stored in an archive.
#[binrw]
#[derive(Clone, Debug)]
pub struct File {
    /// The starting offset of the file data.
    pub offset: u32,
    /// The uncompressed size of the file contents.
    pub uncompressed_size: u32,
    /// The compressed size of the file contents.
    pub compressed_size: u32,
    /// Whether the file is stored compressed.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub compressed: bool,
    /// The CRC32 checksum of the uncompressed file contents.
    pub crc: u32,

    /// Whether this file has unpatched data in the archive that
    /// needs to be ignored.
    ///
    /// Unpatched files are basically just placeholder for actual
    /// data later to be filled in.
    #[brw(ignore)]
    pub is_unpatched: bool,

    #[br(temp)]
    #[bw(calc(name.len() as u32 + 1))]
    name_len: u32,

    /// The name of the file in the archive.
    ///
    /// When accessing this by going through an archive's journal,
    /// expect this string to be empty. Instead, use the map key
    /// for this value.
    #[br(args(name_len as usize, true), parse_with = read_prefixed_string)]
    #[bw(args(true), write_with = write_prefixed_string)]
    pub name: String,
}

impl File {
    #[cfg(feature = "builder")]
    pub(crate) fn binary_size(&self) -> usize {
        22 + self.name.len()
    }

    /// Gets the length of data described by this file in bytes.
    #[inline]
    pub const fn size(&self) -> usize {
        if self.compressed {
            self.compressed_size as usize
        } else {
            self.uncompressed_size as usize
        }
    }

    /// Extracts this file from the given raw archive bytes.
    ///
    /// When the archive is malformed, this returns [`None`].
    pub fn extract<'wad>(&self, raw_archive: &'wad [u8]) -> Option<&'wad [u8]> {
        let offset = self.offset as usize;
        let size = self.size();

        raw_archive.get(offset..offset + size)
    }
}

/// Representation of a KIWAD archive.
///
/// This does not account for the dynamically-sized data
/// which follows after the structured part.
///
/// Implementations must consider this and keep the raw
/// archive bytes around even after parsing this structure.
#[binrw]
#[brw(magic = b"KIWAD")]
#[derive(Clone, Debug)]
pub struct Archive {
    /// The [`Header`] of the archive.
    pub header: Header,
    /// [`File`] metadata describing every stored file.
    #[br(count = header.file_count)]
    pub files: Vec<File>,
}

impl Archive {
    #[cfg(feature = "builder")]
    pub(crate) fn binary_size(&self) -> usize {
        5 + self.header.binary_size() + self.files.iter().map(|f| f.binary_size()).sum::<usize>()
    }

    /// Parses the archive from the given [`Read`]er.
    pub fn parse<R: Read + Seek>(mut reader: R) -> BinResult<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes the archive data to the given [`Write`]r.
    pub fn write<W: Write + Seek>(&self, mut writer: W) -> BinResult<()> {
        writer.write_le(self).map_err(Into::into)
    }

    /// Verifies the CRCs of every file in the archive given the
    /// raw bytes of the archive file.
    ///
    /// # Panics
    ///
    /// Panics when the KIWAD archive encodes file journal entries
    /// with no matching data.
    pub fn verify_crcs(&mut self, raw_archive: &[u8]) -> Result<(), CrcMismatch> {
        self.files.iter_mut().try_for_each(|f| {
            let data = f.extract(raw_archive).unwrap();
            let hash = crc::hash(data);

            if hash == f.crc {
                Ok(())
            } else {
                // Only dismiss files as unpatched if they are all zeroes
                // on CRC mismatch.
                if is_unpatched_file(data) {
                    f.is_unpatched = true;
                    return Ok(());
                }

                Err(CrcMismatch {
                    expected: f.crc,
                    actual: hash,
                })
            }
        })
    }
}
