//! Shared helpers for the Python bindings of Katsuba crates.

use std::io;

use pyo3::{PyErr, create_exception, exceptions::PyException};

create_exception!(katsuba, KatsubaError, PyException);

/// Converts an error from parsing in-memory data to a Python exception.
///
/// Since no actual I/O is involved, all errors stem from invalid data.
pub fn parse_error(err: io::Error) -> PyErr {
    KatsubaError::new_err(err.to_string())
}

/// Implements the `parse` and `open` classmethods for a type with an
/// inherent `parse` method taking a reader.
#[macro_export]
macro_rules! impl_py_parse {
    ($ty:ty) => {
        #[::pyo3::pymethods]
        impl $ty {
            #[classmethod]
            #[pyo3(name = "parse")]
            fn py_parse(
                _cls: &::pyo3::Bound<'_, ::pyo3::types::PyType>,
                py: ::pyo3::Python<'_>,
                data: &[u8],
            ) -> ::pyo3::PyResult<Self> {
                py.detach(|| Self::parse(data))
                    .map_err($crate::python::parse_error)
            }

            #[classmethod]
            #[pyo3(name = "open")]
            fn py_open(
                cls: &::pyo3::Bound<'_, ::pyo3::types::PyType>,
                py: ::pyo3::Python<'_>,
                path: ::std::path::PathBuf,
            ) -> ::pyo3::PyResult<Self> {
                Self::py_parse(cls, py, &::std::fs::read(path)?)
            }
        }
    };
}
