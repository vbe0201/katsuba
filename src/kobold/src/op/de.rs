use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use kobold_object_property::serde;

use super::{format, ClassType};

pub fn process<D: serde::Diagnostics>(
    mut de: serde::Deserializer<D>,
    path: PathBuf,
    _class_type: ClassType,
) -> anyhow::Result<()> {
    // Read the binary data from the given input file.
    // TODO: mmap?
    let data = fs::read(path)?;
    let mut data = data.as_slice();

    // If the data starts with the `BINd` prefix, it is a serialized file
    // in the local game data. These always use a fixed base configuration.
    if data.get(0..4) == Some(b"BINd") {
        de.serde_parts.options.shallow = false;
        de.serde_parts.options.flags |= serde::SerializerFlags::STATEFUL_FLAGS;

        data = data.get(4..).unwrap();
    }

    // Deserialize the type from the given data.
    // TODO: Different class types?
    let value = de.deserialize::<serde::PropertyClass>(data)?;

    // Format the resulting object to stdout.
    {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        format::value(&mut stdout, value)?;
        writeln!(stdout)?;
    }

    Ok(())
}
