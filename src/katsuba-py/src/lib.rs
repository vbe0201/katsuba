//! Python bindings to the Katsuba libraries.
//!
//! All crates in the Katsuba project are unified into a single Python
//! library here to reduce complexity.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]

mod error;
mod op;
mod utils;
mod wad;

use pyo3::{create_exception, exceptions::PyException, prelude::*};

create_exception!(katsuba, KatsubaError, PyException);

/// The entrypoint to the Katsuba extension module for Python.
#[pymodule]
pub fn katsuba(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    // Bind the exception types utilized on the Python side.
    module.add("KatsubaError", py.get_type::<KatsubaError>())?;

    // Declare all the submodules in the package.
    let op = PyModule::new(py, "op")?;
    let utils = PyModule::new(py, "utils")?;
    let wad = PyModule::new(py, "wad")?;

    // Enable `from katsuba.a import b` imports.
    let sys_modules = py.import("sys")?.getattr("modules")?;
    sys_modules.set_item("katsuba.op", &op)?;
    sys_modules.set_item("katsuba.utils", &utils)?;
    sys_modules.set_item("katsuba.wad", &wad)?;

    // Register katsuba_py.op module.
    op::katsuba_op(&op)?;
    module.add_submodule(&op)?;

    // Register katsuba_py.utils module.
    utils::katsuba_utils(&utils)?;
    module.add_submodule(&utils)?;

    // Register katsuba_py.wad module.
    wad::katsuba_wad(&wad)?;
    module.add_submodule(&wad)?;

    Ok(())
}
