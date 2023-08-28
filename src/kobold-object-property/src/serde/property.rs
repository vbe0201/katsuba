use kobold_bit_buf::BitReader;
use kobold_types::Property;

use super::{enum_variant, object, simple_data, Deserializer, Diagnostics, TypeTag};
use crate::value::{List, Value};

pub fn deserialize<D: Diagnostics, T: TypeTag>(
    de: &mut Deserializer<D>,
    property: &Property,
    reader: &mut BitReader<'_>,
    diagnostics: &mut D,
) -> anyhow::Result<Value> {
    if property.dynamic {
        deserialize_list::<_, T>(de, property, reader, diagnostics)
    } else {
        deserialize_value::<_, T>(de, property, reader, diagnostics)
    }
}

fn deserialize_value<D: Diagnostics, T: TypeTag>(
    de: &mut Deserializer<D>,
    property: &Property,
    reader: &mut BitReader<'_>,
    diagnostics: &mut D,
) -> anyhow::Result<Value> {
    if property.is_enum() {
        enum_variant::deserialize(de, property, reader)
    } else {
        // Try to interpret the value as simple data and if that fails,
        // deserialize a new object as a fallback strategy.
        match simple_data::deserialize(de, &property.r#type, reader) {
            Some(v) => Ok(v),
            None => object::deserialize::<_, T>(de, reader, diagnostics),
        }
    }
}

fn deserialize_list<D: Diagnostics, T: TypeTag>(
    de: &mut Deserializer<D>,
    property: &Property,
    reader: &mut BitReader<'_>,
    diagnostics: &mut D,
) -> anyhow::Result<Value> {
    let len = simple_data::read_seq_len(&de.options, reader);
    let mut inner = Vec::with_capacity(len);

    check_recursion! {
        let de = de;
        for _ in 0..len {
            inner.push(deserialize_value::<_, T>(de, property, reader, diagnostics)?);
        }
    }

    Ok(Value::List(List { inner }))
}
