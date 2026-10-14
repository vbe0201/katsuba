use std::fmt;

use indexmap::IndexMap;
use katsuba_types::{Property, TypeList};
use serde::{
    Deserialize,
    de::{
        self, DeserializeSeed, Deserializer, Error as _, MapAccess, SeqAccess, Unexpected, Visitor,
    },
};

use super::*;

pub fn deserialize<'de, D>(deserializer: D, types: &TypeList) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    ObjectSeed { types }.deserialize(deserializer)
}

struct PropertySeed<'a> {
    property: &'a Property,
    types: &'a TypeList,
}

impl<'de> DeserializeSeed<'de> for PropertySeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        if self.property.dynamic {
            deserializer.deserialize_seq(ListVisitor {
                property: self.property,
                types: self.types,
            })
        } else {
            ScalarSeed {
                property: self.property,
                types: self.types,
            }
            .deserialize(deserializer)
        }
    }
}

struct ScalarSeed<'a> {
    property: &'a Property,
    types: &'a TypeList,
}

impl<'de> DeserializeSeed<'de> for ScalarSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        if self.property.is_enum() {
            // Enum variants may be given in integral or string representation.
            // Our serializer accepts everything, so anything goes.
            return deserializer.deserialize_any(EnumVisitor);
        }

        match &*self.property.r#type {
            // Booleans are simply true or false values.
            "bool" => deserializer.deserialize_bool(BoolVisitor),

            // Integer types are always parsed as signed or unsigned values.
            // TODO: Should we move range checks here?
            "char" | "unsigned char" | "short" | "unsigned short" | "wchar_t" | "int" | "long"
            | "unsigned int" | "unsigned long" | "unsigned __int64" | "gid" | "union gid"
            | "bi2" | "bui2" | "bi3" | "bui3" | "bi4" | "bui4" | "bi5" | "bui5" | "bi6"
            | "bui6" | "bi7" | "bui7" | "s24" | "u24" => deserializer.deserialize_any(IntVisitor),

            // Floating-point values are parsed as either floating-point directly or
            // integer values that can be losslessly converted to floating-point.
            "float" | "double" => deserializer.deserialize_any(FloatVisitor),

            // String values have custom deserializers parsing value representations
            // potentially containing raw bytes.
            "std::string" => CxxStr::deserialize(deserializer).map(Value::String),
            "std::wstring" => CxxWStr::deserialize(deserializer).map(Value::WString),

            // These classes are not PropertyClasses but have deserializers already.
            "class Color" => Color::deserialize(deserializer).map(Value::Color),
            "class Vector3D" => Vec3::deserialize(deserializer).map(Value::Vec3),
            "class Quaternion" => Quaternion::deserialize(deserializer).map(Value::Quat),
            "class Euler" => Euler::deserialize(deserializer).map(Value::Euler),
            "class Matrix3x3" => {
                Matrix::deserialize(deserializer).map(|m| Value::Mat3x3(Box::new(m)))
            }
            "class Size<int>" => Size::<i32>::deserialize(deserializer).map(Value::SizeInt),
            "class Point<int>" => Point::<i32>::deserialize(deserializer).map(Value::PointInt),
            "class Point<unsigned char>" => {
                Point::<u8>::deserialize(deserializer).map(Value::PointUChar)
            }
            "class Point<unsigned int>" => {
                Point::<u32>::deserialize(deserializer).map(Value::PointUInt)
            }
            "class Point<float>" => Point::<f32>::deserialize(deserializer).map(Value::PointFloat),
            "class Rect<int>" => Rect::<i32>::deserialize(deserializer).map(Value::RectInt),
            "class Rect<float>" => Rect::<f32>::deserialize(deserializer).map(Value::RectFloat),

            // Anything else is assumed to be a PropertyClass object.
            _ => ObjectSeed { types: self.types }.deserialize(deserializer),
        }
    }
}

struct ObjectSeed<'a> {
    types: &'a TypeList,
}

impl<'de> DeserializeSeed<'de> for ObjectSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ObjectVisitor { types: self.types })
    }
}

struct ObjectVisitor<'a> {
    types: &'a TypeList,
}

impl<'de> Visitor<'de> for ObjectVisitor<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("null, or a map with \"$__type\" as its first key")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Empty)
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Empty)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let Some("$__type") = map.next_key()? else {
            return Err(A::Error::custom("expected \"$__type\" as the first key"));
        };

        let type_hash: u32 = map.next_value()?;
        let type_def = self
            .types
            .0
            .get(&type_hash)
            .ok_or_else(|| A::Error::custom(format!("unknown type hash '{type_hash}'")))?;

        let mut inner = IndexMap::with_capacity(map.size_hint().unwrap_or(0));
        while let Some(name) = map.next_key::<String>()? {
            let property = type_def.property_by_name(&name).ok_or_else(|| {
                A::Error::custom(format!(
                    "type '{}' has no property named \"{name}\"",
                    type_def.name
                ))
            })?;

            let value = map.next_value_seed(PropertySeed {
                property,
                types: self.types,
            })?;
            inner.insert(property.name.clone(), value);
        }

        Ok(Value::Object(Box::new(Object { type_hash, inner })))
    }
}

struct ListVisitor<'a> {
    property: &'a Property,
    types: &'a TypeList,
}

impl<'de> Visitor<'de> for ListVisitor<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a sequence of values")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut inner = Vec::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(v) = seq.next_element_seed(ScalarSeed {
            property: self.property,
            types: self.types,
        })? {
            inner.push(v);
        }

        Ok(Value::List(List { inner }))
    }
}

struct BoolVisitor;

impl Visitor<'_> for BoolVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a boolean")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
}

struct IntVisitor;

impl Visitor<'_> for IntVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an integer")
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Signed(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Unsigned(v))
    }

    fn visit_i128<E: de::Error>(self, v: i128) -> Result<Value, E> {
        i64::try_from(v)
            .map(Value::Signed)
            .map_err(|_| E::custom("integer out of range"))
    }

    fn visit_u128<E: de::Error>(self, v: u128) -> Result<Value, E> {
        u64::try_from(v)
            .map(Value::Unsigned)
            .map_err(|_| E::custom("integer out of range"))
    }
}

struct FloatVisitor;

impl Visitor<'_> for FloatVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a floating-point number")
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Ok(Value::Float(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        let f = v as f64;
        if f != (i64::MAX as f64) && f as i64 == v {
            Ok(Value::Float(f))
        } else {
            Err(E::invalid_value(Unexpected::Signed(v), &self))
        }
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        let f = v as f64;
        if f != (u64::MAX as f64) && f as u64 == v {
            Ok(Value::Float(f))
        } else {
            Err(E::invalid_value(Unexpected::Unsigned(v), &self))
        }
    }
}

struct EnumVisitor;

impl Visitor<'_> for EnumVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an integer or a string naming an enum variant")
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Signed(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Unsigned(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(CxxStr(v.into_bytes())))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(CxxStr(v.as_bytes().to_vec())))
    }
}
