use std::{io, path::PathBuf};

use clap::{Args, Subcommand};
use kobold_poi::Poi as PoiFile;
use kobold_utils::{
    anyhow::{self, Context},
    fs,
};

use crate::utils;

#[derive(Debug, Args)]
pub struct Poi {
    #[clap(subcommand)]
    command: PoiCommand,
}

#[derive(Debug, Subcommand)]
enum PoiCommand {
    /// Deserializes a given Point of Interest file into JSON format.
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

pub fn process(poi: Poi) -> anyhow::Result<()> {
    match poi.command {
        PoiCommand::De { input, output } => {
            let input = fs::open_file(input)?;
            let poi = PoiFile::parse(&mut io::BufReader::new(input))
                .with_context(|| "failed to parse POI file")?;

            utils::json_to_stdout_or_output_file(output, &poi)
        }
    }
}
