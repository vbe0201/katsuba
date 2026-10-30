//! Python bindings for the BCD types.

use std::convert::Infallible;

use pyo3::{prelude::*, types::PyInt};

use crate::{Bcd, Collision, CollisionFlags, Face, GeomParams, ProxyGeometry, ProxyMesh};

katsuba_utils::impl_py_parse!(Bcd);

// Flags are exposed as plain integers to be tested against the module constants.
impl<'py> IntoPyObject<'py> for CollisionFlags {
    type Target = PyInt;
    type Output = Bound<'py, PyInt>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.bits().into_pyobject(py)
    }
}

/// Registers the BCD types in the given Python module.
pub fn katsuba_bcd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Face>()?;
    m.add_class::<GeomParams>()?;
    m.add_class::<ProxyGeometry>()?;
    m.add_class::<ProxyMesh>()?;
    m.add_class::<Collision>()?;
    m.add_class::<Bcd>()?;

    for (name, flag) in CollisionFlags::all().iter_names() {
        m.add(name, flag.bits())?;
    }

    Ok(())
}
