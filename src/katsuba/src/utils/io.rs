use std::{
    borrow::Cow,
    collections::{HashSet, hash_set::IntoIter},
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
/// required system calls.
pub struct DirectoryTree<'a> {
    inner: HashSet<Cow<'a, Path>>,
}

impl<'a> DirectoryTree<'a> {
    /// Creates an empty directory tree.
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    /// Creates an empty directory tree with capacity for `cap`
    /// entries preallocated.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: HashSet::with_capacity(cap),
        }
    }

    /// Given a path to a file, interns the directory tree needed
    /// to be created for it.
    pub fn add<P: Into<Cow<'a, Path>>>(&mut self, path: P) {
        // Convert `path` into `path.parent()`.
        let path = match path.into() {
            Cow::Borrowed(p) => {
                let Some(p) = p.parent() else {
                    return;
                };
                Cow::Borrowed(p)
            }

            Cow::Owned(mut p) => {
                if !p.pop() {
                    return;
                }
                Cow::Owned(p)
            }
        };

        if path.is_empty() {
            return;
        }

        for ancestor in path.ancestors().skip(1) {
            self.inner.remove(ancestor);
        }

        self.inner.insert(path);
    }
}

impl<'a> IntoIterator for DirectoryTree<'a> {
    type Item = Cow<'a, Path>;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
