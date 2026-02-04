//! Common types and structures in the KIWAD format.

use std::io;

use katsuba_utils::binary;
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
#[derive(Clone, Copy, Debug)]
pub struct Header {
    /// The format version in use.
    pub version: u32,
    /// The total number of files in the archive.
    pub file_count: u32,
    /// Implementation-defined config flags associated with
    /// the archive.
    pub flags: Option<u8>,
}

impl Header {
    #[cfg(feature = "builder")]
    fn binary_size(&self) -> usize {
        8 + if self.version >= 2 { 1 } else { 0 }
    }
}

/// Metadata for a file stored in an archive.
#[derive(Clone, Debug)]
pub struct File {
    /// The starting offset of the file data.
    pub offset: u32,
    /// The uncompressed size of the file contents.
    pub uncompressed_size: u32,
    /// The compressed size of the file contents.
    pub compressed_size: u32,
    /// Whether the file is stored compressed.
    pub compressed: bool,
    /// The CRC32 checksum of the uncompressed file contents.
    pub crc: u32,
    /// Whether this file has unpatched data in the archive that
    /// needs to be ignored.
    ///
    /// Unpatched files are basically just placeholder for actual
    /// data later to be filled in.
    pub is_unpatched: bool,
    /// The name of the file in the archive.
    ///
    /// When accessing this by going through an archive's journal,
    /// expect this string to be empty. Instead, use the map key
    /// for this value.
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
#[derive(Clone, Debug)]
pub struct Archive {
    /// The [`Header`] of the archive.
    pub header: Header,
    /// [`File`] metadata describing every stored file.
    pub files: Vec<File>,
}

impl Archive {
    #[cfg(feature = "builder")]
    pub(crate) fn binary_size(&self) -> usize {
        5 + self.header.binary_size() + self.files.iter().map(|f| f.binary_size()).sum::<usize>()
    }

    /// Parses the archive from the given [`Read`]er.
    pub fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        binary::magic(reader, *b"KIWAD")?;

        let mut header = Header {
            version: binary::uint32(reader)?,
            file_count: binary::uint32(reader)?,
            flags: None,
        };

        if header.version >= 2 {
            header.flags = Some(binary::uint8(reader)?);
        }

        let files = binary::seq(reader, header.file_count, |r| {
            Ok(File {
                offset: binary::uint32(r)?,
                uncompressed_size: binary::uint32(r)?,
                compressed_size: binary::uint32(r)?,
                compressed: binary::boolean(r)?,
                crc: binary::uint32(r)?,
                is_unpatched: false,
                name: binary::uint32(r).and_then(|len| binary::str(r, len, true))?,
            })
        })?;

        Ok(Archive { header, files })
    }

    /// Writes the archive data to the given [`Write`]r.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        binary::write_magic(writer, b"KIWAD")?;

        binary::write_uint32(writer, self.header.version)?;
        binary::write_uint32(writer, self.header.file_count)?;
        if let Some(flags) = self.header.flags {
            binary::write_uint8(writer, flags)?;
        }

        binary::write_seq(writer, false, &self.files, |w, f| {
            binary::write_uint32(w, f.offset)?;
            binary::write_uint32(w, f.uncompressed_size)?;
            binary::write_uint32(w, f.compressed_size)?;
            binary::write_boolean(w, f.compressed)?;
            binary::write_uint32(w, f.crc)?;
            binary::write_str(w, &f.name, true)?;

            Ok(())
        })?;

        Ok(())
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
