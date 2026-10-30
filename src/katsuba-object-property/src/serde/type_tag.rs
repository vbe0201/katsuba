use bitter::{LittleEndianReader, LittleEndianWriter};
use katsuba_types::{TypeDef, TypeList};

use super::{Error, utils};
use crate::value::{CoreHeader, Object};

#[inline]
pub fn property_class<'a>(
    reader: &mut LittleEndianReader<'_>,
    types: &'a TypeList,
) -> Result<Option<&'a TypeDef>, Error> {
    let hash = utils::read_bits_aligned(reader, u32::BITS)? as u32;
    find_class_def(types, hash)
}

#[inline]
pub fn core_object<'a>(
    reader: &mut LittleEndianReader<'_>,
    types: &'a TypeList,
) -> Result<(Option<&'a TypeDef>, Option<CoreHeader>), Error> {
    let core_type = utils::read_bits_aligned(reader, u8::BITS)? as u8;
    let namespace = utils::read_bits(reader, u8::BITS)? as u8;
    let type_or_template = utils::read_bits(reader, u32::BITS)? as u32;

    if core_type == 0 {
        return Ok((find_class_def(types, type_or_template)?, None));
    }

    let type_hash = types
        .core_object_factory_types
        .get(&core_type)
        .ok_or(Error::UnknownCoreType(core_type))?
        .type_hash;
    let type_def = find_class_def(types, type_hash)?.ok_or(Error::UnknownType(type_hash))?;

    Ok((
        Some(type_def),
        Some(CoreHeader {
            core_type,
            namespace,
            template_id: type_or_template,
        }),
    ))
}

#[inline]
pub fn write_property_class<'a>(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    types: &'a TypeList,
    hash: u32,
) -> Result<&'a TypeDef, Error> {
    let type_def = find_class_def(types, hash)?.ok_or(Error::UnknownType(hash))?;
    utils::write_bits_aligned(writer, hash as u64, u32::BITS)?;
    Ok(type_def)
}

#[inline]
pub fn write_core_object<'a>(
    writer: &mut LittleEndianWriter<Vec<u8>>,
    types: &'a TypeList,
    obj: &Object,
) -> Result<&'a TypeDef, Error> {
    let def = find_class_def(types, obj.type_hash)?.ok_or(Error::UnknownType(obj.type_hash))?;
    let (core_type, namespace, id) = match obj.core {
        Some(h) if !types.core_object_factory_types.contains_key(&h.core_type) => {
            return Err(Error::UnknownCoreType(h.core_type));
        }
        Some(h) => (h.core_type, h.namespace, h.template_id),
        None => (0, 0, obj.type_hash),
    };

    utils::write_bits_aligned(writer, core_type as u64, u8::BITS)?;
    utils::write_bits(writer, namespace as u64, u8::BITS)?;
    utils::write_bits(writer, id as u64, u32::BITS)?;

    Ok(def)
}

#[inline]
fn find_class_def(types: &TypeList, hash: u32) -> Result<Option<&TypeDef>, Error> {
    if hash == 0 {
        log::debug!("Received null hash for object");
        Ok(None)
    } else if let Some(t) = types.classes.get(&hash) {
        log::debug!("Received object hash for '{}' ({hash})", t.name);
        Ok(Some(t))
    } else {
        Err(Error::UnknownType(hash))
    }
}
