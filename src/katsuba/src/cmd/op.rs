use std::{path::PathBuf, sync::Arc};

use clap::{Args, Subcommand};
use katsuba_object_property::{
    Value,
    serde::{BIND_MAGIC, Serializer, SerializerFlags, SerializerOptions},
};
use katsuba_types::PropertyFlags;

use super::Command;
use crate::{
    cli::{self, InputsOutputs, process},
    utils,
};

mod helpers;

/// Subcommand for working with ObjectProperty serialization.
#[derive(Debug, Args)]
pub struct ObjectProperty {
    #[clap(subcommand)]
    command: ObjectPropertyCommand,

    /// A list of paths to JSON type list files to use.
    ///
    /// These files are used to dynamically source the game's reflected type information
    /// from. They are crucial for interpreting the format of serialized data.
    ///
    /// Multiple files can be provided, which will have their entries merged into one
    /// type list.
    #[clap(short, long)]
    type_lists: Vec<PathBuf>,

    /// Serializer configuration flags to use.
    ///
    /// The provided integer value will be interpreted as a bitmask of `SerializerFlags`.
    /// This influences how the serializer will interpret the data it receives.
    ///
    /// When in doubt what to pick, try the default value of 0 first.
    #[clap(short, long, default_value_t = 0)]
    flags: u32,

    /// Property filter mask to use.
    ///
    /// This mask can be used to conditionally exclude properties
    /// of an object from the serialization.
    ///
    /// When in doubt what to pick, try the default value or 0.
    #[clap(short, long, default_value_t = 0x7)]
    mask: u32,

    /// Whether the object is serialized shallow.
    ///
    /// Deep mode contains additional metadata per object/property, and is the choice for all
    /// persistent state in game files.
    ///
    /// When in doubt what to pick, try the default value.
    #[clap(short, long, default_value_t = false)]
    shallow: bool,

    /// Whether manual compression should be assumed for the object.
    ///
    /// This is rarely used for state transferred over the network.
    ///
    /// When in doubt if you need this, just try it and see if it works.
    #[clap(short, long, default_value_t = false)]
    zlib_manual: bool,
}

#[derive(Debug, Subcommand)]
enum ObjectPropertyCommand {
    /// Deserializes ObjectProperty binary state to JSON.
    De {
        #[clap(flatten)]
        args: InputsOutputs,

        /// Represents enum members as human-readable strings in the output.
        ///
        /// Not to be confused with `-f 4` which causes serialized enum variants
        /// to be interpreted as strings in the binary data.
        ///
        /// Without this flag, enums will always be written in their integer
        /// representation to make processing the JSON output easier for tools.
        #[clap(short, long, default_value_t = false)]
        string_enums: bool,

        /// Skips properties with unknown types during deserialization.
        #[clap(short, long, default_value_t = false)]
        ignore_unknown_types: bool,
    },

    /// Serializes JSON state to ObjectProperty binary format.
    Ser {
        #[clap(flatten)]
        args: InputsOutputs,

        /// Prepends the `BINd` magic header and overwrites configuration
        /// to match what is used for persistent object state shipped in
        /// the game files.
        #[clap(long, default_value_t = false)]
        bind: bool,
    },
}

impl Command for ObjectProperty {
    fn handle(self) -> eyre::Result<()> {
        let type_list = Arc::new(helpers::merge_type_lists(self.type_lists)?);
        let mut options = SerializerOptions {
            flags: SerializerFlags::from_bits_truncate(self.flags),
            property_mask: PropertyFlags::from_bits_truncate(self.mask),
            shallow: self.shallow,
            manual_compression: self.zlib_manual,
            ..Default::default()
        };

        match self.command {
            ObjectPropertyCommand::De {
                args,
                string_enums,
                ignore_unknown_types,
            } => {
                let (inputs, outputs) = args.evaluate("de.xml")?;
                options.skip_unknown_types = ignore_unknown_types;

                let original_flags = options.flags;
                let original_shallow = options.shallow;
                process(
                    inputs,
                    outputs,
                    || {
                        Serializer::new(options, type_list.clone())
                            .map_err(|e| eyre::eyre!("invalid serializer configuration: {e}"))
                    },
                    |de, r| {
                        let buf = cli::read_to_vec(r)?;
                        let mut buf: &[u8] = &buf;

                        // If the data starts with the `BINd` magic, it is a game file.
                        // These always use a fixed base config so we set it here.
                        if buf.get(0..4) == Some(BIND_MAGIC) {
                            de.parts.options.shallow = false;
                            de.parts.options.flags = SerializerFlags::STATEFUL_FLAGS;

                            buf = buf.get(4..).unwrap();
                        } else {
                            de.parts.options.shallow = original_shallow;
                            de.parts.options.flags = original_flags;
                        }

                        let mut value = de.deserialize(buf)?;
                        if string_enums {
                            helpers::stringify_enums(&mut value, &de.parts.types);
                        }

                        Ok(value)
                    },
                    utils::write_as_json,
                )
            }

            ObjectPropertyCommand::Ser { args, bind } => {
                let (inputs, outputs) = args.evaluate("ser.bin")?;

                if bind {
                    options.shallow = false;
                    options.flags = SerializerFlags::STATEFUL_FLAGS
                        | SerializerFlags::COMPACT_LENGTH_PREFIXES
                        | SerializerFlags::HUMAN_READABLE_ENUMS
                        | SerializerFlags::WITH_COMPRESSION;
                }

                process(
                    inputs,
                    outputs,
                    || {
                        Serializer::new(options, type_list.clone())
                            .map_err(|e| eyre::eyre!("invalid serializer configuration: {e}"))
                    },
                    |ser, r| {
                        let data = cli::read_to_vec(r)?;
                        let value = Value::deserialize_obj(
                            &mut serde_json::Deserializer::from_slice(&data),
                            &ser.parts.types,
                        )
                        .map_err(|e| eyre::eyre!("failed to parse input: {e}"))?;

                        let mut buf = Vec::new();
                        ser.serialize_into(&value, &mut buf)?;

                        if bind {
                            Ok((BIND_MAGIC, buf))
                        } else {
                            Ok((&[], buf))
                        }
                    },
                    |inp, (prefix, v), out| {
                        utils::write_to_out(
                            inp,
                            |w| {
                                if !prefix.is_empty() {
                                    w.write_all(prefix)?;
                                }
                                w.write_all(&v)?;

                                Ok(())
                            },
                            out,
                        )
                    },
                )
            }
        }
    }
}
