//! Python bindings to the Kobold libraries.
//!
//! All crates in the Kobold project are unified into a single Python
//! library here to reduce complexity.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]

mod error;
mod op;
mod utils;
mod wad;

use pyo3::{create_exception, exceptions::PyException, prelude::*, types::IntoPyDict};

create_exception!(kobold, KoboldError, PyException);

/// The entrypoint to the Kobold extension module for Python.
#[pymodule]
pub fn kobold(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    // Bind the exception types utilized on the Python side.
    module.add("KoboldError", py.get_type::<KoboldError>())?;

    // Declare all the submodules in the package.
    let op = PyModule::new(py, "kobold_py.op")?;
    let utils = PyModule::new(py, "kobold_py.utils")?;
    let wad = PyModule::new(py, "kobold_py.wad")?;

    // Enable `from kobold_py.x import A` imports.
    let locals = [
        ("op", op.to_object(py)),
        ("utils", utils.to_object(py)),
        ("wad", wad.to_object(py)),
    ]
    .into_py_dict(py);
    py.run(
        r#"
import sys
sys.modules['kobold_py.op'] = op
sys.modules['kobold_py.utils'] = utils
sys.modules['kobold_py.wad'] = wad
"#,
        None,
        Some(locals),
    )?;

    // Register kobold_py.op module.
    op::kobold_op(op)?;
    module.add_submodule(op)?;

    // Register kobold_py.utils module.
    utils::kobold_utils(utils)?;
    module.add_submodule(utils)?;

    // Register kobold_py.wad module.
    wad::kobold_wad(wad)?;
    module.add_submodule(wad)?;

    Ok(())
}
