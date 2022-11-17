use anyhow::{bail, Result};
use flate2::{Decompress, FlushDecompress, Status};

pub struct Inflater {
    scratch: Vec<u8>,
    decompress: Decompress,
}

impl Inflater {
    /// Creates a new, empty inflater instance.
    ///
    /// This method does not allocate by default.
    pub fn new() -> Self {
        Self {
            scratch: Vec::new(),
            decompress: Decompress::new(true),
        }
    }

    /// Decompresses the given data into an internal buffer
    /// and returns an immutable handle to it.
    ///
    /// It is expected that `data` is one full, consecutive
    /// stream of zlib data for decompression and
    /// `expected_size` accurately describes the byte size
    /// of the data after decompression.
    pub fn decompress<'a>(&'a mut self, data: &[u8], expected_size: usize) -> Result<&'a [u8]> {
        // Prepare internal buffer for decompressing the data.
        self.scratch.clear();
        self.scratch.reserve(expected_size);

        // Decompress the data into the internal buffer.
        if self
            .decompress
            .decompress_vec(data, &mut self.scratch, FlushDecompress::Finish)?
            != Status::StreamEnd
        {
            bail!("Received incomplete zlib stream or wrong size expectation");
        }

        // Reset decompress object for next usage.
        self.decompress.reset(true);

        // Return a handle to the data we decompressed.
        Ok(&self.scratch[..])
    }
}
