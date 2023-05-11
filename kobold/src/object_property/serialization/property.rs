use anyhow::bail;

use super::{
    enum_variant::EnumVariantDeserializer, object::ObjectDeserializer,
    simple_data::SimpleDataDeserializer, Deserializer, SerializerFlags,
};
use crate::object_property::{List, Property, PropertyFlags, TypeTag, Value};

fn deserialize_value<T: TypeTag>(
    de: &mut Deserializer<T>,
    property: &Property,
) -> anyhow::Result<(bool, Value)> {
    if property
        .flags
        .intersects(PropertyFlags::BITS | PropertyFlags::ENUM)
    {
        Ok((false, EnumVariantDeserializer { de }.deserialize(property)?))
    } else {
        // Try to interpret the value as simple data and if that fails,
        // deserialize a new object as a fallback strategy.
        SimpleDataDeserializer { de }
            .deserialize(&property.r#type)
            .map(|v| (false, v))
            .or_else(|_| {
                let mut obj = ObjectDeserializer { de, skipped: false };
                let value = obj.deserialize()?;

                Ok((obj.skipped, value))
            })
    }
}

/// Deserializes a property value from its [`Property`] description.
///
/// This handles both dynamic containers and single values.
pub struct PropertyDeserializer<'de, T> {
    pub(crate) de: &'de mut Deserializer<T>,
    pub(crate) skipped: bool,
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
            let mut de = ListDeserializer {
                de: self.de,
                skipped: false,
            };
            let value = de.deserialize(property)?;
            self.skipped = de.skipped;

            Ok(value)
        } else {
            let (skipped, value) = deserialize_value(self.de, property)?;
            self.skipped = skipped;

            Ok(value)
        }
    }
}

struct ListDeserializer<'de, T> {
    de: &'de mut Deserializer<T>,
    skipped: bool,
}

impl<'de, T: TypeTag> ListDeserializer<'de, T> {
    pub fn deserialize(&mut self, property: &Property) -> anyhow::Result<Value> {
        let len = self.de.read_seq_len()?;
        let mut list = Vec::with_capacity(len);

        check_recursion! {
            let this = self;

            for _ in 0..len {
                let (skipped, value) = deserialize_value(this.de, property)?;
                this.skipped |= skipped;

                list.push(value);
            }
        }

        Ok(Value::List(List { inner: list }))
    }
}
