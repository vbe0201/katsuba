use std::{fs, io::BufReader, path::PathBuf};

use eyre::Context;
use katsuba_types::TypeList;

/// Reads all the given type list paths and merges them into a single
/// [`TypeList`] instance.
pub fn merge_type_lists(paths: Vec<PathBuf>) -> eyre::Result<TypeList> {
    let (first, rest) = paths
        .split_first()
        .ok_or_else(|| eyre::eyre!("at least one type list is required for deserialization"))?;

    let first = fs::File::open(first)
        .with_context(|| format!("failed to open type list at '{}'", first.display()))?;
    let mut list = TypeList::from_reader(BufReader::new(first))?;

    // Merge remaining type lists into `list`.
    for path in rest {
        let file = fs::File::open(path)
            .with_context(|| format!("failed to open type list at '{}'", path.display()))?;
        let next = TypeList::from_reader(BufReader::new(file))?;

        list.merge(next);
    }

    Ok(list)
}
