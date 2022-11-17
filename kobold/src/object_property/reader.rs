use std::io;

use bitvec::{
    domain::Domain,
    prelude::*,
    ptr::{bitslice_from_raw_parts, Const},
};
use byteorder::{LittleEndian, ReadBytesExt};

#[inline(always)]
const fn align_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

fn premature_eof() -> io::Error {
    io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "Premature EOF reached while trying to read more data than available",
    )
}

macro_rules! impl_read_literal {
    ($(#[$doc:meta] $de:ident() = $read:ident() -> $ty:ty),* $(,)*) => {
        $(
            #[$doc]
            #[inline]
            pub fn $de(&mut self) -> io::Result<$ty> {
                self.realign_to_byte();

                let mut data = self.data();
                let v = data.$read::<LittleEndian>()?;
                self.current = data.as_bitptr();

                Ok(v)
            }
        )*
    };
}

/// A binary reader that provides bit-level operations on
/// byte-sized input.
pub struct BitReader {
    start: BitPtr<Const, u8, Lsb0>,
    current: BitPtr<Const, u8, Lsb0>,
    len: usize,
    cap: usize,
}

impl BitReader {
    /// Creates a new bit reader over a given slice of bytes.
    pub fn new(data: Vec<u8>) -> Self {
        let (ptr, len, cap) = BitVec::from_vec(data).into_raw_parts();
        let ptr = ptr.to_const();

        Self {
            start: ptr,
            current: ptr,
            len,
            cap,
        }
    }

    pub(crate) fn dangling() -> Self {
        Self {
            start: BitPtr::DANGLING,
            current: BitPtr::DANGLING,
            len: 0,
            cap: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        // First, compute the offset between start and current.
        // SAFETY: Both pointers share the same provenance.
        let bit_offset = unsafe { self.current.offset_from(self.start) };

        // Now we can subtract this offset from the total
        // length we store to obtain the relative length.
        // Offset is positive because current >= start.
        self.len - bit_offset as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn data<'a, 'b: 'a>(&'a self) -> &'b BitSlice<u8, Lsb0> {
        // Make sure we aren't accidentally operating on a
        // BitReader obtained from `BitReader::dangling`.
        debug_assert_ne!(self.current, BitPtr::<Const, u8, Lsb0>::DANGLING);

        // SAFETY: current is a valid pointer derived from start
        // and self.len() does the relative length calculation.
        // Thus, we get a valid slice spanning initialized memory
        // in the same allocated object, which is fine to deref.
        let ptr = bitslice_from_raw_parts(self.current, self.len());
        unsafe { &*ptr }
    }

    /// Attempts to read a single bit from this buffer.
    pub fn read_bit(&mut self) -> io::Result<bool> {
        let data = self.data();

        let (first, remainder) = data.split_first().ok_or_else(premature_eof)?;
        self.current = remainder.as_bitptr();

        Ok(*first)
    }

    /// Attempts to read `n` bits from this buffer and returns a bit slice
    /// holding the data on success.
    #[allow(unsafe_code)]
    pub fn read_bits(&mut self, n: usize) -> io::Result<&BitSlice<u8, Lsb0>> {
        let data = self.data();

        if n <= data.len() {
            // SAFETY: We did the bounds check ourselves.
            let (chunk, remainder) = unsafe { data.split_at_unchecked(n) };
            self.current = remainder.as_bitptr();

            Ok(chunk)
        } else {
            Err(premature_eof())
        }
    }

    /// Attempts to read `n` bits from this buffer into a new [`usize`] value
    /// and returns it on success.
    ///
    /// When using this to read signed values, additional sign extension will
    /// be required.
    pub fn read_value_bits(&mut self, n: usize) -> io::Result<usize> {
        let mut result = 0;
        self.read_bits(n)?
            .into_iter()
            .enumerate()
            .for_each(|(i, b)| result |= (*b as usize) << i);

        Ok(result)
    }

    #[inline]
    pub(super) fn realign_to_byte(&mut self) {
        let data = self.data();

        // SAFETY: `pad_bits` is guaranteed to be <= `data.len()`.
        let pad_bits = data.len() - align_down(data.len(), u8::BITS as _);

        let (_, remainder) = unsafe { data.split_at_unchecked(pad_bits) };
        self.current = remainder.as_bitptr();
    }

    /// Attempts to read `n` bytes from the internal buffer and returns a slice
    /// of those bytes on success.
    pub fn read_bytes(&mut self, n: usize) -> io::Result<Vec<u8>> {
        let nbits = n * u8::BITS as usize;

        self.read_bits(nbits).map(|bs| match bs.domain() {
            Domain::Region {
                head: _,
                body,
                tail: _,
            } => body.to_owned(),
            Domain::Enclave(elem) => vec![elem.load_value()],
        })
    }

    /// Loads an [`u8`] value.
    #[inline]
    pub fn load_u8(&mut self) -> io::Result<u8> {
        self.realign_to_byte();

        let mut data = self.data();
        let v = data.read_u8();
        self.current = data.as_bitptr();

        v
    }

    /// Loads an [`i8`] value.
    #[inline]
    pub fn load_i8(&mut self) -> io::Result<i8> {
        self.realign_to_byte();

        let mut data = self.data();
        let v = data.read_i8();
        self.current = data.as_bitptr();

        v
    }

    impl_read_literal! {
        /// Loads an [`u16`] value in little endian byte order.
        load_u16() = read_u16() -> u16,
        /// Loads an [`u32`] value in little endian byte order.
        load_u32() = read_u32() -> u32,
        /// Loads an [`u64`] value in little endian byte order.
        load_u64() = read_u64() -> u64,

        /// Loads an [`i16`] value in little endian byte order.
        load_i16() = read_i16() -> i16,
        /// Loads an [`i32`] value in little endian byte order.
        load_i32() = read_i32() -> i32,

        /// Loads an [`f32`] value in little endian byte order.
        load_f32() = read_f32() -> f32,
        /// Loads an [`f64`] value in little endian byte order.
        load_f64() = read_f64() -> f64,
    }
}

#[allow(unused)]
const fn assert_send<T: Send + Sync>() {}
const _: () = assert_send::<BitVec<u8, Lsb0>>();

// SAFETY: BitVec is Send and Sync as per above assertion,
// so its raw parts might be as well.
unsafe impl Send for BitReader {}
unsafe impl Sync for BitReader {}

impl Drop for BitReader {
    fn drop(&mut self) {
        if self.start != BitPtr::<Const, u8, Lsb0>::DANGLING {
            // SAFETY: This originally was a mut pointer, so we just
            // restore the write access we have previously taken away.
            let data = unsafe { self.start.to_mut() };

            // Let BitVec do its own cleanup.
            // SAFETY: We checked for dangling pointers, so we know
            // these are parts obtained from a valid BitVec.
            let _ = unsafe { BitVec::from_raw_parts(data, self.len, self.cap) };
        }
    }
}
