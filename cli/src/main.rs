use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use mimalloc::MiMalloc;

mod op;
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
    /// Subcommand for working with ObjectProperty state.
    Op(ObjectProperty),
}

#[derive(Args)]
pub struct Wad {
    #[clap(subcommand)]
    command: WadCommand,
}

#[derive(Args)]
pub struct ObjectProperty {
    #[clap(subcommand)]
    command: ObjectPropertyCommand,

    /// The path to the type list json file.
    #[clap(short, long)]
    type_list: PathBuf,

    /// Serializer configuration flags to use.
    #[clap(short, long, default_value_t = 0)]
    flags: u32,

    /// Property filter mask to use.
    #[clap(short, long, default_value_t = 0x18)]
    mask: u32,

    /// Whether the object is serialized shallow.
    #[clap(short, long, default_value_t = false)]
    shallow: bool,

    /// Whether the object is manually zlib-compressed.
    #[clap(short, long, default_value_t = false)]
    zlib_manual: bool,
}

#[derive(Subcommand)]
pub enum ObjectPropertyCommand {
    /// Deserializes the given ObjectProperty binary state.
    De {
        /// Path to the file to deserialize.
        input: PathBuf,
    },
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
        Command::Wad(wad) => wad::process(wad)?,
        Command::Op(op) => op::process(op)?,
    }

    Ok(())
}
