use std::{
    io::{self, Write},
    marker::PhantomData,
    sync::Arc,
};

use anyhow::bail;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use flate2::write::ZlibDecoder;
#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::{reader::BitReader, type_list::*, TypeTag, Value};

#[macro_use]
mod macros;

mod enum_variant;

mod object;
use object::ObjectDeserializer;

mod property;

mod simple_data;

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
    /// Skips unknown types during deserialization of properties.
    pub skip_unknown_types: bool,
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

    #[getter]
    pub fn get_skip_unknown_types(&self) -> PyResult<bool> {
        Ok(self.skip_unknown_types)
    }

    #[setter]
    pub fn set_skip_unknown_types(&mut self, new: bool) -> PyResult<()> {
        self.skip_unknown_types = new;
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
            skip_unknown_types: false,
        }
    }
}

/// A configurable deserializer for the ObjectProperty binary
/// format, producing [`Value`]s.
pub struct Deserializer<T> {
    pub(crate) reader: BitReader,
    pub(crate) options: DeserializerOptions,
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

macro_rules! impl_deserialize {
    ($($de:ident($ty:ty) = $read:ident()),* $(,)*) => {
        $(
            pub(crate) fn $de(&mut self) -> anyhow::Result<$ty> {
                self.reader.$read().map_err(Into::into)
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

    fn deserialize_str(&mut self) -> anyhow::Result<Vec<u8>> {
        self.read_str_len()
            .and_then(|len| self.reader.read_bytes(len).map_err(Into::into))
    }

    fn deserialize_wstr(&mut self) -> anyhow::Result<Vec<u16>> {
        let len = self.read_str_len()?;

        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.reader.load_u16()?);
        }

        Ok(result)
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

impl<T: TypeTag> Deserializer<T> {
    /// Deserializes an object [`Value`] from previously loaded data.
    pub fn deserialize(&mut self, data: &[u8]) -> anyhow::Result<Value> {
        self.configure(data)?;
        ObjectDeserializer { de: self }.deserialize()
    }
}
