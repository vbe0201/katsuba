use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use katsuba_bit_buf::{utils::sign_extend, BitReader};
use katsuba_utils::align::align_up;

use super::{Error, SerializerFlags, SerializerOptions};
use crate::value::*;

#[inline]
pub const fn bits_to_bytes(bits: usize) -> usize {
    align_up(bits, u8::BITS as _) >> 3
}

#[inline]
pub fn read_bits(reader: &mut BitReader<'_>, nbits: u32) -> Result<u64, Error> {
    if reader.buffered_bits() < nbits {
        reader.refill_bits();
    }

    let v = reader.peek(nbits)?;
    reader.consume(nbits)?;
    Ok(v)
}

#[inline]
pub fn read_signed_bits(reader: &mut BitReader<'_>, nbits: u32) -> Result<i64, Error> {
    let v = read_bits(reader, nbits)?;
    Ok(sign_extend(v, nbits))
}

#[inline]
pub fn read_u64(reader: &mut BitReader<'_>) -> Result<u64, Error> {
    reader.realign_to_byte();
    reader
        .read_bytes(8)
        .map(LittleEndian::read_u64)
        .map_err(Into::into)
}

#[inline]
pub fn read_bool(reader: &mut BitReader<'_>) -> Result<bool, Error> {
    read_bits(reader, 1).map(|v| v != 0)
}

#[inline]
pub fn read_compact_length(reader: &mut BitReader<'_>) -> Result<usize, Error> {
    let is_large = read_bool(reader)?;
    let v = match is_large {
        true => read_bits(reader, u32::BITS - 1),
        false => read_bits(reader, u8::BITS - 1),
    };

    v.map(|v| v as usize)
}

#[inline]
pub fn read_string_length(reader: &mut BitReader<'_>, compact: bool) -> Result<usize, Error> {
    let len = match compact {
        true => read_compact_length(reader)?,
        false => {
            reader.realign_to_byte();
            read_bits(reader, u16::BITS)? as usize
        }
    };

    Ok(len)
}

#[inline]
pub fn read_container_length(reader: &mut BitReader<'_>, compact: bool) -> Result<usize, Error> {
    let len = match compact {
        true => read_compact_length(reader)?,
        false => {
            reader.realign_to_byte();
            read_bits(reader, u32::BITS)? as usize
        }
    };

    Ok(len)
}

#[inline]
pub fn read_string<'a>(
    reader: &mut BitReader<'a>,
    opts: &SerializerOptions,
) -> Result<&'a [u8], Error> {
    let len = read_string_length(
        reader,
        opts.flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    if len != 0 {
        reader.realign_to_byte();
        reader.read_bytes(len).map_err(Into::into)
    } else {
        Ok(&[])
    }
}

#[inline]
pub fn read_wstring(
    reader: &mut BitReader<'_>,
    opts: &SerializerOptions,
) -> Result<Vec<u16>, Error> {
    let len = read_string_length(
        reader,
        opts.flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    let mut out = Vec::with_capacity(len);
    if len != 0 {
        reader.realign_to_byte();
        for _ in 0..len {
            out.push(read_bits(reader, u16::BITS)? as u16);
        }
    }

    Ok(out)
}

#[inline]
pub fn read_color(reader: &mut BitReader<'_>) -> Result<Color, Error> {
    if reader.buffered_bits() < u32::BITS {
        reader.refill_bits();
    }

    let r = reader.peek(u8::BITS)? as u8;
    reader.consume(u8::BITS)?;
    let g = reader.peek(u8::BITS)? as u8;
    reader.consume(u8::BITS)?;
    let b = reader.peek(u8::BITS)? as u8;
    reader.consume(u8::BITS)?;
    let a = reader.peek(u8::BITS)? as u8;
    reader.consume(u8::BITS)?;

    Ok(Color { r, g, b, a })
}

#[inline]
pub fn read_vec3(reader: &mut BitReader<'_>) -> Result<Vec3, Error> {
    let mut data = reader.read_bytes(12)?;

    let x = data.read_f32::<LittleEndian>()?;
    let y = data.read_f32::<LittleEndian>()?;
    let z = data.read_f32::<LittleEndian>()?;

    Ok(Vec3 { x, y, z })
}

#[inline]
pub fn read_quat(reader: &mut BitReader<'_>) -> Result<Quaternion, Error> {
    let mut data = reader.read_bytes(16)?;

    let x = data.read_f32::<LittleEndian>()?;
    let y = data.read_f32::<LittleEndian>()?;
    let z = data.read_f32::<LittleEndian>()?;
    let w = data.read_f32::<LittleEndian>()?;

    Ok(Quaternion { x, y, z, w })
}

#[inline]
pub fn read_euler(reader: &mut BitReader<'_>) -> Result<Euler, Error> {
    let mut data = reader.read_bytes(12)?;

    // TODO: Is this order correct?
    let pitch = data.read_f32::<LittleEndian>()?;
    let roll = data.read_f32::<LittleEndian>()?;
    let yaw = data.read_f32::<LittleEndian>()?;

    Ok(Euler { pitch, roll, yaw })
}

#[inline]
pub fn read_matrix(reader: &mut BitReader<'_>) -> Result<Matrix, Error> {
    let mut data = reader.read_bytes(36)?;

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
