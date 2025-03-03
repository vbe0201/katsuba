//! Serialization support for ObjectProperty values.

use std::{io, sync::Arc};

use bitflags::bitflags;
use katsuba_types::{PropertyFlags, TypeList};
use libdeflater::{DecompressionError, Decompressor};
use thiserror::Error;

mod de;

mod enum_variant;

#[cfg(feature = "option-guessing")]
mod guess;

mod object;

mod property;

mod simple_data;

mod type_tag;
pub use type_tag::*;

mod utils;

/// Magic header for persistent object state shipped with the client.
pub const BIND_MAGIC: &[u8] = b"BINd";

/// Errors that may occur during the ObjectProperty (de)serialization process.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occured while trying to read data from the input source.
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Failed to decompress a zlib object stream.
    #[error("{0}")]
    Decompress(#[from] DecompressionError),

    /// The deserialized object as a whole was a null value.
    #[error("root object must not be null")]
    NullRoot,

    /// The actual size of an inflated object after decompression does
    /// not match the size expectation for it.
    #[error("mismatch for inflated object size: expected {expected}, got {actual}")]
    DecompressedSizeMismatch { expected: usize, actual: usize },

    /// Attempted to construct a serializer from a bad configuration.
    #[error("bad serializer configuration: {0:?}")]
    BadConfig(&'static str),

    /// Configured recursion limit was exceeded during the process.
    #[error("recursion limit exceeded")]
    Recursion,

    /// Failed to decode an UTF-8 string where one was expected.
    #[error("{0}")]
    Decode(#[from] std::str::Utf8Error),

    /// Failed to encode or decode an encountered enum value.
    #[error("{0}")]
    Enum(#[from] katsuba_types::EncodingError),

    /// Failed to identify a type from its type tag during deserialization.
    #[error("failed to identify type with tag '{0}'")]
    UnknownType(u32),

    /// Object stream specifies a property that is not part of the object.
    #[error("unknown property for object with hash '{0}'")]
    UnknownProperty(u32),

    /// Encoded property size did not match the actually consumed data for it.
    #[error("mismatch for property size: expected {expected}, got {actual}")]
    PropertySizeMismatch { expected: usize, actual: usize },

    /// Encoded properties for an object consume more size than the object is
    /// specified to be.
    #[error("overflowed object size while consuming data")]
    ObjectSizeMismatch,

    /// When a delta-encoded property is missing from a stream which enforces
    /// its presence.
    #[error("missing delta value which must be present")]
    MissingDelta,
}

bitflags! {
    /// Configuration bits to customize serialization behavior.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SerializerFlags: u32 {
        /// A serializer configuration is part of the state
        /// and should be used upon deserializing.
        const STATEFUL_FLAGS = 1 << 0;
        /// Small length prefix values may be compressed into
        /// smaller integer types.
        const COMPACT_LENGTH_PREFIXES = 1 << 1;
        /// Whether enums are encoded as integer values or
        /// human-readable strings.
        const HUMAN_READABLE_ENUMS = 1 << 2;
        /// Whether the serialized state is zlib-compressed.
        const WITH_COMPRESSION = 1 << 3;
        /// Any property with the `DELTA_ENCODE` bit must always
        /// have its value serialized.
        const FORBID_DELTA_ENCODE = 1 << 4;
    }
}

/// Serializer configuration which influences how data is interpreted.
#[derive(Clone, Copy, Debug)]
pub struct SerializerOptions {
    /// The [`SerializerFlags`] to use.
    pub flags: SerializerFlags,
    /// A set of [`PropertyFlags`] for conditionally ignoring
    /// unmasked properties in a type.
    pub property_mask: PropertyFlags,
    /// Whether the shallow encoding strategy is used for
    /// the data.
    pub shallow: bool,
    /// Whether the data is manually compressed.
    pub manual_compression: bool,
    /// A recursion limit for nested data to avoid stack
    /// overflows during deserialization.
    ///
    /// Ignored during serialization.
    pub recursion_limit: i8,
    /// Skips unknown types during deserialization of properties.
    ///
    /// Ignored during serialization.
    pub skip_unknown_types: bool,
    /// Uses djb2 for all hashes.
    ///
    /// Used by Pirate101.
    pub djb2_only: bool,
}

impl Default for SerializerOptions {
    fn default() -> Self {
        Self {
            flags: SerializerFlags::empty(),
            property_mask: PropertyFlags::TRANSMIT | PropertyFlags::PRIVILEGED_TRANSMIT,
            shallow: true,
            manual_compression: false,
            recursion_limit: i8::MAX,
            skip_unknown_types: false,
            djb2_only: false,
        }
    }
}

pub(super) struct ZlibParts {
    inflater: Decompressor,

    // Most of the time, only one of these will be in use.
    scratch1: Vec<u8>,
    scratch2: Vec<u8>,
}

// SAFETY: No interior mutability. This should be in libdeflater?
unsafe impl Sync for ZlibParts {}

impl ZlibParts {
    pub fn new() -> Self {
        Self {
            inflater: Decompressor::new(),
            scratch1: Vec::new(),
            scratch2: Vec::new(),
        }
    }
}

/// The inner parts of the serializer state.
pub struct SerializerParts {
    /// The serializer configuration in use.
    pub options: SerializerOptions,
    pub(crate) types: Arc<TypeList>,
}

/// A serializer and deserializer for values in the ObjectProperty system.
pub struct Serializer {
    /// The raw serializer state.
    pub parts: SerializerParts,
    zlib_parts: ZlibParts,
}

impl SerializerParts {
    #[inline]
    pub(super) fn with_recursion_limit<F, T>(&mut self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Error>,
    {
        self.options.recursion_limit -= 1;
        if self.options.recursion_limit < 0 {
            return Err(Error::Recursion);
        }

        let res = f(self);

        self.options.recursion_limit += 1;

        res
    }
}
