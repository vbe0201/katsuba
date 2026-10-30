use std::{io, sync::Arc};

use bitter::LittleEndianReader;
use byteorder::{LE, ReadBytesExt};
use katsuba_types::TypeList;
use libdeflater::Decompressor;

use super::*;
use crate::Value;

pub(super) fn zlib_decompress(
    inflater: &mut Decompressor,
    size: u32,
    data: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    let size = size as usize;
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
        if opts.manual_compression {
            // When the highest bit is set, treat the data as uncompressed.
            let size = data.read_u32::<LE>()?;
            if size & MANUAL_UNCOMPRESSED_BIT != 0 {
                let size = (size & !MANUAL_UNCOMPRESSED_BIT) as usize;
                data = data
                    .get(..size)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
            } else {
                zlib_decompress(&mut self.inflater, size, data, &mut self.scratch1)?;
                data = &self.scratch1;
            }
        }

        // If the serializer flags are stateful, update the options.
        if opts.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            opts.flags = SerializerFlags::from_bits_truncate(data.read_u32::<LE>()?);
        }

        // If the data is compressed, uncompress it into scratch.
        if opts.flags.contains(SerializerFlags::WITH_COMPRESSION) && data.read_u8()? & 1 != 0 {
            let size = data.read_u32::<LE>()?;
            zlib_decompress(&mut self.inflater, size, data, &mut self.scratch2)?;
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
        let opts = self.parts.options;
        let res = self.deserialize_(data);
        self.parts.options = opts;

        res
    }

    fn deserialize_(&mut self, data: &[u8]) -> Result<Value, Error> {
        let mut reader = self.zlib_parts.configure(&mut self.parts.options, data)?;
        log::debug!("Deserializing object with config {:?}", self.parts.options);

        let value = object::deserialize(&mut self.parts, &mut reader)?;
        if let Value::Empty = value {
            return Err(Error::NullRoot);
        }

        Ok(value)
    }
}
