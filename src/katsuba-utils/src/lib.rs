//! Shared utility code throughout the Katsuba project.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

#[cfg(feature = "binrw")]
pub use binrw;
#[cfg(feature = "libdeflater")]
pub use libdeflater;
pub use thiserror;

pub mod align;
#[cfg(feature = "binrw")]
pub mod binrw_ext;
pub mod hash;
