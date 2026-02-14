use std::{fs, io::BufReader, path::PathBuf};

use eyre::Context;
use katsuba_object_property::{Value, value};
use katsuba_types::{Property, TypeList};

/// Reads all the given type list paths and merges them into a single
/// [`TypeList`] instance.
pub fn merge_type_lists(paths: Vec<PathBuf>) -> eyre::Result<TypeList> {
    let (first, rest) = paths
        .split_first()
        .ok_or_else(|| eyre::eyre!("at least one type list is required for deserialization"))?;

    let first = fs::File::open(first)
        .with_context(|| format!("failed to open type list at '{}'", first.display()))?;
    let mut list = TypeList::from_reader(BufReader::new(first))?;

    // Merge remaining type lists into `list`.
    for path in rest {
        let file = fs::File::open(path)
            .with_context(|| format!("failed to open type list at '{}'", path.display()))?;
        let next = TypeList::from_reader(BufReader::new(file))?;

        list.merge(next);
    }

    Ok(list)
}

/// Recursively converts all [`Value::Enum`] components in the provided
/// [`Value`] into strings.
///
/// This makes the CLI output easier to interpret for humans.
pub fn stringify_enums(value: &mut Value, types: &TypeList) {
    fn visit_object(obj: &mut value::Object, types: &TypeList) {
        let type_def = types.0.get(&obj.type_hash).unwrap();
        for property in type_def.properties.values() {
            if let Some(value) = obj.get_mut(&property.name) {
                visit_property(value, types, property);
            }
        }
    }

    fn visit_property(value: &mut Value, types: &TypeList, property: &Property) {
        match value {
            Value::Enum(v) => {
                if let Ok(v) = property.encode_enum_variant(*v) {
                    *value = Value::String(value::CxxStr(v.into_bytes()));
                } else {
                    log::warn!("Failed to stringify enum variant of {}: {v}", property.name);
                }
            }

            Value::List(list) => {
                for nested in list {
                    visit_property(nested, types, property);
                }
            }

            Value::Object(obj) => visit_object(obj, types),

            _ => (),
        }
    }

    if let Value::Object(obj) = value {
        visit_object(obj, types);
    }
}
