//! Hints for the compiler that affect code optimization.

/// A branch prediction hint that indicates the code path is unlikely to
/// be used. This serves as a substitute for [`std::hint::cold_path`]
/// until that is stabilized.
#[cold]
#[inline(always)]
pub fn cold_path() {}
