use std::path::PathBuf;

use clap::{Args, Subcommand};
use kobold_utils::anyhow::{self, Context};
use kobold_wad::Archive;

mod extract;

#[derive(Debug, Args)]
pub struct Wad {
    #[clap(subcommand)]
    command: WadCommand,
}

#[derive(Debug, Subcommand)]
pub enum WadCommand {
    Unpack {
        input: PathBuf,

        #[clap(short, long)]
        out: Option<PathBuf>,

        #[clap(short, long)]
        verify_checksums: bool,
    },
}

pub fn process(wad: Wad) -> anyhow::Result<()> {
    match wad.command {
        WadCommand::Unpack {
            input,
            out,
            verify_checksums,
        } => {
            let mut out = match out {
                Some(out) => out,
                None => std::env::current_dir()?,
            };

            // TODO: Choose between heap and mmap based on file size.
            let archive = Archive::mmap(&input, verify_checksums)
                .with_context(|| format!("failed to open archive '{}'", input.display()))?;

            // At this point, we succeeded in opening `input` as a file.
            // Therefore, we know it has a parent path that is not None.
            out.push(input.file_stem().unwrap());

            extract::extract_all(archive, out)
        }
    }
}
