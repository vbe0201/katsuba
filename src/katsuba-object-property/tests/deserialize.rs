//! Deserialization tests for the ObjectProperty system.
//!
//! These tests use generated test fixtures to verify the deserializer
//! correctly handles various data shapes and edge cases.

use std::sync::Arc;

use katsuba_object_property::{
    Value,
    serde::{Error, Serializer, SerializerFlags, SerializerOptions},
};
use katsuba_types::{PropertyFlags, TypeList};

fn load_types() -> Arc<TypeList> {
    let data = include_str!("fixtures/types.json");
    Arc::new(TypeList::from_str(data).expect("failed to parse types.json"))
}

fn parse_config(json: &str) -> SerializerOptions {
    #[derive(serde::Deserialize)]
    struct Config {
        shallow: bool,
        compress: bool,
        flags: u32,
        property_mask: u32,
    }

    let config: Config = serde_json::from_str(json).expect("failed to parse config");

    SerializerOptions {
        flags: SerializerFlags::from_bits_truncate(config.flags),
        property_mask: PropertyFlags::from_bits_truncate(config.property_mask),
        shallow: config.shallow,
        manual_compression: config.compress,
        recursion_limit: i8::MAX,
        skip_unknown_types: false,
        djb2_only: false,
    }
}

macro_rules! test_deserialize {
    ($name:ident, $bin:literal, $config:literal) => {
        #[test]
        fn $name() {
            let types = load_types();
            let options = parse_config(include_str!($config));
            let data = include_bytes!($bin);

            let mut serializer =
                Serializer::new(options, types).expect("failed to create serializer");
            let result = serializer.deserialize(data);

            assert!(result.is_ok(), "deserialization failed: {:?}", result.err());
            let value = result.unwrap();
            assert!(
                !matches!(value, Value::Empty),
                "deserialized value should not be empty"
            );
        }
    };
}

macro_rules! test_should_fail {
    ($name:ident, $bin:literal, $config:literal, $error_pat:pat) => {
        #[test]
        fn $name() {
            let types = load_types();
            let options = parse_config(include_str!($config));
            let data = include_bytes!($bin);

            let mut serializer =
                Serializer::new(options, types).expect("failed to create serializer");
            let result = serializer.deserialize(data);

            assert!(result.is_err(), "deserialization should have failed");
            assert!(
                matches!(result.as_ref().err().unwrap(), $error_pat),
                "unexpected error type: {:?}",
                result.err()
            );
        }
    };
}

// === Scalar Type Tests ===

test_deserialize!(
    all_scalars_shallow,
    "fixtures/all-scalars-shallow.bin",
    "fixtures/all-scalars-shallow.config.json"
);

test_deserialize!(
    all_scalars_deep,
    "fixtures/all-scalars-deep.bin",
    "fixtures/all-scalars-deep.config.json"
);

// === Bit Integer Tests ===

test_deserialize!(
    bit_integers_shallow,
    "fixtures/bit-integers-shallow.bin",
    "fixtures/bit-integers-shallow.config.json"
);

test_deserialize!(
    bit_integers_deep,
    "fixtures/bit-integers-deep.bin",
    "fixtures/bit-integers-deep.config.json"
);

// === String Tests ===

test_deserialize!(
    strings_shallow,
    "fixtures/strings-shallow.bin",
    "fixtures/strings-shallow.config.json"
);

test_deserialize!(
    strings_deep,
    "fixtures/strings-deep.bin",
    "fixtures/strings-deep.config.json"
);

test_deserialize!(
    strings_compact,
    "fixtures/strings-compact.bin",
    "fixtures/strings-compact.config.json"
);

test_deserialize!(
    property_mask,
    "fixtures/property-mask.bin",
    "fixtures/property-mask.config.json"
);

test_deserialize!(
    deprecated_shallow,
    "fixtures/deprecated-shallow.bin",
    "fixtures/deprecated-shallow.config.json"
);

test_deserialize!(
    deprecated_deep,
    "fixtures/deprecated-deep.bin",
    "fixtures/deprecated-deep.config.json"
);

test_deserialize!(
    delta_encode_present,
    "fixtures/delta-encode-present.bin",
    "fixtures/delta-encode-present.config.json"
);

test_deserialize!(
    delta_encode_absent,
    "fixtures/delta-encode-absent.bin",
    "fixtures/delta-encode-absent.config.json"
);

test_deserialize!(
    scoped_enum_int,
    "fixtures/scoped-enum-int.bin",
    "fixtures/scoped-enum-int.config.json"
);

test_deserialize!(
    scoped_enum_string,
    "fixtures/scoped-enum-string.bin",
    "fixtures/scoped-enum-string.config.json"
);

test_deserialize!(
    bitflags_int,
    "fixtures/bitflags-int.bin",
    "fixtures/bitflags-int.config.json"
);

test_deserialize!(
    bitflags_combined,
    "fixtures/bitflags-combined.bin",
    "fixtures/bitflags-combined.config.json"
);

test_deserialize!(
    bitflags_string,
    "fixtures/bitflags-string.bin",
    "fixtures/bitflags-string.config.json"
);

test_deserialize!(
    bitflags_empty_string,
    "fixtures/bitflags-empty-string.bin",
    "fixtures/bitflags-empty-string.config.json"
);

test_deserialize!(
    nested_object,
    "fixtures/nested-object.bin",
    "fixtures/nested-object.config.json"
);

test_deserialize!(
    list_simple,
    "fixtures/list-simple.bin",
    "fixtures/list-simple.config.json"
);

test_deserialize!(
    list_compact,
    "fixtures/list-compact.bin",
    "fixtures/list-compact.config.json"
);

test_deserialize!(
    nested_lists,
    "fixtures/nested-lists.bin",
    "fixtures/nested-lists.config.json"
);

test_deserialize!(
    ends_with_bits_shallow,
    "fixtures/ends-with-bits-shallow.bin",
    "fixtures/ends-with-bits-shallow.config.json"
);

test_deserialize!(
    ends_with_bits_deep,
    "fixtures/ends-with-bits-deep.bin",
    "fixtures/ends-with-bits-deep.config.json"
);

test_deserialize!(
    binary_string,
    "fixtures/binary-string.bin",
    "fixtures/binary-string.config.json"
);

test_deserialize!(
    with_compression,
    "fixtures/with-compression.bin",
    "fixtures/with-compression.config.json"
);

test_deserialize!(
    empty_string_no_realign,
    "fixtures/empty-string-no-realign.bin",
    "fixtures/empty-string-no-realign.config.json"
);

test_deserialize!(
    deep_size_boundary,
    "fixtures/deep-size-boundary.bin",
    "fixtures/deep-size-boundary.config.json"
);

test_should_fail!(
    null_root_should_fail,
    "fixtures/should-fail/null-root.bin",
    "fixtures/should-fail/null-root.config.json",
    Error::NullRoot
);

test_should_fail!(
    delta_encode_forbid_absent,
    "fixtures/should-fail/delta-encode-forbid-absent.bin",
    "fixtures/should-fail/delta-encode-forbid-absent.config.json",
    Error::MissingDelta
);
