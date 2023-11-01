use std::{io, mem::size_of, ptr};

// The maximum number of bits that can be buffered before comitting to the
// output sink.
//
// We target to have an amount between 56 and 63 bits in the buffer. Since
// we only commit whole bytes, it means the low 3 bits never change.
const BUFFER_SIZE: u32 = u64::BITS - 1;

// The maximum number of bits that can be committed at once.
//
// Since we write whole bytes only, this is the smallest value where a whole
// byte doesn't fit in anymore.
const WRITABLE_BITS: u32 = BUFFER_SIZE & !7;

/// A buffer which enables bit-based serialization of data.
///
/// Individual bit writing starts at the LSB of the byte, working
/// towards the MSB.
#[derive(Debug, Default)]
pub struct BitWriter {
    // The inner buffer where data is being written to.
    inner: Vec<u8>,

    // A buffer for bits which are not committed to the
    // data buffer yet.
    buf: u64,

    // How many bits in `buf` are currently filled.
    count: u32,
}

impl BitWriter {
    /// Creates an empty [`BitWriter`].
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            buf: 0,
            count: 0,
        }
    }

    /// Creates an empty [`BitWriter`] to a given output vector.
    ///
    /// This is useful if you want to reuse existing buffer allocations.
    pub const fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: vec,
            buf: 0,
            count: 0,
        }
    }

    /// Gets the number of bits currently in the buffer.
    #[inline]
    pub fn written_bits(&self) -> usize {
        (self.inner.len() << 3) + self.count as usize
    }

    /// Indicates how much capacity is still left for writing bits until
    /// [`Self::commit`] must be called.
    #[inline]
    pub fn remaining(&self) -> u32 {
        WRITABLE_BITS.saturating_sub(self.count)
    }

    /// Gets a view of the buffer's storage as a byte slice.
    #[inline]
    pub fn view(&self) -> &[u8] {
        &self.inner
    }

    /// Consumes the [`BitWriter`] and returns the byte buffer.
    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }

    /// Reserves capacity for at least `nbytes` more bytes in the
    /// output buffer.
    ///
    /// When the data format allows making educated guesses about
    /// size consumption, use this to optimize memory allocation.
    #[inline]
    pub fn reserve(&mut self, nbytes: usize) {
        self.inner.reserve(nbytes);
    }

    /// Flushes all currently buffered bits to the data buffer.
    pub fn commit(&mut self) {
        debug_assert!(self.count <= BUFFER_SIZE);

        let buf = self.buf.to_le_bytes();
        self.reserve(buf.len());

        // Copy the current buffer into the underlying writer.
        unsafe {
            let dest = self.inner.as_mut_ptr().add(self.inner.len());
            ptr::copy_nonoverlapping(buf.as_ptr(), dest, buf.len());

            self.inner
                .set_len(self.inner.len() + (self.count as usize >> 3));
        }

        // Remove the written bits from the internal state.
        self.buf >>= self.count & WRITABLE_BITS;
        self.count &= 7;
    }

    /// Adds `nbits` bits from `value` to the internal buffer, if capacity
    /// is available in the buffer.
    pub fn offer(&mut self, value: u64, nbits: u32) -> io::Result<()> {
        if nbits <= WRITABLE_BITS && nbits <= (BUFFER_SIZE - self.count) {
            self.buf |= (value & ((1 << nbits) - 1)) << self.count;
            self.count += nbits;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "buffer capacity overflow",
            ))
        }
    }

    /// Flushes remaining bits to the output vector, with partially initialized
    /// bytes being zero-padded.
    pub fn realign_to_byte(&mut self) {
        // Flush whole bytes to the buffer. If no partial byte is left, we're done.
        self.commit();

        // The remainder of our buffer is a partial byte with at most 7 bits set.
        // These bits were already committed, so we can just skip another byte.
        if self.count != 0 {
            unsafe { self.inner.set_len(self.inner.len() + 1) }

            self.buf = 0;
            self.count = 0;
        }
    }

    /// Writes whole bytes from `buf` to the output vector.
    pub fn write_bytes(&mut self, buf: &[u8]) {
        self.inner.extend_from_slice(buf);
    }

    /// Length-prefixes all data produced by the closure `f` with
    /// a 32-bit little endian value.
    ///
    /// The closure itself starts at an aligned writer with 0 stale
    /// bits buffered.
    pub fn length_prefixed<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let bit_start = self.written_bits();
        self.realign_to_byte();

        // Remember the start position and reserve a placeholder.
        let prefix_pos = self.written_bits() >> 3;
        self.inner.extend_from_slice(&[0; 4]);

        // Execute the inner closure with all its operations.
        let t = f(self);

        // Calculate and write back the length prefix value.
        let prefix = (self.written_bits() - bit_start) as u32;
        unsafe {
            let dest = self.inner.as_mut_ptr().add(prefix_pos);
            let src = prefix.to_le_bytes();
            ptr::copy_nonoverlapping(src.as_ptr(), dest, size_of::<u32>());
        }

        t
    }
}
