use clap::{Args, Subcommand};
use katsuba_bcd::Bcd as BcdFile;

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor};

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
                deserialize(args)
            }
        }
    }
}

pub fn deserialize(args: InputsOutputs) -> eyre::Result<()> {
    let (inputs, outputs) = args.evaluate("de.json")?;
    Processor::new(Bias::Current)?
        .read_with(|r, _| BcdFile::parse(r).map_err(Into::into))
        .write_with(helpers::write_as_json)
        .process(inputs, outputs)
}
