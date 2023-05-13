use anyhow::bail;

use super::{
    enum_variant::EnumVariantDeserializer, object::ObjectDeserializer,
    simple_data::SimpleDataDeserializer, Deserializer, SerializerFlags,
};
use crate::object_property::{List, Property, PropertyFlags, TypeTag, Value};

fn deserialize_value<T: TypeTag>(
    de: &mut Deserializer<T>,
    property: &Property,
) -> anyhow::Result<Value> {
    if property
        .flags
        .intersects(PropertyFlags::BITS | PropertyFlags::ENUM)
    {
        EnumVariantDeserializer { de }.deserialize(property)
    } else {
        // Try to interpret the value as simple data and if that fails,
        // deserialize a new object as a fallback strategy.
        SimpleDataDeserializer { de }
            .deserialize(&property.r#type)
            .or_else(|_| ObjectDeserializer { de }.deserialize())
    }
}

/// Deserializes a property value from its [`Property`] description.
///
/// This handles both dynamic containers and single values.
pub struct PropertyDeserializer<'de, T> {
    pub(crate) de: &'de mut Deserializer<T>,
}

impl<'de, T: TypeTag> PropertyDeserializer<'de, T> {
    pub fn deserialize(&mut self, property: &Property) -> anyhow::Result<Value> {
        if property.flags.contains(PropertyFlags::DELTA_ENCODE) && !self.de.deserialize_bool()? {
            if self
                .de
                .options
                .flags
                .contains(SerializerFlags::FORBID_DELTA_ENCODE)
            {
                bail!("missing delta value which is supposed to be present");
            }

            return Ok(Value::Empty);
        }

        if property.dynamic {
            ListDeserializer { de: self.de }.deserialize(property)
        } else {
            deserialize_value(self.de, property)
        }
    }
}

struct ListDeserializer<'de, T> {
    de: &'de mut Deserializer<T>,
}

impl<'de, T: TypeTag> ListDeserializer<'de, T> {
    pub fn deserialize(&mut self, property: &Property) -> anyhow::Result<Value> {
        let len = self.de.read_seq_len()?;
        let mut list = Vec::with_capacity(len);

        check_recursion! {
            let this = self;

            for _ in 0..len {
                list.push(deserialize_value(this.de, property)?);
            }
        }

        Ok(Value::List(List { inner: list }))
    }
}
