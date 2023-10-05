use kobold_bit_buf::BitReader;
use kobold_types::{TypeDef, TypeList};
use kobold_utils::anyhow;

use super::utils;

/// A type tag which defines the encoding of an object
/// identity scheme.
pub trait TypeTag: Sized {
    /// Reads an object identity from the deserializer
    /// and returns a matching type definition.
    fn identity<'a>(
        reader: &mut BitReader<'_>,
        types: &'a TypeList,
    ) -> anyhow::Result<Option<&'a TypeDef>>;
}

/// A [`TypeTag`] that identifies regular PropertyClasses.
pub struct PropertyClass;

impl TypeTag for PropertyClass {
    fn identity<'a>(
        reader: &mut BitReader<'_>,
        types: &'a TypeList,
    ) -> anyhow::Result<Option<&'a TypeDef>> {
        let hash = utils::read_bits(reader, u32::BITS)? as u32;
        find_class_def(types, hash)
    }
}

#[inline]
fn find_class_def(types: &TypeList, hash: u32) -> anyhow::Result<Option<&TypeDef>> {
    if hash == 0 {
        log::debug!("Received null hash for object");
        Ok(None)
    } else if let Some(t) = types.0.get(&hash) {
        log::debug!("Received object hash for '{}' ({hash})", t.name);
        Ok(Some(t))
    } else {
        anyhow::bail!("failed to identify type with hash {hash}")
    }
}
