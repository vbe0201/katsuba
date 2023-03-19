use std::{
    io::{self, Write},
    marker::PhantomData,
    sync::Arc,
};

use anyhow::{anyhow, bail};
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use flate2::write::ZlibDecoder;
#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::{reader::BitReader, type_list::*, HashMap, List, Object, TypeTag, Value};

#[inline]
fn extract_type_argument(ty: &str) -> Option<&str> {
    let generic = ty.split_once('<')?.1;
    let generic = generic.rsplit_once('>')?.0;

    Some(generic)
}

#[inline]
fn zlib_decompress(data: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(Vec::with_capacity(expected_size));
    decoder.write_all(data)?;
    decoder.finish()
}

bitflags! {
    /// Configuration bits to customize serialization
    /// behavior.
    pub struct SerializerFlags: u32 {
        /// A serializer configuration is part of the state
        /// and should be used upon deserializing.
        const STATEFUL_FLAGS = 1 << 0;
        /// Small length prefix values may be compressed
        /// into smaller integer types.
        const COMPACT_LENGTH_PREFIXES = 1 << 1;
        /// Whether enums are encoded as integer values
        /// or human-readable strings.
        const HUMAN_READABLE_ENUMS = 1 << 2;
        /// Whether the serialized state is zlib-compressed.
        const WITH_COMPRESSION = 1 << 3;
        /// Any property with the `DELTA_ENCODE` bit must
        /// always have its value serialized.
        const FORBID_DELTA_ENCODE = 1 << 4;
    }
}

/// Configuration for the [`Deserializer`].
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "python", pyclass)]
pub struct DeserializerOptions {
    /// The [`SerializerFlags`] to use.
    pub flags: SerializerFlags,
    /// A set of [`PropertyFlags`] for conditionally ignoring
    /// unmasked properties of a type.
    pub property_mask: PropertyFlags,
    /// Whether the shallow encoding strategy is used for
    /// the data.
    pub shallow: bool,
    /// Whether the data is manually compressed.
    pub manual_compression: bool,
    /// A recursion limit for nested data to avoid stack
    /// overflows.
    pub recursion_limit: u8,
}

#[cfg(feature = "python")]
#[pymethods]
impl DeserializerOptions {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[getter]
    pub fn get_flags(&self) -> PyResult<u32> {
        Ok(self.flags.bits())
    }

    #[setter]
    pub fn set_flags(&mut self, new: u32) -> PyResult<()> {
        self.flags = SerializerFlags::from_bits_truncate(new);
        Ok(())
    }

    #[getter]
    pub fn get_property_mask(&self) -> PyResult<u32> {
        Ok(self.property_mask.bits())
    }

    #[setter]
    pub fn set_property_mask(&mut self, new: u32) -> PyResult<()> {
        self.property_mask = PropertyFlags::from_bits_truncate(new);
        Ok(())
    }

    #[getter]
    pub fn get_shallow(&self) -> PyResult<bool> {
        Ok(self.shallow)
    }

    #[setter]
    pub fn set_shallow(&mut self, new: bool) -> PyResult<()> {
        self.shallow = new;
        Ok(())
    }

    #[getter]
    pub fn get_manual_compression(&self) -> PyResult<bool> {
        Ok(self.manual_compression)
    }

    #[setter]
    pub fn set_manual_compression(&mut self, new: bool) -> PyResult<()> {
        self.manual_compression = new;
        Ok(())
    }

    #[getter]
    pub fn get_recursion_limit(&self) -> PyResult<u8> {
        Ok(self.recursion_limit)
    }

    #[setter]
    pub fn set_recursion_limit(&mut self, new: u8) -> PyResult<()> {
        self.recursion_limit = new;
        Ok(())
    }
}

impl Default for DeserializerOptions {
    fn default() -> Self {
        Self {
            flags: SerializerFlags::empty(),
            property_mask: PropertyFlags::TRANSMIT | PropertyFlags::PRIVILEGED_TRANSMIT,
            shallow: true,
            manual_compression: false,
            recursion_limit: u8::MAX / 2,
        }
    }
}

/// A configurable deserializer for the ObjectProperty binary
/// format, producing [`Value`]s.
pub struct Deserializer<T> {
    reader: BitReader,
    options: DeserializerOptions,
    pub(crate) types: Arc<TypeList>,
    _t: PhantomData<T>,
}

macro_rules! impl_read_len {
    ($($de:ident() = $read:ident()),* $(,)*) => {
        $(
            #[inline]
            fn $de(&mut self) -> anyhow::Result<usize> {
                self.reader.realign_to_byte();
                if self
                    .options
                    .flags
                    .contains(SerializerFlags::COMPACT_LENGTH_PREFIXES)
                {
                    self.read_compact_length_prefix()
                } else {
                    self.reader.$read().map(|v| v as usize).map_err(Into::into)
                }
            }
        )*
    };
}

impl<T> Deserializer<T> {
    /// Creates a new deserializer with its configuration.
    ///
    /// No data for deserialization has been loaded at this
    /// point. [`Deserializer::feed_data`] should be called
    /// next.
    pub fn new(options: DeserializerOptions, types: Arc<TypeList>) -> Self {
        Self {
            reader: BitReader::dangling(),
            types,
            options,
            _t: PhantomData,
        }
    }

    fn decompress_data(mut data: &[u8]) -> anyhow::Result<BitReader> {
        let size = data.read_u32::<LE>()? as usize;
        let decompressed = zlib_decompress(data, size)?;

        // Assert correct size expectations.
        if decompressed.len() != size {
            bail!(
                "Compression size mismatch - expected {} bytes, got {}",
                decompressed.len(),
                size
            );
        }

        Ok(BitReader::new(decompressed))
    }

    fn configure(&mut self, mut data: &[u8]) -> anyhow::Result<()> {
        let reader = if self.options.manual_compression {
            let mut reader = Self::decompress_data(data)?;

            // If configuration flags are stateful, deserialize them.
            if self.options.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
                self.options.flags = SerializerFlags::from_bits_truncate(reader.load_u32()?);
            }

            reader
        } else {
            // If configuration flags are stateful, deserialize them.
            if self.options.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
                self.options.flags = SerializerFlags::from_bits_truncate(data.read_u32::<LE>()?);
            }

            // Determine whether the data is compressed or not.
            if self
                .options
                .flags
                .contains(SerializerFlags::WITH_COMPRESSION)
                && data.read_u8()? != 0
            {
                Self::decompress_data(data)?
            } else {
                BitReader::new(data.to_owned())
            }
        };

        self.reader = reader;
        Ok(())
    }

    fn read_compact_length_prefix(&mut self) -> anyhow::Result<usize> {
        let is_large = self.reader.read_bit()?;
        if is_large {
            self.reader
                .read_value_bits(u32::BITS as usize - 1)
                .map_err(Into::into)
        } else {
            self.reader
                .read_value_bits(u8::BITS as usize - 1)
                .map_err(Into::into)
        }
    }

    impl_read_len! {
        // Used for strings, where the length is written as a `u16`.
        read_str_len() = load_u16(),

        // Used for sequences, where the length is written as a `u32`.
        read_seq_len() = load_u32(),
    }

    fn read_str(&mut self) -> anyhow::Result<Vec<u8>> {
        self.read_str_len()
            .and_then(|len| self.reader.read_bytes(len).map_err(Into::into))
    }

    fn read_wstr(&mut self) -> anyhow::Result<Vec<u16>> {
        let len = self.read_str_len()?;

        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.reader.load_u16()?);
        }

        Ok(result)
    }

    fn deserialize_bits(&mut self, n: usize) -> anyhow::Result<u64> {
        self.reader
            .read_value_bits(n)
            .map(|v| v as u64)
            .map_err(Into::into)
    }

    fn deserialize_signed_bits(&mut self, n: usize) -> anyhow::Result<i64> {
        self.deserialize_bits(n).map(|v| {
            // Perform sign-extension of the value we got.
            if v & (1 << (n - 1)) != 0 {
                (v as i64) | ((!0) << n)
            } else {
                v as i64
            }
        })
    }
}

macro_rules! check_recursion {
    (let $new_this:ident = $this:ident $($body:tt)*) => {
        $this.options.recursion_limit -= 1;
        if $this.options.recursion_limit == 0 {
            bail!("deserializer recursion limit exceeded");
        }

        let $new_this = $this $($body)*

        $new_this.options.recursion_limit += 1;
    };
}

macro_rules! impl_deserialize {
    ($($de:ident($ty:ty) = $read:ident()),* $(,)*) => {
        $(
            pub(crate) fn $de(&mut self) -> anyhow::Result<$ty> {
                self.reader.$read().map_err(Into::into)
            }
        )*
    };
}

impl<T: TypeTag> Deserializer<T> {
    /// Deserializes an object [`Value`] from previously
    /// loaded data.
    pub fn deserialize(&mut self, data: &[u8]) -> anyhow::Result<Value> {
        self.configure(data)?;
        self.deserialize_impl()
    }

    fn deserialize_impl(&mut self) -> anyhow::Result<Value> {
        check_recursion! {
            let this = self;

            let res = if let Some(type_def) = T::object_identity(&mut this.reader, &this.types)? {
                let object_size = (!this.options.shallow).then(|| this.reader.load_u32()).unwrap_or(Ok(0))?;
                let object = this.deserialize_properties(object_size as usize - u32::BITS as usize, &type_def)?;

                Value::Object(Object { name: type_def.name.to_owned(), inner: object })
            } else {
                Value::Empty
            };
        }

        Ok(res)
    }

    pub(crate) fn deserialize_bool(&mut self) -> anyhow::Result<bool> {
        self.reader.read_bit().map_err(Into::into)
    }

    impl_deserialize! {
        deserialize_u8(u8)   = load_u8(),
        deserialize_u16(u16) = load_u16(),
        deserialize_u32(u32) = load_u32(),
        deserialize_u64(u64) = load_u64(),

        deserialize_i8(i8)   = load_i8(),
        deserialize_i16(i16) = load_i16(),
        deserialize_i32(i32) = load_i32(),

        deserialize_f32(f32) = load_f32(),
        deserialize_f64(f64) = load_f64(),
    }

    fn deserialize_list(&mut self, property: &Property) -> anyhow::Result<Value> {
        let len = self.read_seq_len()?;
        let mut list = Vec::with_capacity(len);

        check_recursion! {
            let this = self;

            for _ in 0..len {
                list.push(this.deserialize_data(property)?);
            }
        }

        Ok(Value::List(List { inner: list }))
    }

    fn deserialize_simple_data(&mut self, ty: &str) -> anyhow::Result<Value> {
        match ty {
            // Primitive C++ types.
            "bool" => self.deserialize_bool().map(Value::Bool),
            "char" => self.deserialize_i8().map(|v| Value::Signed(v as _)),
            "unsigned char" => self.deserialize_u8().map(|v| Value::Unsigned(v as _)),
            "short" => self.deserialize_i16().map(|v| Value::Signed(v as _)),
            "unsigned short" | "wchar_t" => self.deserialize_u16().map(|v| Value::Unsigned(v as _)),
            "int" | "long" => self.deserialize_i32().map(|v| Value::Signed(v as _)),
            "unsigned int" | "unsigned long" => {
                self.deserialize_u32().map(|v| Value::Unsigned(v as _))
            }
            "float" => self.deserialize_f32().map(|v| Value::Float(v as _)),
            "double" => self.deserialize_f64().map(|v| Value::Float(v as _)),
            "unsigned __int64" | "gid" | "union gid" => self.deserialize_u64().map(Value::Unsigned),

            // Bit integers
            "bi2" => self.deserialize_signed_bits(2).map(Value::Signed),
            "bui2" => self.deserialize_bits(2).map(Value::Unsigned),
            "bi3" => self.deserialize_signed_bits(3).map(Value::Signed),
            "bui3" => self.deserialize_bits(3).map(Value::Unsigned),
            "bi4" => self.deserialize_signed_bits(4).map(Value::Signed),
            "bui4" => self.deserialize_bits(4).map(Value::Unsigned),
            "bi5" => self.deserialize_signed_bits(5).map(Value::Signed),
            "bui5" => self.deserialize_bits(5).map(Value::Unsigned),
            "bi6" => self.deserialize_signed_bits(6).map(Value::Signed),
            "bui6" => self.deserialize_bits(6).map(Value::Unsigned),
            "bi7" => self.deserialize_signed_bits(7).map(Value::Signed),
            "bui7" => self.deserialize_bits(7).map(Value::Unsigned),

            "s24" => self.deserialize_signed_bits(24).map(Value::Signed),
            "u24" => self.deserialize_bits(24).map(Value::Unsigned),

            // Strings
            "std::string" | "char*" => self.read_str().map(Value::String),
            "std::wstring" | "wchar_t*" => self.read_wstr().map(Value::WString),

            // Miscellaneous leaf types that are not PropertyClasses.
            "class Color" => Ok(Value::Color {
                b: self.deserialize_u8()?,
                g: self.deserialize_u8()?,
                r: self.deserialize_u8()?,
                a: self.deserialize_u8()?,
            }),
            "class Vector3D" => Ok(Value::Vec3 {
                x: self.deserialize_f32()?,
                y: self.deserialize_f32()?,
                z: self.deserialize_f32()?,
            }),
            "class Quaternion" => Ok(Value::Quat {
                x: self.deserialize_f32()?,
                y: self.deserialize_f32()?,
                z: self.deserialize_f32()?,
                w: self.deserialize_f32()?,
            }),
            "class Euler" => Ok(Value::Euler {
                pitch: self.deserialize_f32()?,
                roll: self.deserialize_f32()?,
                yaw: self.deserialize_f32()?,
            }),
            "class Matrix3x3" => Ok(Value::Mat3x3 {
                i: [
                    self.deserialize_f32()?,
                    self.deserialize_f32()?,
                    self.deserialize_f32()?,
                ],
                j: [
                    self.deserialize_f32()?,
                    self.deserialize_f32()?,
                    self.deserialize_f32()?,
                ],
                k: [
                    self.deserialize_f32()?,
                    self.deserialize_f32()?,
                    self.deserialize_f32()?,
                ],
            }),
            s if s.starts_with("class Size") => {
                let ty_arg = extract_type_argument(s).unwrap();
                Ok(Value::Size {
                    wh: Box::new((
                        self.deserialize_simple_data(ty_arg)?,
                        self.deserialize_simple_data(ty_arg)?,
                    )),
                })
            }
            s if s.starts_with("class Point") => {
                let ty_arg = extract_type_argument(s).unwrap();
                Ok(Value::Point {
                    xy: Box::new((
                        self.deserialize_simple_data(ty_arg)?,
                        self.deserialize_simple_data(ty_arg)?,
                    )),
                })
            }
            s if s.starts_with("class Rect") => {
                let ty_arg = extract_type_argument(s).unwrap();
                Ok(Value::Rect {
                    inner: Box::new((
                        self.deserialize_simple_data(ty_arg)?,
                        self.deserialize_simple_data(ty_arg)?,
                        self.deserialize_simple_data(ty_arg)?,
                        self.deserialize_simple_data(ty_arg)?,
                    )),
                })
            }

            _ => bail!("'{ty}' does not represent simple data"),
        }
    }

    fn deserialize_enum_variant(&mut self, property: &Property) -> anyhow::Result<Value> {
        let value = if self
            .options
            .flags
            .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
        {
            let value = String::from_utf8(self.read_str()?)?;
            StringOrInt::String(value)
        } else {
            let value = self.deserialize_u32()?;
            StringOrInt::Int(value as i64)
        };

        property.decode_enum_variant(value).map(Value::Enum)
    }

    fn deserialize_data(&mut self, property: &Property) -> anyhow::Result<Value> {
        if property
            .flags
            .intersects(PropertyFlags::BITS | PropertyFlags::ENUM)
        {
            self.deserialize_enum_variant(property)
        } else {
            // Try to interpret the value as simple data and if that
            // fails, deserialize a new object as a fallback strategy.
            self.deserialize_simple_data(&property.r#type)
                .or_else(|_| self.deserialize_impl())
        }
    }

    fn deserialize_properties(
        &mut self,
        mut object_size: usize,
        type_def: &TypeDef,
    ) -> anyhow::Result<HashMap<String, Value>> {
        let mut object = HashMap::default();

        if self.options.shallow {
            // In shallow mode, we walk masked properties in order.
            let mask = self.options.property_mask;
            for property in type_def
                .properties
                .iter()
                .filter(|p| p.flags.contains(mask) && !p.flags.contains(PropertyFlags::DEPRECATED))
            {
                object.insert(
                    property.name.to_owned(),
                    self.deserialize_property(property)?,
                );
            }
        } else {
            // When in exhaustive mode, the format dictates which
            // properties there are to discover.
            while object_size > 0 {
                // Back up the current buffer length and read the next property's size.
                // This will also count padding bits to byte boundaries.
                let previous_buf_len = self.reader.len();
                let property_size = self.deserialize_u32()? as usize;

                // Read the property's hash and get its object from type defs.
                let property_hash = self.deserialize_u32()?;
                let property = type_def
                    .properties
                    .iter()
                    .find(|p| p.hash == property_hash)
                    .ok_or_else(|| anyhow!("received unknown property hash {property_hash}"))?;

                // Deserialize the property's value.
                let value = self.deserialize_property(property)?;

                // Validate the size expectations.
                let actual_size = previous_buf_len - self.reader.len();
                if actual_size != property_size {
                    bail!(
                        "size mismatch for property; expected {property_size}, got {actual_size}"
                    );
                }

                // When the size check passed, subtract the property's size from
                // the object's total size to prepare for the next round.
                object_size -= property_size;

                // Lastly, insert the property into our object.
                object.insert(property.name.to_owned(), value);
            }
        }

        Ok(object)
    }

    fn deserialize_property(&mut self, property: &Property) -> anyhow::Result<Value> {
        if property.flags.contains(PropertyFlags::DELTA_ENCODE) && !self.deserialize_bool()? {
            if self
                .options
                .flags
                .contains(SerializerFlags::FORBID_DELTA_ENCODE)
            {
                bail!("missing delta value which is supposed to be there");
            }

            return Ok(Value::Empty);
        }

        if property.dynamic {
            self.deserialize_list(property)
        } else {
            self.deserialize_data(property)
        }
    }
}
