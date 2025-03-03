use libdeflater::{CompressionError, CompressionLvl, Compressor};

/// A zlib inflater for compressing archive files.
///
/// This maintains an internal scratch buffer whose memory will be
/// reused for subsequent compressions with the same [`Deflater`]
/// instance.
///
/// This however comes at the caveat that only one compressed file
/// can be borrowed from the deflater at a time.
pub struct Deflater {
    compressor: Compressor,
    scratch: Vec<u8>,
}

impl Deflater {
    /// Creates an empty deflater at default compression level.
    pub fn new() -> Self {
        Self {
            compressor: Compressor::new(CompressionLvl::best()),
            scratch: Vec::new(),
        }
    }

    /// Compresses a raw buffer into the inner scratch buffer and
    /// returns the subset of the slice occupied by it.
    pub fn compress(&mut self, data: &[u8]) -> Result<&[u8], CompressionError> {
        let max_size = self.compressor.zlib_compress_bound(data.len());
        self.scratch.resize(max_size, 0);

        let real_size = self.compressor.zlib_compress(data, &mut self.scratch)?;
        debug_assert!(real_size <= max_size);

        // SAFETY: We know `real_size` fits in `max_size` and we initialized
        // `max_size` elements in the vector beforehand.
        Ok(unsafe { self.scratch.get_unchecked(..real_size) })
    }

    pub fn compress_into<'a>(
        &mut self,
        out: &'a mut Vec<u8>,
        data: &[u8],
    ) -> Result<&'a [u8], CompressionError> {
        let data_start = out.len();
        let max_size = self.compressor.zlib_compress_bound(data.len());

        // Reserve more memory at the end of the vector and compress into the
        // newly initialized region of `max_size`.
        out.resize(data_start + max_size, 0);
        unsafe {
            // SAFETY: Indexing at `data_start` always makes at least an empty silce.
            let real_size = self
                .compressor
                .zlib_compress(data, out.get_unchecked_mut(data_start..))?;
            debug_assert!(real_size <= max_size);

            // SAFETY: We know `real_size` is smaller than `max_size`, so that many
            // extra elements are guaranteed to be initialized in vector memory.
            out.set_len(data_start + real_size);

            // SAFETY: Same as above.
            Ok(out.get_unchecked(data_start..))
        }
    }
}

impl Default for Deflater {
    fn default() -> Self {
        Self::new()
    }
}
