use std::sync::Arc;

use anyhow::bail;
use byteorder::{ReadBytesExt, LE};
use kobold_bit_buf::BitReader;
use kobold_types::TypeList;
use kobold_utils::{
    anyhow,
    libdeflater::{self, Decompressor},
};

use super::*;
use crate::Value;

pub(super) enum ZlibError {
    Io(std::io::Error),
    Decompress(libdeflater::DecompressionError),
    Verify(anyhow::Error),
}

impl From<ZlibError> for anyhow::Error {
    fn from(value: ZlibError) -> Self {
        match value {
            ZlibError::Io(e) => e.into(),
            ZlibError::Decompress(e) => e.into(),
            ZlibError::Verify(e) => e,
        }
    }
}

#[inline]
pub(super) fn zlib_decompress(
    inflater: &mut Decompressor,
    mut data: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), ZlibError> {
    let size = data.read_u32::<LE>().map_err(ZlibError::Io)? as usize;
    out.resize(size, 0);

    let decompressed = inflater
        .zlib_decompress(data, out)
        .map_err(ZlibError::Decompress)?;
    if decompressed != size {
        return Err(ZlibError::Verify(anyhow::anyhow!(
            "size mismatch for uncompressed data"
        )));
    }

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
            zlib_parts: ZlibParts::new(),
        })
    }

    /// Attempts to guess the serializer configuration based on a
    /// concrete data stream.
    ///
    /// The resulting serializer instance should be ready to use,
    /// but the user may still need to tweak the settings themselves.
    ///
    /// It is generally only advised to use the resulting config as
    /// a first fit, it is not guaranteed to be accurate.
    #[cfg(feature = "enable-option-guessing")]
    pub fn with_guessed_options(types: Arc<TypeList>, data: &[u8]) -> anyhow::Result<Self> {
        Self::with_guessed_options_from_base(Default::default(), types, data)
    }

    /// Attempts to guess the serializer configuration based on a
    /// concrete data stream.
    ///
    /// This one takes a base config and modifies it based on guessed
    /// properties of the input format.
    ///
    /// This is generally recommended when users know something about
    /// the format that can't be guessed precisely, like the property
    /// filter mask.
    #[cfg(feature = "enable-option-guessing")]
    pub fn with_guessed_options_from_base(
        opts: SerializerOptions,
        types: Arc<TypeList>,
        data: &[u8],
    ) -> anyhow::Result<Self> {
        super::guess::Guesser::new(opts, types).guess(data)
    }

    /// Deserializes an object [`Value`] from the given data.
    pub fn deserialize<T: TypeTag>(&mut self, data: &[u8]) -> anyhow::Result<Value> {
        let mut reader = self.zlib_parts.configure(&mut self.parts.options, data)?;

        object::deserialize::<T>(&mut self.parts, &mut reader)
    }
}
