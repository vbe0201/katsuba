use std::{
    ffi::CStr,
    path::PathBuf,
    sync::Arc
};

use libc::{c_char};

use clap::{Args, Subcommand};
use katsuba_object_property::serde::{self, SerializerOptions};
use katsuba_types::{PropertyFlags, TypeList};

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor, HYPHEN};

mod guess;
mod utils;

pub const DEFAULT_FLAGS: u32 = 0;
pub const DEFAULT_MASK: u32 = 0x18;

/// Subcommand for working with ObjectProperty serialization.
#[derive(Debug, Args)]
pub struct ObjectProperty {
    #[clap(subcommand)]
    command: ObjectPropertyCommand,

    /// A list of paths to JSON type list files to use.
    ///
    /// These files are used to dynamically source the game's
    /// reflected type information from. They are crucial for
    /// interpreting the format of serialized data.
    ///
    /// Multiple files can be provided, which will have their
    /// entries merged into one type list.
    #[clap(short, long)]
    type_lists: Vec<PathBuf>,

    /// Serializer configuration flags to use.
    ///
    /// These flags are configuration bits for the serializer
    /// instance and influence how data is interpreted.
    ///
    /// When in doubt what to pick, try 0 or using the guess command.
    #[clap(short, long, default_value_t = DEFAULT_FLAGS)]
    flags: u32,

    /// Property filter mask to use.
    ///
    /// This mask can be used to conditionally exclude properties
    /// of an object from the serialization.
    ///
    /// When in doubt what to pick, try the default value or 0.
    #[clap(short, long, default_value_t = DEFAULT_MASK)]
    mask: u32,

    /// Whether the object is serialized shallow.
    ///
    /// Deep mode contains additional metadata per object/property,
    /// and is the choice for all persistent game files.
    ///
    /// When in doubt what to pick, try the default value or the
    /// guess command.
    #[clap(short, long, default_value_t = false)]
    shallow: bool,

    /// Whether manual compression should be assumed for the object.
    ///
    /// This is rarely used for state transferred over the network.
    ///
    /// When in doubt what to pick, try the default value or the
    /// guess command.
    #[clap(short, long, default_value_t = false)]
    zlib_manual: bool,

    /// Whether we should use only the djb2 hash (Pirate101)
    #[clap(short, long, default_value_t = false)]
    djb2_only: bool,
}

#[derive(Debug, Subcommand)]
enum ObjectPropertyCommand {
    /// Deserializes ObjectProperty binary state to JSON.
    De {
        #[clap(flatten)]
        args: InputsOutputs,

        /// Skips properties with unknown types during deserialization.
        #[clap(short, long, default_value_t = false)]
        ignore_unknown_types: bool,
    },

    /// Attempts to deserialize ObjectProperty binary state
    /// into JSON with a guessed serializer config.
    ///
    /// This means that you shouldn't have to provide most of
    /// the options in the base command to get working output.
    ///
    /// Note however that this command is not a golden bullet;
    /// it will report the configuration it tried regardless of
    /// success or failure and you may want to tweak it manually.
    Guess {
        /// Path to the file to deserialize.
        path: PathBuf,

        /// Whether the deserialized value should be pretty-printed
        /// on success.
        ///
        /// Since the output can get pretty spammy and distract from
        /// the serializer configuration report, users may want to
        /// disable this when analyzing unknown configuration.
        #[clap(short, long)]
        quiet: bool,
    },
}

impl Command for ObjectProperty {
    fn handle(self) -> eyre::Result<()> {
        let type_list = Arc::new(utils::merge_type_lists(self.type_lists)?);
        let options = serde::SerializerOptions {
            flags: serde::SerializerFlags::from_bits_truncate(self.flags),
            property_mask: PropertyFlags::from_bits_truncate(self.mask),
            shallow: self.shallow,
            manual_compression: self.zlib_manual,
            djb2_only: self.djb2_only,
            ..Default::default()
        };

        match self.command {
            ObjectPropertyCommand::De {
                args,
                ignore_unknown_types,
            } => {
                return deserialize(args, type_list, options, ignore_unknown_types)
            }

            ObjectPropertyCommand::Guess { path, quiet } => {
                guess::guess(options, type_list, path, quiet)
            }
        }
    }
}

fn deserialize(
    args: InputsOutputs,
    type_list: Arc<TypeList>,
    mut options: SerializerOptions,
    ignore_unknown_types: bool,
) -> eyre::Result<()> {
    let (inputs, outputs) = args.evaluate("de.xml")?;

    options.skip_unknown_types = ignore_unknown_types;
    let mut de = serde::Serializer::new(options, type_list)?;

    Processor::new(Bias::Current)?
        .read_with(move |mut r, ex| {
            let buf = r.get_buffer(ex)?;
            let mut buf: &[u8] = &buf;

            // If the data starts with the `BINd` magic, it is a game file.
            // These always use a fixed base config so we set it here.
            if buf.get(0..4) == Some(serde::BIND_MAGIC) {
                de.parts.options.shallow = false;
                de.parts.options.flags = serde::SerializerFlags::STATEFUL_FLAGS;

                buf = buf.get(4..).unwrap();
            }

            de.deserialize::<serde::PropertyClass>(buf)
                .map_err(Into::into)
        })
        .write_with(helpers::write_as_json)
        .process(inputs, outputs)
}

fn get_type_lists_from_c(type_lists: *const *const c_char) -> eyre::Result<Arc<TypeList>> {
    let mut type_list_paths = Vec::new();
    let mut i = 0;
    loop {
        let current_type_list = unsafe { *type_lists.add(i) };
        if current_type_list.is_null() {
            break // Null pointer indicates end of array
        }

        match unsafe { CStr::from_ptr(current_type_list) }.to_str() {
            Ok(rust_str) => type_list_paths.push(PathBuf::from(rust_str)),
            Err(_) => continue,
        };

        i += 1;
    }

    match utils::merge_type_lists(type_list_paths) {
        Ok(combined_type_list) => Ok(Arc::new(combined_type_list)),
        Err(report) => Err(report),
    }
}

#[no_mangle]
pub extern "C" fn op_deserialize(
    input: *const c_char,
    output: *const c_char,
    type_lists: *const *const c_char,
    flags: u32,
    mask: u32,
    shallow: bool,
    manual_compression: bool,
    djb2_only: bool,
    ignore_unknown_types: bool,
) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    if input.is_null() || type_lists.is_null() {
        return false
    }

    // Create the InputsOutputs
    let rust_input = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(rust_str) => rust_str.to_owned(),
        Err(_) => return false,
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

    // Create the type_list
    let type_list = match get_type_lists_from_c(type_lists) {
        Ok(list) => list,
        Err(_) => return false,
    };

    // Set the options
    let options = serde::SerializerOptions {
        flags: serde::SerializerFlags::from_bits_truncate(flags),
        property_mask: PropertyFlags::from_bits_truncate(mask),
        shallow: shallow,
        manual_compression: manual_compression,
        djb2_only: djb2_only,
        ..Default::default()
    };

    deserialize(io, type_list, options, ignore_unknown_types).is_ok()
}

#[no_mangle]
pub extern "C" fn op_guess(
    path: *const c_char,
    type_lists: *const *const c_char,
    flags: u32,
    mask: u32,
    shallow: bool,
    manual_compression: bool,
    djb2_only: bool,
    quiet: bool,
) -> bool {

    // Set the options
    let options = serde::SerializerOptions {
        flags: serde::SerializerFlags::from_bits_truncate(flags),
        property_mask: PropertyFlags::from_bits_truncate(mask),
        shallow: shallow,
        manual_compression: manual_compression,
        djb2_only: djb2_only,
        ..Default::default()
    };

    // Create the type_list
    let type_list = match get_type_lists_from_c(type_lists) {
        Ok(list) => list,
        Err(_) => return false,
    };

    let rust_path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(rust_str) => PathBuf::from(rust_str),
        Err(_) => return false,
    };

    guess::guess(options, type_list, rust_path, quiet).is_ok()
}
