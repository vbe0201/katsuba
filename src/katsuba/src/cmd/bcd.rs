use std::{
    ffi::{CStr},
    path::{PathBuf},
};

use libc::{c_char};

use clap::{Args, Subcommand};
use katsuba_bcd::Bcd as BcdFile;

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor, HYPHEN};

/// Subcommand for working with BCD data.
#[derive(Debug, Args)]
pub struct Bcd {
    #[clap(subcommand)]
    command: BcdCommand,
}

#[derive(Debug, Subcommand)]
enum BcdCommand {
    /// Deserializes given Binary Collision Data files into JSON format.
    De(InputsOutputs),
}

impl Command for Bcd {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            BcdCommand::De(args) => {
                deserialize(args)
            }
        }
    }
}

fn deserialize(args: InputsOutputs) -> eyre::Result<()> {
    let (inputs, outputs) = args.evaluate("de.json")?;
    Processor::new(Bias::Current)?
        .read_with(|r, _| BcdFile::parse(r).map_err(Into::into))
        .write_with(helpers::write_as_json)
        .process(inputs, outputs)
}

#[no_mangle]
pub extern "C" fn bcd_deserialize(input: *const c_char, output: *const c_char) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    deserialize(io).is_ok()
}
