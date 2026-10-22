use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use eyre::Context;
use katsuba_wad::{Archive, glob};

use super::Command;
use crate::cli::{InputsOutputs, process};

mod extract;
mod pack;

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

        /// Specifies flags which should be set on the newly created
        /// KIWAD archive.
        ///
        /// Unless you know what you're doing, the use of this option
        /// is generally not recommended. The only exception to that
        /// rule is when repacking Root.wad, in which case a value of
        /// 1 must be set.
        #[clap(short, long, default_value_t = 0)]
        flags: u8,

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

        /// If specified, extracts only files matching the glob pattern.
        #[clap(short, long)]
        glob: Option<String>,
    },
}

impl Command for Wad {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            WadCommand::Pack {
                input,
                flags,
                output,
            } => {
                if !input.is_dir() {
                    eyre::bail!("input for packing must be a directory");
                }

                let output = match output {
                    Some(o) => o,
                    None => match input.file_name() {
                        Some(p) => {
                            let p: &Path = p.as_ref();
                            p.with_extension("wad")
                        }
                        None => eyre::bail!(
                            "failed to determine output file. consider specifying one with '-o'"
                        ),
                    },
                };

                pack::pack_archive(input, flags, output)
            }

            WadCommand::Unpack { args, glob } => {
                let (inputs, outputs) = args.evaluate("")?;
                let matcher = glob
                    .as_deref()
                    .map(glob::Matcher::new)
                    .transpose()
                    .context("invalid glob pattern")?;

                process(
                    inputs,
                    outputs,
                    || Ok(()),
                    |_, r| {
                        r.either(
                            |buf| Archive::from_vec(buf.into_inner()),
                            |f| Archive::mmap(f.into_inner()),
                        )
                        .map_err(Into::into)
                    },
                    |inp, a, out| extract::extract_archive(inp, a, out, matcher.as_ref()),
                )
            }
        }
    }
}
