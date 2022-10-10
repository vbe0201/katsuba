//! Implementation of Kobold core functionality.
//!
//! This provides the structures and parsers for all
//! file formats we plan to support, without doing
//! any I/O by itself.
//!
//! This then further serves as the core for the WASM
//! interface and the CLI.

pub mod formats;

pub mod object_property;
