use kobold_bit_buf::BitReader;
use kobold_types::{Property, PropertyFlags};

use super::{enum_variant, object, simple_data, Deserializer, TypeTag};
use crate::value::{List, Value};

pub fn deserialize<T: TypeTag>(
    de: &mut Deserializer<T>,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
    if property.dynamic {
        deserialize_list(de, property, reader)
    } else {
        deserialize_value(de, property, reader)
    }
}

fn deserialize_value<T: TypeTag>(
    de: &mut Deserializer<T>,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
    if property.is_enum() {
        enum_variant::deserialize(de, property, reader)
    } else {
        // Try to interpret the value as simple data and if that fails,
        // deserialize a new object as a fallback strategy.
        match simple_data::deserialize(de, &property.r#type, reader) {
            Some(v) => Ok(v),
            None => object::deserialize(de, reader),
        }
    }
}

fn deserialize_list<T: TypeTag>(
    de: &mut Deserializer<T>,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
    let len = simple_data::read_seq_len(&de.options, reader);
    let mut inner = Vec::with_capacity(len);

    check_recursion! {
        let de = de;
        for _ in 0..len {
            inner.push(deserialize_value(de, property, reader)?);
        }
    }

    Ok(Value::List(List { inner }))
}
