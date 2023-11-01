use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

use serde::Serialize;

use crate::executor::{Executor, Task};

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
        let mut buffer = ex.request_buffer(4 * 1024 * 1024);
        serde_json::to_writer(buffer.as_vec(), value)?;

        let task = Task::create_file(out, buffer.downgrade(), 0o666);
        for pending in ex.dispatch(task) {
            pending.result?;
        }
    } else {
        // Writing to stdout is generally only done for single elements and it also
        // doesn't have to go through slow `CloseHandle()` calls. Therefore, there
        // is no point in running this code on the executor.
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
