use std::collections::HashMap;

use bitflags::bitflags;
use katsuba_utils::hash;
use serde::{de::Error, Deserialize, Deserializer};
use smartstring::alias::String;
use thiserror::Error;

use super::StringOrInt;

/// Errors that may occur when encoding or decoding enum values.
#[derive(Debug, PartialEq, Error)]
pub enum EncodingError {
    /// Failed to decode an enum variant's string representation.
    #[error("unknown enum variant: {0}")]
    Decode(std::string::String),

    /// Failed to encode an enum variant's integral representation.
    #[error("unknown enum value: {0}")]
    Encode(i64),
}

bitflags! {
    /// The configuration bits for [`Property`] values.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

        // below are editor hint flags
        const NOEDIT = 1 << 16;
        const FILENAME = 1 << 17;
        const COLOR = 1 << 18;
        const CONSTRAINED_VALUE = 1 << 19;
        const BITS = 1 << 20;
        const ENUM = 1 << 21;
        const LOCALIZED = 1 << 22;
        const STRING_KEY = 1 << 23;
        const OBJECT_ID = 1 << 24;
        const REFERENCE_ID = 1 << 25;
        const RADIANS = 1 << 26;
        const OBJECT_NAME = 1 << 27;
        const HAS_BASECLASS = 1 << 28;
        // class is behavior, not property type
        const IS_BEHAVIOR = 1 << 29;
        const ASSET = 1 << 30;

        #[doc(hidden)]
        const ENUM_LIKE = Self::ENUM.bits() | Self::BITS.bits();
    }
}

/// A property that represents a member of a class.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Property {
    /// The name of the property.
    #[serde(skip)]
    pub name: String,
    /// The type of the property.
    pub r#type: String,
    /// The ID of the property.
    pub id: u32,
    /// The associated property flag mask.
    #[serde(deserialize_with = "deserialize_property_flags")]
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

    /// Whether this property holds an enum value.
    pub fn is_enum(&self) -> bool {
        self.flags.intersects(PropertyFlags::ENUM_LIKE) || self.r#type.starts_with("enum")
    }

    /// Encodes an integral enum variant into a string representation
    /// of the value through the property's defined options.
    pub fn encode_enum_variant(&self, variant: i64) -> Result<String, EncodingError> {
        match self.flags.contains(PropertyFlags::BITS) {
            // Given a bitmask, check all available bits in enum_options
            // and build a string representation similar to KI's.
            true => {
                let mut res = String::new();

                for (name, value) in &self.enum_options {
                    if value.to_int().map(|v| variant & v != 0).unwrap_or(false) {
                        if !res.is_empty() {
                            res.push_str(" | ");
                        }

                        res.push_str(name);
                    }
                }

                Ok(res)
            }

            // Otherwise, we just find the name for the given value and
            // return that as-is.
            false => {
                for (name, value) in &self.enum_options {
                    if value.to_int().map(|v| v == variant).unwrap_or(false) {
                        return Ok(name.clone());
                    }
                }

                Err(EncodingError::Encode(variant))
            }
        }
    }

    /// Decodes a given enum variant from its string representation to
    /// an integer value through the property's defined options.
    pub fn decode_enum_variant(&self, variant: &str) -> Result<i64, EncodingError> {
        if self.flags.contains(PropertyFlags::BITS) {
            // For bitflags in string format, we convert them into
            // their integral representation.
            let mut res = 0;

            for bit in variant.split('|') {
                let bit = bit.trim();
                res |= self
                    .enum_options
                    .get(bit)
                    .and_then(|v| v.to_int())
                    .ok_or_else(|| EncodingError::Decode(bit.to_string()))?;
            }

            Ok(res)
        } else {
            // When we got an enum in string format, we look up the
            // corresponding value and return as-is.
            self.enum_options
                .get(variant)
                .and_then(|v| v.to_int())
                .ok_or_else(|| EncodingError::Decode(variant.to_string()))
        }
    }
}

fn deserialize_property_flags<'de, D>(deserializer: D) -> Result<PropertyFlags, D::Error>
where
    D: Deserializer<'de>,
{
    let flags = u32::deserialize(deserializer)?;
    PropertyFlags::from_bits(flags)
        .ok_or_else(|| D::Error::custom(format!("unknown property flags: {:?}", flags)))
}
