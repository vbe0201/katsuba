use std::{io, path::PathBuf};

use anyhow::{self, Context};
use clap::{Args, Subcommand, ValueEnum};
use kobold_nav::{NavigationGraph, ZoneNavigationGraph};

use crate::{fs, utils};

#[derive(Debug, Args)]
pub struct Nav {
    #[clap(subcommand)]
    command: NavCommand,

    /// The NAV file type to assume.
    #[clap(value_enum, default_value_t = FileType::Nav)]
    file_type: FileType,
}

/// The NAV file type to use.
#[derive(Clone, Debug, ValueEnum)]
enum FileType {
    /// Regular navigation graphs.
    Nav,
    /// Zone navigation graphs.
    ZoneNav,
}

#[derive(Debug, Subcommand)]
enum NavCommand {
    /// Deserializes a given navigation graph file into JSON format.
    De {
        /// Path to the file to deserialize.
        input: PathBuf,

        /// An optional path to an output JSON file.
        ///
        /// If this is not supplied, Kobold will print the JSON data to
        /// stdout.
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
}

pub fn process(nav: Nav) -> anyhow::Result<()> {
    match nav.command {
        NavCommand::De { input, output } => {
            let input = fs::open_file(input)?;
            let mut reader = io::BufReader::new(input);

            match nav.file_type {
                FileType::Nav => {
                    let nav = NavigationGraph::parse(&mut reader)
                        .with_context(|| "failed to parse nav file")?;
                    utils::json_to_stdout_or_output_file(output, &nav)
                }

                FileType::ZoneNav => {
                    let zonenav = ZoneNavigationGraph::parse(&mut reader)
                        .with_context(|| "failed to parse zonenav file")?;
                    utils::json_to_stdout_or_output_file(output, &zonenav)
                }
            }
        }
    }
}
