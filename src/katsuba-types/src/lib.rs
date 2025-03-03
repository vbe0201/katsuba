//! Implements the [wiztype] format for game type dumps.
//!
//! A type list is represented through [`TypeList`] and is a mapping
//! of a type's name hash to its static reflection metadata.
//!
//! Therefore, this crate provides a way for Rust code to work with
//! these types in order to mimick runtime serialization behavior.
//!
//! # Version Support
//!
//! This crate generally tries to implement every format version a
//! recent release of wiztype offers to produce.
//!
//! [wiztype]: https://github.com/wizspoil/wiztype

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::{collections::HashMap, io};

use serde::{Deserialize, Deserializer};
use smartstring::alias::String;
use thiserror::Error;

mod property;
pub use property::*;

mod serde_impl;

mod string_or_int;
pub use string_or_int::*;

/// Errors that may occur when working with [`TypeList`]s.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occured while interacting with a type list file.
    #[error("{0}")]
    Io(#[from] io::Error),

    /// An error occurred during JSON deserialization.
    #[error("{0}")]
    Serde(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        use serde_json::error::Category;

        match value.classify() {
            Category::Io => Self::Io(value.into()),
            _ => Self::Serde(value),
        }
    }
}

/// Representation of the list of types dumped from the game client.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeList(pub HashMap<u32, TypeDef>);

impl TypeList {
    /// Deserializes a type list in JSON format from a given reader.
    pub fn from_reader<R: io::Read>(reader: R) -> Result<Self, Error> {
        serde_json::from_reader(reader).map_err(Into::into)
    }

    /// Deserializes a type list in JSON format from a given string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(data: &str) -> Result<Self, Error> {
        serde_json::from_str(data).map_err(Into::into)
    }

    /// Merges all entries from `other` into `self`.
    pub fn merge(&mut self, mut other: TypeList) {
        self.0.reserve(other.0.len());

        for (k, v) in other.0.drain() {
            self.0.insert(k, v);
        }
    }
}

impl<'de> Deserialize<'de> for TypeList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_map(serde_impl::TypeListVisitor { version: 1 })
            .map(Self)
    }
}

/// An individual type definition inside the list.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TypeDef {
    /// The type name.
    #[serde(default)]
    pub name: String,
    /// The properties of the class.
    #[serde(deserialize_with = "deserialize_property_list")]
    pub properties: Vec<Property>,
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
