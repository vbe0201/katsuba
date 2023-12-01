use std::{
    collections::{hash_set::IntoIter, HashSet},
    io::{self, IsTerminal},
    path::Path,
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

/// A structure which interns directory trees from given file paths
/// and returns the minimal amount of paths to be created.
///
/// This ensures that we create a directory tree with the least
/// required system calls. This has shown to greatly impact
/// performance on Windows systems.
pub struct DirectoryTree<'a> {
    inner: HashSet<&'a Path>,
}

impl<'a> DirectoryTree<'a> {
    /// Creates an empty directory tree.
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    /// Given a path to a file, interns the directory tree needed
    /// to be created for it.
    pub fn add(&mut self, path: &'a Path) {
        if let Some(p) = path.parent() {
            self.inner.insert(p);

            // Check for parent of parent so that we can remove
            // entries which would just need unnecessary syscalls.
            if let Some(p) = p.parent() {
                self.inner.remove(p);
            }
        }
    }
}

impl<'a> IntoIterator for DirectoryTree<'a> {
    type Item = &'a Path;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
