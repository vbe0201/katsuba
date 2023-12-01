//! Implementation of a worker pool for file I/O processing.
//!
//! # Motivation
//!
//! Katsuba's primary use case for multithreading is writing files from
//! in-memory structures to disk.
//!
//! This requires coping with various nuisances in different OSes, e.g.
//! Windows running Defender during `CloseHandle()` calls.
//!
//! # Design
//!
//! By design, the main thread produces tasks for the workers and handles
//! their results.
//!
//! Since all kinds of manipulations we do to our structures in memory
//! complete at a much faster rate than any file I/O, all the processing
//! mostly happens on the main thread and can still generate reasonable
//! loads onto the executor.

mod executor;
pub use executor::*;

mod memory;
pub use memory::Buffer;
