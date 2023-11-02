//! Library for interacting with KIWAD archives.
//!
//! Support for both reading and writing archive files is provided,
//! along with a flexible interface for decompressing into user
//! provided buffers.

#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]

mod archive;
pub use archive::*;

pub mod crc;

pub mod glob;

mod inflater;
pub use inflater::*;

pub mod types;
