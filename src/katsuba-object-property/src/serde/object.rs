use std::sync::Arc;

use bitter::{BitReader, LittleEndianReader};
use indexmap::IndexMap;
use katsuba_types::{PropertyFlags, TypeDef};
use katsuba_utils::{
    align::align_down,
    hash::{djb2, string_id},
};

use super::{Error, SerializerFlags, SerializerParts, property, type_tag, utils};
use crate::{Value, value::Object};

pub fn deserialize(
    de: &mut SerializerParts,
    reader: &mut LittleEndianReader<'_>,
) -> Result<Value, Error> {
    de.with_recursion_limit(|de| {
        let types = de.types.clone();
        let res = match type_tag::property_class(reader, &types) {
            // If a type definition exists, read the full object.
            Ok(Some(type_def)) => {
                let object_size = read_bit_size(de, reader)? as usize;
                deserialize_properties(de, object_size, type_def, reader)?
            }

            // If we encountered a null pointer, return an empty value.
            Ok(None) => Value::Empty,

            // If no type definition exists but we're allowed to skip it,
            // consume the bits the object is supposed to occupy.
            Err(Error::UnknownType(tag)) if de.options.skip_unknown_types => {
                log::warn!("Encountered unknown type with tag '{tag}'; skipping it");

                let object_size = read_bit_size(de, reader)? as usize;
                let aligned_object_size = align_down(object_size, u8::BITS as _);
                let remainder_bits = (object_size - aligned_object_size) as u32;

                // When skipping an object, we must make sure to consume
                // exactly as many bits as specified or we might end up
                // with property size mismatches.
                //
                // We first read the whole bytes out of the given bit size,
                // then read the remaining bits.
                utils::read_bytes(reader, utils::bits_to_bytes(aligned_object_size))?;
                if remainder_bits > 0 {
                    utils::read_bits(reader, remainder_bits)?;
                }

                Value::Empty
            }

            // If no type definition was found but we're also not allowed
            // to skip the object, return an error.
            Err(e) => return Err(e),
        };

        Ok(res)
    })
}

fn deserialize_properties(
    de: &mut SerializerParts,
    object_size: usize,
    type_def: &TypeDef,
    reader: &mut LittleEndianReader<'_>,
) -> Result<Value, Error> {
    let mut inner = IndexMap::with_capacity(type_def.properties.len());

    if de.options.shallow {
        deserialize_properties_shallow(&mut inner, de, type_def, reader)?;
    } else {
        deserialize_properties_deep(&mut inner, de, object_size, type_def, reader)?;
    }

    let hash = match de.options.djb2_only {
        true => djb2(type_def.name.as_bytes()),
        false => string_id(type_def.name.as_bytes()),
    };

    Ok(Value::Object(Box::new(Object {
        type_hash: hash,
        inner,
    })))
}

#[inline]
fn deserialize_properties_shallow(
    obj: &mut IndexMap<Arc<str>, Value>,
    de: &mut SerializerParts,
    type_def: &TypeDef,
    reader: &mut LittleEndianReader<'_>,
) -> Result<(), Error> {
    // In shallow mode, we walk masked properties in order.
    let mask = de.options.property_mask;
    for property in type_def
        .properties
        .values()
        .filter(|p| p.flags.contains(mask) && !p.flags.contains(PropertyFlags::DEPRECATED))
    {
        if property.flags.contains(PropertyFlags::DELTA_ENCODE) {
            let delta_present = utils::read_bool(reader)?;
            if !delta_present {
                if de
                    .options
                    .flags
                    .contains(SerializerFlags::FORBID_DELTA_ENCODE)
                {
                    return Err(Error::MissingDelta);
                }

                continue;
            }
        }

        let value = property::deserialize(de, property, reader)?;
        obj.insert(property.name.clone(), value);
    }

    Ok(())
}

#[inline]
fn deserialize_properties_deep(
    obj: &mut IndexMap<Arc<str>, Value>,
    de: &mut SerializerParts,
    mut object_size: usize,
    type_def: &TypeDef,
    reader: &mut LittleEndianReader<'_>,
) -> Result<(), Error> {
    // In deep mode, the properties name themselves.
    while object_size > 0 {
        // Back up the current buffer length and read the property size.
        // This will also count padding bits to byte boundaries.
        let previous_buf_len = reader.bits_remaining().unwrap();
        let property_size = utils::read_bits_aligned(reader, u32::BITS)? as usize;

        // Read the property's hash and find the object in type defs.
        let property_hash = utils::read_bits(reader, u32::BITS)? as u32;
        let property = type_def
            .properties
            .get(&property_hash)
            .ok_or(Error::UnknownProperty(property_hash))?;

        // Deserialize the property's value.
        let value = property::deserialize(de, property, reader)?;

        // Validate the size expectations.
        let actual_size = previous_buf_len - reader.bits_remaining().unwrap();
        if property_size != actual_size {
            return Err(Error::PropertySizeMismatch {
                expected: property_size,
                actual: actual_size,
            });
        }

        // Prepare for the next round of deserialization.
        object_size = object_size
            .checked_sub(property_size)
            .ok_or(Error::ObjectSizeMismatch)?;

        // Lastly, insert the property into the object.
        obj.insert(property.name.clone(), value);
    }

    Ok(())
}

#[inline]
pub(crate) fn read_bit_size(
    de: &SerializerParts,
    reader: &mut LittleEndianReader<'_>,
) -> Result<u32, Error> {
    if !de.options.shallow {
        utils::read_bits(reader, u32::BITS).map(|v| v as u32 - u32::BITS)
    } else {
        Ok(0)
    }
}
