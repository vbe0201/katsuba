use std::{io, path::PathBuf};

use clap::{Args, Subcommand};
use kobold_bcd::Bcd as BcdFile;
use kobold_utils::{
    anyhow::{self, Context},
    fs,
};

use crate::utils;

#[derive(Debug, Args)]
pub struct Bcd {
    #[clap(subcommand)]
    command: BcdCommand,
}

#[derive(Debug, Subcommand)]
enum BcdCommand {
    /// Deserializes a given Binary Collision Data file into JSON format.
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

pub fn process(bcd: Bcd) -> anyhow::Result<()> {
    match bcd.command {
        BcdCommand::De { input, output } => {
            // Parse the given input file.
            let input = fs::open_file(input)?;
            let bcd = BcdFile::parse(&mut io::BufReader::new(input))
                .with_context(|| "failed to parse BCD file")?;

            utils::json_to_stdout_or_output_file(output, &bcd)
        }
    }
}
