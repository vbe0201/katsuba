use bitter::{LittleEndianReader, LittleEndianWriter};
use katsuba_types::Property;

use super::{Error, SerializerFlags, SerializerParts, utils};
use crate::Value;

pub fn deserialize(
    de: &SerializerParts,
    property: &Property,
    reader: &mut LittleEndianReader<'_>,
) -> Result<Value, Error> {
    if de
        .options
        .flags
        .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
    {
        let raw = utils::read_string(reader, &de.options)?;
        let value = std::str::from_utf8(raw)?;
        property
            .decode_enum_variant(value)
            .map(Value::Enum)
            .map_err(Into::into)
    } else {
        let value = utils::read_signed_bits_aligned(reader, u32::BITS)?;
        Ok(Value::Enum(value))
    }
}

pub fn serialize(
    ser: &SerializerParts,
    property: &Property,
    value: &Value,
    writer: &mut LittleEndianWriter<Vec<u8>>,
) -> Result<(), Error> {
    enum Variant<'a> {
        Int(i64),
        Str(&'a str),
    }

    let variant = match value {
        Value::Enum(v) | Value::Signed(v) => Variant::Int(*v),
        Value::Unsigned(v) => u32::try_from(*v)
            .map(|v| Variant::Int(v as i32 as i64))
            .map_err(|_| Error::OutOfRange(property.r#type.clone()))?,
        Value::String(s) => std::str::from_utf8(&s.0).map(Variant::Str)?,

        _ => return Err(Error::TypeMismatch(property.r#type.clone())),
    };

    if ser
        .options
        .flags
        .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
    {
        match variant {
            Variant::Int(i) => {
                let v = property.encode_enum_variant(i)?;
                utils::write_string(writer, v.as_bytes(), &ser.options)
            }
            Variant::Str(s) => utils::write_string(writer, s.as_bytes(), &ser.options),
        }
    } else {
        match variant {
            Variant::Int(i) => write_enum_bits(writer, property, i),
            Variant::Str(s) => {
                let v = property.decode_enum_variant(s)?;
                write_enum_bits(writer, property, v)
            }
        }
    }
}

fn write_enum_bits(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    property: &Property,
    v: i64,
) -> Result<(), Error> {
    // We intentionally accept both the range of signed values and the range of unsigned
    // values which produce 32-bit patterns. Since C++ enums can use different underlying
    // storage types, this is the best compromise to cover all use cases.
    let min = i32::MIN as i64;
    let max = u32::MAX as i64;
    if !(min..=max).contains(&v) {
        return Err(Error::OutOfRange(property.r#type.clone()));
    }

    utils::write_signed_bits_aligned(writer, v, u32::BITS)
}
