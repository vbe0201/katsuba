use std::{
    io::{self, IsTerminal},
    process,
};

use clap::CommandFactory;

use crate::cli::Cli;

/// Obtains a buffered reader over the contents of stdin.
///
/// This function will terminate the process and print the running
/// command's help if stdin is connected to a terminal.
pub fn stdin_reader() -> io::BufReader<io::StdinLock<'static>> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        let _ = Cli::command().print_help();
        process::exit(2);
    }

    io::BufReader::new(stdin.lock())
}
