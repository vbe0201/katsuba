use std::sync::Arc;

use anyhow::bail;
use byteorder::{ReadBytesExt, LE};
use kobold_bit_buf::BitReader;
use kobold_types::TypeList;
use kobold_utils::{anyhow, libdeflater::Decompressor};

use super::*;
use crate::Value;

#[inline]
fn zlib_decompress(
    inflater: &mut Decompressor,
    mut data: &[u8],
    out: &mut Vec<u8>,
) -> anyhow::Result<()> {
    let size = data.read_u32::<LE>()? as usize;
    out.resize(size, 0);

    let decompressed = inflater.zlib_decompress(data, out)?;
    anyhow::ensure!(decompressed == size, "size mismatch for uncompressed data");

    Ok(())
}

impl ZlibParts {
    fn configure<'a>(
        &'a mut self,
        opts: &mut SerializerOptions,
        mut data: &'a [u8],
    ) -> anyhow::Result<BitReader<'a>> {
        // If the data is manually compressed, uncompress into scratch.
        if opts.manual_compression {
            zlib_decompress(&mut self.inflater, data, &mut self.scratch1)?;
            data = &self.scratch1;
        }

        // If the serializer flags are stateful, update the options.
        if opts.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            opts.flags = SerializerFlags::from_bits_truncate(data.read_u32::<LE>()?);
        }

        // If the data is compressed, uncompress it into scratch.
        if opts.flags.contains(SerializerFlags::WITH_COMPRESSION) && data.read_u8()? != 0 {
            zlib_decompress(&mut self.inflater, data, &mut self.scratch2)?;
            data = &self.scratch2;
        }

        Ok(BitReader::new(data))
    }
}

impl Serializer {
    /// Creates a new deserializer with its configuration.
    ///
    /// No data for deserialization has been loaded at this point.
    /// [`Deserializer::load_data`] should be called next.
    pub fn new(options: SerializerOptions, types: Arc<TypeList>) -> anyhow::Result<Self> {
        if options.shallow && options.skip_unknown_types {
            bail!("cannot skip unknown types in shallow mode");
        }

        Ok(Self {
            parts: SerializerParts { options, types },
            zlib_parts: ZlibParts {
                inflater: Decompressor::new(),
                scratch1: Vec::new(),
                scratch2: Vec::new(),
            },
        })
    }

    /// Deserializes an object [`Value`] from the given data.
    pub fn deserialize<D: Diagnostics, T: TypeTag>(
        &mut self,
        data: &[u8],
        mut diagnostics: D,
    ) -> anyhow::Result<Value> {
        let mut reader = self.zlib_parts.configure(&mut self.parts.options, data)?;

        object::deserialize::<_, T>(&mut self.parts, &mut reader, &mut diagnostics)
    }
}
