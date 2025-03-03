//! Shared utility code throughout the Katsuba project.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

pub mod align;
#[cfg(feature = "binrw-ext")]
pub mod binrw_ext;
pub mod hash;
