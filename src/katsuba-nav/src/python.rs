//! Python bindings for the NAV types.

use pyo3::prelude::*;

use crate::{NavigationGraph, NavigationLink, NavigationNode, ZoneNavigationGraph};

katsuba_utils::impl_py_parse!(NavigationGraph);
katsuba_utils::impl_py_parse!(ZoneNavigationGraph);

/// Registers the NAV types in the given Python module.
pub fn katsuba_nav(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NavigationNode>()?;
    m.add_class::<NavigationLink>()?;
    m.add_class::<NavigationGraph>()?;
    m.add_class::<ZoneNavigationGraph>()?;

    Ok(())
}
