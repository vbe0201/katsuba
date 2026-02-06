use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Component, Path, PathBuf},
};

use eyre::bail;
use katsuba_wad::{Archive, Inflater};
use rayon::prelude::*;

use crate::{cli::OutputSource, utils::DirectoryTree};

fn validate_extract_path(base: &Path, archive_path: &str) -> eyre::Result<PathBuf> {
    let path = Path::new(archive_path);

    // Reject absolute paths outright.
    if path.is_absolute() {
        bail!("absolute path not allowed in archive: '{archive_path}'");
    }

    // Traverse the path while checking for directory escapes.
    let mut result = base.to_path_buf();
    let base_depth = base.components().count();

    for component in path.components() {
        match component {
            Component::Normal(c) => {
                result.push(c);
            }
            Component::ParentDir => {
                if result.components().count() <= base_depth {
                    bail!("path traversal detected in archive path '{archive_path}'");
                }
                result.pop();
            }
            Component::CurDir => (),
            Component::Prefix(_) | Component::RootDir => {
                bail!("invalid path component in archive path '{archive_path}'");
            }
        }
    }

    Ok(result)
}

fn create_directory_tree(archive: &Archive, out: &Path) -> eyre::Result<()> {
    // Pre-compute the directory structure we need to create.
    let mut tree = DirectoryTree::new();
    for file in archive.files().keys() {
        validate_extract_path(out, file)?;
        tree.add(file.as_ref());
    }

    // Create all the directories with minimal required syscalls.
    for path in tree {
        let path = validate_extract_path(out, &path.to_string_lossy())?;
        fs::create_dir_all(&path)?;
    }

    Ok(())
}

/// Writes a buffer to a file, handling platform-specific optimizations.
///
/// On Windows, closing files has significant overhead due to NTFS metadata
/// updates. We offload the file close to a separate thread pool to avoid
/// blocking the extraction pipeline.
fn write_file(path: PathBuf, data: &[u8], _mode: u32) -> eyre::Result<()> {
    let file = File::create(&path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = file.set_permissions(fs::Permissions::from_mode(_mode));
    }

    let mut writer = BufWriter::new(file);
    writer.write_all(data)?;

    #[cfg(windows)]
    blocking::unblock(move || {
        let _ = writer.flush();
        drop(writer);
    })
    .detach();

    #[cfg(not(windows))]
    writer.flush()?;

    Ok(())
}

pub fn extract_archive(
    inpath: Option<PathBuf>,
    archive: Archive,
    out: OutputSource,
) -> eyre::Result<()> {
    // Determine the output directory for the archive files.
    // Since we can't print here, we use the cwd instead.
    let input_stem = inpath.as_ref().and_then(|p| p.file_stem()).unwrap();
    let mut out = match out {
        OutputSource::Stdout => env::current_dir()?,
        OutputSource::File(p) | OutputSource::Dir(p, ..) => p,
    };
    out.push(input_stem);

    // First, create all the directories for the output files.
    create_directory_tree(&archive, &out)?;

    let mode = archive.mode();

    // Collect files that need extraction, validating paths upfront.
    let files_to_extract: Vec<_> = archive
        .files()
        .iter()
        .filter_map(|(path, file)| {
            if file.is_unpatched {
                log::warn!("Skipping unpatched file '{path}'");
                return None;
            }

            match validate_extract_path(&out, path) {
                Ok(dest_path) => Some((path.as_str(), file, dest_path)),
                Err(e) => {
                    log::error!("Skipping file due to path validation error: {e}");
                    None
                }
            }
        })
        .collect();

    // Extract files in parallel using rayon.
    // Each thread gets its own Inflater for decompression.
    files_to_extract.par_iter().try_for_each_init(
        Inflater::new,
        |inflater, (path, file, dest_path)| {
            let contents = archive
                .file_contents(file)
                .ok_or_else(|| eyre::eyre!("missing file contents for '{path}'"))?;

            let data = match file.compressed {
                true => {
                    let len = file.uncompressed_size as usize;
                    inflater.decompress(contents, len)?
                }
                false => contents,
            };
            write_file(dest_path.clone(), data, mode)?;

            Ok(())
        },
    )
}
