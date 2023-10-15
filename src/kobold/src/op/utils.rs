use std::{io::BufReader, path::PathBuf};

use kobold_types::TypeList;
use kobold_utils::{anyhow, fs};

pub fn merge_type_lists(mut paths: Vec<PathBuf>) -> anyhow::Result<TypeList> {
    anyhow::ensure!(
        !paths.is_empty(),
        "at least one type list is required for deserialization"
    );

    // Take the first type list path and read it.
    let first = fs::open_file(paths.swap_remove(0))?;
    let mut list = TypeList::from_reader(BufReader::new(first))?;

    // Merge remaining type lists into `list`.
    for path in paths {
        let file = fs::open_file(path)?;
        let other = TypeList::from_reader(BufReader::new(file))?;

        list.merge(other);
    }

    Ok(list)
}
