//! Python bindings to the `kobold` crate.

use pyo3::{create_exception, exceptions::PyException, prelude::*};

mod nav;
mod object_property;
mod poi;
mod wad;

create_exception!(kobold_py, KoboldError, PyException);

#[pymodule]
fn kobold_py(py: Python, m: &PyModule) -> PyResult<()> {
    // Bind the generic exception type used by all submodules.
    m.add("KoboldError", py.get_type::<KoboldError>())?;

    // Initialize the kobold.nav submodule.
    let nav = PyModule::new(py, "nav")?;
    nav::kobold_nav(nav)?;
    m.add_submodule(nav)?;

    // Initialize the kobold.op submodule.
    let op = PyModule::new(py, "op")?;
    object_property::kobold_op(op)?;
    m.add_submodule(op)?;

    // Initialize the kobold.poi submodule.
    let poi = PyModule::new(py, "poi")?;
    poi::kobold_poi(poi)?;
    m.add_submodule(poi)?;

    // Initialize the kobold.wad submodule.
    let wad = PyModule::new(py, "wad")?;
    wad::kobold_wad(wad)?;
    m.add_submodule(wad)?;

    Ok(())
}
