use std::io;

use bitter::{BitReader, LittleEndianReader, sign_extend};
use byteorder::{LittleEndian, ReadBytesExt};
use katsuba_utils::align::align_down;

use super::{Error, SerializerFlags, SerializerOptions};
use crate::value::*;

#[inline]
pub fn bits_to_bytes(bits: usize) -> usize {
    bits.next_multiple_of(u8::BITS as usize) >> 3
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
pub fn read_bits(reader: &mut LittleEndianReader<'_>, nbits: u32) -> Result<u64, Error> {
    reader
        .read_bits(nbits)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "reached premature EOF").into())
}

#[inline]
pub fn read_signed_bits(reader: &mut LittleEndianReader<'_>, nbits: u32) -> Result<i64, Error> {
    let v = read_bits(reader, nbits)?;
    Ok(sign_extend(v, nbits))
}

#[inline]
pub fn read_bits_aligned(reader: &mut LittleEndianReader<'_>, nbits: u32) -> Result<u64, Error> {
    align(reader)?;
    reader
        .read_bits(nbits)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "reached premature EOF").into())
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
pub fn read_bool(reader: &mut LittleEndianReader<'_>) -> Result<bool, Error> {
    read_bits(reader, 1).map(|v| v != 0)
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
pub fn read_string_length(
    reader: &mut LittleEndianReader<'_>,
    compact: bool,
) -> Result<usize, Error> {
    let len = match compact {
        true => read_compact_length(reader)?,
        false => {
            align(reader)?;
            read_bits(reader, u16::BITS)? as usize
        }
    };

    Ok(len)
}

#[inline]
pub fn read_container_length(
    reader: &mut LittleEndianReader<'_>,
    compact: bool,
) -> Result<usize, Error> {
    let len = match compact {
        true => read_compact_length(reader)?,
        false => {
            align(reader)?;
            read_bits(reader, u32::BITS)? as usize
        }
    };

    Ok(len)
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
pub fn read_color(reader: &mut LittleEndianReader<'_>) -> Result<Color, Error> {
    let b = read_bits_aligned(reader, u8::BITS)? as u8;
    let g = read_bits(reader, u8::BITS)? as u8;
    let r = read_bits(reader, u8::BITS)? as u8;
    let a = read_bits(reader, u8::BITS)? as u8;

    Ok(Color { r, g, b, a })
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
pub fn read_quat(reader: &mut LittleEndianReader<'_>) -> Result<Quaternion, Error> {
    let mut data = read_bytes(reader, 16)?;

    let w = data.read_f32::<LittleEndian>()?;
    let x = data.read_f32::<LittleEndian>()?;
    let y = data.read_f32::<LittleEndian>()?;
    let z = data.read_f32::<LittleEndian>()?;

    Ok(Quaternion { x, y, z, w })
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
pub fn read_size_int(reader: &mut LittleEndianReader<'_>) -> Result<Size<i32>, Error> {
    let width = read_signed_bits_aligned(reader, i32::BITS)? as i32;
    let height = read_signed_bits(reader, i32::BITS)? as i32;
    Ok(Size { width, height })
}

#[inline]
pub fn read_point_int(reader: &mut LittleEndianReader<'_>) -> Result<Point<i32>, Error> {
    let x = read_signed_bits_aligned(reader, i32::BITS)? as i32;
    let y = read_signed_bits(reader, i32::BITS)? as i32;
    Ok(Point { x, y })
}

#[inline]
pub fn read_point_uint(reader: &mut LittleEndianReader<'_>) -> Result<Point<u32>, Error> {
    let x = read_bits_aligned(reader, u32::BITS)? as u32;
    let y = read_bits(reader, u32::BITS)? as u32;
    Ok(Point { x, y })
}

#[inline]
pub fn read_point_uchar(reader: &mut LittleEndianReader<'_>) -> Result<Point<u8>, Error> {
    let x = read_bits_aligned(reader, u8::BITS)? as u8;
    let y = read_bits(reader, u8::BITS)? as u8;
    Ok(Point { x, y })
}

#[inline]
pub fn read_point_float(reader: &mut LittleEndianReader<'_>) -> Result<Point<f32>, Error> {
    let x = read_bits_aligned(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    let y = read_bits(reader, u32::BITS).map(|v| f32::from_bits(v as u32))?;
    Ok(Point { x, y })
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
