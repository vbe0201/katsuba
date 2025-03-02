use std::{ptr, sync::Arc};

use katsuba_object_property::value::*;
use pyo3::{prelude::*, types::PyBytes};

use super::{lazy::*, leaf_types};

fn convert_to_utf16(py: Python<'_>, x: &[u16]) -> PyObject {
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
        match PyObject::from_owned_ptr_or_opt(py, unicode) {
            Some(v) => v,
            None => PyBytes::from_ptr(py, ptr, len).into(),
        }
    }
}

// SAFETY: `value` must be derived from `base` in some way.
pub unsafe fn value_to_python(base: Arc<Value>, value: &Value, py: Python<'_>) -> PyObject {
    match value {
        Value::Empty => py.None(),

        Value::Unsigned(v) => v.into_py(py),
        Value::Signed(v) | Value::Enum(v) => v.into_py(py),
        Value::Float(v) => v.into_py(py),
        Value::Bool(v) => v.into_py(py),

        Value::String(v) => v.0.as_slice().into_py(py),
        Value::WString(v) => convert_to_utf16(py, &v.0),

        Value::List(v) => unsafe { LazyList::new(base, v).into_py(py) },
        Value::Object { hash, obj } => unsafe { LazyObject::new(base, *hash, obj).into_py(py) },

        Value::Color(v) => {
            let Color { r, g, b, a } = *v;
            leaf_types::Color { r, g, b, a }.into_py(py)
        }
        Value::Vec3(v) => {
            let Vec3 { x, y, z } = *v;
            leaf_types::Vec3 { x, y, z }.into_py(py)
        }
        Value::Quat(v) => {
            let Quaternion { x, y, z, w } = *v;
            leaf_types::Quaternion { x, y, z, w }.into_py(py)
        }
        Value::Euler(v) => {
            let Euler { pitch, roll, yaw } = *v;
            leaf_types::Euler { pitch, roll, yaw }.into_py(py)
        }
        Value::Mat3x3(v) => {
            let Matrix { i, j, k } = **v;
            leaf_types::Matrix { i, j, k }.into_py(py)
        }

        Value::PointInt(v) => {
            let Point { x, y } = *v;
            leaf_types::PointInt { x, y }.into_py(py)
        }
        Value::PointFloat(v) => {
            let Point { x, y } = *v;
            leaf_types::PointFloat { x, y }.into_py(py)
        }

        Value::SizeInt(v) => {
            let Size { width, height } = *v;
            leaf_types::SizeInt { width, height }.into_py(py)
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
            .into_py(py)
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
            .into_py(py)
        }
    }
}
