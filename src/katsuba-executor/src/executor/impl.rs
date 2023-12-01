use std::{
    fs,
    io::{self, Write},
    path::Path,
};

/// Creates a new file in the filesystem.
///
/// The file mode may be optionally respected on UNIX platforms,
/// but is ignored everywhere else.
pub fn write_file(path: &Path, contents: &[u8], _mode: u32) -> io::Result<()> {
    let mut opts = fs::OpenOptions::new();

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(_mode);
    }

    let mut file = opts.write(true).create(true).truncate(true).open(path)?;
    file.write_all(contents)
}

/// Creates a new directory in the filesystem.
///
/// Subdirectories in the `path` are also created, when missing.
pub fn create_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}
