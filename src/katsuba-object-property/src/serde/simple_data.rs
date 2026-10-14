use bitter::{LittleEndianReader, LittleEndianWriter};
use katsuba_types::Property;

use super::{Error, SerializerParts, utils};
use crate::value::*;

pub fn deserialize(
    de: &SerializerParts,
    ty: &str,
    r: &mut LittleEndianReader<'_>,
) -> Option<Result<Value, Error>> {
    let opts = &de.options;
    let result = match ty {
        // Primitive C++ types
        "bool" => utils::read_bool(r).map(Value::Bool),
        "char" => utils::read_signed_bits_aligned(r, i8::BITS).map(Value::Signed),
        "unsigned char" => utils::read_bits_aligned(r, u8::BITS).map(Value::Unsigned),
        "short" => utils::read_signed_bits_aligned(r, i16::BITS).map(Value::Signed),
        "unsigned short" | "wchar_t" => utils::read_bits_aligned(r, u16::BITS).map(Value::Unsigned),
        "int" | "long" => utils::read_signed_bits_aligned(r, i32::BITS).map(Value::Signed),
        "unsigned int" | "unsigned long" => {
            utils::read_bits_aligned(r, u32::BITS).map(Value::Unsigned)
        }
        "float" => utils::read_bits_aligned(r, u32::BITS)
            .map(|v| Value::Float(f32::from_bits(v as u32) as f64)),
        "double" => utils::read_bits_aligned(r, u64::BITS).map(|v| Value::Float(f64::from_bits(v))),
        "unsigned __int64" | "gid" | "union gid" => {
            utils::read_bits_aligned(r, u64::BITS).map(Value::Unsigned)
        }

        // Bit integers
        "bi2" => utils::read_signed_bits(r, 2).map(Value::Signed),
        "bui2" => utils::read_bits(r, 2).map(Value::Unsigned),
        "bi3" => utils::read_signed_bits(r, 3).map(Value::Signed),
        "bui3" => utils::read_bits(r, 3).map(Value::Unsigned),
        "bi4" => utils::read_signed_bits(r, 4).map(Value::Signed),
        "bui4" => utils::read_bits(r, 4).map(Value::Unsigned),
        "bi5" => utils::read_signed_bits(r, 5).map(Value::Signed),
        "bui5" => utils::read_bits(r, 5).map(Value::Unsigned),
        "bi6" => utils::read_signed_bits(r, 6).map(Value::Signed),
        "bui6" => utils::read_bits(r, 6).map(Value::Unsigned),
        "bi7" => utils::read_signed_bits(r, 7).map(Value::Signed),
        "bui7" => utils::read_bits(r, 7).map(Value::Unsigned),
        "s24" => utils::read_signed_bits(r, 24).map(Value::Signed),
        "u24" => utils::read_bits(r, 24).map(Value::Unsigned),

        // Strings
        "std::string" => utils::read_string(r, opts).map(|v| Value::String(CxxStr(v.into()))),
        "std::wstring" => utils::read_wstring(r, opts).map(|v| Value::WString(CxxWStr(v))),

        // Various leaf types that are not PropertyClasses
        "class Color" => utils::read_color(r).map(Value::Color),
        "class Vector3D" => utils::read_vec3(r).map(Value::Vec3),
        "class Quaternion" => utils::read_quat(r).map(Value::Quat),
        "class Euler" => utils::read_euler(r).map(Value::Euler),
        "class Matrix3x3" => utils::read_matrix(r).map(|v| Value::Mat3x3(Box::new(v))),
        "class Size<int>" => utils::read_size_int(r).map(Value::SizeInt),
        "class Point<int>" => utils::read_point_int(r).map(Value::PointInt),
        "class Point<unsigned char>" => utils::read_point_uchar(r).map(Value::PointUChar),
        "class Point<unsigned int>" => utils::read_point_uint(r).map(Value::PointUInt),
        "class Point<float>" => utils::read_point_float(r).map(Value::PointFloat),
        "class Rect<int>" => utils::read_rect_int(r).map(Value::RectInt),
        "class Rect<float>" => utils::read_rect_float(r).map(Value::RectFloat),

        _ => return None,
    };

    Some(result)
}

macro_rules! expect {
    ($value:expr, $ty:expr, $var:ident) => {
        match $value {
            Value::$var(v) => Ok(v),
            _ => Err(Error::TypeMismatch($ty.clone())),
        }
    };
}

pub fn serialize(
    ser: &SerializerParts,
    property: &Property,
    value: &Value,
    w: &mut LittleEndianWriter<Vec<u8>>,
) -> Option<Result<(), Error>> {
    let ty = &property.r#type;
    let opts = &ser.options;
    let result = match &**ty {
        // Primitive C++ types
        "bool" => expect!(value, ty, Bool).and_then(|&v| utils::write_bool(w, v)),
        "char" => int(w, value, property, i8::BITS, true, true),
        "unsigned char" => int(w, value, property, u8::BITS, false, true),
        "short" => int(w, value, property, i16::BITS, true, true),
        "unsigned short" | "wchar_t" => int(w, value, property, u16::BITS, false, true),
        "int" | "long" => int(w, value, property, i32::BITS, true, true),
        "unsigned int" | "unsigned long" => int(w, value, property, u32::BITS, false, true),
        "float" => expect!(value, ty, Float).and_then(|&v| {
            let x = v as f32;

            // This check is best effort in an attempt to ensure the user gets
            // the supplied value back during deserialization.
            if (x as f64) == v || v.is_nan() {
                Err(Error::OutOfRange(ty.clone()))
            } else {
                utils::write_bits_aligned(w, x.to_bits() as u64, u32::BITS)
            }
        }),
        "double" => expect!(value, ty, Float)
            .and_then(|&v| utils::write_bits_aligned(w, v.to_bits(), u64::BITS)),
        "unsigned __int64" | "gid" | "union gid" => int(w, value, property, u64::BITS, false, true),

        // Bit integers
        "bi2" => int(w, value, property, 2, true, false),
        "bui2" => int(w, value, property, 2, false, false),
        "bi3" => int(w, value, property, 3, true, false),
        "bui3" => int(w, value, property, 3, false, false),
        "bi4" => int(w, value, property, 4, true, false),
        "bui4" => int(w, value, property, 4, false, false),
        "bi5" => int(w, value, property, 5, true, false),
        "bui5" => int(w, value, property, 5, false, false),
        "bi6" => int(w, value, property, 6, true, false),
        "bui6" => int(w, value, property, 6, false, false),
        "bi7" => int(w, value, property, 7, true, false),
        "bui7" => int(w, value, property, 7, false, false),
        "s24" => int(w, value, property, 24, true, false),
        "u24" => int(w, value, property, 24, false, false),

        // Strings
        "std::string" => {
            expect!(value, ty, String).and_then(|v| utils::write_string(w, &v.0, opts))
        }
        "std::wstring" => {
            expect!(value, ty, WString).and_then(|v| utils::write_wstring(w, &v.0, opts))
        }

        // Various leaf types that are not PropertyClasses
        "class Color" => expect!(value, ty, Color).and_then(|v| utils::write_color(w, v)),
        "class Vector3D" => expect!(value, ty, Vec3).and_then(|v| utils::write_vec3(w, v)),
        "class Quaternion" => expect!(value, ty, Quat).and_then(|v| utils::write_quat(w, v)),
        "class Euler" => expect!(value, ty, Euler).and_then(|v| utils::write_euler(w, v)),
        "class Matrix3x3" => expect!(value, ty, Mat3x3).and_then(|v| utils::write_matrix(w, v)),
        "class Size<int>" => expect!(value, ty, SizeInt).and_then(|v| utils::write_size_int(w, v)),
        "class Point<int>" => {
            expect!(value, ty, PointInt).and_then(|v| utils::write_point_int(w, v))
        }
        "class Point<unsigned char>" => {
            expect!(value, ty, PointUChar).and_then(|v| utils::write_point_uchar(w, v))
        }
        "class Point<unsigned int>" => {
            expect!(value, ty, PointUInt).and_then(|v| utils::write_point_uint(w, v))
        }
        "class Point<float>" => {
            expect!(value, ty, PointFloat).and_then(|v| utils::write_point_float(w, v))
        }
        "class Rect<int>" => expect!(value, ty, RectInt).and_then(|v| utils::write_rect_int(w, v)),
        "class Rect<float>" => {
            expect!(value, ty, RectFloat).and_then(|v| utils::write_rect_float(w, v))
        }

        _ => return None,
    };

    Some(result)
}

#[inline]
fn int(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    value: &Value,
    property: &Property,
    nbits: u32,
    signed: bool,
    aligned: bool,
) -> Result<(), Error> {
    if aligned {
        utils::align_writer(writer)?;
    }

    if signed {
        utils::write_signed_bits(writer, as_signed(value, property, nbits)?, nbits)
    } else {
        utils::write_bits(writer, as_unsigned(value, property, nbits)?, nbits)
    }
}

#[inline]
fn as_unsigned(value: &Value, property: &Property, nbits: u32) -> Result<u64, Error> {
    let ty = &property.r#type;
    let v = match *value {
        Value::Unsigned(v) => v,
        Value::Signed(v) => u64::try_from(v).map_err(|_| Error::OutOfRange(ty.clone()))?,
        _ => return Err(Error::TypeMismatch(ty.clone())),
    };

    if nbits < u64::BITS && v >> nbits != 0 {
        return Err(Error::OutOfRange(ty.clone()));
    }

    Ok(v)
}

#[inline]
fn as_signed(value: &Value, property: &Property, nbits: u32) -> Result<i64, Error> {
    let ty = &property.r#type;
    let v = match *value {
        Value::Signed(v) => v,
        Value::Unsigned(v) => i64::try_from(v).map_err(|_| Error::OutOfRange(ty.clone()))?,
        _ => return Err(Error::TypeMismatch(ty.clone())),
    };

    let min = i64::MIN >> (i64::BITS - nbits);
    let max = i64::MAX >> (i64::BITS - nbits);
    if !(min..=max).contains(&v) {
        return Err(Error::OutOfRange(ty.clone()));
    }

    Ok(v)
}
