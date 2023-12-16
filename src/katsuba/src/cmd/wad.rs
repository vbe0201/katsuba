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
    },
}

impl Command for Wad {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
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
