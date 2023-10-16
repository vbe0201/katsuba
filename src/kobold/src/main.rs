use clap::{Parser, Subcommand, ValueEnum};
use env_logger::{Builder, WriteStyle};

mod bcd;
mod cs;
mod fs;
mod hash;
mod nav;
mod op;
mod poi;
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

    #[clap(long, value_enum, default_value_t = Color::Auto)]
    color: Color,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Color {
    Auto,
    Always,
    Never,
}

impl From<Color> for WriteStyle {
    fn from(value: Color) -> Self {
        match value {
            Color::Auto => WriteStyle::Auto,
            Color::Always => WriteStyle::Always,
            Color::Never => WriteStyle::Never,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Subcommand for working with Binary Collision Data.
    Bcd(bcd::Bcd),
    /// Subcommand for working with ClientSig binary state.
    Cs(cs::ClientSig),
    /// Subcommand for working with string hashing algorithms.
    Hash(hash::Hash),
    /// Subcommand for working with Navigation Graphs.
    Nav(nav::Nav),
    /// Subcommand for working with ObjectProperty binary state.
    Op(op::ObjectProperty),
    /// Subcommand for working with Points of Interest.
    Poi(poi::Poi),
    /// Subcommand for working with KIWAD archives.
    Wad(wad::Wad),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Configure global logging with color setting.
    Builder::from_default_env()
        .write_style(cli.color.into())
        .try_init()?;

    match cli.command {
        Command::Bcd(bcd) => bcd::process(bcd),
        Command::Cs(cs) => cs::process(cs),
        Command::Hash(hash) => hash::process(hash),
        Command::Nav(nav) => nav::process(nav),
        Command::Op(op) => op::process(op),
        Command::Poi(poi) => poi::process(poi),
        Command::Wad(wad) => wad::process(wad),
    }
}
