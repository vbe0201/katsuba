//! Shared utility code throughout the Kobold project.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

pub use anyhow;
#[cfg(feature = "binrw")]
pub use binrw;
#[cfg(feature = "libdeflater")]
pub use libdeflater;

pub mod align;
#[cfg(feature = "binrw")]
pub mod binrw_ext;
pub mod fs;
pub mod hash;
