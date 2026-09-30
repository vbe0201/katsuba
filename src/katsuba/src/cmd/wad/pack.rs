use std::{fs, path::PathBuf};

use eyre::Context;
use katsuba_wad::ArchiveBuilder;

/// Packs a KIWAD archive from the files in the `input` directory and
/// writes the resulting file to the `output` path.
pub fn pack_archive(input: PathBuf, flags: u8, output: PathBuf) -> eyre::Result<()> {
    let input = std::path::absolute(input)?;
    let output = std::path::absolute(output)?;

    let mut builder = ArchiveBuilder::new(2, flags, &output)
        .with_context(|| format!("failed to build output archive at '{}'", output.display()))?;

    for entry in walkdir::WalkDir::new(&input).sort_by_file_name() {
        let entry = entry.context("failed to query input directory")?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path == output {
            continue;
        }

        let contents = fs::read(path)
            .with_context(|| format!("failed to read file at '{}'", path.display()))?;

        let rel = path.strip_prefix(&input)?;
        builder.add_file_compressed(rel, &contents)?;
    }

    builder.finish()?;

    Ok(())
}
