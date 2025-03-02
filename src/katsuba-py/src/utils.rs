use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyByteArray, PyBytes, PyString},
};

#[inline]
unsafe fn downcast_to_bytes<'a>(value: &'a Bound<'_, PyAny>) -> PyResult<&'a [u8]> {
    if let Ok(value) = value.downcast::<PyString>() {
        value.to_str().map(str::as_bytes)
    } else if let Ok(value) = value.downcast::<PyBytes>() {
        Ok(value.as_bytes())
    } else if let Ok(value) = value.downcast::<PyByteArray>() {
        Ok(unsafe { value.as_bytes() })
    } else {
        Err(PyTypeError::new_err("Cannot hash the given type"))
    }
}

/// Hashes the given `input` using the KingsIsle String ID algorithm.
#[pyfunction]
fn string_id(input: &Bound<'_, PyAny>) -> PyResult<u32> {
    unsafe { downcast_to_bytes(input).map(katsuba_utils::hash::string_id) }
}

/// Hashes the given `input` using the DJB2 algorithm.
#[pyfunction]
fn djb2(input: &Bound<'_, PyAny>) -> PyResult<u32> {
    unsafe { downcast_to_bytes(input).map(katsuba_utils::hash::djb2) }
}

pub fn katsuba_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(string_id, m)?)?;
    m.add_function(wrap_pyfunction!(djb2, m)?)?;

    Ok(())
}
