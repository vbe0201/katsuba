use std::{mem::size_of, ptr};

// The maximum number of bits that can be buffered before
// committing to the output vector.
//
// We target to have an amount between 56 and 63 bits in the
// buffer. Since we only commit whole bytes, it means the
// low 3 bits never change.
//
// The choice of 63 instead of 64 is conscious because the
// logic for advancing the bit pointer obeys several traits
// for algebraic refactoring to improve codegen.
const BUFFER_SIZE: u32 = u64::BITS - 1;

// The maximum number of bits that can be committed at once.
//
// Since we write whole bytes only, this is the smallest
// value where a whole byte doesn't fit in anymore.
const WRITABLE_BITS: u32 = BUFFER_SIZE & !7;

macro_rules! impl_write_literal {
    ($($(#[$doc:meta])* $write_fn:ident($ty:ty)),* $(,)?) => {
        $(
            $(#[$doc])*
            #[inline]
            pub fn $write_fn(&mut self, value: $ty) {
                self.inner.reserve(size_of::<$ty>());

                // SAFETY: We pre-allocated the needed memory.
                unsafe {
                    let buf = value.to_le_bytes();
                    let dest = self.inner.as_mut_ptr().add(self.inner.len());

                    ptr::copy_nonoverlapping(buf.as_ptr(), dest, buf.len());
                    self.inner.set_len(self.inner.len() + size_of::<$ty>());
                }
            }
        )*
    };
}

/// A reserved length prefix, which can be committed at a later time.
pub struct LengthMarker(usize, usize);

/// A buffer which enables bit-based serialization of data.
///
/// Quantities of multiple bytes (except byte slices) are always
/// written in little-endian byte ordering. Individual bit writing
/// starts at the LSB of the byte, working towards the MSB.
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
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            buf: 0,
            count: 0,
        }
    }

    /// Gets the number of bits currently in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        (self.inner.len() << 3) + self.count as usize
    }

    /// Indicates how much capacity is still left for writing
    /// bits until [`Self::flush_bits`] must be called.
    #[inline]
    pub fn remaining(&self) -> u32 {
        WRITABLE_BITS.saturating_sub(self.count)
    }

    /// Indicates if the writer doesn't contain any bits.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn flush_bits(&mut self) {
        debug_assert!(self.count <= BUFFER_SIZE);

        let buf = self.buf.to_le_bytes();
        self.reserve(buf.len());

        // SAFETY: We reserve enough bytes in advance for this
        // write to not go out of bounds.
        unsafe {
            let dest = self.inner.as_mut_ptr().add(self.inner.len());
            ptr::copy_nonoverlapping(buf.as_ptr(), dest, buf.len());
        }

        // Remove the bits we just wrote from the buffer.
        self.buf >>= self.count & WRITABLE_BITS;

        unsafe {
            self.inner
                .set_len(self.inner.len() + (self.count as usize >> 3));
        }

        // Remove the written bytes from the count.
        self.count &= 7;
    }

    /// Writes bits to the internal buffer, if possible.
    ///
    /// These will not show up in the output bytes until
    /// [`Self::flush_bits`] was called.
    ///
    /// # Panics
    ///
    /// The caller must ensure enough capacity is left for
    /// `nbits` bits in the data buffer.
    #[inline]
    pub fn write_bits(&mut self, value: u64, nbits: u32) {
        assert!(nbits <= WRITABLE_BITS);
        assert!(nbits <= (BUFFER_SIZE - self.count));

        self.buf |= (value & ((1 << nbits) - 1)) << self.count;
        self.count += nbits;
    }

    /// Realigns the buffer to the boundaries of the next
    /// untouched byte.
    ///
    /// Untouched byte in this case means no partial bit
    /// writes overlap with its memory region.
    pub fn realign_to_byte(&mut self) {
        // Flush whole bytes to the buffer.
        self.flush_bits();

        // The remainder of our buffer is a partial byte with at
        // most 7 bits set. In the case of 0, the buffer is
        // already aligned, otherwise we need to discard the bits.
        if self.count != 0 {
            // SAFETY: Since we partially started the next byte already,
            // the memory for it is reserved and initialized.
            unsafe {
                self.inner.set_len(self.inner.len() + 1);
            }

            self.buf = 0;
            self.count = 0;
        }
    }

    /// Appends a given slice of bytes to the output buffer.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.inner.extend_from_slice(bytes);
    }

    /// Reserves a [`u32`] length prefix at the current buffer position.
    ///
    /// The returned [`LengthMarker`] can be passed to [`Self::commit_len`]
    /// at a later time to patch back the amount of bits that have been
    /// written since marking.
    pub fn mark_len(&mut self) -> LengthMarker {
        // Back up the current bit position for length calculation.
        let bit_start = self.len();

        // Write the placeholder value and remember its start offset.
        self.realign_to_byte();
        let bit_pos = self.len();
        self.u32(0);

        LengthMarker(bit_start, bit_pos)
    }

    /// Commits a previously reserved length prefix to the buffer.
    pub fn commit_len(&mut self, len: LengthMarker) {
        let LengthMarker(bit_start, bit_pos) = len;

        // Calculate the length prefix value.
        let prefix = (self.len() - bit_start) as u32;
        let prefix = prefix.to_le_bytes();

        // SAFETY: We created `LengthMarker` ourselves, so we
        // can trust `bit_pos` to be a valid offset.
        unsafe {
            let dest = self.inner.as_mut_ptr().add(bit_pos >> 3);
            ptr::copy_nonoverlapping(prefix.as_ptr(), dest, prefix.len());
        }
    }

    /// Writes a [`bool`] value to the bit buffer.
    #[inline]
    pub fn bool(&mut self, value: bool) {
        self.write_bits(value as u64, 1);
    }

    // fn $write_fn(&mut self, value: $ty)
    impl_write_literal! {
        /// Writes a [`u8`] value to the current position in
        /// the byte buffer.
        u8(u8),
        /// Writes a [`i8`] value to the current position in
        /// the byte buffer.
        i8(i8),

        /// Writes a [`u16`] value to the current position in
        /// the byte buffer.
        u16(u16),
        /// Writes a [`i16`] value to the current position in
        /// the byte buffer.
        i16(i16),

        /// Writes a [`u32`] value to the current position in
        /// the byte buffer.
        u32(u32),
        /// Writes a [`i32`] value to the current position in
        /// the byte buffer.
        i32(i32),

        /// Writes a [`u64`] value to the current position in
        /// the byte buffer.
        u64(u64),
        /// Writes a [`i64`] value to the current position in
        /// the byte buffer.
        i64(i64),
    }

    /// Writes a [`f32`] value to the current position in
    /// the byte buffer.
    #[inline]
    pub fn f32(&mut self, value: f32) {
        self.u32(value.to_bits());
    }

    /// Writes a [`f64`] value to the current position in
    /// the byte buffer.
    #[inline]
    pub fn f64(&mut self, value: f64) {
        self.u64(value.to_bits());
    }
}
