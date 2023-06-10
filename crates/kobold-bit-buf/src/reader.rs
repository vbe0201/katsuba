use std::{borrow::Cow, mem::size_of, ptr, slice};

// The maximum number of bits that can be stored in lookahead.
//
// We target to have an amount between 56 and 63 bits in the
// buffer. Since we only refill by whole bytes, it means the
// low 3 bits never change.
//
// The choice of 63 instead of 64 is conscious because the
// logic for advancing the bit pointer obeys several traits
// for algebraic refactoring to improve codegen.
const BUFFER_SIZE: u32 = u64::BITS - 1;

// The maximum number of bits to be consumed after a refill.
//
// Since we refill by whole bytes only, this is the smallest
// value where a whole byte doesn't fit in anymore.
const CONSUMABLE_BITS: u32 = BUFFER_SIZE & !7;

// Reads a 64-bit value from memory in little-endian ordering.
#[inline(always)]
unsafe fn read_64_le(ptr: *const u8) -> u64 {
    // SAFETY: Caller must provide a valid pointer.
    let value = unsafe { ptr::read_unaligned(ptr.cast::<u64>()) };
    value.to_le()
}

// There are no stable branch prediction hints for us.
// We emulate them by nudging the compiler into optimizing
// around the paths which do not call this function.
#[cold] // :)
#[inline(always)]
fn cold() {}

// Sign-extends an `nbits` wide value to [`i64`].
#[inline]
fn sign_extend(value: u64, nbits: u32) -> i64 {
    let shift = u64::BITS - nbits;
    (value << shift) as i64 >> shift
}

macro_rules! impl_read_literal {
    ($($(#[$doc:meta])* $read_fn:ident() -> $ty:ty),* $(,)?) => {
        $(
            $(#[$doc])*
            #[doc = "# Panics"]
            #[doc = "Caller must check that enough bytes are left for the read."]
            #[inline]
            pub fn $read_fn(&mut self) -> $ty {
                assert!(size_of::<$ty>() <= self.untouched_bytes());

                // SAFETY: We did the bounds check for reading bytes.
                #[allow(clippy::size_of_in_element_count)] // false positive
                unsafe {
                    let value = ptr::read_unaligned(self.ptr.cast::<$ty>());
                    self.ptr = self.ptr.add(size_of::<$ty>());
                    value.to_le()
                }
            }
        )*
    };
}

/// A buffer which enables bit-based deserialization of data.
///
/// Quantities of multiple bytes (except byte slices) are always
/// read in little-endian byte ordering. Individual bit reading
/// starts at the LSB of the byte, working towards the MSB.
///
/// When reading bits, users must manually call [`Self::refill_bits`]
/// first and do appropriate checks based on how many bits are left.
///
/// When wanting to read from full byte boundaries with some stale
/// buffered bits, [`Self::invalidate_and_realign_ptr`] can help.
#[derive(Debug)]
pub struct BitReader<'a> {
    // The underlying data buffer; either an owned Vec or
    // a borrowed slice.
    // Invariant: This must never be modified.
    data: Cow<'a, [u8]>,

    // Pointer to the next byte where the bit lookahead
    // buffer will be fetched from.
    ptr: *const u8,

    // A marker pointer past which it is not safe anymore
    // to take the fast refill path which blindly copies.
    safeguard: *const u8,

    // One-past-the-end pointer in the spanned byte view.
    end: *const u8,

    // The pre-fetched lookahead bufer to extract bits at.
    lookahead: u64,

    // The number of bits available for consumption from
    // the `lookahead` buffer.
    remaining: u32,
}

impl<'a> BitReader<'a> {
    /// Creates a new [`BitReader`] over an owned buffer.
    pub fn new(data: Vec<u8>) -> Self {
        let (ptr, len) = (data.as_ptr(), data.len());

        // SAFETY: All pointer arithmetic is within bounds
        // or one past the end of the allocated slice object.
        unsafe {
            Self {
                data: Cow::Owned(data),
                ptr,
                safeguard: ptr.add(len.saturating_sub(7)),
                end: ptr.add(len),
                lookahead: 0,
                remaining: 0,
            }
        }
    }

    /// Creates a new [`BitReader`] over a given byte slice.
    pub const fn new_borrowed(data: &'a [u8]) -> Self {
        let (ptr, len) = (data.as_ptr(), data.len());

        // SAFETY: All pointer arithmetic is within bounds
        // or one past the end of the allocated slice object.
        unsafe {
            Self {
                data: Cow::Borrowed(data),
                ptr,
                safeguard: ptr.add(len.saturating_sub(7)),
                end: ptr.add(len),
                lookahead: 0,
                remaining: 0,
            }
        }
    }

    /// Consumes the reader and returns its inner data.
    ///
    /// This may be used to reclaim memory of a moved [`Vec`].
    #[inline]
    pub fn into_inner(self) -> Cow<'a, [u8]> {
        self.data
    }

    #[inline(always)]
    fn can_read_in_fast_path(&self) -> bool {
        self.ptr < self.safeguard
    }

    /// Gets the remaining untouched bytes in the reader.
    #[inline]
    pub fn untouched_bytes(&self) -> usize {
        // SAFETY: Both byte pointers are derived from the same object,
        // with `ptr <= end` being an interally maintained invariant.
        unsafe { self.end.offset_from(self.ptr) as usize }
    }

    /// Gets the total number of remaining bits in the reader.
    #[inline]
    pub fn remaining_bits(&self) -> usize {
        (self.untouched_bytes() << 3) + self.remaining as usize
    }

    /// Invalidates the current bit lookahead and resets the pointer
    /// back to the first untouched byte.
    ///
    /// Untouched in this case means no partial bit reads overlapping
    /// with the memory region of a byte have happened yet.
    ///
    /// As a side effect, unread bits from a previous partially touched
    /// byte will be discarded.
    #[inline(always)]
    pub fn invalidate_and_realign_ptr(&mut self) {
        // SAFETY: Decrementing the pointer is fine since we always move
        // within a fraction of the increment done by a refill operation.
        self.ptr = unsafe { self.ptr.sub(self.remaining as usize >> 3) };

        // Invalidate the bit buffer state for refill.
        self.lookahead = 0;
        self.remaining = 0;
    }

    // SAFETY: Caller must make sure `self.ptr` is valid for 8 byte read.
    #[inline]
    unsafe fn refill_branchless(&mut self) {
        // Read from current bit pointer and fill up the lookahead
        // buffer with missing bits.
        self.lookahead |= unsafe { read_64_le(self.ptr) } << self.remaining;

        // Advance the read cursor for the next refill.
        //
        // This code seemingly increases the number of instructions
        // from 3 (sub, shr, add) to 4 (add, shr, and, sub), but
        // with dedicated bitfield extraction support it stays at 3.
        //
        // Note that the dependency chain decreases from 3 to 2
        // however, which may result in higher throughput.
        unsafe {
            self.ptr = self.ptr.add(CONSUMABLE_BITS as usize >> 3);
            self.ptr = self.ptr.sub((self.remaining as usize >> 3) & 7);
        }

        // Update the available bit count to reflect full buffer.
        self.remaining |= CONSUMABLE_BITS;
    }

    #[inline]
    fn refill_slow(&mut self) {
        while self.remaining < CONSUMABLE_BITS {
            if self.ptr == self.end {
                cold();
                break;
            }

            // SAFETY: `ptr` hasn't reached the buffer end yet.
            unsafe {
                self.lookahead |= (*self.ptr as u64) << self.remaining;
                self.ptr = self.ptr.add(1);
            }

            self.remaining += u8::BITS;
        }
    }

    /// Refills the internal bit lookahead buffer and returns the
    /// available number of bits for consumption.
    ///
    /// When this buffer is exhausted, another refill must be done.
    pub fn refill_bits(&mut self) -> u32 {
        // Make sure our state still keeps itself together.
        debug_assert!(self.ptr <= self.end);
        debug_assert!(self.remaining <= BUFFER_SIZE);

        if self.can_read_in_fast_path() {
            // SAFETY: We have enough bytes for an unchecked refill.
            unsafe {
                self.refill_branchless();
            }
        } else {
            cold();
            self.refill_slow();
        }

        self.remaining
    }

    fn peek(&mut self, count: u32) -> u64 {
        assert!(count <= CONSUMABLE_BITS);
        assert!(count <= self.remaining);

        self.lookahead & ((1 << count) - 1)
    }

    fn consume(&mut self, count: u32) {
        assert!(count <= self.remaining);

        self.lookahead >>= count;
        self.remaining -= count;
    }

    /// Reads `nbytes` raw bytes from the byte buffer.
    ///
    /// They are borrowed from the underlying byte view without
    /// copying them; however we cannot safely hand it out for
    /// `'a` since a `'static` lifetime with an owned underlying
    /// buffer would cause unsoundness.
    ///
    /// # Panics
    ///
    /// Caller must check that enough bytes are left for the read.
    pub fn read_bytes(&mut self, nbytes: usize) -> &[u8] {
        assert!(nbytes <= self.untouched_bytes());

        // SAFETY: A bounds check was done and an appropriate lifetime is
        // inferred through the function signature.
        unsafe {
            let value = slice::from_raw_parts(self.ptr, nbytes);
            self.ptr = self.ptr.add(nbytes);
            value
        }
    }

    /// Reads `nbits` from the bit lookahead and consumes them.
    ///
    /// Use [`Self::refill_bits()`] to ensure enough bits are
    /// buffered for consumption before calling this method.
    ///
    /// # Panics
    ///
    /// Caller must check that enough bits are left for the read.
    #[inline]
    pub fn read_bits(&mut self, nbits: u32) -> u64 {
        let value = self.peek(nbits);
        self.consume(nbits);

        value
    }

    /// Reads an `nbits` sized value from the bit lookahead.
    ///
    /// Use [`Self::refill_bits()`] to ensure enough bits are
    /// buffered for consumption before calling this method.
    ///
    /// # Panics
    ///
    /// Caller must check that enough bits are left for the read.
    #[inline]
    pub fn read_signed_bits(&mut self, nbits: u32) -> i64 {
        sign_extend(self.read_bits(nbits), nbits)
    }

    /// Reads a [`bool`] value from the bit lookahead, if possible.
    ///
    /// Booleans are represented as individual bits.
    ///
    /// # Panics
    ///
    /// Caller must check that enough bits are left for the read.
    #[inline]
    pub fn bool(&mut self) -> bool {
        self.read_bits(1) != 0
    }

    // fn $read_fn(&mut self) -> $ty
    impl_read_literal! {
        /// Reads a [`u8`] value from the byte buffer, if possible.
        u8() -> u8,
        /// Reads a [`i8`] value from the byte buffer, if possible.
        i8() -> i8,

        /// Reads a [`u16`] value from the byte buffer, if possible.
        u16() -> u16,
        /// Reads a [`i16`] value from the byte buffer, if possible.
        i16() -> i16,

        /// Reads a [`u32`] value from the byte buffer, if possible.
        u32() -> u32,
        /// Reads a [`i32`] value from the byte buffer, if possible.
        i32() -> i32,

        /// Reads a [`u64`] value from the byte buffer, if possible.
        u64() -> u64,
        /// Reads a [`i64`] value from the byte buffer, if possible.
        i64() -> i64,
    }

    /// Reads a [`f32`] value from the byte buffer, if possible.
    ///
    /// # Panics
    ///
    /// Caller must check that enough bytes are left for the read.
    #[inline]
    pub fn f32(&mut self) -> f32 {
        f32::from_bits(self.u32())
    }

    /// Reads a [`f64`] value from the byte buffer, if possible.
    ///
    /// # Panics
    ///
    /// Caller must check that enough bytes are left for the read.
    #[inline]
    pub fn f64(&mut self) -> f64 {
        f64::from_bits(self.u64())
    }
}

impl Default for BitReader<'_> {
    fn default() -> Self {
        Self::new_borrowed(&[])
    }
}

// SAFETY: The `BitReader` API does not expose the internal pointers
// in ways which can compromise Rust's safety guarantees.
unsafe impl Send for BitReader<'_> {}
unsafe impl Sync for BitReader<'_> {}
