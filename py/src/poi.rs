use std::io::Cursor;

use kobold::formats::poi as kobold;
use pyo3::prelude::*;

use crate::KoboldError;

#[pyfunction]
pub fn deserialize(bytes: &[u8]) -> PyResult<kobold::Poi> {
    let mut cursor = Cursor::new(bytes);
    kobold::Poi::parse(&mut cursor).map_err(|e| KoboldError::new_err(e.to_string()))
}

pub fn kobold_poi(m: &PyModule) -> PyResult<()> {
    m.add_class::<kobold::PointOfInterest>()?;
    m.add_class::<kobold::Poi>()?;

    m.add_function(wrap_pyfunction!(deserialize, m)?)?;

    Ok(())
}
