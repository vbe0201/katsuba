use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use katsuba_object_property::{
    serde::{self, BIND_MAGIC},
    Value,
};
use katsuba_types::TypeList;

use crate::utils;

struct Report {
    value: Result<Value, serde::Error>,
    opts: serde::SerializerOptions,
}

pub fn guess(
    opts: serde::SerializerOptions,
    types: Arc<TypeList>,
    path: PathBuf,
    quiet: bool,
) -> eyre::Result<()> {
    let report = try_guess(opts, types, path)?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    write_status(&mut stdout, &report)?;
    writeln!(stdout)?;

    write_config(&mut stdout, &report)?;
    writeln!(stdout)?;

    write_value(&mut stdout, &report, quiet)?;

    Ok(())
}

fn try_guess(
    opts: serde::SerializerOptions,
    types: Arc<TypeList>,
    path: PathBuf,
) -> eyre::Result<Report> {
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

    // If that doesn't work, retry with human readable enums if that's realistic.
    if !opts.shallow && !opts.flags.contains(serde::SerializerFlags::STATEFUL_FLAGS) {
        de.parts.options.flags |= serde::SerializerFlags::HUMAN_READABLE_ENUMS;

        res = de.deserialize::<serde::PropertyClass>(data);
        if res.is_ok() {
            return Ok(Report {
                value: res,
                opts: de.parts.options,
            });
        }

        // This didn't work, so reset the bit.
        de.parts.options.flags &= !serde::SerializerFlags::HUMAN_READABLE_ENUMS;
    }

    Ok(Report {
        value: res,
        opts: de.parts.options,
    })
}

fn write_status<W: Write>(mut writer: W, report: &Report) -> io::Result<()> {
    let text = match report.value.is_ok() {
        true => "Deserialization succeeded!",
        false => "Deserialization failed!",
    };

    writeln!(writer, "{text}")
}

fn write_config<W: Write>(mut writer: W, report: &Report) -> io::Result<()> {
    writeln!(writer, "Config:")?;
    writeln!(
        writer,
        "  Shallow: {}",
        utils::human_bool(report.opts.shallow)
    )?;
    writeln!(writer, "  Serializer flags: {:?}", report.opts.flags)?;
    writeln!(
        writer,
        "  Manually compressed: {}",
        utils::human_bool(report.opts.manual_compression)
    )?;
    writeln!(writer, "  Property mask: {:?}", report.opts.property_mask)?;

    Ok(())
}

fn write_value<W: Write>(mut writer: W, report: &Report, quiet: bool) -> io::Result<()> {
    writeln!(writer, "Output:")?;
    match &report.value {
        Ok(v) if !quiet => {
            serde_json::to_writer_pretty(&mut writer, v)?;
            writeln!(writer)
        }

        Ok(_) => writeln!(writer, "<omitted>"),

        Err(e) => writeln!(writer, "Error: {e}"),
    }
}
