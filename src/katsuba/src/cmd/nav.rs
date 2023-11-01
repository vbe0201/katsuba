use clap::{Args, Subcommand, ValueEnum};
use katsuba_nav::{NavigationGraph, ZoneNavigationGraph};

use super::Command;
use crate::cli::{helpers, Bias, InputsOutputs, Processor};

/// Subcommand for working with NAV data.
#[derive(Debug, Args)]
pub struct Nav {
    #[clap(subcommand)]
    command: NavCommand,

    /// The NAV type to assume for the given data.
    #[clap(value_enum, default_value_t = FileType::Nav)]
    file_type: FileType,
}

/// The NAV file type to use.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum FileType {
    /// Regular navigation graphs.
    Nav,
    /// Zone navigation graphs.
    ZoneNav,
}

#[derive(Debug, Subcommand)]
enum NavCommand {
    /// Deserializes given Navigation Graph files into JSON format.
    De(InputsOutputs),
}

impl Command for Nav {
    fn handle(self) -> eyre::Result<()> {
        match self.command {
            NavCommand::De(args) => {
                let (inputs, outputs) = args.evaluate("de.json")?;
                let processor = Processor::new(Bias::Current)?;

                match self.file_type {
                    FileType::Nav => processor
                        .read_with(|r, _| NavigationGraph::parse(r).map_err(Into::into))
                        .write_with(helpers::write_as_json)
                        .process(inputs, outputs),

                    FileType::ZoneNav => processor
                        .read_with(|r, _| ZoneNavigationGraph::parse(r).map_err(Into::into))
                        .write_with(helpers::write_as_json)
                        .process(inputs, outputs),
                }
            }
        }
    }
}
