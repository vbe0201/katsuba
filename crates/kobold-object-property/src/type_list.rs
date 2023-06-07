//! Representation of game type lists in JSON format.

use std::{borrow::Cow, collections::HashMap, io};

use anyhow::anyhow;
use bitflags::bitflags;
use serde::{Deserialize, Deserializer};

use super::hash;

bitflags! {
    /// The configuration bits for [`Property`] values.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
    #[serde(transparent)]
    #[repr(transparent)]
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

/// Representation of the list of types dumped from the game client.
#[derive(Clone, Debug, Deserialize)]
pub struct TypeList {
    /// The format version in use.
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

    /// Merges all entries from `other` into `self`.
    pub fn merge(&mut self, mut other: TypeList) {
        self.classes.reserve(other.classes.len());

        for (k, v) in other.classes.drain() {
            self.classes.insert(k, v);
        }
    }
}

/// An individual type definition inside the list.
#[derive(Clone, Debug, Deserialize)]
pub struct TypeDef {
    /// The type name.
    pub name: String,
    /// The properties of the class.
    #[serde(deserialize_with = "deserialize_property_list")]
    pub properties: Vec<Property>,
}

/// A property that represents a member of a class.
#[derive(Clone, Debug, Deserialize)]
pub struct Property {
    /// The name of the property.
    #[serde(skip)]
    pub name: String,
    /// The type of the property.
    pub r#type: String,
    /// The ID of the property.
    pub id: u32,
    /// The associated property flag mask.
    pub flags: PropertyFlags,
    /// Whether the property's storage is dynamically allocated.
    pub dynamic: bool,
    /// A combined hash of the property's name and of its type.
    pub hash: u32,
    /// A mapping of all enum options defined on a property.
    #[serde(default)]
    pub enum_options: HashMap<String, StringOrInt>,
}

impl Property {
    /// Gets the hash of this property's type.
    pub fn type_hash(&self) -> u32 {
        self.hash.wrapping_sub(hash::djb2(self.name.as_bytes()))
    }

    /// Decodes any given enum representation into a readable string,
    /// using the property's options.
    pub fn decode_enum_variant<'a>(
        &'a self,
        variant: &'a StringOrInt,
    ) -> anyhow::Result<Cow<'a, str>> {
        match (self.flags.contains(PropertyFlags::ENUM), variant) {
            // When the variant is already in string format, return as-is.
            (_, StringOrInt::String(value)) => Ok(Cow::Borrowed(value)),

            // When we're an enum but only got the integer value,
            // we look up the associated variant name and return it.
            (true, &StringOrInt::Int(value)) => {
                let variant = self
                    .enum_options
                    .iter()
                    .find(|(_, v)| v.compare_to_int(value))
                    .ok_or_else(|| anyhow!("unknown enum variant received: {value}"))?;

                Ok(Cow::Borrowed(variant.0))
            }

            // When we're a bitmask, walk through all the bits, look
            // up the names and build a new string that matches above
            // representation.
            (false, StringOrInt::Int(value)) => {
                let mut bits = String::new();

                for (name, bit) in self.enum_options.iter() {
                    if let StringOrInt::Int(bit) = bit {
                        if value & bit != 0 {
                            if !bits.is_empty() {
                                bits.push_str(" | ");
                            }

                            bits.push_str(name);
                        }
                    }
                }

                Ok(Cow::Owned(bits))
            }
        }
    }
}

/// A value that is either a string or an integer.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StringOrInt {
    /// A string value.
    String(String),
    /// An integer value.
    Int(i64),
}

impl StringOrInt {
    /// Compares a given `rhs` integer to the `self` value.
    ///
    /// If self is a string, `rhs` is expected to be the value
    /// matching the numeric string representation in self.
    pub fn compare_to_int(&self, rhs: i64) -> bool {
        match self {
            &StringOrInt::Int(v) => v == rhs,
            StringOrInt::String(s) => s.parse().map(|v: i64| v == rhs).unwrap_or(false),
        }
    }

    /// Compares a given `rhs` string to the `self` value.
    ///
    /// If self is an integer, `rhs` is expected to be the
    /// string representation of the same value.
    pub fn compare_to_string(&self, rhs: &str) -> bool {
        match self {
            StringOrInt::String(s) => s == rhs,
            &StringOrInt::Int(v) => rhs.parse().map(|rhs: i64| v == rhs).unwrap_or(false),
        }
    }
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
