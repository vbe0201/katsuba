use super::{Deserializer, SerializerFlags};
use crate::object_property::{Property, StringOrInt, TypeTag, Value};

/// Deserializes enum variants either from string or int representation.
///
/// A configuration bit on the serializer chooses the exact behavior.
pub struct EnumVariantDeserializer<'de, T> {
    pub(crate) de: &'de mut Deserializer<T>,
}

impl<'de, T: TypeTag> EnumVariantDeserializer<'de, T> {
    /// Deserializes an enum variant given a property that references it.
    pub fn deserialize(&mut self, property: &Property) -> anyhow::Result<Value> {
        let value = if self
            .de
            .options
            .flags
            .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
        {
            let value = String::from_utf8(self.de.deserialize_str()?)?;
            StringOrInt::String(value)
        } else {
            let value = self.de.deserialize_u32()?;
            StringOrInt::Int(value as i64)
        };

        property.decode_enum_variant(value).map(Value::Enum)
    }
}
