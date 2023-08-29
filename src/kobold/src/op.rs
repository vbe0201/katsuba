use std::{path::PathBuf, sync::Arc};

use clap::{Args, Subcommand, ValueEnum};
use kobold_object_property::serde;
use kobold_types::PropertyFlags;
use kobold_utils::anyhow;

mod de;
mod display;
mod format;
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
        } => {
            options.skip_unknown_types = ignore_unknown_types;

            let de = serde::Deserializer::new(options, Arc::new(type_list), serde::Quiet)?;
            de::process(de, path, class_type)
        }
    }
}
