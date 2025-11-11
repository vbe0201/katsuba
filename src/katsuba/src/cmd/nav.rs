use std::{
    ffi::{CStr},
    path::PathBuf,
};

use libc::{c_char};

use clap::{Args, Subcommand, ValueEnum};
use katsuba_nav::{NavigationGraph, ZoneNavigationGraph};

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor, HYPHEN};

/// Subcommand for working with NAV data.
#[derive(Debug, Args)]
pub struct Nav {
    #[clap(subcommand)]
    command: NavCommand,

    /// The NAV type to assume for the given data.
    #[clap(value_enum, default_value_t = FileType::Nav)]
    file_type: FileType,
}

/// The NAV file type to use.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum FileType {
    /// Regular navigation graphs.
    Nav,
    /// Zone navigation graphs.
    ZoneNav,
}

#[derive(Debug, Subcommand)]
enum NavCommand {
    /// Deserializes given Navigation Graph files into JSON format.
    De(InputsOutputs),
}

impl Command for Nav {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            NavCommand::De(args) => {
                match self.file_type {
                    FileType::Nav => deserialize_nav(args),
                    FileType::ZoneNav => deserialize_zonenav(args),
                }
            }
        }
    }
}

fn deserialize_nav(args: InputsOutputs) -> eyre::Result<()> {
    let (inputs, outputs) = args.evaluate("de.json")?;
    Processor::new(Bias::Current)?
        .read_with(|r, _| NavigationGraph::parse(r).map_err(Into::into))
        .write_with(helpers::write_as_json)
        .process(inputs, outputs)
}

fn deserialize_zonenav(args: InputsOutputs) -> eyre::Result<()> {
    let (inputs, outputs) = args.evaluate("de.json")?;
    Processor::new(Bias::Current)?
        .read_with(|r, _| ZoneNavigationGraph::parse(r).map_err(Into::into))
        .write_with(helpers::write_as_json)
        .process(inputs, outputs)
}

#[no_mangle]
pub extern "C" fn nav_deserialize(
    input: *const c_char,
    output: *const c_char,
) -> bool {
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

    deserialize_nav(io).is_ok()
}

#[no_mangle]
pub extern "C" fn zonenav_deserialize(
    input: *const c_char,
    output: *const c_char,
) -> bool {
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

    deserialize_zonenav(io).is_ok()
}
