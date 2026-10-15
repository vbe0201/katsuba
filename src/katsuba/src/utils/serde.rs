use std::{
    fs,
    io::{self, BufWriter, IsTerminal, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::cli::OutputSource;

/// Serializes the given value to the respective output source.
///
/// This will produce valid JSON. If the output is a file or piped to
/// another application, a minified representation will be emitted.
///
/// Output to stdout always gets pretty-printed.
pub fn serialize_to_output_source<T: Serialize>(
    out: Option<PathBuf>,
    value: &T,
) -> eyre::Result<()> {
    if let Some(out) = out {
        let file = fs::File::create(&out)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, value)?;
        writer.flush()?;
    } else {
        let mut stdout = io::stdout().lock();
        if stdout.is_terminal() {
            serde_json::to_writer_pretty(&mut stdout, value)?;
            writeln!(stdout)?;
        } else {
            serde_json::to_writer(&mut stdout, value)?;
        }
    }

    Ok(())
}

/// Helper function for serializing a `T` value to an output source.
pub fn write_as_json<T: serde::Serialize>(
    inpath: Option<&Path>,
    value: T,
    out: OutputSource,
) -> eyre::Result<()> {
    match (out, inpath) {
        (OutputSource::Stdout, _) => serialize_to_output_source(None, &value),
        (OutputSource::File(path), _) => serialize_to_output_source(Some(path), &value),
        (OutputSource::Dir(mut out, suffix), Some(path)) => {
            // Create a file named after the input in the output directory.
            let infile = path.with_extension(suffix);
            out.push(infile.file_name().unwrap());

            serialize_to_output_source(Some(out), &value)
        }

        (OutputSource::Dir(..), None) => Err(eyre::eyre!(
            "output path for stdin input is a directory; please specify a file path instead"
        )),
    }
}
