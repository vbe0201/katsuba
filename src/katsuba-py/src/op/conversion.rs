use std::{ptr, sync::Arc};

use katsuba_object_property::value::*;
use pyo3::{IntoPyObjectExt, prelude::*, types::PyBytes};

use super::{lazy::*, leaf_types};

fn convert_to_utf16(py: Python<'_>, x: &[u16]) -> Py<PyAny> {
    let ptr = x.as_ptr().cast::<u8>();
    let len = x.len() * 2;

    unsafe {
        let unicode = pyo3::ffi::PyUnicode_DecodeUTF16(
            ptr.cast(),
            len as isize,
            // "strict" errors are the default
            ptr::null(),
            // native byte ordering
            ptr::null_mut(),
        );

        // If we successfully decode the string, we can return it as-is.
        // Otherwise, handing back the raw bytes seems most reasonable.
        match Py::from_owned_ptr_or_opt(py, unicode) {
            Some(v) => v,
            None => PyBytes::from_ptr(py, ptr, len).into(),
        }
    }
}

// SAFETY: `value` must be derived from `base` in some way.
pub unsafe fn value_to_python(base: Arc<Value>, value: &Value, py: Python<'_>) -> Py<PyAny> {
    match value {
        Value::Empty => py.None(),

        Value::Unsigned(v) => v.into_py_any(py).unwrap(),
        Value::Signed(v) | Value::Enum(v) => v.into_py_any(py).unwrap(),
        Value::Float(v) => v.into_py_any(py).unwrap(),
        Value::Bool(v) => v.into_py_any(py).unwrap(),

        Value::String(v) => v.0.as_slice().into_py_any(py).unwrap(),
        Value::WString(v) => convert_to_utf16(py, &v.0),

        Value::List(v) => unsafe { LazyList::new(base, v).into_py_any(py).unwrap() },
        Value::Object { hash, obj } => unsafe { LazyObject::new(base, *hash, obj).into_py_any(py).unwrap() },

        Value::Color(v) => {
            let Color { r, g, b, a } = *v;
            leaf_types::Color { r, g, b, a }.into_py_any(py).unwrap()
        }
        Value::Vec3(v) => {
            let Vec3 { x, y, z } = *v;
            leaf_types::Vec3 { x, y, z }.into_py_any(py).unwrap()
        }
        Value::Quat(v) => {
            let Quaternion { x, y, z, w } = *v;
            leaf_types::Quaternion { x, y, z, w }.into_py_any(py).unwrap()
        }
        Value::Euler(v) => {
            let Euler { pitch, roll, yaw } = *v;
            leaf_types::Euler { pitch, roll, yaw }.into_py_any(py).unwrap()
        }
        Value::Mat3x3(v) => {
            let Matrix { i, j, k } = **v;
            leaf_types::Matrix { i, j, k }.into_py_any(py).unwrap()
        }

        Value::PointInt(v) => {
            let Point { x, y } = *v;
            leaf_types::PointInt { x, y }.into_py_any(py).unwrap()
        }
        Value::PointFloat(v) => {
            let Point { x, y } = *v;
            leaf_types::PointFloat { x, y }.into_py_any(py).unwrap()
        }

        Value::SizeInt(v) => {
            let Size { width, height } = *v;
            leaf_types::SizeInt { width, height }.into_py_any(py).unwrap()
        }

        Value::RectInt(v) => {
            let Rect {
                left,
                top,
                right,
                bottom,
            } = *v;
            leaf_types::RectInt {
                left,
                top,
                right,
                bottom,
            }
            .into_py_any(py).unwrap()
        }
        Value::RectFloat(v) => {
            let Rect {
                left,
                top,
                right,
                bottom,
            } = *v;
            leaf_types::RectFloat {
                left,
                top,
                right,
                bottom,
            }
            .into_py_any(py).unwrap()
        }
    }
}
