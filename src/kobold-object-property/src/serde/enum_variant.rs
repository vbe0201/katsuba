use kobold_bit_buf::BitReader;
use kobold_types::Property;
use kobold_utils::anyhow;

use super::{utils, SerializerFlags, SerializerParts};
use crate::Value;

pub fn deserialize(
    de: &SerializerParts,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
    if de
        .options
        .flags
        .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
    {
        let value = std::str::from_utf8(utils::read_string(reader, &de.options)?)?;
        property.decode_enum_variant(value).map(Value::Enum)
    } else {
        let value = utils::read_bits(reader, u32::BITS)?;
        Ok(Value::Enum(value as i64))
    }
}
