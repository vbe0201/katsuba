use anyhow::{anyhow, bail};

use super::{property::PropertyDeserializer, Deserializer};
use crate::object_property::{HashMap, Object, PropertyFlags, TypeDef, TypeTag, Value};

pub struct ObjectDeserializer<'de, T> {
    pub(crate) de: &'de mut Deserializer<T>,
}

impl<'de, T: TypeTag> ObjectDeserializer<'de, T> {
    pub fn deserialize(&mut self) -> anyhow::Result<Value> {
        check_recursion! {
            let this = self;

            let res = match T::object_identity(&mut this.de.reader, &this.de.types) {
                // If a type definition exists, read the full object.
                Ok(Some(type_def)) => {
                    let object_size = this.read_bit_size()? as usize;
                    let object = this.deserialize_properties(object_size, &type_def)?;

                    Value::Object(Object {
                        name: type_def.name.to_owned(),
                        inner: object,
                    })
                }

                // If we encountered a null pointer, return an empty value.
                Ok(None) => Value::Empty,

                // If no type definition exists but we're allowed to skip it,
                // consume the bits the object is supposed to occupy.
                Err(_) if this.de.options.skip_unknown_types => {
                    let object_size = this.read_bit_size()? as usize;
                    this.de.reader.read_bits(object_size)?;

                    Value::Empty
                }

                // If no type definition was found but we're also not allowed
                // to skip the object, return an error.
                Err(e) => return Err(e),
            };
        }

        Ok(res)
    }

    fn read_bit_size(&mut self) -> anyhow::Result<u32> {
        (!self.de.options.shallow)
            .then(|| Ok(self.de.deserialize_u32()? - u32::BITS))
            .unwrap_or(Ok(0))
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
                    PropertyDeserializer { de: self.de }.deserialize(property)?,
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

                // 64 bits for size + hash
                // 32 bits for the int itself
                // 103 - 96 = 7 and where the fuck are THOSE coming from

                // delta encode works by writing one bit before the value to indicate if it is present
                // but the thing is for int it would have to add 7 padding bits after it
                //if property.name == "m_levelRestriction" {
                //    println!(
                //        "delta encode? {}",
                //        property.flags.contains(PropertyFlags::DELTA_ENCODE)
                //    );
                //}

                // Deserialize the property's value.
                let value = PropertyDeserializer { de: self.de }.deserialize(property)?;

                // Validate the size expectations.
                let actual_size = previous_buf_len - self.de.reader.len();
                if property_size != actual_size {
                    bail!(
                        "size mismatch for property; expected {property_size}, got {actual_size}"
                    );
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
