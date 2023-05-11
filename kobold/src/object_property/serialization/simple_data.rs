use anyhow::bail;

use super::Deserializer;
use crate::object_property::{TypeTag, Value};

#[inline]
fn extract_type_argument(ty: &str) -> Option<&str> {
    let generic = ty.split_once('<')?.1;
    let generic = generic.rsplit_once('>')?.0;

    Some(generic)
}

/// Deserializes primitive types and tuple-like types composed of these.
pub struct SimpleDataDeserializer<'de, T> {
    pub(crate) de: &'de mut Deserializer<T>,
}

impl<'de, T: TypeTag> SimpleDataDeserializer<'de, T> {
    /// Deserializes "simple data" for a given string name.
    pub fn deserialize(&mut self, ty: &str) -> anyhow::Result<Value> {
        match ty {
            // Primitive C++ types.
            "bool" => self.de.deserialize_bool().map(Value::Bool),
            "char" => self.de.deserialize_i8().map(|v| Value::Signed(v as _)),
            "unsigned char" => self.de.deserialize_u8().map(|v| Value::Unsigned(v as _)),
            "short" => self.de.deserialize_i16().map(|v| Value::Signed(v as _)),
            "unsigned short" | "wchar_t" => {
                self.de.deserialize_u16().map(|v| Value::Unsigned(v as _))
            }
            "int" | "long" => self.de.deserialize_i32().map(|v| Value::Signed(v as _)),
            "unsigned int" | "unsigned long" => {
                self.de.deserialize_u32().map(|v| Value::Unsigned(v as _))
            }
            "float" => self.de.deserialize_f32().map(|v| Value::Float(v as _)),
            "double" => self.de.deserialize_f64().map(|v| Value::Float(v as _)),
            "unsigned __int64" | "gid" | "union gid" => {
                self.de.deserialize_u64().map(Value::Unsigned)
            }

            // Bit integers
            "bi2" => self.de.deserialize_signed_bits(2).map(Value::Signed),
            "bui2" => self.de.deserialize_bits(2).map(Value::Unsigned),
            "bi3" => self.de.deserialize_signed_bits(3).map(Value::Signed),
            "bui3" => self.de.deserialize_bits(3).map(Value::Unsigned),
            "bi4" => self.de.deserialize_signed_bits(4).map(Value::Signed),
            "bui4" => self.de.deserialize_bits(4).map(Value::Unsigned),
            "bi5" => self.de.deserialize_signed_bits(5).map(Value::Signed),
            "bui5" => self.de.deserialize_bits(5).map(Value::Unsigned),
            "bi6" => self.de.deserialize_signed_bits(6).map(Value::Signed),
            "bui6" => self.de.deserialize_bits(6).map(Value::Unsigned),
            "bi7" => self.de.deserialize_signed_bits(7).map(Value::Signed),
            "bui7" => self.de.deserialize_bits(7).map(Value::Unsigned),

            "s24" => self.de.deserialize_signed_bits(24).map(Value::Signed),
            "u24" => self.de.deserialize_bits(24).map(Value::Unsigned),

            // Strings
            "std::string" | "char*" => self.de.deserialize_str().map(Value::String),
            "std::wstring" | "wchar_t*" => self.de.deserialize_wstr().map(Value::WString),

            // Miscellaneous leaf types that are not PropertyClasses.
            "class Color" => Ok(Value::Color {
                b: self.de.deserialize_u8()?,
                g: self.de.deserialize_u8()?,
                r: self.de.deserialize_u8()?,
                a: self.de.deserialize_u8()?,
            }),
            "class Vector3D" => Ok(Value::Vec3 {
                x: self.de.deserialize_f32()?,
                y: self.de.deserialize_f32()?,
                z: self.de.deserialize_f32()?,
            }),
            "class Quaternion" => Ok(Value::Quat {
                x: self.de.deserialize_f32()?,
                y: self.de.deserialize_f32()?,
                z: self.de.deserialize_f32()?,
                w: self.de.deserialize_f32()?,
            }),
            "class Euler" => Ok(Value::Euler {
                pitch: self.de.deserialize_f32()?,
                roll: self.de.deserialize_f32()?,
                yaw: self.de.deserialize_f32()?,
            }),
            "class Matrix3x3" => Ok(Value::Mat3x3 {
                i: [
                    self.de.deserialize_f32()?,
                    self.de.deserialize_f32()?,
                    self.de.deserialize_f32()?,
                ],
                j: [
                    self.de.deserialize_f32()?,
                    self.de.deserialize_f32()?,
                    self.de.deserialize_f32()?,
                ],
                k: [
                    self.de.deserialize_f32()?,
                    self.de.deserialize_f32()?,
                    self.de.deserialize_f32()?,
                ],
            }),
            s if s.starts_with("class Size") => {
                let ty_arg = extract_type_argument(s).unwrap();
                Ok(Value::Size {
                    wh: Box::new((self.deserialize(ty_arg)?, self.deserialize(ty_arg)?)),
                })
            }
            s if s.starts_with("class Point") => {
                let ty_arg = extract_type_argument(s).unwrap();
                Ok(Value::Point {
                    xy: Box::new((self.deserialize(ty_arg)?, self.deserialize(ty_arg)?)),
                })
            }
            s if s.starts_with("class Rect") => {
                let ty_arg = extract_type_argument(s).unwrap();
                Ok(Value::Rect {
                    inner: Box::new((
                        self.deserialize(ty_arg)?,
                        self.deserialize(ty_arg)?,
                        self.deserialize(ty_arg)?,
                        self.deserialize(ty_arg)?,
                    )),
                })
            }

            _ => bail!("'{ty}' does not represent simple data"),
        }
    }
}
