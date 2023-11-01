//! Provides high-performance bit level manipulation of data.
//!
//! Bit-based serialization is fairly common in Katsuba's areas
//! of focus. Hardware often lacks dedicated support to deal
//! with it, making performance a concern for implementations.
//!
//! Therefore, this crate aims to provide reusable components
//! that take advantage of today's superscalar, out-of-order
//! processors.
//!
//! # Implementation
//!
//! The implementation itself is based on Fabian Giesen's
//! [writeups], specifically variant 4.
//!
//! Additional techniques are employed which take advantage
//! of byte-sized reads starting at byte-aligned boundaries.
//!
//! [writeups]: https://fgiesen.wordpress.com/2018/02/20/reading-bits-in-far-too-many-ways-part-2/

#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]

mod reader;
pub use reader::BitReader;

mod writer;
pub use writer::BitWriter;

pub mod utils;
