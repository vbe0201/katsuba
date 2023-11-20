mod io;
pub use io::*;

mod serde;
pub use serde::*;

/// Converts a [`bool`] value into a human-readable description.
#[inline]
pub fn human_bool(v: bool) -> &'static str {
    if v {
        "Yes"
    } else {
        "No"
    }
}
