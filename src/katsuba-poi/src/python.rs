//! Python bindings for the POI types.

use pyo3::prelude::*;

use crate::{Poi, Point, Teleporter};

katsuba_utils::impl_py_parse!(Poi);

/// Registers the POI types in the given Python module.
pub fn katsuba_poi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Point>()?;
    m.add_class::<Teleporter>()?;
    m.add_class::<Poi>()?;

    Ok(())
}
