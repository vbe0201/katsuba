use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

use katsuba_executor::{Executor, Task};
use serde::Serialize;

/// Serializes the given value to the respective output source.
///
/// This will produce valid JSON. If the output is a file or piped to
/// another application, a minified representation will be emitted.
///
/// Output to stdout always gets pretty-printed.
///
/// This will use the given executor to dispatch the work, so a call
/// to [`Executor::join`] is necessary to ensure all tasks complete.
pub fn serialize_to_output_source<T: Serialize>(
    ex: &Executor,
    out: Option<PathBuf>,
    value: &T,
) -> eyre::Result<()> {
    if let Some(out) = out {
        // We use a blanket size for buffers since they will grow as needed anyway.
        // But also most files shouldn't be this large so the memory can be reused.
        let buffer = ex.request_buffer(1024 * 1024, |buf| serde_json::to_writer(buf, value))?;

        let task = Task::create_file(out, buffer, 0o666);
        for pending in ex.dispatch(task) {
            pending?;
        }
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
