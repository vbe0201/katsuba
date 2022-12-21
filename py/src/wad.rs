use std::io::Cursor;

use kobold::formats::wad as kobold;
use pyo3::prelude::*;

use crate::KoboldError;

#[pyfunction]
pub fn deserialize(bytes: &[u8]) -> PyResult<kobold::Archive> {
    let mut cursor = Cursor::new(bytes);
    kobold::Archive::parse(&mut cursor).map_err(|e| KoboldError::new_err(e.to_string()))
}

pub fn kobold_wad(m: &PyModule) -> PyResult<()> {
    m.add_class::<kobold::Header>()?;
    m.add_class::<kobold::File>()?;
    m.add_class::<kobold::Archive>()?;

    m.add_function(wrap_pyfunction!(deserialize, m)?)?;

    Ok(())
}
