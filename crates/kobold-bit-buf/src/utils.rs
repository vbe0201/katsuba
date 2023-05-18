/// Aligns `value` down to the next multiple of `align`.
///
/// # Panics
///
/// Panics in debug mode when `align` is not a power of two.
#[inline(always)]
pub const fn align_down(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    value & !(align - 1)
}
