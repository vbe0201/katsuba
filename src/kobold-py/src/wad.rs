use pyo3::prelude::*;

#[pyclass]
struct Archive(kobold_wad::Archive);

// TODO: Figure out API.

pub fn kobold_wad(m: &PyModule) -> PyResult<()> {
    m.add_class::<Archive>()?;

    Ok(())
}
