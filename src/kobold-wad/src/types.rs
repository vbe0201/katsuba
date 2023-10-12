//! Common types and structures in the KIWAD format.

use kobold_utils::{
    anyhow,
    binrw::{
        self, binrw,
        io::{Read, Seek, Write},
        BinReaderExt, BinWriterExt,
    },
    binrw_ext::{read_prefixed_string, write_prefixed_string},
};

use crate::crc;

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
    /// # Panics
    ///
    /// This may panic when `raw_archive` is indexed incorrectly with
    /// offset and length of the described file bytes.
    pub fn extract<'wad>(&self, raw_archive: &'wad [u8]) -> &'wad [u8] {
        let offset = self.offset as usize;
        let size = self.size();

        &raw_archive[offset..offset + size]
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
    /// Parses the archive from the given [`Read`]er.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes the archive data to the given [`Write`]r.
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_le(self).map_err(Into::into)
    }

    /// Verifies the CRCs of every file in the archive given the
    /// raw bytes of the archive file.
    pub fn verify_crcs(&self, raw_archive: &[u8]) -> anyhow::Result<()> {
        self.files.iter().try_for_each(|f| {
            let hash = crc::hash(f.extract(raw_archive));
            anyhow::ensure!(
                hash == f.crc,
                "CRC mismatch - expected {}, got {hash}",
                f.crc
            );

            Ok(())
        })
    }
}
