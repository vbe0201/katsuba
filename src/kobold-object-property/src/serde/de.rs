use std::sync::Arc;

use anyhow::bail;
use byteorder::{ReadBytesExt, LE};
use kobold_bit_buf::BitReader;
use kobold_types::TypeList;
use libdeflater::Decompressor;

use super::{object, Diagnostics, SerializerFlags, SerializerOptions, TypeTag};
use crate::Value;

#[inline]
fn decompress(
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

struct ZlibParts {
    inflater: Decompressor,

    // We need two scratch buffers to handle the case of data being compressed
    // twice. Note that this has not been observed in practice yet, so most of
    // the time one buffer will stay empty and never allocates.
    scratch1: Vec<u8>,
    scratch2: Vec<u8>,
}

impl ZlibParts {
    fn configure<'a>(
        &'a mut self,
        opts: &mut SerializerOptions,
        mut data: &'a [u8],
    ) -> anyhow::Result<BitReader<'a>> {
        // If the data is manually compressed, uncompress into scratch.
        if opts.manual_compression {
            decompress(&mut self.inflater, data, &mut self.scratch1)?;
            data = &self.scratch1;
        }

        // If the serializer flags are stateful, update the options.
        if opts.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            opts.flags = SerializerFlags::from_bits_truncate(data.read_u32::<LE>()?);
        }

        // If the data is compressed, uncompress it into scratch.
        if opts.flags.contains(SerializerFlags::WITH_COMPRESSION) && data.read_u8()? != 0 {
            decompress(&mut self.inflater, data, &mut self.scratch2)?;
            data = &self.scratch2;
        }

        Ok(BitReader::new(data))
    }
}

/// The guts of a [`Deserializer`].
pub struct DeserializerParts<D> {
    /// The serializer configuration in use.
    pub options: SerializerOptions,
    pub(crate) types: Arc<TypeList>,
    pub(crate) diagnostics: Option<D>,
}

impl<D> DeserializerParts<D> {
    #[inline]
    pub fn with_recursion_limit<F, T>(&mut self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut Self) -> anyhow::Result<T>,
    {
        self.options.recursion_limit -= 1;
        anyhow::ensure!(self.options.recursion_limit > 0, "recursion limit exceeded");

        let res = f(self);

        self.options.recursion_limit += 1;

        res
    }
}

/// A deserializer for dynamic ObjectProperty [`Value`]s.
pub struct Deserializer<D> {
    /// The raw deserializer state.
    pub serde_parts: DeserializerParts<D>,
    zlib_parts: ZlibParts,
}

impl<D: Diagnostics> Deserializer<D> {
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
            serde_parts: DeserializerParts {
                types,
                options,
                diagnostics: Some(diagnostics),
            },
            zlib_parts: ZlibParts {
                inflater: Decompressor::new(),
                scratch1: Vec::new(),
                scratch2: Vec::new(),
            },
        })
    }

    /// Deserializes an object [`Value`] from the given data.
    pub fn deserialize<T: TypeTag>(&mut self, data: &[u8]) -> anyhow::Result<Value> {
        let mut reader = self
            .zlib_parts
            .configure(&mut self.serde_parts.options, data)?;
        let mut diagnostics = self.serde_parts.diagnostics.take().unwrap();

        let res = object::deserialize::<_, T>(&mut self.serde_parts, &mut reader, &mut diagnostics);
        self.serde_parts.diagnostics = Some(diagnostics);

        res
    }
}
