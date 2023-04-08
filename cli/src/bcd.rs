use std::{fs::File, io, path::PathBuf};

use clap::{Args, Subcommand};
use kobold::formats::bcd::Bcd as BcdFormat;

#[derive(Args)]
pub struct Bcd {
    #[clap(subcommand)]
    command: BcdCommand,
}

#[derive(Subcommand)]
enum BcdCommand {
    /// Deserializes the given BCD file and prints its
    /// JSON representation to stdout.
    De {
        /// Path to the file to deserialize.
        input: PathBuf,
    },
}

/// Processes the user's requested BCD command.
pub fn process(bcd: Bcd) -> anyhow::Result<()> {
    match bcd.command {
        BcdCommand::De { input } => {
            let file = File::open(input)?;

            let bcd = BcdFormat::parse(&mut io::BufReader::new(file))?;
            serde_json::to_writer_pretty(io::stdout().lock(), &bcd)?;

            Ok(())
        }
    }
}
