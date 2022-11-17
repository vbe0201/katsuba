use std::{
    fs::File,
    io::{self, Write},
    sync::Arc,
};

use anyhow::bail;
use kobold::object_property::{
    Deserializer, DeserializerOptions, PropertyClass, PropertyFlags, SerializerFlags, TypeList,
    Value,
};
use memmap2::MmapOptions;

use super::{ObjectProperty, ObjectPropertyCommand};

mod display;
use self::display::*;

/// Processes the user's requested ObjectProperty command.
pub fn process(op: ObjectProperty) -> anyhow::Result<()> {
    let types = {
        let file = File::open(&op.type_list)?;
        TypeList::from_reader(io::BufReader::new(file))?
    };
    let options = DeserializerOptions {
        flags: SerializerFlags::from_bits_truncate(op.flags),
        property_mask: PropertyFlags::from_bits_truncate(op.mask),
        shallow: op.shallow,
        manual_compression: op.zlib_manual,
        recursion_limit: u8::MAX,
    };
    let mut deserializer = Deserializer::<PropertyClass>::new(options, Arc::new(types));

    match op.command {
        ObjectPropertyCommand::De { input } => {
            let file = File::open(&input)?;
            // SAFETY: `file` remains unmodified for the entire duration of the mapping.
            let data = unsafe { MmapOptions::new().populate().map(&file)? };

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

            let obj = deserializer.deserialize(data)?;
            pretty_print_value(&obj, &mut handle, None).map_err(Into::into)
        }
    }
}

fn pretty_print_value(
    value: &Value,
    handle: &mut io::StdoutLock,
    list_property: Option<&str>,
) -> io::Result<()> {
    match value {
        Value::Unsigned(i) => write!(handle, "{i}"),
        Value::Signed(i) => write!(handle, "{i}"),
        Value::Float(f) => write!(handle, "{f}"),
        Value::Bool(b) => write!(handle, "{b}"),

        Value::String(str) => write!(handle, "{}", CxxStr(str)),
        Value::WString(wstr) => write!(handle, "{}", CxxWStr(wstr)),

        Value::Enum(str) => write!(handle, "{str}"),

        Value::List(list) => {
            let list_property = list_property.unwrap();
            list.iter().try_for_each(|value| {
                writeln!(handle, "<{list_property}>")?;
                pretty_print_value(value, handle, None)?;
                writeln!(handle, "</{list_property}>")
            })
        }

        Value::Color { r, g, b, a } => write!(handle, "#{r:X}{g:X}{b:X}{a:X}"),
        Value::Vec3 { x, y, z } => write!(handle, "{x},{y},{z}"),
        Value::Quat { x, y, z, w } => write!(handle, "{x},{y},{z},{w}"),
        Value::Euler { pitch, roll, yaw } => write!(handle, "{pitch},{roll},{yaw}"),
        Value::Mat3x3 {
            i: [i0, i1, i2],
            j: [j0, j1, j2],
            k: [k0, k1, k2],
        } => write!(handle, "{i0},{i1},{i2},{j0},{j1},{j2},{k0},{k1},{k2}"),
        Value::Point { xy } => {
            pretty_print_value(&xy.0, handle, None)?;
            write!(handle, ",")?;
            pretty_print_value(&xy.1, handle, None)
        }
        Value::Size { wh } => {
            pretty_print_value(&wh.0, handle, None)?;
            write!(handle, ",")?;
            pretty_print_value(&wh.1, handle, None)
        }
        Value::Rect { inner } => {
            pretty_print_value(&inner.0, handle, None)?;
            write!(handle, ",")?;
            pretty_print_value(&inner.1, handle, None)?;
            write!(handle, ",")?;
            pretty_print_value(&inner.2, handle, None)?;
            write!(handle, ",")?;
            pretty_print_value(&inner.3, handle, None)
        }

        Value::Object(object) => {
            writeln!(handle, "<Objects>")?;
            writeln!(handle, "<Class Name={}>", object.name)?;
            for (name, value) in object {
                match value {
                    value @ Value::List(_) => pretty_print_value(value, handle, Some(name))?,
                    value => {
                        write!(handle, "<{name}>")?;
                        pretty_print_value(value, handle, None)?;
                        write!(handle, "</{name}>")?;
                    }
                }
            }
            writeln!(handle, "</Class>")?;
            writeln!(handle, "</Objects>")
        }

        Value::Empty => Ok(()),
    }
}
