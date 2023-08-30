use std::{
    io::{self, Write},
    path::PathBuf,
};

use kobold_utils::{anyhow, fs};
use serde::Serialize;

pub fn json_to_stdout_or_output_file<T: Serialize>(
    out: Option<PathBuf>,
    v: &T,
) -> anyhow::Result<()> {
    // If we are given an output file, write a compact data to that.
    // Otherwise, pretty-print formatted JSON to stdout.
    if let Some(out) = out {
        let output = fs::open_file(out)?;
        let mut writer = io::BufWriter::new(output);

        serde_json::to_writer(&mut writer, v)?;
    } else {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        serde_json::to_writer_pretty(&mut stdout, v)?;
        writeln!(stdout)?;
    }

    Ok(())
}
