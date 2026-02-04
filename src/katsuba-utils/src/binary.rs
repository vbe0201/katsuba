//! Utilities for reading and writing structured binary data.

use std::{
    io::{self, Read, Write},
    mem,
};

/// Reads a magic value of `N` bytes from the stream.
#[inline]
pub fn magic<R: Read, const N: usize>(data: &mut R, expected: [u8; N]) -> io::Result<()> {
    let mut v = [0; N];
    data.read_exact(&mut v)?;

    if v == expected {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Magic mismatch in input stream",
        ))
    }
}

/// Writes a magic value to the stream.
#[inline]
pub fn write_magic<W: Write>(out: &mut W, magic: &[u8]) -> io::Result<()> {
    out.write_all(magic)
}

/// Parses an unsigned byte off the data stream.
#[inline]
pub fn uint8<R: Read>(data: &mut R) -> io::Result<u8> {
    let mut v = [0; 1];
    data.read_exact(&mut v)?;
    Ok(v[0])
}

/// Writes an unsigned byte to the data stream.
#[inline]
pub fn write_uint8<W: Write>(out: &mut W, v: u8) -> io::Result<()> {
    out.write_all(&[v])
}

/// Parses a signed byte off the data stream.
#[inline]
pub fn int8<R: Read>(data: &mut R) -> io::Result<i8> {
    uint8(data).map(|v| v as i8)
}

/// Writes a signed byte to the data stream.
#[inline]
pub fn write_int8<W: Write>(out: &mut W, v: i8) -> io::Result<()> {
    out.write_all(&[v as u8])
}

/// Reads a [`bool`] from the data stream.
#[inline]
pub fn boolean<R: Read>(data: &mut R) -> io::Result<bool> {
    uint8(data).map(|v| v != 0)
}

/// Writes a [`bool`] to the data stream.
#[inline]
pub fn write_boolean<W: Write>(out: &mut W, v: bool) -> io::Result<()> {
    out.write_all(&[v as u8])
}

macro_rules! int_read_impl {
    ($($fn:ident() -> $ty:ty),* $(,)*) => {
        $(
            #[doc = concat!("Parses a [`", stringify!($ty), "`] value off the data stream.")]
            #[inline]
            pub fn $fn<R: io::Read>(data: &mut R) -> io::Result<$ty> {
                let mut v = [0; mem::size_of::<$ty>()];
                data.read_exact(&mut v)?;
                Ok(<$ty>::from_le_bytes(v))
        }
        )*
    };
}

macro_rules! int_write_impl {
    ($($fn:ident($ty:ty)),* $(,)*) => {
        $(
            #[doc = concat!("Writes a [`", stringify!($ty), "`] value to the data stream.")]
            #[inline]
            pub fn $fn<W: Write>(out: &mut W, v: $ty) -> io::Result<()> {
                out.write_all(&v.to_le_bytes())
            }
        )*
    };
}

int_read_impl! {
    uint16() -> u16,
    int16() -> i16,
    uint32() -> u32,
    int32() -> i32,
    uint64() -> u64,
    int64() -> i64,
}

int_write_impl! {
    write_uint16(u16),
    write_int16(i16),
    write_uint32(u32),
    write_int32(i32),
    write_uint64(u64),
    write_int64(i64),
}

/// Parses a string given its length and removes a potential null
/// terminator from the end of the value.
///
/// Fails if the string is not valid UTF-8.
#[inline]
pub fn str<R: Read>(data: &mut R, len: u32, null_terminated: bool) -> io::Result<String> {
    let mut v = Vec::with_capacity(len as usize);
    data.take(len as u64).read_to_end(&mut v)?;

    if v.len() != len as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Premature EOF while reading string data",
        ));
    }

    if null_terminated && !v.is_empty() && v.remove(v.len().wrapping_sub(1)) != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected null terminator for string",
        ));
    }

    String::from_utf8(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Writes a length-prefixed string to the output stream.
#[inline]
pub fn write_str<W: Write>(out: &mut W, v: &str, null_terminated: bool) -> io::Result<()> {
    write_uint32(out, v.len() as u32 + null_terminated as u32)?;
    out.write_all(v.as_bytes())?;
    if null_terminated {
        out.write_all(&[0])?;
    }

    Ok(())
}

/// Parses a sequence of `count` elements using the given parser.
///
/// The parser function freely defines how to parse one element
/// of the sequence.
#[inline]
pub fn seq<F, R, T>(data: &mut R, count: u32, mut f: F) -> io::Result<Vec<T>>
where
    F: FnMut(&mut R) -> io::Result<T>,
    R: Read,
{
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let element = f(data)?;
        out.push(element);
    }
    Ok(out)
}

#[inline]
pub fn write_seq<F, T, W>(out: &mut W, prefixed: bool, seq: &[T], mut f: F) -> io::Result<()>
where
    F: FnMut(&mut W, &T) -> io::Result<()>,
    W: Write,
{
    if prefixed {
        write_uint32(out, seq.len() as u32)?;
    }
    for v in seq {
        f(out, v)?;
    }

    Ok(())
}
