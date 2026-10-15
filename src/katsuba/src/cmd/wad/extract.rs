use std::{
    cell::RefCell,
    env, fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use eyre::bail;
use katsuba_wad::{Archive, Inflater, glob};
use rayon::prelude::*;

use crate::{cli::OutputSource, utils::DirectoryTree};

thread_local! {
    static INFLATER: RefCell<Inflater> = RefCell::new(Inflater::new());
}

fn validate_extract_path(base: &Path, file: &str) -> eyre::Result<PathBuf> {
    let mut result = PathBuf::with_capacity(base.as_os_str().len() + file.len() + 1);
    result.push(base);

    let mut named = false;
    for component in Path::new(file).components() {
        match component {
            Component::Normal(c) => {
                // `name:stream` would create an NTFS alternate data stream.
                #[cfg(windows)]
                if c.as_encoded_bytes().contains(&b':') {
                    bail!("invalid character in archive path: '{file}'");
                }

                result.push(c);
                named = true;
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("invalid path in archive: '{file}'");
            }
        }
    }

    if !named {
        bail!("archive path '{file}' does not name a file");
    }

    Ok(result)
}

fn write_file(path: &Path, data: &[u8], _mode: u32) -> eyre::Result<()> {
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(_mode);
    }

    let mut file = opts.open(path)?;
    file.write_all(data)?;

    #[cfg(windows)]
    blocking::unblock(move || drop(file)).detach();

    Ok(())
}

fn choose_destination(inp: Option<&Path>, out: OutputSource) -> eyre::Result<PathBuf> {
    let (mut out, explicit) = match out {
        OutputSource::Stdout => (env::current_dir()?, false),
        OutputSource::Dir(p, ..) => (p, true),
        OutputSource::File(p) => eyre::bail!("'{}' is not a directory", p.display()),
    };

    match inp {
        Some(p) => {
            let stem = p.file_stem().ok_or_else(|| {
                eyre::eyre!("cannot determine archive name from '{}'", p.display())
            })?;
            out.push(stem);
        }
        None if explicit => (),
        None => eyre::bail!("must specify explicit extraction directory with -o/--output"),
    }

    Ok(out)
}

pub fn extract_archive(
    inpath: Option<&Path>,
    archive: Archive,
    out: OutputSource,
    glob: Option<&glob::Matcher>,
) -> eyre::Result<()> {
    let out = choose_destination(inpath, out)?;

    // Collect files to extract and validate all paths upfront.
    let mut files = Vec::new();
    for (path, file) in archive.files().iter() {
        if let Some(matcher) = glob
            && !matcher.is_match(path)
        {
            continue;
        }

        if file.is_unpatched {
            log::warn!("Skipping unpatched file '{path}'");
            continue;
        }

        let dst = validate_extract_path(&out, path)?;
        files.push((path.as_str(), file, dst));
    }

    // Create the directory tree required for extraction.
    let mut dirs = DirectoryTree::with_capacity(files.len());
    for (_, _, dst) in &files {
        dirs.add(dst);
    }
    for path in dirs {
        fs::create_dir_all(path)?;
    }

    // Extract files in parallel using rayon.
    let mode = archive.mode();
    files.par_iter().try_for_each(|(path, file, dst)| {
        let contents = archive
            .file_contents(file)
            .ok_or_else(|| eyre::eyre!("missing file contents for '{path}'"))?;

        match file.compressed {
            true => {
                let len = file.uncompressed_size as usize;
                INFLATER.with_borrow_mut(|inf| -> eyre::Result<()> {
                    let data = inf.decompress(contents, len)?;
                    write_file(dst, data, mode)
                })
            }
            false => write_file(dst, contents, mode),
        }
    })
}
