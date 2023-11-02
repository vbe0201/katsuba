//! Python bindings to the Katsuba libraries.
//!
//! All crates in the Katsuba project are unified into a single Python
//! library here to reduce complexity.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]

mod error;
mod op;
mod utils;
mod wad;

use pyo3::{create_exception, exceptions::PyException, prelude::*, types::IntoPyDict};

create_exception!(katsuba, KatsubaError, PyException);

/// The entrypoint to the Katsuba extension module for Python.
#[pymodule]
pub fn katsuba(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    // Bind the exception types utilized on the Python side.
    module.add("KatsubaError", py.get_type::<KatsubaError>())?;

    // Declare all the submodules in the package.
    let op = PyModule::new(py, "op")?;
    let utils = PyModule::new(py, "utils")?;
    let wad = PyModule::new(py, "wad")?;

    // Enable `from katsuba_py.x import A` imports.
    let locals = [
        ("op", op.to_object(py)),
        ("utils", utils.to_object(py)),
        ("wad", wad.to_object(py)),
    ]
    .into_py_dict(py);
    py.run(
        r#"
import sys
sys.modules['katsuba.op'] = op
sys.modules['katsuba.utils'] = utils
sys.modules['katsuba.wad'] = wad
"#,
        None,
        Some(locals),
    )?;

    // Register katsuba_py.op module.
    op::katsuba_op(op)?;
    module.add_submodule(op)?;

    // Register katsuba_py.utils module.
    utils::katsuba_utils(utils)?;
    module.add_submodule(utils)?;

    // Register katsuba_py.wad module.
    wad::katsuba_wad(wad)?;
    module.add_submodule(wad)?;

    Ok(())
}
