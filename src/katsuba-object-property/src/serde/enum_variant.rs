use katsuba_bit_buf::BitReader;
use katsuba_types::Property;

use super::{utils, Error, SerializerFlags, SerializerParts};
use crate::Value;

pub fn deserialize(
    de: &SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> Result<Value, Error> {
    if de
        .options
        .flags
        .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
    {
        let raw = utils::read_string(reader, &de.options)?;
        let value = std::str::from_utf8(raw)?;
        property
            .decode_enum_variant(value)
            .map(Value::Enum)
            .map_err(Into::into)
    } else {
        let value = utils::read_bits(reader, u32::BITS)?;
        Ok(Value::Enum(value as i64))
    }
}
