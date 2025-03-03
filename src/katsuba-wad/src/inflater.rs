use libdeflater::{DecompressionError, Decompressor};

/// A zlib inflater for decompressing archive files.
///
/// This maintains an internal scratch buffer whose allocation
/// will be re-used for subsequent decompression on the same
/// [`Inflater`] object.
///
/// This however comes at the caveat that only one decompressed
/// file can be borrowed from the archive at a time.
pub struct Inflater {
    raw: Decompressor,
    scratch: Vec<u8>,
}

impl Inflater {
    /// Creates a new inflater for zlib decompression.
    pub fn new() -> Self {
        Self {
            raw: Decompressor::new(),
            scratch: Vec::new(),
        }
    }

    /// Creates a new inflater from a pre-allocated memory buffer.
    pub fn new_with(buf: Vec<u8>) -> Self {
        Self {
            raw: Decompressor::new(),
            scratch: buf,
        }
    }

    /// Consumes the inflater and returns its scratch buffer.
    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.scratch
    }

    /// Decompresses the given `data` into a provided external
    /// buffer and returns a reference to it back.
    pub fn decompress_into<'a>(
        &mut self,
        out: &'a mut [u8],
        data: &[u8],
    ) -> Result<&'a [u8], DecompressionError> {
        let written = self.raw.zlib_decompress(data, out)?;
        if written != out.len() {
            return Err(DecompressionError::BadData);
        }

        Ok(out)
    }

    /// Decompresses the given `data` into the internal scratch
    /// buffer and returns a reference to it.
    ///
    /// `size_hint` must be the size of inflated output, otherwise
    /// this method will error.
    pub fn decompress(
        &mut self,
        data: &[u8],
        size_hint: usize,
    ) -> Result<&[u8], DecompressionError> {
        self.scratch.resize(size_hint, 0);

        let written = self.raw.zlib_decompress(data, &mut self.scratch)?;
        if written != size_hint {
            return Err(DecompressionError::BadData);
        }

        Ok(&self.scratch)
    }
}

impl Default for Inflater {
    fn default() -> Self {
        Self::new()
    }
}
