use std::{fs, io::BufReader, path::PathBuf};

use katsuba_types::TypeList;

/// Reads all the given type list paths and merges them into a single
/// [`TypeList`] instance.
pub fn merge_type_lists(paths: Vec<PathBuf>) -> eyre::Result<TypeList> {
    let (first, rest) = paths
        .split_first()
        .ok_or_else(|| eyre::eyre!("at least one type list is required for deserialization"))?;

    let first = fs::File::open(first)?;
    let mut list = TypeList::from_reader(BufReader::new(first))?;

    // Merge remaining type lists into `list`.
    for path in rest {
        let file = fs::File::open(path)?;
        let next = TypeList::from_reader(BufReader::new(file))?;

        list.merge(next);
    }

    Ok(list)
}
