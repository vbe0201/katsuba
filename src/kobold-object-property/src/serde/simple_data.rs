use kobold_bit_buf::BitReader;
use phf::phf_map;

use crate::value::*;

use super::{Deserializer, SerializerFlags, SerializerOptions};

static DESERIALIZER_LUT: phf::Map<
    &'static str,
    fn(opts: &SerializerOptions, &mut BitReader<'_>) -> Value,
> = phf_map! {
    // Primitive C++ types
    "bool" => |_, r| Value::Bool(bool(r)),
    "char" => |_, r| Value::Signed(r.i8() as _),
    "unsigned char" => |_, r| Value::Unsigned(r.u8() as _),
    "short" => |_, r| Value::Signed(r.i16() as _),
    "unsigned short" => |_, r| Value::Unsigned(r.u16() as _),
    "wchar_t" => |_, r| Value::Unsigned(r.u16() as _),
    "int" => |_, r| Value::Signed(r.i32() as _),
    "unsigned int" => |_, r| Value::Unsigned(r.u32() as _),
    "long" => |_, r| Value::Signed(r.i32() as _),
    "unsigned long" => |_, r| Value::Unsigned(r.u32() as _),
    "float" => |_, r| Value::Float(r.f32() as _),
    "double" => |_, r| Value::Float(r.f64()),
    "unsigned __int64" => |_, r| Value::Unsigned(r.u64()),
    "gid" => |_, r| Value::Unsigned(r.u64()),
    "union gid" => |_, r| Value::Unsigned(r.u64()),

    // Bit integers
    "bi2" => |_, r| Value::Signed(signed_bits(r, 2)),
    "bui2" => |_, r| Value::Unsigned(bits(r, 2)),
    "bi3" => |_, r| Value::Signed(signed_bits(r, 3)),
    "bui3" => |_, r| Value::Unsigned(bits(r, 3)),
    "bi4" => |_, r| Value::Signed(signed_bits(r, 4)),
    "bui4" => |_, r| Value::Unsigned(bits(r, 4)),
    "bi5" => |_, r| Value::Signed(signed_bits(r, 5)),
    "bui5" => |_, r| Value::Unsigned(bits(r, 5)),
    "bi6" => |_, r| Value::Signed(signed_bits(r, 6)),
    "bui6" => |_, r| Value::Unsigned(bits(r, 6)),
    "bi7" => |_, r| Value::Signed(signed_bits(r, 7)),
    "bui7" => |_, r| Value::Unsigned(bits(r, 7)),
    "s24" => |_, r| Value::Signed(signed_bits(r, 24)),
    "u24" => |_, r| Value::Unsigned(bits(r, 24)),

    // Strings
    "std::string" => |opts, r| Value::String(deserialize_str(opts, r).to_owned()),
    "std::wstring" => |opts, r| Value::WString(deserialize_wstr(opts, r)),

    // Miscellaneous leaf types that are not PropertyClasses
    "class Color" => |_, r| Value::Color(Color {
        b: r.u8(),
        g: r.u8(),
        r: r.u8(),
        a: r.u8(),
    }),
    "class Vector3D" => |_, r| Value::Vec3(Vec3 {
        x: r.f32(),
        y: r.f32(),
        z: r.f32(),
    }),
    "class Quaternion" => |_, r| Value::Quat(Quaternion {
        x: r.f32(),
        y: r.f32(),
        z: r.f32(),
        w: r.f32(),
    }),
    "class Euler" => |_, r| Value::Euler(Euler {
        pitch: r.f32(),
        roll: r.f32(),
        yaw: r.f32(),
    }),
    "class Matrix3x3" => |_, r| Value::Mat3x3(Box::new(Matrix {
        i: [r.f32(), r.f32(), r.f32()],
        j: [r.f32(), r.f32(), r.f32()],
        k: [r.f32(), r.f32(), r.f32()],
    })),
    "class Size<int>" => |_, r| Value::Size {
        wh: Box::new((Value::Signed(r.i32() as _), Value::Signed(r.i32() as _))),
    },
    "class Point<int>" => |_, r| Value::Point {
        xy: Box::new((Value::Signed(r.i32() as _), Value::Signed(r.i32() as _))),
    },
    "class Point<float>" => |_, r| Value::Point {
        xy: Box::new((Value::Float(r.f32() as _), Value::Float(r.f32() as _))),
    },
    "class Rect<int>" => |_, r| Value::Rect {
        inner: Box::new((
            Value::Signed(r.i32() as _),
            Value::Signed(r.i32() as _),
            Value::Signed(r.i32() as _),
            Value::Signed(r.i32() as _),
        )),
    },
    "class Rect<float>" => |_, r| Value::Rect {
        inner: Box::new((
            Value::Float(r.f32() as _),
            Value::Float(r.f32() as _),
            Value::Float(r.f32() as _),
            Value::Float(r.f32() as _),
        )),
    },

};

pub fn deserialize<T>(de: &Deserializer<T>, ty: &str, reader: &mut BitReader<'_>) -> Option<Value> {
    DESERIALIZER_LUT.get(ty).map(|f| f(&de.options, reader))
}

macro_rules! impl_read_len {
    ($($de:ident() = $read:ident()),* $(,)*) => {
        $(
            #[inline]
            pub fn $de(options: &SerializerOptions, reader: &mut BitReader<'_>) -> usize {
                // TODO: realign to byte?
                if options.flags.contains(SerializerFlags::COMPACT_LENGTH_PREFIXES) {
                    read_compact_length_prefix(reader)
                } else {
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
