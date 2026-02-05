use bitter::LittleEndianReader;
use phf::phf_map;

use super::{Error, SerializerOptions, SerializerParts, utils};
use crate::value::*;

type ReadFn = fn(&mut LittleEndianReader<'_>, &SerializerOptions) -> Result<Value, Error>;

static LUT: phf::Map<&'static str, ReadFn> = phf_map! {
    // Primitive C++ types
    "bool" => |r, _| utils::read_bool(r).map(Value::Bool),
    "char" => |r, _| utils::read_signed_bits_aligned(r, i8::BITS).map(Value::Signed),
    "unsigned char" => |r, _| utils::read_bits_aligned(r, u8::BITS).map(Value::Unsigned),
    "short" => |r, _| utils::read_signed_bits_aligned(r, i16::BITS).map(Value::Signed),
    "unsigned short" => |r, _| utils::read_bits_aligned(r, u16::BITS).map(Value::Unsigned),
    "wchar_t" => |r, _| utils::read_bits_aligned(r, u16::BITS).map(Value::Unsigned),
    "int" => |r, _| utils::read_signed_bits_aligned(r, i32::BITS).map(Value::Signed),
    "unsigned int" => |r, _| utils::read_bits_aligned(r, u32::BITS).map(Value::Unsigned),
    "long" => |r, _| utils::read_signed_bits_aligned(r, i32::BITS).map(Value::Signed),
    "unsigned long" => |r, _| utils::read_bits_aligned(r, u32::BITS).map(Value::Unsigned),
    "float" => |r, _| utils::read_bits_aligned(r, u32::BITS).map(|v| Value::Float(f32::from_bits(v as u32) as f64)),
    "double" => |r, _| utils::read_bits_aligned(r, u64::BITS).map(|v| Value::Float(f64::from_bits(v))),
    "unsigned __int64" => |r, _| utils::read_bits_aligned(r, u64::BITS).map(Value::Unsigned),
    "gid" => |r, _| utils::read_bits_aligned(r, u64::BITS).map(Value::Unsigned),
    "union gid" => |r, _| utils::read_bits_aligned(r, u64::BITS).map(Value::Unsigned),

    // Bit integers
    "bi2" => |r, _| utils::read_signed_bits(r, 2).map(Value::Signed),
    "bui2" => |r, _| utils::read_bits(r, 2).map(Value::Unsigned),
    "bi3" => |r, _| utils::read_signed_bits(r, 3).map(Value::Signed),
    "bui3" => |r, _| utils::read_bits(r, 3).map(Value::Unsigned),
    "bi4" => |r, _| utils::read_signed_bits(r, 4).map(Value::Signed),
    "bui4" => |r, _| utils::read_bits(r, 4).map(Value::Unsigned),
    "bi5" => |r, _| utils::read_signed_bits(r, 5).map(Value::Signed),
    "bui5" => |r, _| utils::read_bits(r, 5).map(Value::Unsigned),
    "bi6" => |r, _| utils::read_signed_bits(r, 6).map(Value::Signed),
    "bui6" => |r, _| utils::read_bits(r, 6).map(Value::Unsigned),
    "bi7" => |r, _| utils::read_signed_bits(r, 7).map(Value::Signed),
    "bui7" => |r, _| utils::read_bits(r, 7).map(Value::Unsigned),
    "s24" => |r, _| utils::read_signed_bits(r, 24).map(Value::Signed),
    "u24" => |r, _| utils::read_bits(r, 24).map(Value::Unsigned),

    // Strings
    "std::string" => |r, opts| utils::read_string(r, opts).map(|v| Value::String(CxxStr(v.into()))),
    "std::wstring" => |r, opts| utils::read_wstring(r, opts).map(|v| Value::WString(CxxWStr(v))),

    // Various leaf types that are not PropertyClasses
    "class Color" => |r, _| utils::read_color(r).map(Value::Color),
    "class Vector3D" => |r, _| utils::read_vec3(r).map(Value::Vec3),
    "class Quaternion" => |r, _| utils::read_quat(r).map(Value::Quat),
    "class Euler" => |r, _| utils::read_euler(r).map(Value::Euler),
    "class Matrix3x3" => |r, _| utils::read_matrix(r).map(|v| Value::Mat3x3(Box::new(v))),
    "class Size<int>" => |r, _| {
        let width = utils::read_signed_bits_aligned(r, i32::BITS)? as i32;
        let height = utils::read_signed_bits(r, i32::BITS)? as i32;
        Ok(Value::SizeInt(Size { width, height }))
    },
    "class Point<int>" => |r, _| {
        let x = utils::read_signed_bits_aligned(r, i32::BITS)? as i32;
        let y = utils::read_signed_bits(r, i32::BITS)? as i32;
        Ok(Value::PointInt(Point { x, y }))
    },
    "class Point<float>" => |r, _| {
        let x = utils::read_bits_aligned(r, u32::BITS).map(|v| f32::from_bits(v as u32))?;
        let y = utils::read_bits(r, u32::BITS).map(|v| f32::from_bits(v as u32))?;
        Ok(Value::PointFloat(Point { x, y }))
    },
    "class Rect<int>" => |r, _| {
        let left = utils::read_signed_bits_aligned(r, i32::BITS)? as i32;
        let top = utils::read_signed_bits(r, i32::BITS)? as i32;
        let right = utils::read_signed_bits(r, i32::BITS)? as i32;
        let bottom = utils::read_signed_bits(r, i32::BITS)? as i32;
        Ok(Value::RectInt(Rect { left, top, right, bottom }))
    },
    "class Rect<float>" => |r, _| {
        let left = utils::read_bits_aligned(r, u32::BITS).map(|v| f32::from_bits(v as u32))?;
        let top = utils::read_bits(r, u32::BITS).map(|v| f32::from_bits(v as u32))?;
        let right = utils::read_bits(r, u32::BITS).map(|v| f32::from_bits(v as u32))?;
        let bottom = utils::read_bits(r, u32::BITS).map(|v| f32::from_bits(v as u32))?;
        Ok(Value::RectFloat(Rect {left,top,right,bottom}))
    },
};

#[inline]
pub fn deserialize(
    de: &SerializerParts,
    ty: &str,
    reader: &mut LittleEndianReader<'_>,
) -> Option<Result<Value, Error>> {
    LUT.get(ty).map(|f| f(reader, &de.options))
}
