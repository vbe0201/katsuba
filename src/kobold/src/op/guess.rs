use std::{
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use kobold_object_property::{
    serde::{self, BIND_MAGIC},
    Value,
};
use kobold_types::TypeList;
use kobold_utils::{anyhow, fs};

use crate::utils::*;

struct Report {
    value: anyhow::Result<Value>,
    opts: serde::SerializerOptions,
}

pub fn guess(
    opts: serde::SerializerOptions,
    types: Arc<TypeList>,
    path: PathBuf,
    no_value: bool,
) -> anyhow::Result<()> {
    let report = try_guess(opts, types, path)?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    print_status(&mut stdout, &report)?;
    writeln!(stdout)?;

    print_config(&mut stdout, &report)?;
    writeln!(stdout)?;

    print_value(&mut stdout, &report, no_value)?;

    Ok(())
}

fn try_guess(
    opts: serde::SerializerOptions,
    types: Arc<TypeList>,
    path: PathBuf,
) -> anyhow::Result<Report> {
    // Read the binary data from the given input file.
    // TODO: mmap?
    let data = fs::read(path)?;
    let mut data = data.as_slice();

    let mut de = serde::Serializer::with_guessed_options_from_base(opts, types, data)?;
    let mut res;

    if data.get(0..4) == Some(BIND_MAGIC) {
        data = data.get(4..).unwrap();
    }

    // First, try to deserialize with the current config.
    res = de.deserialize::<serde::PropertyClass>(data);
    if res.is_ok() {
        return Ok(Report {
            value: res,
            opts: de.parts.options,
        });
    }

    // If that doesn't work, check if it is realistic to retry with human-readable enums.
    if !opts.shallow && !opts.flags.contains(serde::SerializerFlags::STATEFUL_FLAGS) {
        de.parts.options.flags |= serde::SerializerFlags::HUMAN_READABLE_ENUMS;

        res = de.deserialize::<serde::PropertyClass>(data);
        if res.is_ok() {
            return Ok(Report {
                value: res,
                opts: de.parts.options,
            });
        }

        // Even if this bit is actually part of the config, we keep it the smallest
        // confirmed set of options to not confuse users with false positives.
        de.parts.options.flags &= !serde::SerializerFlags::HUMAN_READABLE_ENUMS;
    }

    Ok(Report {
        value: res,
        opts: de.parts.options,
    })
}

fn print_status<W: Write>(writer: &mut W, report: &Report) -> io::Result<()> {
    let text = match report.value.is_ok() {
        true => "Deserialization succeeded!",
        false => "Deserialization failed!",
    };

    writeln!(writer, "{}", text)
}

fn print_config<W: Write>(writer: &mut W, report: &Report) -> io::Result<()> {
    writeln!(writer, "Config:")?;
    writeln!(writer, "  Shallow? {}", human_bool(report.opts.shallow))?;
    writeln!(writer, "  Serializer flags: {:?}", report.opts.flags)?;
    writeln!(
        writer,
        "  Manually compressed? {}",
        human_bool(report.opts.manual_compression)
    )?;
    writeln!(writer, "  Property mask: {:?}", report.opts.property_mask)?;

    Ok(())
}

fn print_value<W: Write>(mut writer: W, report: &Report, no_value: bool) -> io::Result<()> {
    writeln!(writer, "Output:")?;
    match &report.value {
        Ok(v) if !no_value => {
            serde_json::to_writer_pretty(&mut writer, v)?;
            writeln!(writer)?;
        }

        Ok(_) => writeln!(writer, "<omitted>")?,

        Err(e) => {
            writeln!(writer, "Error: {}", e)?;
        }
    }

    Ok(())
}
