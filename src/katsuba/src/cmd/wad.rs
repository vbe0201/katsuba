use clap::{Args, Subcommand};
use katsuba_wad::Archive;

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
    /// Unpacks all files in a given KIWAD archive into a directory.
    Unpack {
        #[clap(flatten)]
        args: InputsOutputs,

        /// Verifies the CRC32 checksums for every file in the archive.
        ///
        /// Since most files are zlib-compressed and are automatically
        /// verified by Adler32, this only has relevance for a very
        /// limited number of files.
        ///
        /// Therefore, this option is disabled by default.
        #[clap(short, long, default_value_t = false)]
        verify_checksums: bool,
    },
}

impl Command for Wad {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            WadCommand::Unpack {
                args,
                verify_checksums,
            } => {
                let (inputs, outputs) = args.evaluate("")?;
                Processor::new(Bias::Threaded)?
                    .read_with(move |r, _| {
                        let res = match r {
                            Reader::Stdin(buf) => {
                                Archive::from_vec(buf.into_inner(), verify_checksums)
                            }
                            Reader::File(_, f) => Archive::mmap(f.into_inner(), verify_checksums),
                        };

                        res.map_err(Into::into)
                    })
                    .write_with(extract::extract_archive)
                    .process(inputs, outputs)
            }
        }
    }
}
