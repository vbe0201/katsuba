use clap::{Args, Subcommand};
use katsuba_poi::Poi as PoiFile;

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor};

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
                return deserialize(args)
            }
        }
    }
}

pub fn deserialize(args: InputsOutputs) -> eyre::Result<()> {
    let (inputs, outputs) = args.evaluate("de.json")?;
    Processor::new(Bias::Current)?
        .read_with(|r, _| PoiFile::parse(r).map_err(Into::into))
        .write_with(helpers::write_as_json)
        .process(inputs, outputs)
}
