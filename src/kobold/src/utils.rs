use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

use kobold_utils::{anyhow, fs};
use serde::Serialize;

pub fn human_bool(v: bool) -> &'static str {
    if v {
        "Yes"
    } else {
        "No"
    }
}

pub fn json_to_stdout_or_output_file<T: Serialize>(
    out: Option<PathBuf>,
    v: &T,
) -> anyhow::Result<()> {
    // If we are given an output file, write a compact data to that.
    // Otherwise, print the data to stdout.
    if let Some(out) = out {
        let output = fs::open_file(out)?;
        let mut writer = io::BufWriter::new(output);

        serde_json::to_writer(&mut writer, v)?;
    } else {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        // Determine whether we should pretty-print the output for terminals
        // or if another program is going to process what we give it.
        if stdout.is_terminal() {
            serde_json::to_writer_pretty(&mut stdout, v)?;
            writeln!(stdout)?;
        } else {
            serde_json::to_writer(&mut stdout, v)?;
        }
    }

    Ok(())
}
