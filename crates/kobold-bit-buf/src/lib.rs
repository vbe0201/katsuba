//! Provides buffers for bit-level serialization and deserialization
//! of data.
//!
//! Every operation on types from this crate starts reading at a byte's
//! LSB, working towards the MSB. The exception are units of whole
//! bytes, where little endian ordering is used.

#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]

mod reader;
pub use reader::BitReader;

mod utils;
