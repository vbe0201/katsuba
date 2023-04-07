use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;

mod bcd;
mod nav;
mod op;
mod poi;
mod progress_bar;
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
    Wad(wad::Wad),
    /// Subcommand for working with Binary Collision Data.
    Bcd(bcd::Bcd),
    /// Subcommand for working with Point of Interest data.
    Poi(poi::Poi),
    /// Subcommand for working with Navigation Graph data.
    Nav(nav::Nav),
    /// Subcommand for working with ObjectProperty state.
    Op(op::ObjectProperty),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Wad(wad) => wad::process(wad)?,
        Command::Bcd(bcd) => bcd::process(bcd)?,
        Command::Poi(poi) => poi::process(poi)?,
        Command::Nav(nav) => nav::process(nav)?,
        Command::Op(op) => op::process(op)?,
    }

    Ok(())
}
