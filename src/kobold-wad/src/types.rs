//! Common types and structures in the KIWAD format.

use binrw::{
    binrw,
    io::{Read, Seek, SeekFrom, Write},
    BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, VecArgs,
};
use kobold_utils::anyhow;

use crate::crc;

/// The header of a KIWAD archive.
#[binrw]
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
    #[br(args(name_len as usize), parse_with = parse_file_name)]
    #[bw(write_with = write_file_name)]
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
                "CRC mismatch - expected {hash}, got {}",
                f.crc
            );

            Ok(())
        })
    }
}

#[binrw::parser(reader, endian)]
fn parse_file_name(len: usize) -> BinResult<String> {
    let out: Vec<u8> = <_>::read_options(
        reader,
        endian,
        VecArgs::builder().count(len.saturating_sub(1)).finalize(),
    )?;

    let new_pos = reader.seek(SeekFrom::Current(1))?;
    String::from_utf8(out).map_err(|e| binrw::Error::Custom {
        pos: new_pos - len as u64,
        err: Box::new(e.utf8_error()),
    })
}

#[binrw::writer(writer, endian)]
fn write_file_name(name: &String) -> BinResult<()> {
    name.as_bytes().write_options(writer, endian, ())?;
    0_u8.write_options(writer, endian, ())
}
