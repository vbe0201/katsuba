//! Implementation of the KIWAD archive format.

use binrw::{
    binread,
    io::{Read, Seek},
    BinReaderExt, FilePtr, NullString,
};

mod sealed {
    use std::io::SeekFrom;

    #[binrw::binread]
    #[derive(Clone, Copy, Debug)]
    pub struct CurrentPosition;

    impl binrw::file_ptr::IntoSeekFrom for CurrentPosition {
        fn into_seek_from(self) -> SeekFrom {
            SeekFrom::Current(0)
        }
    }
}

/// The header of a WAD [`Archive`].
#[binread]
#[derive(Debug, PartialEq, Eq)]
#[br(magic = b"KIWAD")]
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
#[derive(Debug, PartialEq)]
pub struct File {
    #[br(temp)]
    start_offset: u32,
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
    pub name: NullString,
    /// The data referenced by this file.
    #[br(offset = start_offset as u64)]
    #[br(count = if compressed { size_compressed } else { size_uncompressed })]
    pub data: FilePtr<sealed::CurrentPosition, Vec<u8>>,
}

/// Representation of a KIWAD archive.
#[binread]
#[derive(Debug, PartialEq)]
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
