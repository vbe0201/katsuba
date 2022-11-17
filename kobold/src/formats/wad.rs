//! Implementation of the KIWAD archive format.

use binrw::{
    binread,
    io::{Read, Seek},
    BinReaderExt,
};

use super::utils;

/// The header of a WAD [`Archive`].
#[binread]
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    /// The version of the WAD file format in use.
    pub version: u32,
    /// The number of files stored in this archive.
    pub file_count: u32,
    /// The configuration flags associated with the
    /// WAD archive.
    ///
    /// These are only present when the archive
    /// version is 2 or greater.
    #[br(if(version >= 2))]
    pub flags: Option<u8>,
}

/// A file encoded in a WAD [`Archive`].
#[binread]
#[derive(Debug, PartialEq, Eq)]
pub struct File {
    /// The starting offset of the file in the archive.
    pub start_offset: u32,
    /// The uncompressed size of the file contents.
    pub size_uncompressed: u32,
    /// The compressed size of the file contents.
    ///
    /// When the file is stored uncompressed, this will be
    /// set to [`u32::MAX`].
    pub size_compressed: u32,
    /// Whether the file is stored compressed.
    #[br(map = |x: u8| x != 0)]
    pub compressed: bool,
    /// The CRC32 checksum of the uncompressed file contents.
    pub crc: u32,
    #[br(temp)]
    name_len: u32,
    /// The name of the file in the archive.
    #[br(args(name_len as usize), parse_with = utils::parse_string)]
    #[br(map = |mut x: String| {
        // Get rid of the null terminator byte.
        x.truncate(name_len as usize - 1);
        x
    })]
    pub name: String,
}

/// Representation of a KIWAD archive.
#[binread]
#[derive(Debug, PartialEq, Eq)]
#[br(magic = b"KIWAD")]
pub struct Archive {
    /// The archive [`Header`].
    pub header: Header,
    /// The data for all archived files.
    #[br(count = header.file_count)]
    pub files: Vec<File>,
}

impl Archive {
    /// Attempts to parse an archive from a given input source.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }
}
