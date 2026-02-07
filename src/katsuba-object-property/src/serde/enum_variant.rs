use bitter::LittleEndianReader;
use katsuba_types::Property;

use super::{Error, SerializerFlags, SerializerParts, utils};
use crate::Value;

pub fn deserialize(
    de: &SerializerParts,
    property: &Property,
    reader: &mut LittleEndianReader<'_>,
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
        let value = utils::read_bits(reader, u32::BITS)? as u32;
        Ok(Value::Enum(value as i64))
    }
}
