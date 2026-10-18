use std::mem;

use bitter::LittleEndianWriter;
use byteorder::{ByteOrder, LE};
use libdeflater::Compressor;

use super::*;
use crate::Value;

pub(super) fn zlib_compress(
    deflater: &mut Compressor,
    data: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    let size = u32::try_from(data.len()).map_err(|_| Error::LengthOverflow(data.len()))?;
    out.extend_from_slice(&size.to_le_bytes());

    let start = out.len();
    out.resize(start + deflater.zlib_compress_bound(data.len()), 0);

    let compressed = deflater.zlib_compress(data, &mut out[start..])?;
    out.truncate(start + compressed);

    Ok(())
}

impl ZlibParts {
    fn finish(&mut self, opts: &SerializerOptions, mut data: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut header = [0u8; 5];
        let mut header_len = 0;

        // Serialize stateful flags into the header, if requested.
        if opts.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            LE::write_u32(&mut header[0..4], opts.flags.bits());
            header_len += 4;
        }

        // Try to compress the data, if requested.
        if opts.flags.contains(SerializerFlags::WITH_COMPRESSION) {
            self.scratch2.clear();

            self.scratch2.push(1);
            zlib_compress(&mut self.deflater, &data, &mut self.scratch2)?;

            if !self.scratch2.is_empty() && self.scratch2.len() < data.len() {
                mem::swap(&mut data, &mut self.scratch2);
            } else {
                // No compression used because it occupies more space than the raw input.
                // Write the compression marker to the header, since we won't be using
                // the one we prematurely placed in the scratch2 buffer anymore.
                header[header_len] = 0;
                header_len += 1;
            }
        }

        // If we have a header, merge it with the data buffer.
        if header_len > 0 {
            self.scratch1.clear();
            self.scratch1.extend_from_slice(&header[..header_len]);
            self.scratch1.extend_from_slice(&data);
            mem::swap(&mut data, &mut self.scratch1);
        }

        // Compress the entire data buffer if manual compression is requested.
        if opts.manual_compression {
            self.scratch1.clear();
            zlib_compress(&mut self.deflater, &data, &mut self.scratch1)?;
            mem::swap(&mut data, &mut self.scratch1);
        }

        Ok(data)
    }
}

impl Serializer {
    /// Serializes an object [`Value`] into a freshly allocated buffer.
    ///
    /// For repeated use on the same [`Serializer`], prefer
    /// [`Serializer::serialize_into`] with a buffer retained across calls,
    /// to avoid reallocating it every time.
    pub fn serialize(&mut self, value: &Value) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        self.serialize_into(value, &mut out)?;
        Ok(out)
    }

    /// Serializes an object [`Value`] into `out`, reusing its existing
    /// allocation where possible.
    ///
    /// `out` is cleared before writing. Reusing the same buffer across
    /// repeated calls avoids reallocating it each time.
    pub fn serialize_into(&mut self, value: &Value, out: &mut Vec<u8>) -> Result<(), Error> {
        let object = match value {
            Value::Object(o) => o,
            Value::Empty => return Err(Error::NullRoot),
            _ => return Err(Error::TypeMismatch("PropertyClass".into())),
        };
        log::debug!("Serializing object with config {:?}", self.parts.options);

        let mut buf = mem::take(out);
        buf.clear();

        let mut writer = LittleEndianWriter::new(buf);
        object::serialize(&mut self.parts, Some(object), &mut writer)?;
        let data = writer.into_inner().unwrap();

        *out = self.zlib_parts.finish(&self.parts.options, data)?;

        Ok(())
    }
}
