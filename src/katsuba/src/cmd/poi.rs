use clap::{Args, Subcommand};
use katsuba_poi::Poi as PoiFile;

use super::Command;
use crate::{
    cli::{self, InputsOutputs, process},
    utils,
};

/// Subcommand for working with POI data.
#[derive(Debug, Args)]
pub struct Poi {
    #[clap(subcommand)]
    command: PoiCommand,
}

#[derive(Debug, Subcommand)]
enum PoiCommand {
    /// Deserializes given Point of Interest files into JSON format.
    De(InputsOutputs),

    /// Serializes given JSON files into Point of Interest format.
    Ser(InputsOutputs),
}

impl Command for Poi {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            PoiCommand::De(args) => {
                let (inputs, outputs) = args.evaluate("de.json")?;
                process(
                    inputs,
                    outputs,
                    || Ok(()),
                    |_, r| PoiFile::parse(r).map_err(Into::into),
                    utils::write_as_json,
                )
            }

            PoiCommand::Ser(args) => {
                let (inputs, outputs) = args.evaluate("se.bin")?;
                process(
                    inputs,
                    outputs,
                    || Ok(()),
                    |_, r| {
                        let data = cli::read_to_vec(r)?;
                        let value: PoiFile = serde_json::from_slice(&data)?;
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
