use std::{collections::HashMap, fmt, sync::Arc};

use serde::de::{Error, MapAccess, Visitor};

use super::{TypeDef, TypeList};

impl TypeDef {
    // Converts a deserialized v1 TypeDef into a v2 one.
    #[inline]
    fn into_v2(mut self, name: Arc<str>) -> (u32, Self) {
        self.name = name;
        (self.hash, self)
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
    type Value = TypeList;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("v1, v2 or v3 type list")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut list = TypeList::default();

        // Start by trying to extract the version entry of the format.
        if let Some(key) = map.next_key::<Arc<str>>()? {
            if &*key == "version" {
                self.version = map.next_value::<u32>()?;
            } else {
                list.classes.reserve(map.size_hint().unwrap_or(0));

                // This is a v1 type list.
                // We must not swallow the entry we just read.
                let (key, value) = TypeDef::into_v2(map.next_value()?, key);
                list.classes.insert(key, value);
            }
        }

        // Process remaining elements as dictated by the version.
        match self.version {
            // For a v1 list, continue eating entries and convert them into new format.
            1 => {
                while let Some((key, value)) = map.next_entry()? {
                    let (key, value) = TypeDef::into_v2(value, key);
                    list.classes.insert(key, value);
                }
            }

            // For v2 and v3 lists, we can deserialize the entries directly.
            2 | 3 => {
                let mut classes = None;
                let mut aliases = HashMap::<u32, u32>::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "classes" => classes = Some(map.next_value()?),
                        "aliases" if self.version >= 3 => aliases = map.next_value()?,
                        "core_object_factory_types" if self.version >= 3 => {
                            list.core_object_factory_types = map.next_value()?;
                        }
                        _ => {
                            return Err(A::Error::custom(format!(
                                "unexpected '{key}' entry for v{} list",
                                self.version
                            )));
                        }
                    }
                }
                list.classes = classes.ok_or_else(|| A::Error::missing_field("classes"))?;

                // Aliases deduplicate classes in v3 lists. For compatibility
                // with older versions, we resolve them into copies. Those of
                // classes from other lists will wait for a merge.
                list.pending_aliases = aliases;
                list.resolve_aliases();
            }

            // Reject any potentially newer version until proper support is added.
            _ => {
                return Err(A::Error::custom(format!(
                    "unknown version: {}",
                    self.version
                )));
            }
        }

        Ok(list)
    }
}
