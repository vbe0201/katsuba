use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use kobold_utils::anyhow;
use kobold_wad::{Archive, Inflater};

pub fn extract_all(archive: Archive, out: PathBuf) -> anyhow::Result<()> {
    let mut inflater = Inflater::new();

    // Pre-compute the directory structure we need to create.
    let mut out_paths = HashSet::new();
    for (file, _) in archive.files() {
        let file: &Path = file.as_ref();
        if let Some(p) = file.parent() {
            out_paths.insert(p);
        }
    }

    // Now create all the directories.
    for path in out_paths {
        fs::create_dir_all(out.join(path))?;
    }

    // Extract all the files in the archive.
    for (path, file) in archive.files() {
        // Get the uncompressed contents of the file.
        let contents = archive.file_contents(file);
        let contents = match file.compressed {
            true => inflater.decompress(contents, file.uncompressed_size as _)?,
            false => contents,
        };

        // Write uncompressed file contents.
        fs::write(out.join(path), contents)?;
    }

    Ok(())
}
