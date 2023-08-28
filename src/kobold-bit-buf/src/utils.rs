//! Miscellaneous utilities for working with bits.

/// Sign-extends an `nbits` wide value to [`i64`].
#[inline]
pub fn sign_extend(value: u64, nbits: u32) -> i64 {
    let shift = u64::BITS - nbits;
    (value << shift) as i64 >> shift
}
