use bitter::{LittleEndianReader, LittleEndianWriter};
use katsuba_types::Property;

use super::*;
use crate::value::{List, Value};

pub fn deserialize(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut LittleEndianReader<'_>,
) -> Result<Value, Error> {
    log::debug!("Deserializing value for property '{}'", property.name);

    let value = if property.dynamic {
        deserialize_list(de, property, reader)?
    } else {
        deserialize_value(de, property, reader)?
    };

    log::trace!("Got '{value:?}'");

    Ok(value)
}

pub fn serialize(
    ser: &mut SerializerParts,
    property: &Property,
    value: &Value,
    writer: &mut LittleEndianWriter<Vec<u8>>,
) -> Result<(), Error> {
    log::debug!("Serializing value for property '{}'", property.name);

    if property.dynamic {
        let Value::List(list) = value else {
            let ty = format!("list of {}", property.r#type);
            return Err(Error::TypeMismatch(Arc::from(ty)));
        };

        serialize_list(ser, property, list, writer)
    } else {
        serialize_value(ser, property, value, writer)
    }
}

fn deserialize_value(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut LittleEndianReader<'_>,
) -> Result<Value, Error> {
    if property.is_enum() {
        enum_variant::deserialize(de, property, reader)
    } else {
        // Try to interpret the value as simple data and if that fails,
        // deserialize a new object as a fallback strategy.
        match simple_data::deserialize(de, &property.r#type, reader) {
            Some(v) => v,
            None => object::deserialize(de, reader),
        }
    }
}

fn serialize_value(
    ser: &mut SerializerParts,
    property: &Property,
    value: &Value,
    writer: &mut LittleEndianWriter<Vec<u8>>,
) -> Result<(), Error> {
    if property.is_enum() {
        enum_variant::serialize(ser, property, value, writer)
    } else {
        // Try to serialize the value as simple data and if the type is
        // not one, serialize an object as a fallback strategy.
        match simple_data::serialize(ser, property, value, writer) {
            Some(r) => r,
            None => match value {
                Value::Object(obj) => object::serialize(ser, Some(obj), writer),
                Value::Empty => object::serialize(ser, None, writer),
                _ => Err(Error::TypeMismatch(property.r#type.clone())),
            },
        }
    }
}

fn deserialize_list(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut LittleEndianReader<'_>,
) -> Result<Value, Error> {
    let len = utils::read_container_length(
        reader,
        de.options
            .flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;
    let mut inner = Vec::with_capacity(len);

    de.with_recursion_limit(|de| {
        for _ in 0..len {
            inner.push(deserialize_value(de, property, reader)?);
        }
        Ok(())
    })?;

    Ok(Value::List(List { inner }))
}

fn serialize_list(
    ser: &mut SerializerParts,
    property: &Property,
    list: &List,
    writer: &mut LittleEndianWriter<Vec<u8>>,
) -> Result<(), Error> {
    utils::write_container_length(
        writer,
        list.len(),
        ser.options
            .flags
            .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES),
    )?;

    ser.with_recursion_limit(|ser| {
        list.iter()
            .try_for_each(|v| serialize_value(ser, property, v, writer))
    })
}
