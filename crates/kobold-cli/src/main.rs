use std::{fs::File, io::BufReader, sync::Arc};

use kobold_object_property::serde::*;
use kobold_types::TypeList;

fn main() {
    let file = File::open("types.json").unwrap();
    let reader = BufReader::new(file);

    let types = Arc::new(TypeList::from_reader(reader).unwrap());

    let mut de = Deserializer::<PropertyClass>::new(
        SerializerOptions {
            flags: SerializerFlags::STATEFUL_FLAGS,
            shallow: false,
            skip_unknown_types: true,
            ..Default::default()
        },
        types,
    )
    .unwrap();
    let mut scratch = Vec::new();

    let data = std::fs::read("GDN_SM_MoonflowerTemplate.xml").unwrap();
    let obj = de.deserialize(&mut scratch, &data[4..]).unwrap();

    println!("{obj:?}");
}
