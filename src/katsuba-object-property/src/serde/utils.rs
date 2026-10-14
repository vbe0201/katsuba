use std::io;

use bitter::{BitReader, BitWriter, LittleEndianReader, LittleEndianWriter, sign_extend};
use byteorder::{ByteOrder, LE, LittleEndian, ReadBytesExt};
use katsuba_utils::align::align_down;

use super::{Error, SerializerFlags, SerializerOptions};
use crate::value::*;

#[inline]
pub fn bits_to_bytes(bits: usize) -> usize {
    bits.next_multiple_of(u8::BITS as usize) >> 3
}

#[inline]
fn bit_len(writer: &mut LittleEndianWriter<Vec<u8>>) -> Result<u32, Error> {
    let len = writer.get_ref().len() * u8::BITS as usize;
    let rem = writer.unaligned_bits() as usize;

    u32::try_from(len + rem).map_err(|_| Error::LengthOverflow(len + rem))
}

#[inline]
pub fn write_prefixed<F>(writer: &mut LittleEndianWriter<Vec<u8>>, f: F) -> Result<(), Error>
where
    F: FnOnce(&mut LittleEndianWriter<Vec<u8>>) -> Result<(), Error>,
{
    let start = bit_len(writer)?;

    // Carve out a 32-bit length prefix for the data about to be written.
    // Since this is guaranteed to be at a bit-aligned position, we flush
    // to the underlying Vec here to avoid bit_len misreporting potential
    // padding bits from a later flush.
    write_bits_aligned(writer, 0, u32::BITS)?;
    let size_pos = ((bit_len(writer)? - u32::BITS) / u8::BITS) as usize;
    writer.flush()?;

    // Delegate to the supplied callback for writing data.
    f(writer)?;

    // Since the placeholder bytes are already flushed, overwrite them
    // with the real length of the written data now.
    let size = bit_len(writer)? - start;
    let buf = writer.get_mut();
    LE::write_u32(&mut buf[size_pos..size_pos + 4], size);

    Ok(())
}

#[inline]
pub fn align(reader: &mut LittleEndianReader<'_>) -> Result<(), Error> {
    let bits = reader.lookahead_bits() as usize;
    let aligned_bits = align_down(bits, u8::BITS as usize);

    let pad = (bits - aligned_bits) as u32;
    if pad != 0 {
        read_bits(reader, pad)?;
    }

    Ok(())
}

#[inline]
pub fn align_writer(writer: &mut LittleEndianWriter<Vec<u8>>) -> Result<(), Error> {
    let bits = writer.unaligned_bits();
    if bits != 0 {
        writer.write_bits(u8::BITS - bits, 0)?;
    }

    Ok(())
}

#[inline]
pub fn read_bits(reader: &mut LittleEndianReader<'_>, nbits: u32) -> Result<u64, Error> {
    reader
        .read_bits(nbits)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "reached premature EOF").into())
}

#[inline]
pub fn write_bits(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: u64,
    nbits: u32,
) -> Result<(), Error> {
    writer.write_bits(nbits, value).map_err(Into::into)
}

#[inline]
pub fn read_signed_bits(reader: &mut LittleEndianReader<'_>, nbits: u32) -> Result<i64, Error> {
    let v = read_bits(reader, nbits)?;
    Ok(sign_extend(v, nbits))
}

#[inline]
pub fn write_signed_bits(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: i64,
    nbits: u32,
) -> Result<(), Error> {
    writer.write_signed_bits(nbits, value).map_err(Into::into)
}

#[inline]
pub fn read_bits_aligned(reader: &mut LittleEndianReader<'_>, nbits: u32) -> Result<u64, Error> {
    align(reader)?;
    read_bits(reader, nbits)
}

#[inline]
pub fn write_bits_aligned(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: u64,
    nbits: u32,
) -> Result<(), Error> {
    align_writer(writer)?;
    write_bits(writer, value, nbits)
}

#[inline]
pub fn read_signed_bits_aligned(
    reader: &mut LittleEndianReader<'_>,
    nbits: u32,
) -> Result<i64, Error> {
    let v = read_bits_aligned(reader, nbits)?;
    Ok(sign_extend(v, nbits))
}

#[inline]
pub fn write_signed_bits_aligned(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: i64,
    nbits: u32,
) -> Result<(), Error> {
    align_writer(writer)?;
    write_signed_bits(writer, value, nbits)
}

#[inline]
pub fn read_bool(reader: &mut LittleEndianReader<'_>) -> Result<bool, Error> {
    read_bits(reader, 1).map(|v| v != 0)
}

#[inline]
pub fn write_bool(writer: &mut LittleEndianWriter<Vec<u8>>, value: bool) -> Result<(), Error> {
    write_bits(writer, value as u64, 1)
}

#[inline]
pub fn read_compact_length(reader: &mut LittleEndianReader<'_>) -> Result<usize, Error> {
    let is_large = read_bool(reader)?;
    let v = match is_large {
        true => read_bits(reader, u32::BITS - 1),
        false => read_bits(reader, u8::BITS - 1),
    };

    v.map(|v| v as usize)
}

#[inline]
pub fn write_compact_length(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    len: usize,
) -> Result<(), Error> {
    if len < 1 << (u8::BITS - 1) {
        write_bool(writer, false)?;
        write_bits(writer, len as u64, u8::BITS - 1)
    } else if len < 1 << (u32::BITS - 1) {
        write_bool(writer, true)?;
        write_bits(writer, len as u64, u32::BITS - 1)
    } else {
        Err(Error::LengthOverflow(len))
    }
}

#[inline]
pub fn read_string_length(
    reader: &mut LittleEndianReader<'_>,
    compact: bool,
) -> Result<usize, Error> {
    let len = match compact {
        true => read_compact_length(reader)?,
        false => read_bits_aligned(reader, u16::BITS)? as usize,
    };

    Ok(len)
}

#[inline]
pub fn write_string_length(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    len: usize,
    compact: bool,
) -> Result<(), Error> {
    match compact {
        true => write_compact_length(writer, len),
        false => {
            let len = u16::try_from(len).map_err(|_| Error::LengthOverflow(len))?;
            write_bits_aligned(writer, len as u64, u16::BITS)
        }
    }
}

#[inline]
pub fn read_container_length(
    reader: &mut LittleEndianReader<'_>,
    compact: bool,
) -> Result<usize, Error> {
    let len = match compact {
        true => read_compact_length(reader)?,
        false => read_bits_aligned(reader, u32::BITS)? as usize,
    };

    Ok(len)
}

#[inline]
pub fn write_container_length(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    len: usize,
    compact: bool,
) -> Result<(), Error> {
    match compact {
        true => write_compact_length(writer, len),
        false => {
            let len = u32::try_from(len).map_err(|_| Error::LengthOverflow(len))?;
            write_bits_aligned(writer, len as u64, u32::BITS)
        }
    }
}

#[inline]
pub fn read_bytes<'a>(reader: &mut LittleEndianReader<'a>, len: usize) -> Result<&'a [u8], Error> {
    align(reader)?;

    let rem = reader.remainder().data();
    let (buf, rem) = rem
        .split_at_checked(len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "reached premature EOF"))?;

    *reader = LittleEndianReader::new(rem);
    Ok(buf)
}

#[inline]
pub fn write_bytes(writer: &mut LittleEndianWriter<Vec<u8>>, data: &[u8]) -> Result<(), Error> {
    align_writer(writer)?;
    writer.write_bytes(data)?;
    Ok(())
}

#[inline]
pub fn read_string<'a>(
    reader: &mut LittleEndianReader<'a>,
    opts: &SerializerOptions,
) -> Result<&'a [u8], Error> {
    let len = read_string_length(
        reader,
        opts.flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    if len != 0 {
        read_bytes(reader, len)
    } else {
        Ok(&[])
    }
}

#[inline]
pub fn write_string(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    data: &[u8],
    opts: &SerializerOptions,
) -> Result<(), Error> {
    write_string_length(
        writer,
        data.len(),
        opts.flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    if !data.is_empty() {
        write_bytes(writer, data)?;
    }

    Ok(())
}

#[inline]
pub fn read_wstring(
    reader: &mut LittleEndianReader<'_>,
    opts: &SerializerOptions,
) -> Result<Vec<u16>, Error> {
    let len = read_string_length(
        reader,
        opts.flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    let mut out = Vec::with_capacity(len);
    if len != 0 {
        align(reader)?;

        // TODO: Optimize this with manual mode.
        for _ in 0..len {
            out.push(read_bits(reader, u16::BITS)? as u16);
        }
    }

    Ok(out)
}

#[inline]
pub fn write_wstring(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    data: &[u16],
    opts: &SerializerOptions,
) -> Result<(), Error> {
    write_string_length(
        writer,
        data.len(),
        opts.flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    if !data.is_empty() {
        // TODO: Optimize this with manual mode.
        align_writer(writer)?;
        for &c in data {
            writer.write_u16(c)?;
        }
    }

    Ok(())
}

#[inline]
pub fn read_color(reader: &mut LittleEndianReader<'_>) -> Result<Color, Error> {
    let b = read_bits_aligned(reader, u8::BITS)? as u8;
    let g = read_bits(reader, u8::BITS)? as u8;
    let r = read_bits(reader, u8::BITS)? as u8;
    let a = read_bits(reader, u8::BITS)? as u8;

    Ok(Color { r, g, b, a })
}

#[inline]
pub fn write_color(writer: &mut LittleEndianWriter<Vec<u8>>, color: &Color) -> Result<(), Error> {
    write_bytes(writer, &[color.b, color.g, color.r, color.a])
}

#[inline]
pub fn read_vec3(reader: &mut LittleEndianReader<'_>) -> Result<Vec3, Error> {
    let mut data = read_bytes(reader, 12)?;

    let x = data.read_f32::<LittleEndian>()?;
    let y = data.read_f32::<LittleEndian>()?;
    let z = data.read_f32::<LittleEndian>()?;

    Ok(Vec3 { x, y, z })
}

#[inline]
pub fn write_vec3(writer: &mut LittleEndianWriter<Vec<u8>>, value: &Vec3) -> Result<(), Error> {
    align_writer(writer)?;
    writer.write_f32(value.x)?;
    writer.write_f32(value.y)?;
    writer.write_f32(value.z)?;

    Ok(())
}

#[inline]
pub fn read_quat(reader: &mut LittleEndianReader<'_>) -> Result<Quaternion, Error> {
    let mut data = read_bytes(reader, 16)?;

    let w = data.read_f32::<LittleEndian>()?;
    let x = data.read_f32::<LittleEndian>()?;
    let y = data.read_f32::<LittleEndian>()?;
    let z = data.read_f32::<LittleEndian>()?;

    Ok(Quaternion { x, y, z, w })
}

#[inline]
pub fn write_quat(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Quaternion,
) -> Result<(), Error> {
    align_writer(writer)?;
    writer.write_f32(value.w)?;
    writer.write_f32(value.x)?;
    writer.write_f32(value.y)?;
    writer.write_f32(value.z)?;

    Ok(())
}

#[inline]
pub fn read_euler(reader: &mut LittleEndianReader<'_>) -> Result<Euler, Error> {
    let mut data = read_bytes(reader, 12)?;

    let pitch = data.read_f32::<LittleEndian>()?;
    let yaw = data.read_f32::<LittleEndian>()?;
    let roll = data.read_f32::<LittleEndian>()?;

    Ok(Euler { pitch, roll, yaw })
}

#[inline]
pub fn write_euler(writer: &mut LittleEndianWriter<Vec<u8>>, value: &Euler) -> Result<(), Error> {
    align_writer(writer)?;
    writer.write_f32(value.pitch)?;
    writer.write_f32(value.yaw)?;
    writer.write_f32(value.roll)?;

    Ok(())
}

#[inline]
pub fn read_matrix(reader: &mut LittleEndianReader<'_>) -> Result<Matrix, Error> {
    let mut data = read_bytes(reader, 36)?;

    let i = [
        data.read_f32::<LittleEndian>()?,
        data.read_f32::<LittleEndian>()?,
        data.read_f32::<LittleEndian>()?,
    ];
    let j = [
        data.read_f32::<LittleEndian>()?,
        data.read_f32::<LittleEndian>()?,
        data.read_f32::<LittleEndian>()?,
    ];
    let k = [
        data.read_f32::<LittleEndian>()?,
        data.read_f32::<LittleEndian>()?,
        data.read_f32::<LittleEndian>()?,
    ];

    Ok(Matrix { i, j, k })
}

#[inline]
pub fn write_matrix(writer: &mut LittleEndianWriter<Vec<u8>>, value: &Matrix) -> Result<(), Error> {
    align_writer(writer)?;
    value.i.iter().try_for_each(|&v| writer.write_f32(v))?;
    value.j.iter().try_for_each(|&v| writer.write_f32(v))?;
    value.k.iter().try_for_each(|&v| writer.write_f32(v))?;

    Ok(())
}

#[inline]
pub fn read_size_int(reader: &mut LittleEndianReader<'_>) -> Result<Size<i32>, Error> {
    let width = read_signed_bits_aligned(reader, i32::BITS)? as i32;
    let height = read_signed_bits(reader, i32::BITS)? as i32;
    Ok(Size { width, height })
}

#[inline]
pub fn write_size_int(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Size<i32>,
) -> Result<(), Error> {
    write_signed_bits_aligned(writer, value.width as i64, i32::BITS)?;
    write_signed_bits(writer, value.height as i64, i32::BITS)
}

#[inline]
pub fn read_point_int(reader: &mut LittleEndianReader<'_>) -> Result<Point<i32>, Error> {
    let x = read_signed_bits_aligned(reader, i32::BITS)? as i32;
    let y = read_signed_bits(reader, i32::BITS)? as i32;
    Ok(Point { x, y })
}

#[inline]
pub fn write_point_int(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Point<i32>,
) -> Result<(), Error> {
    write_signed_bits_aligned(writer, value.x as i64, i32::BITS)?;
    write_signed_bits(writer, value.y as i64, i32::BITS)
}

#[inline]
pub fn read_point_uint(reader: &mut LittleEndianReader<'_>) -> Result<Point<u32>, Error> {
    let x = read_bits_aligned(reader, u32::BITS)? as u32;
    let y = read_bits(reader, u32::BITS)? as u32;
    Ok(Point { x, y })
}

#[inline]
pub fn write_point_uint(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Point<u32>,
) -> Result<(), Error> {
    write_bits_aligned(writer, value.x as u64, u32::BITS)?;
    write_bits(writer, value.y as u64, u32::BITS)
}

#[inline]
pub fn read_point_uchar(reader: &mut LittleEndianReader<'_>) -> Result<Point<u8>, Error> {
    let x = read_bits_aligned(reader, u8::BITS)? as u8;
    let y = read_bits(reader, u8::BITS)? as u8;
    Ok(Point { x, y })
}

#[inline]
pub fn write_point_uchar(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Point<u8>,
) -> Result<(), Error> {
    write_bits_aligned(writer, value.x as u64, u8::BITS)?;
    write_bits(writer, value.y as u64, u8::BITS)
}

#[inline]
pub fn read_point_float(reader: &mut LittleEndianReader<'_>) -> Result<Point<f32>, Error> {
    let x = read_bits_aligned(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    let y = read_bits(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    Ok(Point { x, y })
}

#[inline]
pub fn write_point_float(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Point<f32>,
) -> Result<(), Error> {
    align_writer(writer)?;
    writer.write_f32(value.x)?;
    writer.write_f32(value.y)?;

    Ok(())
}

#[inline]
pub fn read_rect_int(reader: &mut LittleEndianReader<'_>) -> Result<Rect<i32>, Error> {
    let left = read_signed_bits_aligned(reader, i32::BITS)? as i32;
    let top = read_signed_bits(reader, i32::BITS)? as i32;
    let right = read_signed_bits(reader, i32::BITS)? as i32;
    let bottom = read_signed_bits(reader, i32::BITS)? as i32;

    Ok(Rect {
        left,
        top,
        right,
        bottom,
    })
}

#[inline]
pub fn write_rect_int(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Rect<i32>,
) -> Result<(), Error> {
    write_signed_bits_aligned(writer, value.left as i64, i32::BITS)?;
    write_signed_bits(writer, value.top as i64, i32::BITS)?;
    write_signed_bits(writer, value.right as i64, i32::BITS)?;
    write_signed_bits(writer, value.bottom as i64, i32::BITS)
}

#[inline]
pub fn read_rect_float(reader: &mut LittleEndianReader<'_>) -> Result<Rect<f32>, Error> {
    let left = read_bits_aligned(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    let top = read_bits(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    let right = read_bits(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    let bottom = read_bits(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;

    Ok(Rect {
        left,
        top,
        right,
        bottom,
    })
}

#[inline]
pub fn write_rect_float(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Rect<f32>,
) -> Result<(), Error> {
    align_writer(writer)?;
    writer.write_f32(value.left)?;
    writer.write_f32(value.top)?;
    writer.write_f32(value.right)?;
    writer.write_f32(value.bottom)?;

    Ok(())
}
