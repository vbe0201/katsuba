use std::{io, mem};

use bitvec::{
    domain::Domain,
    field::BitField,
    prelude::*,
    ptr::{bitslice_from_raw_parts, Const},
};
use funty::Integral;

use crate::utils::align_down;

#[cold]
#[inline(never)]
fn premature_eof() -> io::Error {
    io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "attempted to read more bits than available",
    )
}

macro_rules! impl_read_literal {
    ($($(#[$doc:meta])* $read_fn:ident() -> $ty:ty),* $(,)?) => {
        $(
            $(#[$doc])*
            #[inline]
            pub fn $read_fn(&mut self) -> io::Result<$ty> {
                self.realign_to_byte();
                self.read_bitint::<$ty>(<$ty>::BITS as _)
            }
        )*
    };
}

/// A buffer which enables bit-based deserialization of data.
///
/// Quantities of multiple bytes (except byte slices) are always read
/// in little-endian byte ordering. Individual bit reading starts at
/// the LSB of the byte, working towards the MSB.
///
/// Ownership of the data buffer to read from must be transferred to
/// this type to make FFI easier. [`BitReader::into_inner`] can be used
/// to reclaim the memory.
pub struct BitReader {
    // Raw parts of a `BitVec`.
    start: BitPtr<Const, u8, Lsb0>,
    len: usize,
    cap: usize,

    // Current position into the spanned bit region.
    current: BitPtr<Const, u8, Lsb0>,
}

impl BitReader {
    /// Constructs a new [`BitReader`] over an owned byte buffer.
    pub fn new(data: Vec<u8>) -> Self {
        Self::new_from_bitvec(BitVec::from_vec(data))
    }

    /// Creates a [`BitReader`] over an empty buffer.
    ///
    /// Trying to read from it will always fail.
    pub fn empty() -> Self {
        Self::new_from_bitvec(BitVec::EMPTY)
    }

    fn new_from_bitvec(bv: BitVec<u8, Lsb0>) -> Self {
        let (start, len, cap) = bv.into_raw_parts();
        let start = start.to_const();

        Self {
            start,
            len,
            cap,

            current: start,
        }
    }

    /// Consumes the [`BitReader`] and returns the unmodified byte buffer
    /// it was constructed over.
    ///
    /// This method should be used to reclaim transferred ownership of the
    /// allocated memory for the buffer.
    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        // SAFETY: By construction invariant, we can reverse `new_from_bitvec`.
        let bv = unsafe { BitVec::from_raw_parts(self.start.to_mut(), self.len, self.cap) };
        mem::forget(self);

        bv.into_vec()
    }

    /// Gets the total bits in the underlying buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Indicates whether the underlying buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets the remaining bits available in the buffer.
    #[inline]
    pub fn remaining(&self) -> usize {
        // First, compute the offset between start and current.
        // SAFETY: Both pointers are derived from the same object.
        let offset = unsafe { self.current.offset_from(self.start) };

        // Now we can subtract this offset from the total length to obtain
        // a relative, positive length value (due to current >= start).
        self.len - offset as usize
    }

    #[inline]
    fn data(&self) -> *const BitSlice<u8, Lsb0> {
        // SAFETY: `current` is a valid pointer derived from `start` and
        // `.remaining()` does the relative length calculation. Thus, we
        // get a valid slice spanning initialized memory in one allocation.
        bitslice_from_raw_parts(self.current, self.remaining())
    }

    /// Attempts to read a single bit from the buffer.
    #[inline]
    pub fn read_bit(&mut self) -> io::Result<bool> {
        let data = unsafe { &*self.data() };

        let (first, remainder) = data.split_first().ok_or_else(premature_eof)?;
        self.current = remainder.as_bitptr();

        Ok(*first)
    }

    /// Attempts to read `n` bits from this buffer and returns a bit
    /// slice holding the data on success.
    #[inline]
    pub fn read_bits(&mut self, nbits: usize) -> io::Result<&BitSlice<u8, Lsb0>> {
        let data = unsafe { &*self.data() };

        if nbits <= data.len() {
            // SAFETY: We did the bounds check ourselves.
            let (chunk, remainder) = unsafe { data.split_at_unchecked(nbits) };
            self.current = remainder.as_bitptr();

            Ok(chunk)
        } else {
            Err(premature_eof())
        }
    }

    /// Attempts to read a given number of bits from the buffer into an integer.
    #[inline]
    pub fn read_bitint<I: Integral>(&mut self, nbits: usize) -> io::Result<I> {
        debug_assert!(
            0 < nbits && nbits <= I::BITS as _,
            "bit count overflows capacity of target type"
        );

        self.read_bits(nbits).map(|bs| bs.load_le())
    }

    #[inline]
    fn realign_to_byte(&mut self) {
        let data = unsafe { &*self.data() };

        // SAFETY: `pad_bits` is guaranteed to be <= `data.len()`.
        let pad_bits = data.len() - align_down(data.len(), u8::BITS as _);
        let (_, remainder) = unsafe { data.split_at_unchecked(pad_bits) };

        self.current = remainder.as_bitptr();
    }

    /// Attempts to read a given number of bytes from the buffer and
    /// returns a byte slice to them.
    ///
    /// This will force-align the buffer to full byte boundaries
    /// before reading; effectively discarding the remaining bits
    /// until then.
    #[inline]
    pub fn read_bytes(&mut self, nbytes: usize) -> io::Result<&[u8]> {
        self.realign_to_byte();
        self.read_bits(nbytes * u8::BITS as usize)
            .map(|bs| match bs.domain() {
                // SAFETY: Since we're starting at byte boundary and only reading
                // full bytes, we don't have to consider any partial elements.
                Domain::Region { body, .. } => body,
                Domain::Enclave(..) => unsafe { std::hint::unreachable_unchecked() },
            })
    }

    /// Reads a [`bool`] value from the buffer, if possible.
    ///
    /// Booleans are represented as individual bits and do not force a
    /// realign of the buffer to full byte boundaries.
    #[inline]
    pub fn bool(&mut self) -> io::Result<bool> {
        self.read_bit()
    }

    // fn $read_fn(&mut self) -> io::Result<$ty>
    impl_read_literal! {
        /// Reads a [`u8`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u8() -> u8,
        /// Reads a [`i8`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i8() -> i8,

        /// Reads a [`u16`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u16() -> u16,
        /// Reads a [`i16`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i16() -> i16,

        /// Reads a [`u32`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u32() -> u32,
        /// Reads a [`i32`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i32() -> i32,

        /// Reads a [`u64`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u64() -> u64,
        /// Reads a [`i64`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i64() -> i64,
    }

    /// Reads a [`f32`] value from the buffer, if possible.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// reading; effectively discarding the remaining bits until then.
    #[inline]
    pub fn f32(&mut self) -> io::Result<f32> {
        self.u32().map(f32::from_bits)
    }

    /// Reads a [`f64`] value from the buffer, if possible.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// reading; effectively discarding the remaining bits until then.
    #[inline]
    pub fn f64(&mut self) -> io::Result<f64> {
        self.u64().map(f64::from_bits)
    }
}

#[allow(unused)]
const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<BitVec<u8, Lsb0>>();

// SAFETY: None of the `BitReader` methods compromise `BitVec`'s integrity.
// Therefore, its parts are safe to share just like the composed type.
unsafe impl Send for BitReader {}
unsafe impl Sync for BitReader {}

impl Drop for BitReader {
    fn drop(&mut self) {
        // Re-build the `BitVec` from raw parts and let it do its own cleanup.
        unsafe {
            // SAFETY: By construction invariant, we can reverse `new_from_bitvec`.
            let _ = BitVec::from_raw_parts(self.start.to_mut(), self.len, self.cap);
        }
    }
}
