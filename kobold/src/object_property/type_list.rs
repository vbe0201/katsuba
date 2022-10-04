use std::{collections::HashMap, io};

use bitflags::bitflags;
use serde::{Deserialize, Deserializer};

bitflags! {
    /// The configuration bits for [`Property`] values.
    #[derive(Deserialize)]
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
#[derive(Deserialize)]
pub struct TypeList {
    /// A mapping of type definitions.
    #[serde(flatten)]
    pub list: HashMap<String, TypeDef>,
}

impl TypeList {
    /// Deserializes a type list in JSON format from a given reader.
    pub fn read<R: io::Read>(reader: R) -> anyhow::Result<Self> {
        serde_json::from_reader(reader).map_err(Into::into)
    }
}

/// An individual type definition inside the list.
#[derive(Deserialize)]
pub struct TypeDef {
    /// The base classes of a type, if any.
    pub bases: Vec<String>,
    /// A hash of the type name.
    pub hash: u32,
    #[serde(deserialize_with = "deserialize_property_list")]
    pub properties: Vec<Property>,
}

/// A property that represents a member of a class.
#[derive(Deserialize)]
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

/// Hack to deal with some inconsistencies in how options are stored.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StringOrInt {
    String(String),
    Int(u32),
}

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
