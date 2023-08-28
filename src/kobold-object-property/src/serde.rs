//! Serialization support for ObjectProperty values.

use bitflags::bitflags;
use kobold_types::PropertyFlags;

mod de;
pub use de::*;

mod enum_variant;

mod diagnostic;
pub use diagnostic::*;

mod object;

mod property;

mod ser;
pub use ser::*;

mod simple_data;

mod type_tag;
pub use type_tag::*;

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
