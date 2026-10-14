use bitter::{LittleEndianReader, LittleEndianWriter};
use katsuba_types::{TypeDef, TypeList};

use super::{Error, utils};

#[inline]
pub fn property_class<'a>(
    reader: &mut LittleEndianReader<'_>,
    types: &'a TypeList,
) -> Result<Option<&'a TypeDef>, Error> {
    let hash = utils::read_bits_aligned(reader, u32::BITS)? as u32;
    find_class_def(types, hash)
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
fn find_class_def(types: &TypeList, hash: u32) -> Result<Option<&TypeDef>, Error> {
    if hash == 0 {
        log::debug!("Received null hash for object");
        Ok(None)
    } else if let Some(t) = types.0.get(&hash) {
        log::debug!("Received object hash for '{}' ({hash})", t.name);
        Ok(Some(t))
    } else {
        Err(Error::UnknownType(hash))
    }
}
