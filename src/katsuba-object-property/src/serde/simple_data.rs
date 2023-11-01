use katsuba_bit_buf::BitReader;
use phf::phf_map;

use crate::value::*;

use super::{utils, Error, SerializerOptions, SerializerParts};

type ReadCallback = fn(&mut BitReader<'_>, &SerializerOptions) -> Result<Value, Error>;

static DESERIALIZER_LUT: phf::Map<&'static str, (bool, ReadCallback)> = phf_map! {
    // Primitive C++ types
    "bool" => (true, |r, _| utils::read_bool(r).map(Value::Bool)),
    "char" => (false, |r, _| utils::read_signed_bits(r, i8::BITS).map(Value::Signed)),
    "unsigned char" => (false, |r, _| utils::read_bits(r, u8::BITS).map(Value::Unsigned)),
    "short" => (false, |r, _| utils::read_signed_bits(r, i16::BITS).map(Value::Signed)),
    "unsigned short" => (false, |r, _| utils::read_bits(r, u16::BITS).map(Value::Unsigned)),
    "wchar_t" => (false, |r, _| utils::read_bits(r, u16::BITS).map(Value::Unsigned)),
    "int" => (false, |r, _| utils::read_signed_bits(r, i32::BITS).map(Value::Signed)),
    "unsigned int" => (false, |r, _| utils::read_bits(r, u32::BITS).map(Value::Unsigned)),
    "long" => (false, |r, _| utils::read_signed_bits(r, i32::BITS).map(Value::Signed)),
    "unsigned long" => (false, |r, _| utils::read_bits(r, u32::BITS).map(Value::Unsigned)),
    "float" => (false, |r, _| utils::read_bits(r, u32::BITS).map(|v| Value::Float(f32::from_bits(v as _) as f64))),
    "double" => (false, |r, _| utils::read_u64(r).map(|v| Value::Float(f64::from_bits(v)))),
    "unsigned __int64" => (false, |r, _| utils::read_u64(r).map(Value::Unsigned)),
    "gid" => (false, |r, _| utils::read_u64(r).map(Value::Unsigned)),
    "union gid" => (false, |r, _| utils::read_u64(r).map(Value::Unsigned)),

    // Bit integers
    "bi2" => (true, |r, _| utils::read_signed_bits(r, 2).map(Value::Signed)),
    "bui2" => (true, |r, _| utils::read_bits(r, 2).map(Value::Unsigned)),
    "bi3" => (true, |r, _| utils::read_signed_bits(r, 3).map(Value::Signed)),
    "bui3" => (true, |r, _| utils::read_bits(r, 3).map(Value::Unsigned)),
    "bi4" => (true, |r, _| utils::read_signed_bits(r, 4).map(Value::Signed)),
    "bui4" => (true, |r, _| utils::read_bits(r, 4).map(Value::Unsigned)),
    "bi5" => (true, |r, _| utils::read_signed_bits(r, 5).map(Value::Signed)),
    "bui5" => (true, |r, _| utils::read_bits(r, 5).map(Value::Unsigned)),
    "bi6" => (true, |r, _| utils::read_signed_bits(r, 6).map(Value::Signed)),
    "bui6" => (true, |r, _| utils::read_bits(r, 6).map(Value::Unsigned)),
    "bi7" => (true, |r, _| utils::read_signed_bits(r, 7).map(Value::Signed)),
    "bui7" => (true, |r, _| utils::read_bits(r, 7).map(Value::Unsigned)),
    "s24" => (true, |r, _| utils::read_signed_bits(r, 24).map(Value::Signed)),
    "u24" => (true, |r, _| utils::read_bits(r, 24).map(Value::Unsigned)),

    // Strings
    "std::string" => (true, |r, opts| utils::read_string(r, opts).map(|v| Value::String(CxxStr(v.to_owned())))),
    "std::wstring" => (true, |r, opts| utils::read_wstring(r, opts).map(|v| Value::WString(CxxWStr(v)))),

    // Miscellaneous leaf types that are not PropertyClasses
    "class Color" => (false, |r, _| utils::read_color(r).map(Value::Color)),
    "class Vector3D" => (false, |r, _| utils::read_vec3(r).map(Value::Vec3)),
    "class Quaternion" => (false, |r, _| utils::read_quat(r).map(Value::Quat)),
    "class Euler" => (false, |r, _| utils::read_euler(r).map(Value::Euler)),
    "class Matrix3x3" => (false, |r, _| utils::read_matrix(r).map(|v| Value::Mat3x3(Box::new(v)))),
    "class Size<int>" => (false, |r, _| {
        let width = utils::read_signed_bits(r, i32::BITS)? as i32;
        let height = utils::read_signed_bits(r, i32::BITS)? as i32;

        Ok(Value::SizeInt(Size { width, height }))
    }),
    "class Point<int>" => (false, |r, _| {
        let x = utils::read_signed_bits(r, i32::BITS)? as i32;
        let y = utils::read_signed_bits(r, i32::BITS)? as i32;

        Ok(Value::PointInt(Point { x, y }))
    }),
    "class Point<float>" => (false, |r, _| {
        let x = f32::from_bits(utils::read_bits(r, u32::BITS)? as _);
        let y = f32::from_bits(utils::read_bits(r, u32::BITS)? as _);

        Ok(Value::PointFloat(Point { x, y }))
    }),
    "class Rect<int>" => (false, |r, _| {
        let left = utils::read_signed_bits(r, i32::BITS)? as i32;
        let top = utils::read_signed_bits(r, i32::BITS)? as i32;
        let right = utils::read_signed_bits(r, i32::BITS)? as i32;
        let bottom = utils::read_signed_bits(r, i32::BITS)? as i32;

        Ok(Value::RectInt(Rect { left, top, right, bottom }))
    }),
    "class Rect<float>" => (false, |r, _| {
        let left = f32::from_bits(utils::read_signed_bits(r, u32::BITS)? as _);
        let top = f32::from_bits(utils::read_signed_bits(r, u32::BITS)? as _);
        let right = f32::from_bits(utils::read_signed_bits(r, u32::BITS)? as _);
        let bottom = f32::from_bits(utils::read_signed_bits(r, u32::BITS)? as _);

        Ok(Value::RectFloat(Rect { left, top, right, bottom }))
    }),
};

pub fn deserialize(
    de: &SerializerParts,
    ty: &str,
    reader: &mut BitReader<'_>,
) -> Option<Result<Value, Error>> {
    DESERIALIZER_LUT.get(ty).map(|(bits, f)| {
        if de.options.shallow && !bits {
            reader.realign_to_byte();
        }

        f(reader, &de.options)
    })
}
