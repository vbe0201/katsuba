use std::marker::PhantomData;

use bitvec::prelude::*;
use funty::Integral;

use crate::utils::{align_up, IntCast};

macro_rules! write_bytes_to_bitslice {
    ($bs:ident, $buf:expr) => {
        // SAFETY: The iterator is consumed while only ever holding
        // onto one `slot` at the same time.
        unsafe { $bs.chunks_exact_mut(u8::BITS as _).remove_alias() }
            .zip($buf)
            .for_each(|(slot, byte)| slot.store_be(byte));
    };
}

macro_rules! impl_write_literal {
    ($($(#[$doc:meta])* $write_fn:ident($ty:ty)),* $(,)?) => {
        $(
            $(#[$doc])*
            #[inline]
            pub fn $write_fn(&mut self, v: $ty) {
                self.realign_to_byte();

                let len = self.inner.len();
                self.inner.resize(len + <$ty>::BITS as usize, false);

                // SAFETY: `len` was the former end of the buffer before reallocation,
                // now it denotes where the newly allocated memory starts.
                let bs = unsafe { self.inner.get_unchecked_mut(len..) };
                write_bytes_to_bitslice!(bs, v.to_le_bytes());
            }
        )*
    };
}

/// A reserved length prefix, which can be committed at a later time.
pub struct LengthMarker<I>(usize, usize, PhantomData<I>);

/// A buffer which enables bit-based serialization of data.
///
/// Quantities of multiple bytes (except byte slices) are always written
/// in little-endian byte ordering. Individual bit writing starts at
/// the LSB of the byte, working towards the MSB.
#[derive(Clone, Debug, Default)]
pub struct BitWriter {
    inner: BitVec<u8, Lsb0>,
}

impl BitWriter {
    /// Creates a new, empty [`BitWriter`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of bits in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Indicates whether the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets a view of the buffer's storage as a byte slice.
    #[inline]
    pub fn view(&self) -> &[u8] {
        self.inner.as_raw_slice()
    }

    /// Consumes the [`BitWriter`] and returns the byte buffer.
    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.inner.into_vec()
    }

    /// Writes a single bit to the buffer.
    #[inline]
    pub fn write_bit(&mut self, b: bool) {
        self.inner.push(b);
    }

    /// Writes all bits in `buf` to the buffer.
    #[inline]
    pub fn write_bits(&mut self, buf: &BitSlice<u8, Lsb0>) {
        self.inner.extend_from_bitslice(buf);
    }

    /// Writes a given number of bits from `value` to the buffer.
    #[inline]
    pub fn write_bitint<I: Integral>(&mut self, value: I, nbits: usize) {
        let len = self.inner.len();
        self.inner.resize(len + nbits, false);

        // SAFETY: `len` was the former end of the buffer before reallocation,
        // now it denotes where the newly allocated memory starts.
        let bs = unsafe { self.inner.get_unchecked_mut(len..) };
        bs.store_le(value);
    }

    #[inline]
    fn realign_to_byte(&mut self) {
        let new_len = align_up(self.inner.len(), u8::BITS as _);
        self.inner.resize(new_len, false);
    }

    /// Writes the bytes from `buf` to the buffer.
    ///
    /// This will force-align the underlying buffer to full byte boundaries
    /// before writing; effectively filling skipped bits with zeroes.
    #[inline]
    pub fn write_bytes(&mut self, buf: &[u8]) {
        self.realign_to_byte();

        let len = self.inner.len();
        self.inner.resize(len + buf.len() * 8, false);

        // SAFETY: `len` was the former end of the buffer before reallocation,
        // now it denotes where the newly allocated memory starts.
        let bs = unsafe { self.inner.get_unchecked_mut(len..) };
        write_bytes_to_bitslice!(bs, buf.iter().copied());
    }

    /// Reserves a length prefix of a literal type `I` at the current
    /// buffer position.
    ///
    /// The returned [`LengthMarker`] can be passed to [`BitWriter::commit_len`]
    /// at a later time to patch back the amount of bits that have been
    /// written since marking.
    pub fn mark_len<I: Integral>(&mut self) -> LengthMarker<I> {
        // Back up the current bit position for length calculation.
        let bit_start = self.len();

        // Write the `I` placeholder value and remember its start offset.
        self.realign_to_byte();
        let bit_pos = self.len();
        self.inner.resize(bit_pos + I::BITS as usize, false);

        LengthMarker(bit_start, bit_pos, PhantomData)
    }

    /// Commits a previously reserved length prefix to the buffer.
    pub fn commit_len<I: Integral>(&mut self, len: LengthMarker<I>)
    where
        usize: IntCast<I>,
        I::Bytes: IntoIterator<Item = u8>,
    {
        let LengthMarker(bit_start, bit_pos, ..) = len;

        // Calculate the length prefix value.
        let prefix: I = (self.inner.len() - bit_start).cast_as();

        // SAFETY: We created `LengthMarker` ourselves, so we
        // can trust `bit_pos` to be a valid offset.
        let bs = unsafe { self.inner.get_unchecked_mut(bit_pos..) };
        write_bytes_to_bitslice!(bs, prefix.to_le_bytes());
    }

    /// Writes a given [`bool`] value to the buffer.
    ///
    /// Booleans are represented as single bits and do not force a realign
    /// to full byte boundaries.
    #[inline]
    pub fn bool(&mut self, v: bool) {
        self.write_bit(v);
    }

    impl_write_literal! {
        /// Writes a given [`u8`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u8(u8),
        /// Writes a given [`i8`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i8(i8),

        /// Writes a given [`u16`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u16(u16),
        /// Writes a given [`i16`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i16(i16),

        /// Writes a given [`u32`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u32(u32),
        /// Writes a given [`i32`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i32(i32),

        /// Writes a given [`u64`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u64(u64),
        /// Writes a given [`i64`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i64(i64),
    }

    /// Writes the bits of a given [`f32`] value to the buffer.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// writing; effectively filling remaining bits with zeroes.
    #[inline]
    pub fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }

    /// Writes the bits of a given [`f64`] value to the buffer.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// writing; effectively filling remaining bits with zeroes.
    #[inline]
    pub fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
}
