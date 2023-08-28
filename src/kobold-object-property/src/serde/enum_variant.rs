use kobold_bit_buf::BitReader;
use kobold_types::Property;

use super::{simple_data, DeserializerParts, SerializerFlags};
use crate::Value;

pub fn deserialize<D>(
    de: &DeserializerParts<D>,
    property: &Property,
    reader: &mut BitReader<'_>,
) -> anyhow::Result<Value> {
    if de
        .options
        .flags
        .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
    {
        let value = std::str::from_utf8(simple_data::deserialize_str(&de.options, reader))?;
        property.decode_enum_variant(value).map(Value::Enum)
    } else {
        let value = reader.u32();
        Ok(Value::Enum(value as i64))
    }
}
