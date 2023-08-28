use std::sync::Arc;

use anyhow::bail;
use byteorder::{ReadBytesExt, LE};
use kobold_bit_buf::BitReader;
use kobold_types::TypeList;
use libdeflater::Decompressor;

use super::{object, Diagnostics, SerializerFlags, SerializerOptions, TypeTag};
use crate::Value;

#[inline]
fn zlib_decompress(out: &mut [u8], data: &[u8]) -> anyhow::Result<()> {
    let mut decoder = Decompressor::new();
    decoder.zlib_decompress(data, out)?;

    Ok(())
}

/// A deserializer for dynamic ObjectProperty [`Value`]s.
pub struct Deserializer<D> {
    /// The serializer configuration in use.
    pub options: SerializerOptions,
    pub(crate) types: Arc<TypeList>,
    pub(crate) diagnostics: Option<D>,
}

impl<D> Deserializer<D> {
    /// Creates a new deserializer with its configuration.
    ///
    /// No data for deserialization has been loaded at this point.
    /// [`Deserializer::load_data`] should be called next.
    pub fn new(
        options: SerializerOptions,
        types: Arc<TypeList>,
        diagnostics: D,
    ) -> anyhow::Result<Self> {
        if options.shallow && options.skip_unknown_types {
            bail!("cannot skip unknown types in shallow mode");
        }

        Ok(Self {
            types,
            options,
            diagnostics: Some(diagnostics),
        })
    }
}

impl<D: Diagnostics> Deserializer<D> {
    fn decompress_data(scratch: &mut Vec<u8>, mut data: &[u8]) -> anyhow::Result<()> {
        let size = data.read_u32::<LE>()? as usize;

        scratch.resize(size, 0);
        zlib_decompress(scratch, data)
    }

    fn configure<'a>(
        &mut self,
        scratch: &'a mut Vec<u8>,
        mut data: &'a [u8],
    ) -> anyhow::Result<BitReader<'a>> {
        scratch.clear();

        let reader = if self.options.manual_compression {
            Self::decompress_data(scratch, data)?;
            let mut reader = BitReader::new(scratch);

            // If configuration flags are stateful, deserialize them.
            if self.options.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
                self.options.flags = SerializerFlags::from_bits_truncate(reader.u32());
            }

            reader
        } else {
            // If configuration flags are stateful, deserialize them.
            if self.options.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
                self.options.flags = SerializerFlags::from_bits_truncate(data.read_u32::<LE>()?);
            }

            // Determine whether data is compressed or not.
            if self
                .options
                .flags
                .contains(SerializerFlags::WITH_COMPRESSION)
                && data.read_u8()? != 0
            {
                Self::decompress_data(scratch, data)?;
                BitReader::new(scratch)
            } else {
                BitReader::new(data)
            }
        };

        Ok(reader)
    }

    /// Deserializes an object [`Value`] from the given data.
    pub fn deserialize<T: TypeTag>(
        &mut self,
        scratch: &mut Vec<u8>,
        data: &[u8],
    ) -> anyhow::Result<Value> {
        let mut reader = self.configure(scratch, data)?;
        let mut diagnostics = self.diagnostics.take().unwrap();

        let res = object::deserialize::<_, T>(self, &mut reader, &mut diagnostics);
        self.diagnostics = Some(diagnostics);

        res
    }
}
