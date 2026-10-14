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
        #[serde(default)]
        recursion_limit: Option<i8>,
    }

    let config: Config = serde_json::from_str(json).expect("failed to parse config");

    SerializerOptions {
        flags: SerializerFlags::from_bits_truncate(config.flags),
        property_mask: PropertyFlags::from_bits_truncate(config.property_mask),
        shallow: config.shallow,
        manual_compression: config.compress,
        recursion_limit: config.recursion_limit.unwrap_or(i8::MAX),
        skip_unknown_types: false,
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

            assert_roundtrips(&mut serializer, &value);
        }
    };
}

fn assert_roundtrips(serializer: &mut Serializer, value: &Value) {
    let mut buf = Vec::new();
    serializer
        .serialize_into(value, &mut buf)
        .unwrap_or_else(|e| panic!("serialization failed: {e:?}"));

    let roundtripped = serializer
        .deserialize(&buf)
        .unwrap_or_else(|e| panic!("re-deserialization failed: {e:?}"));

    assert_eq!(*value, roundtripped, "value did not round-trip");
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
    Error::MissingDelta(..)
);

#[test]
fn strings_compact_aligned() {
    let types = load_types();
    let options = parse_config(include_str!("fixtures/strings-compact-aligned.config.json"));
    let data = include_bytes!("fixtures/strings-compact-aligned.bin");

    let mut serializer = Serializer::new(options, types).expect("failed to create serializer");
    let value = serializer
        .deserialize(data)
        .expect("deserialization failed");
    assert_roundtrips(&mut serializer, &value);

    let Value::Object(obj) = value else {
        panic!("expected an object, got {value:?}");
    };
    assert_eq!(obj.get("m_flag"), Some(&Value::Bool(true)));

    let Some(Value::String(s)) = obj.get("m_string") else {
        panic!("expected m_string to be a string");
    };
    assert_eq!(s.0, b"Short");

    let Some(Value::WString(s)) = obj.get("m_wstring") else {
        panic!("expected m_wstring to be a wide string");
    };
    assert_eq!(s.0, vec!['A' as u16; 200]);
}

#[test]
fn list_compact_aligned() {
    let types = load_types();
    let options = parse_config(include_str!("fixtures/list-compact-aligned.config.json"));
    let data = include_bytes!("fixtures/list-compact-aligned.bin");

    let mut serializer = Serializer::new(options, types).expect("failed to create serializer");
    let value = serializer
        .deserialize(data)
        .expect("deserialization failed");
    assert_roundtrips(&mut serializer, &value);

    let Value::Object(obj) = value else {
        panic!("expected an object, got {value:?}");
    };
    assert_eq!(obj.get("m_flag"), Some(&Value::Bool(true)));

    let Some(Value::List(list)) = obj.get("m_values") else {
        panic!("expected m_values to be a list");
    };
    let expected: Vec<Value> = (0..10).map(Value::Unsigned).collect();
    assert_eq!(list.inner, expected);
    assert_eq!(obj.get("m_count"), Some(&Value::Unsigned(10)));
}

#[test]
fn nested_recursive() {
    let types = load_types();
    let options = parse_config(include_str!("fixtures/nested-recursive.config.json"));
    let data = include_bytes!("fixtures/nested-recursive.bin");

    let mut serializer = Serializer::new(options, types).expect("failed to create serializer");
    let value = serializer
        .deserialize(data)
        .expect("deserialization failed");
    assert_roundtrips(&mut serializer, &value);

    fn as_object(v: &Value) -> &katsuba_object_property::value::Object {
        match v {
            Value::Object(obj) => obj,
            other => panic!("expected an object, got {other:?}"),
        }
    }

    fn value_of(v: &Value) -> u64 {
        match as_object(v).get("m_value") {
            Some(Value::Unsigned(v)) => *v,
            other => panic!("expected m_value to be an unsigned int, got {other:?}"),
        }
    }

    fn children_of(v: &Value) -> &[Value] {
        match as_object(v).get("m_children") {
            Some(Value::List(list)) => &list.inner,
            other => panic!("expected m_children to be a list, got {other:?}"),
        }
    }

    assert_eq!(value_of(&value), 1);
    let children = children_of(&value);
    assert_eq!(children.len(), 1);

    assert_eq!(value_of(&children[0]), 2);
    let grandchildren = children_of(&children[0]);
    assert_eq!(grandchildren.len(), 1);

    assert_eq!(value_of(&grandchildren[0]), 3);
    assert!(children_of(&grandchildren[0]).is_empty());
}

#[test]
fn enum_alignment_shallow() {
    let types = load_types();
    let options = parse_config(include_str!("fixtures/enum-alignment-shallow.config.json"));
    let data = include_bytes!("fixtures/enum-alignment-shallow.bin");

    let mut serializer = Serializer::new(options, types).expect("failed to create serializer");
    let value = serializer
        .deserialize(data)
        .expect("deserialization failed");
    assert_roundtrips(&mut serializer, &value);

    let Value::Object(obj) = value else {
        panic!("expected an object, got {value:?}");
    };
    assert_eq!(obj.get("m_flag"), Some(&Value::Bool(true)));
    assert_eq!(obj.get("m_enum"), Some(&Value::Enum(3)));
}

test_should_fail!(
    recursion_limit_exceeded,
    "fixtures/should-fail/recursion-limit-exceeded.bin",
    "fixtures/should-fail/recursion-limit-exceeded.config.json",
    Error::Recursion
);

test_should_fail!(
    huge_container,
    "fixtures/should-fail/huge-container.bin",
    "fixtures/should-fail/huge-container.config.json",
    Error::ContainerTooLarge { .. }
);
