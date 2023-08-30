use clap::{Parser, Subcommand};
use kobold_utils::anyhow;

mod bcd;
mod cs;
mod op;
mod utils;
mod wad;

// When not stuck with Windows, use a more performant global
// allocator than the default one Rust uses.
#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Subcommand for working with Binary Collision Data.
    Bcd(bcd::Bcd),
    /// Subcommand for working with ClientSig binary state.
    Cs(cs::ClientSig),
    /// Subcommand for working with ObjectProperty binary state.
    Op(op::ObjectProperty),
    /// Subcommand for working with KIWAD archives.
    Wad(wad::Wad),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Bcd(bcd) => bcd::process(bcd),
        Command::Cs(cs) => cs::process(cs),
        Command::Op(op) => op::process(op),
        Command::Wad(wad) => wad::process(wad),
    }
}
