use std::{fs, io, path::Path};

use katsuba_types::*;

fn read_type_list<P: AsRef<Path>>(path: P) -> Result<TypeList, Error> {
    let file = fs::File::open(path)?;
    TypeList::from_reader(io::BufReader::new(file))
}

#[test]
fn equal_parse_between_versions() -> Result<(), Error> {
    let v1 = read_type_list("tests/data/types_v1.json")?;
    let v2 = read_type_list("tests/data/types_v2.json")?;

    assert_eq!(v1, v2);

    Ok(())
}

#[test]
fn query_types() -> Result<(), Error> {
    let list = read_type_list("tests/data/types_v2.json")?;

    let matrix = list.0.get(&1479974833).unwrap();
    assert_eq!(matrix.name, "class Matrix3x3");
    assert!(matrix.properties.is_empty());

    Ok(())
}

#[test]
fn query_properties() -> Result<(), Error> {
    let list = read_type_list("tests/data/types_v1.json")?;

    let cls = list.0.get(&135649998).unwrap();
    assert_eq!(cls.name, "class EquipmentSetList");
    assert_eq!(cls.properties.len(), 1);

    let property = cls.properties.first().unwrap();
    assert_eq!(property.name, "m_equipmentSetList");
    assert_eq!(property.r#type, "class SharedPointer<class EquipmentSet>");
    assert_eq!(property.id, 0);
    assert_eq!(property.hash, 1788831224);
    assert!(property.dynamic);

    Ok(())
}
