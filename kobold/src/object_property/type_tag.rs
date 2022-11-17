use anyhow::bail;

use super::{BitReader, TypeDef, TypeList};

/// A type tag that defines deserialization behavior to
/// identify object types.
pub trait TypeTag: Sized {
    /// Reads the object identity from the deserializer
    /// and returns a matching type def.
    fn object_identity(reader: &mut BitReader, types: &TypeList)
        -> anyhow::Result<Option<TypeDef>>;
}

/// A [`TypeTag`] that identifies regular PropertyClasses.
pub struct PropertyClass;

/// A [`TypeTag`] that identifies CoreObjects.
pub struct CoreObject;

impl TypeTag for PropertyClass {
    fn object_identity(
        reader: &mut BitReader,
        types: &TypeList,
    ) -> anyhow::Result<Option<TypeDef>> {
        let hash = reader.load_u32()?;
        find_property_class_def(types, hash)
    }
}

impl TypeTag for CoreObject {
    fn object_identity(
        reader: &mut BitReader,
        types: &TypeList,
    ) -> anyhow::Result<Option<TypeDef>> {
        let class_id = reader.load_u8()?;
        let namespace_id = reader.load_u8()?;
        let template_or_type = reader.load_u32()?;

        match (class_id, namespace_id) {
            // With a class ID of 0, this is just a regular PropertyClass.
            (0, 0) => find_property_class_def(types, template_or_type),

            // TODO: Add missing mappings.

            // class WizClientObjectItem
            (115, 9) if template_or_type == 175484 => Ok(types.classes.get(&1653772158).cloned()),
            // class WizClientObject
            (104, 2) if template_or_type == 1 => Ok(types.classes.get(&766500222).cloned()),
            // class ClientObject
            (5, 2) if template_or_type == 560 => Ok(types.classes.get(&350837933).cloned()),
            // class ClientPetSnackItem
            (148, 9) if template_or_type == 220190 => Ok(types.classes.get(&1748894102).cloned()),
            // class WizClientPet
            (106, 2) if template_or_type == 2 => Ok(types.classes.get(&1167581154).cloned()),
            // class WizClientObjectItem
            (115, 9) if template_or_type == 4 => Ok(types.classes.get(&1653772158).cloned()),
            // class ClientReagentItem
            (132, 9) if template_or_type == 106931 => Ok(types.classes.get(&398229815).cloned()),
            // class ClientObject
            (2, 2) if template_or_type == 600 => Ok(types.classes.get(&350837933).cloned()),
            // class ClientRecipe
            (131, 131) if template_or_type == 232026 => Ok(types.classes.get(&958775582).cloned()),
            // class WizClientMount
            (108, 2) if template_or_type == 3 => Ok(types.classes.get(&2109552587).cloned()),
            // class ClientObject
            (9, 9) if template_or_type == 1001 => Ok(types.classes.get(&350837933).cloned()),

            _ => bail!("received unknown core type {class_id} and namespace {namespace_id}"),
        }
    }
}

#[inline]
fn find_property_class_def(types: &TypeList, hash: u32) -> anyhow::Result<Option<TypeDef>> {
    if hash == 0 {
        Ok(None)
    } else if let Some(t) = types.classes.get(&hash) {
        Ok(Some(t.clone()))
    } else {
        bail!("Failed to identify type with hash {hash}");
    }
}
