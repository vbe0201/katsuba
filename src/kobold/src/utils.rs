mod io;
pub use io::*;

mod serde;
pub use serde::*;

#[inline]
pub fn human_bool(v: bool) -> &'static str {
    if v {
        "Yes"
    } else {
        "No"
    }
}
