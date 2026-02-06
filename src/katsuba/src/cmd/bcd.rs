use clap::{Args, Subcommand};
use katsuba_bcd::Bcd as BcdFile;

use super::Command;
use crate::cli::{InputsOutputs, helpers, process_par};

/// Subcommand for working with BCD data.
#[derive(Debug, Args)]
pub struct Bcd {
    #[clap(subcommand)]
    command: BcdCommand,
}

#[derive(Debug, Subcommand)]
enum BcdCommand {
    /// Deserializes given Binary Collision Data files into JSON format.
    De(InputsOutputs),
}

impl Command for Bcd {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            BcdCommand::De(args) => {
                let (inputs, outputs) = args.evaluate("de.json")?;
                process_par(
                    inputs,
                    outputs,
                    || (),
                    |_, r| BcdFile::parse(r).map_err(Into::into),
                    helpers::write_as_json,
                )
            }
        }
    }
}
