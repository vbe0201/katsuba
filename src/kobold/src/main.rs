use clap::Parser;

mod cli;
use cli::Cli;

mod cmd;
use cmd::Command;

mod executor;

mod utils;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    cli.verbosity.setup();
    cli.command.handle()
}
