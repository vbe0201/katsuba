use std::{fs::File, io, path::PathBuf};

use clap::{Args, Subcommand};
use kobold::formats::poi::Poi as PoiFormat;

#[derive(Args)]
pub struct Poi {
    #[clap(subcommand)]
    command: PoiCommand,
}

#[derive(Subcommand)]
enum PoiCommand {
    /// Deserializes the given POI file and prints its
    /// JSON representation to stdout.
    De {
        /// Path to the file to deserialize.
        input: PathBuf,
    },
}

/// Processes the user's requested BCD command.
pub fn process(poi: Poi) -> anyhow::Result<()> {
    match poi.command {
        PoiCommand::De { input } => {
            let file = File::open(input)?;

            let poi = PoiFormat::parse(&mut io::BufReader::new(file))?;
            serde_json::to_writer_pretty(io::stdout().lock(), &poi)?;

            Ok(())
        }
    }
}
