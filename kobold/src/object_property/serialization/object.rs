use anyhow::{anyhow, bail};

use super::{property::PropertyDeserializer, Deserializer};
use crate::object_property::{HashMap, Object, PropertyFlags, TypeDef, TypeTag, Value};

pub struct ObjectDeserializer<'de, T> {
    pub(crate) de: &'de mut Deserializer<T>,
    pub(crate) skipped: bool,
}

impl<'de, T: TypeTag> ObjectDeserializer<'de, T> {
    pub fn deserialize(&mut self) -> anyhow::Result<Value> {
        check_recursion! {
            let this = self;

            let res = match T::object_identity(&mut this.de.reader, &this.de.types) {
                Ok(Some(type_def)) => {
                    let object_size = (!this.de.options.shallow).then(|| this.de.deserialize_u32()).unwrap_or(Ok(0))?;
                    let object = this.deserialize_properties((object_size - u32::BITS) as usize, &type_def)?;

                    Value::Object(Object {
                        name: type_def.name.to_owned(),
                        inner: object,
                    })
                }

                Ok(None) => Value::Empty,
                Err(_) if this.de.options.skip_unknown_types => {
                    this.skipped = true;
                    Value::Empty
                }

                Err(e) => return Err(e),
            };
        }

        Ok(res)
    }

    fn deserialize_properties(
        &mut self,
        mut object_size: usize,
        type_def: &TypeDef,
    ) -> anyhow::Result<HashMap<String, Value>> {
        let mut object = HashMap::default();

        if self.de.options.shallow {
            // In shallow mode, we walk masked properties in order.
            let mask = self.de.options.property_mask;
            for property in type_def
                .properties
                .iter()
                .filter(|p| p.flags.contains(mask) && !p.flags.contains(PropertyFlags::DEPRECATED))
            {
                object.insert(
                    property.name.to_owned(),
                    PropertyDeserializer {
                        de: self.de,
                        skipped: false,
                    }
                    .deserialize(property)?,
                );
            }
        } else {
            // When in deep mode, the format dictates which properties
            // there are to discover.
            while object_size > 0 {
                // Back up the current buffer length and read the property's size.
                // This will also count padding bits to byte boundaries.
                let previous_buf_len = self.de.reader.len();
                let property_size = self.de.deserialize_u32()? as usize;

                // Read the property's hash and get the object from type defs.
                let property_hash = self.de.deserialize_u32()?;
                let property = type_def
                    .properties
                    .iter()
                    .find(|p| p.hash == property_hash)
                    .ok_or_else(|| anyhow!("received unknown property hash {property_hash}"))?;

                // Deserialize the property's value.
                let mut de = PropertyDeserializer {
                    de: self.de,
                    skipped: false,
                };
                let value = de.deserialize(property)?;
                let skipped = de.skipped;

                // Validate the size expectations.
                let actual_size = previous_buf_len - self.de.reader.len();
                let delta = property_size.wrapping_sub(actual_size);
                if !skipped && delta != 0 {
                    bail!(
                        "size mismatch for property; expected {property_size}, got {actual_size}"
                    );
                } else {
                    // Consume the bits we skipped over.
                    self.de.reader.read_bits(delta)?;
                }

                // When the size check passed, subtract the property's size from
                // the object's total size to prepare for the next round.
                object_size -= property_size;

                // Lastly, insert the property into our object.
                object.insert(property.name.to_owned(), value);
            }
        }

        Ok(object)
    }
}
