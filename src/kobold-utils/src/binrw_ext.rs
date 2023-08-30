//! Utilities and extensions for common data types we work with.

use binrw::{io::SeekFrom, BinRead, BinResult, BinWrite, VecArgs};

/// Reads a length-prefixed UTF-8 string from the input stream.
#[binrw::parser(reader, endian)]
pub fn read_prefixed_string(len: usize, null: bool) -> BinResult<String> {
    let out: Vec<u8> = <_>::read_options(
        reader,
        endian,
        VecArgs::builder()
            .count(len.saturating_sub(null as usize))
            .finalize(),
    )?;

    let new_pos = reader.seek(SeekFrom::Current(null as i64))?;
    String::from_utf8(out).map_err(|e| binrw::Error::Custom {
        pos: new_pos - len as u64,
        err: Box::new(e.utf8_error()),
    })
}

/// Writes a length-prefixed UTF-8 string to the output stream.
#[binrw::writer(writer, endian)]
pub fn write_prefixed_string(name: &String, null: bool) -> BinResult<()> {
    name.as_bytes().write_options(writer, endian, ())?;
    if null {
        0_u8.write_options(writer, endian, ())?;
    }

    Ok(())
}
