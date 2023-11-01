use clap::{Args, ValueEnum};

use katsuba_utils::hash::*;

use super::Command;

/// Subcommand for hashing strings with common KingsIsle algorithms.
#[derive(Debug, Args)]
pub struct Hash {
    /// The hash algorithm to apply.
    #[clap(value_enum)]
    algo: Algo,

    /// The input string to hash.
    input: String,
}

/// The hash algorithm to apply.
#[derive(Clone, Debug, ValueEnum)]
enum Algo {
    /// The KingsIsle string ID algorithm.
    StringId,
    /// The DJB2 algorithm.
    Djb2,
}

impl Command for Hash {
    fn handle(self) -> eyre::Result<()> {
        let input = self.input.as_bytes();
        let hash = match self.algo {
            Algo::StringId => string_id(input),
            Algo::Djb2 => djb2(input),
        };

        println!("{hash}");
        Ok(())
    }
}
