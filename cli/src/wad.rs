use std::{fs::File, path::PathBuf};

use clap::{Args, Subcommand};

mod crc;

mod ctx;
use ctx::WadContext;

mod inflater;

#[derive(Args)]
pub struct Wad {
    #[clap(subcommand)]
    command: WadCommand,
}

#[derive(Subcommand)]
pub enum WadCommand {
    /// Unpacks a given KIWAD archive file.
    Unpack {
        /// Path to the archive file to unpack.
        input: Vec<PathBuf>,

        /// An optional path to extract the archived files to.
        ///
        /// By default, a new directory named after the input
        /// file will be created in the same directory and
        /// files will be extracted to it.
        #[clap(short, long)]
        out: Option<PathBuf>,

        /// Optionally validates CRCs for all files in the
        /// archive.
        ///
        /// Most of the time, files are stored compressed and
        /// naturally have corruption resilience by nature of
        /// the algorithm in use.
        ///
        /// It is recommended to use this setting only for
        /// testing custom archives for correctness.
        #[clap(short, long)]
        verify_checksums: bool,
    },
}

/// Processes the user's requested WAD command.
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

            for file in input {
                let archive = File::open(&file)?;
                // We opened `file` as a file prior to this, so
                // we can be sure it actually is a file here.
                let out = out.join(file.file_stem().unwrap());

                let mut ctx = WadContext::map_for_unpack(&archive, out, verify_checksums)?;
                ctx.extract_all()?;
            }

            Ok(())
        }
    }
}
