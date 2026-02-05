use bitter::LittleEndianReader;
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
