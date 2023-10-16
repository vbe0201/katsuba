use std::path::PathBuf;

use kobold_object_property::serde::{self, BIND_MAGIC};

use super::ClassType;
use crate::{fs, utils::json_to_stdout_or_output_file};

pub fn process(
    mut de: serde::Serializer,
    path: PathBuf,
    out: Option<PathBuf>,
    _class_type: ClassType,
) -> anyhow::Result<()> {
    // Read the binary data from the given input file.
    // TODO: mmap?
    let data = fs::read(path)?;
    let mut data = data.as_slice();

    // If the data starts with the `BINd` prefix, it is a serialized file
    // in the local game data. These always use a fixed base configuration.
    if data.get(0..4) == Some(BIND_MAGIC) {
        de.parts.options.shallow = false;
        de.parts.options.flags |= serde::SerializerFlags::STATEFUL_FLAGS;

        data = data.get(4..).unwrap();
    }

    // Deserialize the type from the given data.
    // TODO: Different class types?
    let value = de.deserialize::<serde::PropertyClass>(data)?;
    json_to_stdout_or_output_file(out, &value)
}
