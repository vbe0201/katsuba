use funty::Integral;

/// Aligns `value` up to the next multiple of `align`.
///
/// # Panics
///
/// Panics in debug mode when `align` is not a power of two.
#[inline(always)]
pub const fn align_up(value: usize, align: usize) -> usize {
    align_down(value + align - 1, align)
}

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

/// Defines casting behavior from one integer type to another.
pub trait IntCast<T: Integral> {
    /// Casts `self` to target type `T`.
    ///
    /// When `self` is too large to fit `T`, bits will be chopped
    /// off. Expect this to behave like an `as` cast.
    fn cast_as(self) -> T;
}

macro_rules! impl_intcast_from_usize {
    ($($ty:ty),* $(,)*) => {
        $(
            impl IntCast<$ty> for usize {
                #[inline(always)]
                fn cast_as(self) -> $ty {
                    self as $ty
                }
            }
        )*
    };
}

impl_intcast_from_usize! {
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
}
