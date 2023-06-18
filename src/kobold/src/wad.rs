use std::path::PathBuf;

use clap::{Args, Subcommand};
use kobold_wad::Archive;

mod extract;

#[derive(Debug, Args)]
pub struct Wad {
    #[clap(subcommand)]
    pub command: WadCommand,
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
            let out = match out {
                Some(out) => out,
                None => std::env::current_dir()?,
            };

            extract::extract_all(
                Archive::mmap(&input, verify_checksums)?,
                out.join(input.file_stem().unwrap()),
            )
        }
    }
}
