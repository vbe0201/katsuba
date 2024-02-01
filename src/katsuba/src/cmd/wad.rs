use std::{fs, path::PathBuf};

use clap::{Args, Subcommand};
use eyre::Context;
use katsuba_wad::{Archive, ArchiveBuilder};

use super::Command;
use crate::cli::{Bias, InputsOutputs, Processor, Reader};

mod extract;

/// Subcommand for working with KIWAD archives.
#[derive(Debug, Args)]
pub struct Wad {
    #[clap(subcommand)]
    command: WadCommand,
}

#[derive(Debug, Subcommand)]
enum WadCommand {
    /// Packs a directory into a KIWAD archive.
    Pack {
        /// The path to the input directory to pack.
        ///
        /// The directory will be recursively scanned and all its
        /// subdirectories and files will be added to the archive.
        ///
        /// Note that this does not follow symbolic links.
        input: PathBuf,

        /// The optional output file to write the archive to.
        ///
        /// If missing, a file named after the input directory will
        /// be created in the same parent directory.
        #[clap(short)]
        output: Option<PathBuf>,
    },

    /// Unpacks all files in a given KIWAD archive into a directory.
    Unpack {
        #[clap(flatten)]
        args: InputsOutputs,
    },
}

impl Command for Wad {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            WadCommand::Pack { input, output } => {
                if !input.is_dir() {
                    eyre::bail!("input for packing must be a directory");
                }

                let output = if let Some(output) = output {
                    output
                } else {
                    match input.parent() {
                        Some(p) => p.with_extension("wad"),
                        None => eyre::bail!("failed to determine parent directory of input"),
                    }
                };

                let mut builder = ArchiveBuilder::new(2, 0, &output).with_context(|| {
                    format!("failed to build output archive at '{}'", output.display())
                })?;

                for entry in walkdir::WalkDir::new(input) {
                    let entry = entry.context("failed to query input directory")?;
                    if !entry
                        .metadata()
                        .context("failed to obtain metadata for path")?
                        .is_file()
                    {
                        continue;
                    }

                    let path = entry.path();
                    let contents = fs::read(path)
                        .with_context(|| format!("failed to read file at '{}'", path.display()))?;

                    builder.add_file_compressed(entry.path(), &contents)?;
                }

                builder.finish()?;

                Ok(())
            }

            WadCommand::Unpack { args } => {
                let (inputs, outputs) = args.evaluate("")?;
                Processor::new(Bias::Threaded)?
                    .read_with(move |r, _| {
                        let res = match r {
                            Reader::Stdin(buf) => Archive::from_vec(buf.into_inner()),
                            Reader::File(_, f) => Archive::mmap(f.into_inner()),
                        };

                        res.map_err(Into::into)
                    })
                    .write_with(extract::extract_archive)
                    .process(inputs, outputs)
            }
        }
    }
}
