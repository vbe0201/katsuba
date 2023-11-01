use std::{path::PathBuf, sync::Arc};

use clap::{Args, Subcommand};
use kobold_object_property::serde;
use kobold_types::PropertyFlags;

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor};

mod guess;
mod utils;

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
    #[clap(short, long, default_value_t = 0)]
    flags: u32,

    /// Property filter mask to use.
    ///
    /// This mask can be used to conditionally exclude properties
    /// of an object from the serialization.
    ///
    /// When in doubt what to pick, try the default value or 0.
    #[clap(short, long, default_value_t = 0x18)]
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
        let mut options = serde::SerializerOptions {
            flags: serde::SerializerFlags::from_bits_truncate(self.flags),
            property_mask: PropertyFlags::from_bits_truncate(self.mask),
            shallow: self.shallow,
            manual_compression: self.zlib_manual,
            ..Default::default()
        };

        match self.command {
            ObjectPropertyCommand::De {
                args,
                ignore_unknown_types,
            } => {
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
                            de.parts.options.flags |= serde::SerializerFlags::STATEFUL_FLAGS;

                            buf = buf.get(4..).unwrap();
                        }

                        de.deserialize::<serde::PropertyClass>(buf)
                            .map_err(Into::into)
                    })
                    .write_with(helpers::write_as_json)
                    .process(inputs, outputs)
            }

            ObjectPropertyCommand::Guess { path, quiet } => {
                guess::guess(options, type_list, path, quiet)
            }
        }
    }
}
