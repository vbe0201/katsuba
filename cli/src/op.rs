use std::{fs::File, io, path::PathBuf, sync::Arc};

use anyhow::bail;
use clap::{Args, Subcommand, ValueEnum};
use kobold::object_property::{
    CoreObject, Deserializer, DeserializerOptions, PropertyClass, PropertyFlags, SerializerFlags,
    TypeList, Value,
};
use memmap2::MmapOptions;
use serde_json::json;

mod display;
use self::display::*;

#[derive(Args)]
pub struct ObjectProperty {
    #[clap(subcommand)]
    command: ObjectPropertyCommand,

    /// The ObjectProperty class type to use.
    #[clap(value_enum, default_value_t = ClassType::Basic)]
    class_type: ClassType,

    /// The path to the type list json file.
    #[clap(short, long)]
    type_list: PathBuf,

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
#[derive(Clone, ValueEnum, PartialEq)]
enum ClassType {
    /// Ordinary PropertyClasses.
    Basic,
    /// CoreObject subclasses.
    Core,
    /// BINd XML files.
    Bind,
}

#[derive(Subcommand)]
enum ObjectPropertyCommand {
    /// Deserializes the given ObjectProperty binary state
    /// and prints its JSON representation to stdout.
    De {
        /// Path to the file to deserialize.
        input: PathBuf,
    },
}

/// Processes the user's requested ObjectProperty command.
pub fn process(mut op: ObjectProperty) -> anyhow::Result<()> {
    let types = {
        let file = File::open(&op.type_list)?;
        TypeList::from_reader(io::BufReader::new(file))?
    };
    let mut options = DeserializerOptions {
        flags: SerializerFlags::from_bits_truncate(op.flags),
        property_mask: PropertyFlags::from_bits_truncate(op.mask),
        shallow: op.shallow,
        manual_compression: op.zlib_manual,
        recursion_limit: u8::MAX,
    };

    match op.command {
        ObjectPropertyCommand::De { input } => {
            let file = File::open(input)?;
            // SAFETY: `file` remains unmodified for the entire duration of the mapping.
            let data = unsafe { MmapOptions::new().populate().map(&file)? };

            if op.class_type == ClassType::Bind {
                options.shallow = false;
                options.flags |= SerializerFlags::STATEFUL_FLAGS;

                op.class_type = ClassType::Basic;
            }

            let data = if op.shallow {
                &data
            } else {
                let (magic, data) = data.split_at(4);
                if magic != b"BINd" {
                    bail!("File does not start with BINd magic");
                }

                data
            };

            let stdout = io::stdout();
            let mut handle = stdout.lock();

            let obj = match op.class_type {
                ClassType::Basic => Deserializer::<PropertyClass>::new(options, Arc::new(types))
                    .deserialize(data)?,
                ClassType::Core => {
                    Deserializer::<CoreObject>::new(options, Arc::new(types)).deserialize(data)?
                }

                _ => unreachable!(),
            };
            pretty_print_value(obj, &mut handle)
        }
    }
}

fn build_json_object(value: Value) -> serde_json::Value {
    match value {
        Value::Unsigned(i) => json!(i),
        Value::Signed(i) => json!(i),
        Value::Float(f) => json!(f),
        Value::Bool(b) => json!(b),

        Value::String(str) => json!(CxxStr(&str)),
        Value::WString(wstr) => json!(CxxWStr(&wstr)),

        Value::Enum(str) => json!(str),

        Value::List(list) => list.into_iter().map(build_json_object).collect(),

        Value::Color { r, g, b, a } => json!([r, g, b, a]),
        Value::Vec3 { x, y, z } => json!([x, y, z]),
        Value::Quat { x, y, z, w } => json!([x, y, z, w]),
        Value::Euler { pitch, roll, yaw } => json!([pitch, roll, yaw]),
        Value::Mat3x3 { i, j, k } => json!([i, j, k]),
        Value::Point { xy } => {
            json!([build_json_object(xy.0), build_json_object(xy.1)])
        }
        Value::Size { wh } => {
            json!([build_json_object(wh.0), build_json_object(wh.1)])
        }
        Value::Rect { inner } => json!([
            build_json_object(inner.0),
            build_json_object(inner.1),
            build_json_object(inner.2),
            build_json_object(inner.3)
        ]),

        Value::Object(object) => {
            let name = json!(object.name);
            let mut map: serde_json::Map<_, _> = object
                .into_iter()
                .map(|(name, value)| (name, build_json_object(value)))
                .collect();

            map.insert("__type".to_string(), name);

            serde_json::Value::Object(map)
        }

        Value::Empty => json!(null),
    }
}

fn pretty_print_value(value: Value, handle: &mut io::StdoutLock) -> anyhow::Result<()> {
    let json = build_json_object(value);
    serde_json::to_writer_pretty(handle, &json).map_err(Into::into)
}
