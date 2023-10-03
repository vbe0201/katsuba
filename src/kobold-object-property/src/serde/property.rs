use kobold_bit_buf::BitReader;
use kobold_types::Property;
use kobold_utils::anyhow;

use super::{enum_variant, object, simple_data, utils, SerializerFlags, SerializerParts, TypeTag};
use crate::value::{List, Value};

pub fn deserialize<T: TypeTag>(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
    if property.dynamic {
        deserialize_list::<T>(de, property, reader)
    } else {
        deserialize_value::<T>(de, property, reader)
    }
}

fn deserialize_value<T: TypeTag>(
    de: &mut SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
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
) -> anyhow::Result<Value> {
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
