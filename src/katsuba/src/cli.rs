use clap::{Parser, Subcommand};

use crate::cmd::*;

mod args;

pub mod helpers;

pub mod io;
pub use io::*;

mod processor;
pub use processor::*;

pub const HYPHEN: &str = "-";

/// The CLI interface for the Katsuba application.
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// The selected command.
    #[clap(subcommand)]
    pub command: KatsubaCommand,

    #[clap(flatten)]
    pub verbosity: args::Verbosity,
}

/// The top-level commands supported by Katsuba.
#[derive(Debug, Subcommand)]
pub enum KatsubaCommand {
    Bcd(bcd::Bcd),
    Cs(cs::ClientSig),
    Hash(hash::Hash),
    Nav(nav::Nav),
    Op(op::ObjectProperty),
    Poi(poi::Poi),
    Wad(wad::Wad),
}

impl Command for KatsubaCommand {
    fn handle(self) -> eyre::Result<()> {
        match self {
            Self::Bcd(bcd) => bcd.handle(),
            Self::Cs(cs) => cs.handle(),
            Self::Hash(hash) => hash.handle(),
            Self::Nav(nav) => nav.handle(),
            Self::Op(op) => op.handle(),
            Self::Poi(poi) => poi.handle(),
            Self::Wad(wad) => wad.handle(),
        }
    }
}
