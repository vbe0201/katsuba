use std::sync::Arc;

use byteorder::{ReadBytesExt, LE};
use katsuba_bit_buf::BitReader;
use katsuba_types::TypeList;
use libdeflater::Decompressor;

use super::*;
use crate::Value;

#[inline]
pub(super) fn zlib_decompress(
    inflater: &mut Decompressor,
    mut data: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    let size = data.read_u32::<LE>()? as usize;
    out.resize(size, 0);

    let decompressed = inflater.zlib_decompress(data, out)?;
    if decompressed != size {
        return Err(Error::DecompressedSizeMismatch {
            expected: size,
            actual: decompressed,
        });
    }

    Ok(())
}

impl ZlibParts {
    fn configure<'a>(
        &'a mut self,
        opts: &mut SerializerOptions,
        mut data: &'a [u8],
    ) -> Result<BitReader<'a>, Error> {
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
    pub fn new(options: SerializerOptions, types: Arc<TypeList>) -> Result<Self, Error> {
        if options.shallow && options.skip_unknown_types {
            return Err(Error::BadConfig(
                "cannot skip unknown types in shallow mode",
            ));
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
    #[cfg(feature = "option-guessing")]
    pub fn with_guessed_options(types: Arc<TypeList>, data: &[u8]) -> Result<Self, Error> {
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
    #[cfg(feature = "option-guessing")]
    pub fn with_guessed_options_from_base(
        opts: SerializerOptions,
        types: Arc<TypeList>,
        data: &[u8],
    ) -> Result<Self, Error> {
        super::guess::Guesser::new(opts, types).guess(data)
    }

    /// Deserializes an object [`Value`] from the given data.
    pub fn deserialize<T: TypeTag>(&mut self, data: &[u8]) -> Result<Value, Error> {
        let mut reader = self.zlib_parts.configure(&mut self.parts.options, data)?;
        log::info!("Deserializing object with config {:?}", self.parts.options);

        let value = object::deserialize::<T>(&mut self.parts, &mut reader)?;
        if let Value::Empty = value {
            return Err(Error::NullRoot);
        }

        Ok(value)
    }
}
