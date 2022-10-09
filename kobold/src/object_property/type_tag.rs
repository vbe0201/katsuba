use anyhow::bail;

use super::{Deserializer, TypeDef, TypeList};

/// A type tag that defines deserialization behavior to
/// identify object types.
pub trait TypeTag: Sized {
    /// Reads the object identity from the deserializer
    /// and returns a matching type def.
    fn object_identity<'a, 'de>(
        de: &mut Deserializer<'de, Self>,
        types: &'a TypeList,
    ) -> anyhow::Result<Option<&'a TypeDef>>;
}

/// A [`TypeTag`] that identifies regular PropertyClasses.
pub struct PropertyClass;

impl TypeTag for PropertyClass {
    fn object_identity<'a, 'de>(
        de: &mut Deserializer<'de, Self>,
        types: &'a TypeList,
    ) -> anyhow::Result<Option<&'a TypeDef>> {
        let hash = de.deserialize_u32()?;
        if hash == 0 {
            Ok(None)
        } else if let Some(t) = types.list.get(&hash) {
            Ok(Some(t))
        } else {
            bail!("Failed to identify type with hash {hash}");
        }
    }
}
