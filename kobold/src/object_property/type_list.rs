use std::{collections::HashMap, io};

use anyhow::anyhow;
use bitflags::bitflags;
use serde::{Deserialize, Deserializer};

bitflags! {
    /// The configuration bits for [`Property`] values.
    #[derive(Deserialize)]
    #[serde(transparent)]
    pub struct PropertyFlags: u32 {
        const SAVE = 1 << 0;
        const COPY = 1 << 1;
        const PUBLIC = 1 << 2;
        const TRANSMIT = 1 << 3;
        const PRIVILEGED_TRANSMIT = 1 << 4;
        const PERSIST = 1 << 5;
        const DEPRECATED = 1 << 6;
        const NOSCRIPT = 1 << 7;
        const DELTA_ENCODE = 1 << 8;
        const BLOB = 1 << 9;

        const NOEDIT = 1 << 16;
        const FILENAME = 1 << 17;
        const COLOR = 1 << 18;
        const BITS = 1 << 20;
        const ENUM = 1 << 21;
        const LOCALIZED = 1 << 22;
        const STRING_KEY = 1 << 23;
        const OBJECT_ID = 1 << 24;
        const REFERENCE_ID = 1 << 25;
        const OBJECT_NAME = 1 << 27;
        const HAS_BASECLASS = 1 << 28;
    }
}

/// Representation of a type list for all the embedded
/// type information in the game client.
#[derive(Clone, Deserialize)]
pub struct TypeList {
    pub version: u32,
    /// A mapping of type definitions.
    pub classes: HashMap<u32, TypeDef>,
}

impl TypeList {
    /// Deserializes a type list in JSON format from a given reader.
    pub fn from_reader<R: io::Read>(reader: R) -> anyhow::Result<Self> {
        serde_json::from_reader(reader).map_err(Into::into)
    }

    /// Deserializes a type list in JSON format from a given string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(data: &str) -> anyhow::Result<Self> {
        serde_json::from_str(data).map_err(Into::into)
    }
}

/// An individual type definition inside the list.
#[derive(Clone, Deserialize)]
pub struct TypeDef {
    /// The base classes of a type, if any.
    pub bases: Vec<String>,
    /// The type name.
    #[serde(skip)]
    pub name: String,
    /// The hash of the type name.
    pub hash: u32,
    /// The properties of the class.
    #[serde(deserialize_with = "deserialize_property_list")]
    pub properties: Vec<Property>,
}

/// A property that represents a member of a class.
#[derive(Clone, Deserialize)]
pub struct Property {
    /// The name of the property.
    #[serde(skip)]
    pub name: String,
    /// The type of the property.
    pub r#type: String,
    /// The ID of the property.
    pub id: u32,
    /// The offset of the property into the class.
    pub offset: usize,
    /// The associated property flag mask.
    pub flags: PropertyFlags,
    /// The underlying container of the property.
    pub container: String,
    /// Whether the property's storage is dynamically allocated.
    pub dynamic: bool,
    /// Whether the property's type is a global singleton.
    pub singleton: bool, // FIXME: I'm at the wrong place.
    /// Whether the property is a pointer.
    pub pointer: bool,
    /// A combined hash of the property's name and of its type name.
    pub hash: u32,
    /// A mapping of all enum options defined on a property.
    #[serde(default)]
    pub enum_options: HashMap<String, StringOrInt>,
}

impl Property {
    /// Decodes any given enum representation into a readable
    /// string, using the property's options.
    pub fn decode_enum_variant(&self, variant: StringOrInt) -> anyhow::Result<String> {
        match (self.flags.contains(PropertyFlags::ENUM), variant) {
            // When the variant is already in bitflag list format,
            // we have no work to do and can just return the string.
            (false, StringOrInt::String(value)) => Ok(value),

            // When we're an enum and got the string representation
            // of the variant, we just want to prepend the correct
            // type prefix for easier lookup upon introspection.
            (true, StringOrInt::String(mut value)) => {
                value.insert_str(0, "::");
                value.insert_str(0, &self.r#type);

                Ok(value)
            }

            // When we're an enum but only got the integer value,
            // we look up the associated variant name and build
            // a similar string as above.
            (true, StringOrInt::Int(value)) => {
                let variant = self
                    .enum_options
                    .iter()
                    .find(|(_, v)| match v {
                        StringOrInt::Int(v) => *v == value,
                        StringOrInt::String(v) => {
                            v.parse::<u32>().map(|v| v == value).unwrap_or(false)
                        }
                    })
                    .ok_or_else(|| anyhow!("unknown enum variant received: {value}"))?;

                let mut value = variant.0.to_owned();
                value.insert_str(0, "::");
                value.insert_str(0, &self.r#type);

                Ok(value)
            }

            // And lastly, when we're a bitmask, we walk through
            // all the bits, look up the names and build a new
            // bitmask string that matches above representation.
            (false, StringOrInt::Int(value)) => {
                let mut bits = String::new();

                for b in 0..u32::BITS {
                    if !bits.is_empty() {
                        bits.push_str(" | ");
                    }

                    if value & 1 << b != 0 {
                        let variant = self
                            .enum_options
                            .iter()
                            .find(|(_, v)| match v {
                                StringOrInt::Int(v) => *v == value,
                                StringOrInt::String(v) => {
                                    v.parse::<u32>().map(|v| v == value).unwrap_or(false)
                                }
                            })
                            .ok_or_else(|| anyhow!("unknown enum variant received: {value}"))?;

                        bits.push_str(variant.0);
                    }
                }

                Ok(bits)
            }
        }
    }
}

/// Hack to deal with some inconsistencies in how options are stored.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StringOrInt {
    String(String),
    Int(u32),
}

// fn deserialize_type_list<'de, D>(deserializer: D) -> Result<HashMap<u32, TypeDef>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     Ok(<HashMap<String, TypeDef>>::deserialize(deserializer)?
//         .drain()
//         .map(|(name, mut t)| {
//             t.name = name;
//             (t.hash, t)
//         })
//         .collect())
// }

fn deserialize_property_list<'de, D>(deserializer: D) -> Result<Vec<Property>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut properties: Vec<_> = HashMap::<String, Property>::deserialize(deserializer)?
        .drain()
        .map(|(name, mut property)| {
            property.name = name;
            property
        })
        .collect();

    // Sort properties by ID for correct order.
    properties.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(properties)
}
