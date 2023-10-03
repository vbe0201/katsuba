//! Serialization support for ObjectProperty values.

use std::sync::Arc;

use bitflags::bitflags;
use kobold_types::{PropertyFlags, TypeList};
use kobold_utils::{anyhow, libdeflater::Decompressor};

mod de;

mod enum_variant;

#[cfg(feature = "enable-option-guessing")]
mod guess;

mod object;

mod property;

mod simple_data;

mod type_tag;
pub use type_tag::*;

mod utils;

/// Magic header for persistent object state shipped with the client.
pub const BIND_MAGIC: &[u8] = b"BINd";

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
    pub recursion_limit: u8,
    /// Skips unknown types during deserialization of properties.
    ///
    /// Ignored during serialization.
    pub skip_unknown_types: bool,
}

impl Default for SerializerOptions {
    fn default() -> Self {
        Self {
            flags: SerializerFlags::empty(),
            property_mask: PropertyFlags::TRANSMIT | PropertyFlags::PRIVILEGED_TRANSMIT,
            shallow: true,
            manual_compression: false,
            recursion_limit: u8::MAX / 2,
            skip_unknown_types: false,
        }
    }
}

pub(super) struct ZlibParts {
    inflater: Decompressor,

    // Most of the time, only one of these will be in use.
    scratch1: Vec<u8>,
    scratch2: Vec<u8>,
}

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
    pub(super) fn with_recursion_limit<F, T>(&mut self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut Self) -> anyhow::Result<T>,
    {
        self.options.recursion_limit -= 1;
        anyhow::ensure!(self.options.recursion_limit > 0, "recursion limit exceeded");

        let res = f(self);

        self.options.recursion_limit += 1;

        res
    }
}
