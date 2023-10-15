use clap::{Args, ValueEnum};

use kobold_utils::{anyhow, hash::*};

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
    /// The KingsIsle String ID algorithm.
    StringId,
    /// The DJB2 algorithm.
    Djb2,
}

pub fn process(hash: Hash) -> anyhow::Result<()> {
    match hash.algo {
        Algo::StringId => {
            let hash = string_id(hash.input.as_bytes());
            println!("{hash}");
        }

        Algo::Djb2 => {
            let hash = djb2(hash.input.as_bytes());
            println!("{hash}");
        }
    }

    Ok(())
}
