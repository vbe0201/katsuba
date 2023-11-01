use clap::Parser;

mod cli;
use cli::Cli;

mod cmd;
use cmd::Command;

mod executor;

mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    cli.verbosity.setup();
    cli.command.handle()
}
