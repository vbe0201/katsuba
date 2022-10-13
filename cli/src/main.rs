use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use mimalloc::MiMalloc;

mod wad;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Subcommand for working with KIWAD archives.
    Wad(Wad),
}

#[derive(Args)]
struct Wad {
    #[clap(subcommand)]
    command: WadCommand,
}

#[derive(Subcommand)]
pub enum WadCommand {
    /// Unpacks a given KIWAD archive file.
    Unpack {
        /// Path to the archive file to unpack.
        input: PathBuf,

        /// An optional path to extract the archived files to.
        ///
        /// By default, a new directory named after the input
        /// file will be created in the same directory and
        /// files will be extracted to it.
        #[clap(short, long)]
        out: Option<PathBuf>,

        /// Optionally validates CRCs for all files in the
        /// archive.
        ///
        /// Most of the time, files are stored compressed and
        /// naturally have corruption resilience by nature of
        /// the algorithm in use.
        ///
        /// It is recommended to use this setting only for
        /// testing custom archives for correctness.
        #[clap(short, long)]
        verify_checksums: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Wad(wad) => wad::process(wad.command)?,
    }

    Ok(())
}
