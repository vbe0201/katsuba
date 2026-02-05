use std::sync::Arc;

use bitter::LittleEndianReader;
use byteorder::{LE, ReadBytesExt};
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
    ) -> Result<LittleEndianReader<'a>, Error> {
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

        Ok(LittleEndianReader::new(data))
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

    /// Deserializes an object [`Value`] from the given data.
    pub fn deserialize(&mut self, data: &[u8]) -> Result<Value, Error> {
        let mut reader = self.zlib_parts.configure(&mut self.parts.options, data)?;
        log::info!("Deserializing object with config {:?}", self.parts.options);

        let value = object::deserialize(&mut self.parts, &mut reader)?;
        if let Value::Empty = value {
            return Err(Error::NullRoot);
        }

        Ok(value)
    }
}
