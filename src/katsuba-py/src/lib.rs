//! Python bindings to the Katsuba libraries.
//!
//! All crates in the Katsuba project are unified into a single Python
//! library here to reduce complexity.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use pyo3::prelude::*;

mod error;
mod op;
mod utils;
mod wad;

pub use katsuba_utils::python::KatsubaError;

/// The entrypoint to the Katsuba extension module for Python.
#[pymodule]
pub fn katsuba(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    // Bind the exception types utilized on the Python side.
    module.add("KatsubaError", py.get_type::<KatsubaError>())?;

    // Declare all the submodules in the package.
    let bcd = PyModule::new(py, "bcd")?;
    let nav = PyModule::new(py, "nav")?;
    let op = PyModule::new(py, "op")?;
    let poi = PyModule::new(py, "poi")?;
    let utils = PyModule::new(py, "utils")?;
    let wad = PyModule::new(py, "wad")?;

    // Enable `from katsuba.a import b` imports.
    let sys_modules = py.import("sys")?.getattr("modules")?;
    sys_modules.set_item("katsuba.bcd", &bcd)?;
    sys_modules.set_item("katsuba.nav", &nav)?;
    sys_modules.set_item("katsuba.op", &op)?;
    sys_modules.set_item("katsuba.poi", &poi)?;
    sys_modules.set_item("katsuba.utils", &utils)?;
    sys_modules.set_item("katsuba.wad", &wad)?;

    // Register katsuba_py.bcd module.
    katsuba_bcd::python::katsuba_bcd(&bcd)?;
    module.add_submodule(&bcd)?;

    // Register katsuba_py.nav module.
    katsuba_nav::python::katsuba_nav(&nav)?;
    module.add_submodule(&nav)?;

    // Register katsuba_py.op module.
    op::katsuba_op(&op)?;
    module.add_submodule(&op)?;

    // Register katsuba_py.poi module.
    katsuba_poi::python::katsuba_poi(&poi)?;
    module.add_submodule(&poi)?;

    // Register katsuba_py.utils module.
    utils::katsuba_utils(&utils)?;
    module.add_submodule(&utils)?;

    // Register katsuba_py.wad module.
    wad::katsuba_wad(&wad)?;
    module.add_submodule(&wad)?;

    Ok(())
}
