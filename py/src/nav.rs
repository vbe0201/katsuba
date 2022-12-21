use std::io::Cursor;

use kobold::formats::nav as kobold;
use pyo3::prelude::*;

use crate::KoboldError;

#[pyfunction]
pub fn deserialize_nav(bytes: &[u8]) -> PyResult<kobold::NavigationGraph> {
    let mut cursor = Cursor::new(bytes);
    kobold::NavigationGraph::parse(&mut cursor).map_err(|e| KoboldError::new_err(e.to_string()))
}

#[pyfunction]
pub fn deserialize_zonenav(bytes: &[u8]) -> PyResult<kobold::ZoneNavigationGraph> {
    let mut cursor = Cursor::new(bytes);
    kobold::ZoneNavigationGraph::parse(&mut cursor).map_err(|e| KoboldError::new_err(e.to_string()))
}

pub fn kobold_nav(m: &PyModule) -> PyResult<()> {
    m.add_class::<kobold::NavigationLink>()?;
    m.add_class::<kobold::NavigationNode>()?;
    m.add_class::<kobold::NavigationGraph>()?;
    m.add_class::<kobold::ZoneNavigationGraph>()?;

    m.add_function(wrap_pyfunction!(deserialize_nav, m)?)?;
    m.add_function(wrap_pyfunction!(deserialize_zonenav, m)?)?;

    Ok(())
}
