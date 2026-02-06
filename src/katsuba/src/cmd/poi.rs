use clap::{Args, Subcommand};
use katsuba_poi::Poi as PoiFile;

use super::Command;
use crate::cli::{InputsOutputs, helpers, process_par};

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
}

impl Command for Poi {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            PoiCommand::De(args) => {
                let (inputs, outputs) = args.evaluate("de.json")?;
                process_par(
                    inputs,
                    outputs,
                    || (),
                    |_, r| PoiFile::parse(r).map_err(Into::into),
                    helpers::write_as_json,
                )
            }
        }
    }
}
