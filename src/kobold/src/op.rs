use std::{path::PathBuf, sync::Arc};

use clap::{Args, Subcommand, ValueEnum};
use kobold_object_property::serde;
use kobold_types::PropertyFlags;
use kobold_utils::anyhow;

mod de;
mod guess;
mod utils;

#[derive(Debug, Args)]
pub struct ObjectProperty {
    #[clap(subcommand)]
    command: ObjectPropertyCommand,

    /// A list of paths to JSON type list files to use.
    #[clap(short, long)]
    type_lists: Vec<PathBuf>,

    /// Serializer configuration flags to use.
    #[clap(short, long, default_value_t = 0)]
    flags: u32,

    /// Property filter mask to use.
    #[clap(short, long, default_value_t = 0x18)]
    mask: u32,

    /// Whether the object is serialized shallow.
    #[clap(short, long, default_value_t = false)]
    shallow: bool,

    /// Whether the object is manually zlib-compressed.
    #[clap(short, long, default_value_t = false)]
    zlib_manual: bool,
}

/// The class type to work with.
#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum ClassType {
    /// Ordinary PropertyClasses.
    Basic,
    /// CoreObject subclasses.
    Core,
}

#[derive(Debug, Subcommand)]
enum ObjectPropertyCommand {
    /// Deserializes ObjectProperty binary state.
    De {
        /// Path to the file to deserialize.
        path: PathBuf,

        /// Skips properties with unknown types during deserialization.
        #[clap(short, long, default_value_t = false)]
        ignore_unknown_types: bool,

        /// The ObjectProperty class type to use.
        #[clap(value_enum, default_value_t = ClassType::Basic)]
        class_type: ClassType,

        /// The path to the output file, if desired.
        #[clap(short, long)]
        out: Option<PathBuf>,
    },

    /// Deserializes ObjectProperty binary state while trying to guess
    /// its config.
    Guess {
        /// Path to the file to deserialize.
        path: PathBuf,

        /// Whether the deserialized value should be pretty-printed in the
        /// event of success.
        ///
        /// Since this can get pretty spammy and distract from the actual
        /// serializer configuration used, users may not want this.
        #[clap(short, long)]
        no_value: bool,
    },
}

pub fn process(op: ObjectProperty) -> anyhow::Result<()> {
    let type_list = utils::merge_type_lists(op.type_lists)?;
    let mut options = serde::SerializerOptions {
        flags: serde::SerializerFlags::from_bits_truncate(op.flags),
        property_mask: PropertyFlags::from_bits_truncate(op.mask),
        shallow: op.shallow,
        manual_compression: op.zlib_manual,
        recursion_limit: u8::MAX,
        skip_unknown_types: false,
    };

    match op.command {
        ObjectPropertyCommand::De {
            path,
            ignore_unknown_types,
            class_type,
            out,
        } => {
            options.skip_unknown_types = ignore_unknown_types;

            let de = serde::Serializer::new(options, Arc::new(type_list))?;
            de::process(de, path, out, class_type, serde::Quiet)
        }

        ObjectPropertyCommand::Guess { path, no_value } => guess::guess(options, Arc::new(type_list), path, no_value),
    }
}
