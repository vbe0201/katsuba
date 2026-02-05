//! Implements KingsIsle's ObjectProperty serialization system.
//!
//! ObjectProperty is a reflection and serialization system for C++ classes.
//! Serialized object state can be found in various places of the networking
//! protocol or the game files.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

pub mod serde;

pub mod value;
pub use value::Value;
