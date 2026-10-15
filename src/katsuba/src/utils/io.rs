use std::{
    borrow::Cow,
    collections::{HashSet, hash_set::IntoIter},
    fs,
    io::{self, IsTerminal, Write},
    path::Path,
    process,
};

use clap::CommandFactory;

use crate::cli::{Cli, OutputSource};

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

/// Writes data to an output [`Writer`], in compliance with the
/// given [`OutputSource`].
///
/// The user must provide a closure which does the actual data
/// writing, which allows full customization.
pub fn write_to_out<F>(inpath: Option<&Path>, f: F, out: OutputSource) -> eyre::Result<()>
where
    F: FnOnce(&mut dyn Write) -> eyre::Result<()>,
{
    match (out, inpath) {
        (OutputSource::Stdout, _) => f(&mut std::io::stdout()),

        (OutputSource::File(path), _) => fs::File::create(path)
            .map_err(Into::into)
            .and_then(|mut v| f(&mut v)),

        (OutputSource::Dir(mut out, suffix), Some(path)) => {
            // Create a file named after the input in the output directory.
            let infile = path.with_extension(suffix);
            out.push(infile.file_name().unwrap());

            fs::File::create(out)
                .map_err(Into::into)
                .and_then(|mut v| f(&mut v))
        }

        (OutputSource::Dir(..), None) => Err(eyre::eyre!(
            "output path for stdin input is a directory; please specify a file path instead"
        )),
    }
}

/// Helper function for writing raw encoded bytes to an output source.
pub fn write_bytes(inpath: Option<&Path>, value: Vec<u8>, out: OutputSource) -> eyre::Result<()> {
    write_to_out(inpath, |w| w.write_all(&value).map_err(Into::into), out)
}

/// A structure which interns directory trees from given file paths
/// and returns the minimal amount of paths to be created.
///
/// This ensures that we create a directory tree with the least
/// required system calls. For optimal results, path should be
/// sorted prior to inserting.
pub struct DirectoryTree<'a> {
    inner: HashSet<Cow<'a, Path>>,
}

impl<'a> DirectoryTree<'a> {
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

        if let Some(p) = path.parent() {
            self.inner.remove(p);
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
