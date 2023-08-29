use kobold_utils::{anyhow, libdeflater::Decompressor};

/// A zlib inflater for decompressing archive files.
///
/// This maintains an internal scratch buffer whose allocation
/// will be re-used for subsequent decompression on the same
/// [`Inflater`] object.
///
/// This however comes at the caveat that only one decompressed
/// file can be borrowed from the archive at a given moment.
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

    /// Decompresses the given `data` into the internal scratch
    /// buffer and returns a reference to it.
    ///
    /// `size_hint` must be the size of inflated output, otherwise
    /// this method will error.
    pub fn decompress(&mut self, data: &[u8], size_hint: usize) -> anyhow::Result<&[u8]> {
        self.scratch.resize(size_hint, 0);

        let out = self.raw.zlib_decompress(data, &mut self.scratch)?;
        anyhow::ensure!(out == size_hint, "inflated size mismatch");

        Ok(&self.scratch)
    }
}

impl Default for Inflater {
    fn default() -> Self {
        Self::new()
    }
}
