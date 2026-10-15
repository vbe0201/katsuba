use clap::{Args, Subcommand};
use katsuba_bcd::Bcd as BcdFile;

use super::Command;
use crate::{
    cli::{self, InputsOutputs, process},
    utils,
};

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

    /// Serializes given JSON files into Binary Collision Data format.
    Ser(InputsOutputs),
}

impl Command for Bcd {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            BcdCommand::De(args) => {
                let (inputs, outputs) = args.evaluate("de.json")?;
                process(
                    inputs,
                    outputs,
                    || Ok(()),
                    |_, r| BcdFile::parse(r).map_err(Into::into),
                    utils::write_as_json,
                )
            }

            BcdCommand::Ser(args) => {
                let (inputs, outputs) = args.evaluate("se.bin")?;
                process(
                    inputs,
                    outputs,
                    || Ok(()),
                    |_, r| {
                        let data = cli::read_to_vec(r)?;
                        let value: BcdFile = serde_json::from_slice(&data)?;
                        let mut buf = Vec::new();
                        value.write(&mut buf)?;
                        Ok(buf)
                    },
                    utils::write_bytes,
                )
            }
        }
    }
}
