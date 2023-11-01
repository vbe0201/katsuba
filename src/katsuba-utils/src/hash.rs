//! Commonly used dictionary hash functions.

/// Implementation of the String ID algorithm.
///
/// This algorithm is hand-rolled by KingsIsle.
#[inline(always)]
pub fn string_id(input: &[u8]) -> u32 {
    let mut state = 0;

    for (i, &b) in input.iter().enumerate() {
        let value = (b as i32) - 32;
        let shift = (i as u32 * 5) & 31;

        state ^= value.wrapping_shl(shift);
        if shift > 24 {
            state ^= value.wrapping_shr(32 - shift);
        }
    }

    state.unsigned_abs()
}

/// Implementation of the [DJB2] hash function.
///
/// [DJB2]: https://theartincode.stanis.me/008-djb2/
#[inline(always)]
pub fn djb2(input: &[u8]) -> u32 {
    let state: u32 = input
        .iter()
        .copied()
        .fold(5381, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u32));

    // NOTE: KI's implementation strips the MSB.
    state & (u32::MAX >> 1)
}
