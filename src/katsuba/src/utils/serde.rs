use std::{
    fs,
    io::{self, BufWriter, IsTerminal, Write},
    path::PathBuf,
};

use serde::Serialize;

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
