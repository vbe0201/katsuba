use pyo3::{
    buffer::PyBuffer,
    prelude::*,
    types::{PyBytes, PyString},
};

fn hash_with<F: Fn(&[u8]) -> u32>(input: &Bound<'_, PyAny>, f: F) -> PyResult<u32> {
    // str doesn't implement buffer protocol
    if let Ok(s) = input.cast::<PyString>() {
        return s.to_str().map(|s| f(s.as_bytes()));
    }

    // bytes has safe zero-copy access
    if let Ok(b) = input.cast::<PyBytes>() {
        return Ok(f(b.as_bytes()));
    }

    // For bytearray, memoryview, etc., use buffer protocol (requires copy)
    let buffer: PyBuffer<u8> = PyBuffer::get(input)?;
    Ok(f(&buffer.to_vec(input.py())?))
}

/// Hashes the given `input` using the KingsIsle String ID algorithm.
#[pyfunction]
fn string_id(input: &Bound<'_, PyAny>) -> PyResult<u32> {
    hash_with(input, katsuba_utils::hash::string_id)
}

/// Hashes the given `input` using the DJB2 algorithm.
#[pyfunction]
fn djb2(input: &Bound<'_, PyAny>) -> PyResult<u32> {
    hash_with(input, katsuba_utils::hash::djb2)
}

pub fn katsuba_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(string_id, m)?)?;
    m.add_function(wrap_pyfunction!(djb2, m)?)?;

    Ok(())
}
