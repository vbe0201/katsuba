use std::{fs::File, path::PathBuf};

use super::WadCommand;

mod crc;

mod ctx;
use ctx::WadContext;

mod inflater;

/// Processes the user's requested WAD command.
pub fn process(cmd: WadCommand) -> anyhow::Result<()> {
    match cmd {
        WadCommand::Unpack {
            input,
            out,
            verify_checksums,
        } => {
            let archive = File::open(&input)?;
            let out = out.unwrap_or_else(|| {
                let mut new = PathBuf::new();
                // We opened `input` as a file prior to this, so
                // we can be sure that it actually is a file here.
                new.push(input.parent().unwrap());
                new.push(input.file_stem().unwrap());
                new
            });

            let mut ctx = WadContext::map_for_unpack(&archive, out, verify_checksums)?;
            ctx.extract_all()
        }
    }
}
