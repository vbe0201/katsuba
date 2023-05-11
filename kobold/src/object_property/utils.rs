/// Implementation of the [DJB2] hash function.
///
/// [DJB2]: https://theartincode.stanis.me/008-djb2/
#[inline(always)]
pub const fn djb2(input: &str) -> u32 {
    let bytes = input.as_bytes();
    let mut state: u32 = 5381;

    let mut i = 0;
    while i < bytes.len() {
        // state * 33 + bytes[i]
        state = (state << 5)
            .wrapping_add(state)
            .wrapping_add(bytes[i] as u32);

        i += 1;
    }

    // XXX: KingsIsle's implementation strips the MSB.
    state & (u32::MAX >> 1)
}
