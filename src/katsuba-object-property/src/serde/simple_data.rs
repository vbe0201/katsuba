use bitter::LittleEndianReader;

use super::{Error, SerializerParts, utils};
use crate::value::*;

#[inline]
pub fn deserialize(
    de: &SerializerParts,
    ty: &str,
    reader: &mut LittleEndianReader<'_>,
) -> Option<Result<Value, Error>> {
    let opts = &de.options;
    let result = match ty {
        // Primitive C++ types
        "bool" => utils::read_bool(reader).map(Value::Bool),
        "char" => utils::read_signed_bits_aligned(reader, i8::BITS).map(Value::Signed),
        "unsigned char" => utils::read_bits_aligned(reader, u8::BITS).map(Value::Unsigned),
        "short" => utils::read_signed_bits_aligned(reader, i16::BITS).map(Value::Signed),
        "unsigned short" => utils::read_bits_aligned(reader, u16::BITS).map(Value::Unsigned),
        "wchar_t" => utils::read_bits_aligned(reader, u16::BITS).map(Value::Unsigned),
        "int" => utils::read_signed_bits_aligned(reader, i32::BITS).map(Value::Signed),
        "unsigned int" => utils::read_bits_aligned(reader, u32::BITS).map(Value::Unsigned),
        "long" => utils::read_signed_bits_aligned(reader, i32::BITS).map(Value::Signed),
        "unsigned long" => utils::read_bits_aligned(reader, u32::BITS).map(Value::Unsigned),
        "float" => utils::read_bits_aligned(reader, u32::BITS)
            .map(|v| Value::Float(f32::from_bits(v as u32) as f64)),
        "double" => {
            utils::read_bits_aligned(reader, u64::BITS).map(|v| Value::Float(f64::from_bits(v)))
        }
        "unsigned __int64" => utils::read_bits_aligned(reader, u64::BITS).map(Value::Unsigned),
        "gid" => utils::read_bits_aligned(reader, u64::BITS).map(Value::Unsigned),
        "union gid" => utils::read_bits_aligned(reader, u64::BITS).map(Value::Unsigned),

        // Bit integers
        "bi2" => utils::read_signed_bits(reader, 2).map(Value::Signed),
        "bui2" => utils::read_bits(reader, 2).map(Value::Unsigned),
        "bi3" => utils::read_signed_bits(reader, 3).map(Value::Signed),
        "bui3" => utils::read_bits(reader, 3).map(Value::Unsigned),
        "bi4" => utils::read_signed_bits(reader, 4).map(Value::Signed),
        "bui4" => utils::read_bits(reader, 4).map(Value::Unsigned),
        "bi5" => utils::read_signed_bits(reader, 5).map(Value::Signed),
        "bui5" => utils::read_bits(reader, 5).map(Value::Unsigned),
        "bi6" => utils::read_signed_bits(reader, 6).map(Value::Signed),
        "bui6" => utils::read_bits(reader, 6).map(Value::Unsigned),
        "bi7" => utils::read_signed_bits(reader, 7).map(Value::Signed),
        "bui7" => utils::read_bits(reader, 7).map(Value::Unsigned),
        "s24" => utils::read_signed_bits(reader, 24).map(Value::Signed),
        "u24" => utils::read_bits(reader, 24).map(Value::Unsigned),

        // Strings
        "std::string" => utils::read_string(reader, opts).map(|v| Value::String(CxxStr(v.into()))),
        "std::wstring" => utils::read_wstring(reader, opts).map(|v| Value::WString(CxxWStr(v))),

        // Various leaf types that are not PropertyClasses
        "class Color" => utils::read_color(reader).map(Value::Color),
        "class Vector3D" => utils::read_vec3(reader).map(Value::Vec3),
        "class Quaternion" => utils::read_quat(reader).map(Value::Quat),
        "class Euler" => utils::read_euler(reader).map(Value::Euler),
        "class Matrix3x3" => utils::read_matrix(reader).map(|v| Value::Mat3x3(Box::new(v))),
        "class Size<int>" => utils::read_size_int(reader).map(Value::SizeInt),
        "class Point<int>" => utils::read_point_int(reader).map(Value::PointInt),
        "class Point<unsigned char>" => utils::read_point_uchar(reader).map(Value::PointUChar),
        "class Point<unsigned int>" => utils::read_point_uint(reader).map(Value::PointUInt),
        "class Point<float>" => utils::read_point_float(reader).map(Value::PointFloat),
        "class Rect<int>" => utils::read_rect_int(reader).map(Value::RectInt),
        "class Rect<float>" => utils::read_rect_float(reader).map(Value::RectFloat),

        _ => return None,
    };

    Some(result)
}
