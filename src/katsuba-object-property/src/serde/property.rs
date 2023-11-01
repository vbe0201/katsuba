use katsuba_bit_buf::BitReader;
use katsuba_types::Property;

use super::*;
use crate::value::{List, Value};

pub fn deserialize<T: TypeTag>(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> Result<Value, Error> {
    log::debug!("Deserializing value for property '{}'", property.name);

    let value = if property.dynamic {
        deserialize_list::<T>(de, property, reader)?
    } else {
        deserialize_value::<T>(de, property, reader)?
    };

    log::trace!("Got '{value:?}'");

    Ok(value)
}

fn deserialize_value<T: TypeTag>(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> Result<Value, Error> {
    if property.is_enum() {
        enum_variant::deserialize(de, property, reader)
    } else {
        // Try to interpret the value as simple data and if that fails,
        // deserialize a new object as a fallback strategy.
        match simple_data::deserialize(de, &property.r#type, reader) {
            Some(v) => v,
            None => object::deserialize::<T>(de, reader),
        }
    }
}

fn deserialize_list<T: TypeTag>(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
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
            inner.push(deserialize_value::<T>(de, property, reader)?);
        }

        Ok(())
    })?;

    Ok(Value::List(List { inner }))
}
