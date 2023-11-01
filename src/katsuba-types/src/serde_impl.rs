use std::{collections::HashMap, fmt};

use katsuba_utils::hash;
use serde::de::{Error, MapAccess, Visitor};
use smartstring::alias::String;

use super::TypeDef;

impl TypeDef {
    // Converts a deserialized v1 TypeDef into a v2 one.
    #[inline]
    fn into_v2(mut self, name: String) -> (u32, Self) {
        self.name = name;
        (hash::string_id(self.name.as_bytes()), self)
    }
}

/// A custom visitor for deserializing type mappings into a
/// uniform representation.
///
/// Currently, this implements v1 and v2 of the [wiztype] format.
///
/// [wiztype]: https://github.com/wizspoil/wiztype
pub struct TypeListVisitor {
    pub version: u32,
}

impl<'de> Visitor<'de> for TypeListVisitor {
    type Value = HashMap<u32, TypeDef>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("v1 or v2 type list")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut classes = HashMap::new();

        // Start by trying to extract the version entry of the format.
        if let Some(key) = map.next_key()? {
            if key == "version" {
                self.version = map.next_value::<u32>()?;
            } else {
                classes.reserve(map.size_hint().unwrap_or(0));

                // This is a v1 type list.
                // We must not swallow the entry we just read.
                let (key, value) = TypeDef::into_v2(map.next_value()?, key);
                classes.insert(key, value);
            }
        }

        // Process remaining elements as dictated by the version.
        if self.version == 1 {
            // For a v1 list, continue eating entries and convert them into new format.
            while let Some((key, value)) = map.next_entry()? {
                let (key, value) = TypeDef::into_v2(value, key);
                classes.insert(key, value);
            }
        } else if self.version == 2 {
            // For a v2 list, we can deserialize the entries directly.
            if let Some((key, value)) = map.next_entry::<String, _>()? {
                if key != "classes" {
                    return Err(A::Error::custom("expected 'classes' entry for v2 list"));
                }

                return Ok(value);
            }
        } else {
            // Reject any potentially newer version until proper support is added.
            return Err(A::Error::custom(format!(
                "unknown version: {}",
                self.version
            )));
        }

        Ok(classes)
    }
}
