use std::{io, mem, ops::Deref};

use bitvec::{domain::Domain, prelude::*};
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
                let value = self.data.$read::<LittleEndian>()?;
                Ok(value)
            }
        )*
    };
}

/// A binary reader that provides bit-level operations on
/// byte-sized input.
#[derive(Default)]
pub struct BitReader {
    data: BitVec<u8, Lsb0>,
}

impl BitReader {
    /// Creates a new bit reader over a given slice of bytes.
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: BitVec::from_vec(data),
        }
    }

    /// Attempts to read a single bit from this buffer.
    pub fn read_bit(&mut self) -> io::Result<bool> {
        if self.data.is_empty() {
            Err(premature_eof())
        } else {
            Ok(self.data.remove(0))
        }
    }

    /// Attempts to read `n` bits from this buffer and returns a bit slice
    /// holding the data on success.
    #[allow(unsafe_code)]
    pub fn read_bits(&mut self, n: usize) -> io::Result<BitVec<u8, Lsb0>> {
        if n <= self.len() {
            let mut chunk = self.data.split_off(n);
            mem::swap(&mut chunk, &mut self.data);

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
            .for_each(|(i, b)| result |= (b as usize) << i);

        Ok(result)
    }

    #[inline]
    pub(super) fn realign_to_byte(&mut self) {
        let pad_bits = self.data.len() - align_down(self.data.len(), u8::BITS as _);
        self.data = self.data.split_off(pad_bits);
        self.data.force_align();
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
        self.data.read_u8()
    }

    /// Loads an [`i8`] value.
    #[inline]
    pub fn load_i8(&mut self) -> io::Result<i8> {
        self.realign_to_byte();
        self.data.read_i8()
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

impl Deref for BitReader {
    type Target = BitSlice<u8, Lsb0>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
