use std::{fs::File, io, path::PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use kobold::formats::nav::{NavigationGraph, ZoneNavigationGraph};

#[derive(Args)]
pub struct Nav {
    #[clap(subcommand)]
    command: NavCommand,

    /// The NAV file type to use.
    #[clap(value_enum, default_value_t = FileType::Nav)]
    file_type: FileType,
}

/// The NAV file type to use.
#[derive(Clone, ValueEnum)]
enum FileType {
    /// Regular navigation graphs.
    Nav,
    /// Zone navigation graphs.
    ZoneNav,
}

#[derive(Subcommand)]
enum NavCommand {
    /// Deserializes the given NAV file and prints its
    /// JSON representation to stdout.
    De {
        /// Path to the file to deserialize.
        input: PathBuf,
    },
}

/// Processes the user's requested NAV command.
pub fn process(nav: Nav) -> anyhow::Result<()> {
    match nav.command {
        NavCommand::De { input } => {
            let file = File::open(input)?;
            let mut reader = io::BufReader::new(file);

            let stdout = io::stdout();

            match nav.file_type {
                FileType::Nav => {
                    let nav = NavigationGraph::parse(&mut reader)?;
                    serde_json::to_writer_pretty(stdout.lock(), &nav)?;
                }

                FileType::ZoneNav => {
                    let nav = ZoneNavigationGraph::parse(&mut reader)?;
                    serde_json::to_writer_pretty(stdout.lock(), &nav)?;
                }
            }

            Ok(())
        }
    }
}
