//! CLI-friendly wrappers around [`std::fs`], providing error
//! messages aimed at humans.

use std::{fs, path::Path};

use anyhow::Context;

/// Attempts to open a file in read-only mode.
///
/// This wraps [`std::fs::File::open`], but with additional context
/// provided in the error case.
pub fn open_file<P: AsRef<Path>>(path: P) -> anyhow::Result<fs::File> {
    let path = path.as_ref();
    fs::File::open(path).with_context(|| format!("failed to open file: `{}`", path.display()))
}

/// Reads the entire contents of a file into a bytes vector.
///
/// This wraps [`std::fs::read`], but with additional context
/// provided in the error case.
pub fn read<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).with_context(|| format!("failed to read file: `{}`", path.display()))
}

/// Reads the entire contents of a file into a string.
///
/// This wraps [`std::fs::read_to_string`], but with additional
/// context provided in the error case.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).with_context(|| format!("failed to read file: `{}`", path.display()))
}

/// Writes a slice of bytes as the entire contents of a file.
///
/// This wraps [`std::fs::write`], but with additional context
/// provided in the error case.
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> anyhow::Result<()> {
    let path = path.as_ref();
    fs::write(path, contents).with_context(|| format!("failed to write file: `{}`", path.display()))
}

/// Recursively creates a directory and all of its parent components.
///
/// This wraps [`std::fs::create_dir_all`], but with additional
/// context provided in the error case.
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory tree: `{}`", path.display()))
}
