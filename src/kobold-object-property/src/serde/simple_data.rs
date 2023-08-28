use kobold_bit_buf::BitReader;
use phf::phf_map;

use crate::value::*;

use super::{DeserializerParts, SerializerFlags, SerializerOptions};

type ReadCallback = fn(&SerializerOptions, &mut BitReader<'_>) -> Value;

static DESERIALIZER_LUT: phf::Map<&'static str, (bool, ReadCallback)> = phf_map! {
    // Primitive C++ types
    "bool" => (true, |_, r| Value::Bool(bool(r))),
    "char" => (false, |_, r| Value::Signed(r.i8() as _)),
    "unsigned char" => (false, |_, r| Value::Unsigned(r.u8() as _)),
    "short" => (false, |_, r| Value::Signed(r.i16() as _)),
    "unsigned short" => (false,|_, r| Value::Unsigned(r.u16() as _)),
    "wchar_t" => (false,|_, r| Value::Unsigned(r.u16() as _)),
    "int" => (false,|_, r| Value::Signed(r.i32() as _)),
    "unsigned int" => (false,|_, r| Value::Unsigned(r.u32() as _)),
    "long" => (false, |_, r| Value::Signed(r.i32() as _)),
    "unsigned long" => (false, |_, r| Value::Unsigned(r.u32() as _)),
    "float" => (false, |_, r| Value::Float(r.f32() as _)),
    "double" => (false, |_, r| Value::Float(r.f64())),
    "unsigned __int64" => (false, |_, r| Value::Unsigned(r.u64())),
    "gid" => (false, |_, r| Value::Unsigned(r.u64())),
    "union gid" => (false, |_, r| Value::Unsigned(r.u64())),

    // Bit integers
    "bi2" => (true, |_, r| Value::Signed(signed_bits(r, 2))),
    "bui2" =>(true,  |_, r| Value::Unsigned(bits(r, 2))),
    "bi3" => (true, |_, r| Value::Signed(signed_bits(r, 3))),
    "bui3" =>(true,  |_, r| Value::Unsigned(bits(r, 3))),
    "bi4" => (true, |_, r| Value::Signed(signed_bits(r, 4))),
    "bui4" =>(true,  |_, r| Value::Unsigned(bits(r, 4))),
    "bi5" => (true, |_, r| Value::Signed(signed_bits(r, 5))),
    "bui5" =>(true,  |_, r| Value::Unsigned(bits(r, 5))),
    "bi6" => (true, |_, r| Value::Signed(signed_bits(r, 6))),
    "bui6" =>(true,  |_, r| Value::Unsigned(bits(r, 6))),
    "bi7" => (true, |_, r| Value::Signed(signed_bits(r, 7))),
    "bui7" =>(true,  |_, r| Value::Unsigned(bits(r, 7))),
    "s24" => (true, |_, r| Value::Signed(signed_bits(r, 24))),
    "u24" => (true, |_, r| Value::Unsigned(bits(r, 24))),

    // Strings
    "std::string" => (false, |opts, r| Value::String(deserialize_str(opts, r).to_owned())),
    "std::wstring" =>(false,  |opts, r| Value::WString(deserialize_wstr(opts, r))),

    // Miscellaneous leaf types that are not PropertyClasses
    "class Color" => (false, |_, r| Value::Color(Color {
        b: r.u8(),
        g: r.u8(),
        r: r.u8(),
        a: r.u8(),
    })),
    "class Vector3D" => (false, |_, r| Value::Vec3(Vec3 {
        x: r.f32(),
        y: r.f32(),
        z: r.f32(),
    })),
    "class Quaternion" => (false, |_, r| Value::Quat(Quaternion {
        x: r.f32(),
        y: r.f32(),
        z: r.f32(),
        w: r.f32(),
    })),
    "class Euler" => (false, |_, r| Value::Euler(Euler {
        pitch: r.f32(),
        roll: r.f32(),
        yaw: r.f32(),
    })),
    "class Matrix3x3" => (false, |_, r| Value::Mat3x3(Box::new(Matrix {
        i: [r.f32(), r.f32(), r.f32()],
        j: [r.f32(), r.f32(), r.f32()],
        k: [r.f32(), r.f32(), r.f32()],
    }))),
    "class Size<int>" =>(false, |_, r| Value::Size {
        wh: Box::new((Value::Signed(r.i32() as _), Value::Signed(r.i32() as _))),
    }),
    "class Point<int>" => (false, |_, r| Value::Point {
        xy: Box::new((Value::Signed(r.i32() as _), Value::Signed(r.i32() as _))),
    }),
    "class Point<float>" => (false, |_, r| Value::Point {
        xy: Box::new((Value::Float(r.f32() as _), Value::Float(r.f32() as _))),
    }),
    "class Rect<int>" => (false, |_, r| Value::Rect {
        inner: Box::new((
            Value::Signed(r.i32() as _),
            Value::Signed(r.i32() as _),
            Value::Signed(r.i32() as _),
            Value::Signed(r.i32() as _),
        )),
    }),
    "class Rect<float>" => (false, |_, r| Value::Rect {
        inner: Box::new((
            Value::Float(r.f32() as _),
            Value::Float(r.f32() as _),
            Value::Float(r.f32() as _),
            Value::Float(r.f32() as _),
        )),
    }),
};

pub fn deserialize<D>(
    de: &DeserializerParts<D>,
    ty: &str,
    reader: &mut BitReader<'_>,
) -> Option<Value> {
    DESERIALIZER_LUT.get(ty).map(|(bits, f)| {
        if de.options.shallow && !bits {
            reader.invalidate_and_realign_ptr();
        }

        f(&de.options, reader)
    })
}

macro_rules! impl_read_len {
    ($($de:ident() = $read:ident()),* $(,)*) => {
        $(
            #[inline]
            pub fn $de(options: &SerializerOptions, reader: &mut BitReader<'_>) -> usize {
                if options.flags.contains(SerializerFlags::COMPACT_LENGTH_PREFIXES) {
                    read_compact_length_prefix(reader)
                } else {
                    reader.invalidate_and_realign_ptr();
                    reader.$read() as usize
                }
            }
        )*
    };
}

#[inline]
pub fn bool(reader: &mut BitReader<'_>) -> bool {
    if reader.buffered_bits() == 0 {
        reader.refill_bits();
    }

    reader.bool()
}

#[inline]
pub fn bits(reader: &mut BitReader<'_>, nbits: u32) -> u64 {
    if reader.buffered_bits() < nbits {
        reader.refill_bits();
    }

    reader.read_bits(nbits)
}

#[inline]
pub fn signed_bits(reader: &mut BitReader<'_>, nbits: u32) -> i64 {
    if reader.buffered_bits() < nbits {
        reader.refill_bits();
    }

    reader.read_signed_bits(nbits)
}

#[inline]
fn read_compact_length_prefix(reader: &mut BitReader<'_>) -> usize {
    if reader.buffered_bits() < u32::BITS {
        reader.refill_bits();
    }

    let res = if reader.bool() {
        reader.read_bits(u32::BITS - 1) as usize
    } else {
        reader.read_bits(u8::BITS - 1) as usize
    };

    reader.invalidate_and_realign_ptr();

    res
}

impl_read_len! {
    // Used for strings, where the length is written as `u16`.
    read_str_len() = u16(),

    // Used for sequences, where the length is written as `u32`.
    read_seq_len() = u32(),
}

#[inline]
pub fn deserialize_str<'a>(opts: &SerializerOptions, reader: &'a mut BitReader<'_>) -> &'a [u8] {
    let len = read_str_len(opts, reader);
    reader.read_bytes(len)
}

#[inline]
pub fn deserialize_wstr(opts: &SerializerOptions, reader: &mut BitReader<'_>) -> Vec<u16> {
    let len = read_str_len(opts, reader);

    let mut res = Vec::with_capacity(len);
    for _ in 0..len {
        res.push(reader.u16());
    }

    res
}
