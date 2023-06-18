use std::{fs, path::PathBuf};

use kobold_wad::{Archive, Inflater};

pub fn extract_all(archive: Archive, out: PathBuf) -> anyhow::Result<()> {
    let mut inflater = Inflater::new();

    for (path, file) in archive.files() {
        // Get the uncompressed contents of the file.
        let contents = archive.file_contents(file);
        let contents = if file.compressed {
            inflater.decompress(contents, file.uncompressed_size as _)?
        } else {
            contents
        };

        // Make sure the parent directory for the output file exists.
        let out = out.join(path);
        if let Some(p) = out.parent() {
            if !p.exists() {
                fs::create_dir_all(p)?;
            }
        }

        // Write uncompressed file contents.
        fs::write(out, contents)?;
    }

    Ok(())
}
