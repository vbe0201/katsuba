use std::{
    ffi::{CStr},
};

use libc::{c_char};

use clap::{Args, ValueEnum};

use katsuba_utils::hash::*;

use super::Command;

/// Subcommand for hashing strings with common KingsIsle algorithms.
#[derive(Debug, Args)]
pub struct Hash {
    /// The hash algorithm to apply.
    #[clap(value_enum)]
    algo: Algo,

    /// The input string to hash.
    input: String,
}

/// The hash algorithm to apply.
#[derive(Clone, Debug, ValueEnum)]
enum Algo {
    /// The KingsIsle string ID algorithm.
    StringId,
    /// The DJB2 algorithm.
    Djb2,
}

impl Command for Hash {
    fn handle(self) -> eyre::Result<()> {
        hash(&self.input, self.algo)
    }
}

fn hash(input: &String, algo: Algo) -> eyre::Result<()> {
    let input = input.as_bytes();
    let hash = match algo {
        Algo::StringId => string_id(input),
        Algo::Djb2 => djb2(input),
    };

    println!("{hash}");
    Ok(())
}

/// The hash algorithm to apply. Duplicate of Algo enum.
///
/// This enum is accessible from C.
#[repr(C)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CAlgo {
    /// The KingsIsle string ID algorithm.
    StringId,
    /// The DJB2 algorithm.
    Djb2,
}
impl From<&CAlgo> for Algo {
    fn from(algo: &CAlgo) -> Self {
        match algo {
            CAlgo::StringId => Algo::StringId,
            CAlgo::Djb2 => Algo::Djb2,
        }
    }
}

#[no_mangle]
pub extern "C" fn hash_c(input: *const c_char, algo: CAlgo) -> bool {
    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_algo = Algo::from(&algo);

    hash(&rust_input, rust_algo).is_ok()
}
