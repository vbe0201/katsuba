use std::{fs, io};

use kobold::formats::wad::Archive;

fn main() {
    let nav = fs::read("WizardCity-WorldData.wad").unwrap();
    let mut cursor = io::Cursor::new(nav);

    let nav = Archive::parse(&mut cursor).unwrap();
    for file in &nav.files {
        println!("{}", file.name);
    }
}
