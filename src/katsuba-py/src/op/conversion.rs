use std::sync::Arc;

use katsuba_object_property::value::*;
use pyo3::{IntoPyObjectExt, prelude::*, types::PyBytes};

use super::{lazy::*, leaf_types};

fn convert_to_utf16<'py>(py: Python<'py>, x: &[u16]) -> Bound<'py, PyAny> {
    match String::from_utf16(x) {
        Ok(s) => s.into_bound_py_any(py).unwrap(),
        Err(_) => PyBytes::new(py, bytemuck::cast_slice(x)).into_any(),
    }
}

pub fn value_to_python<'py>(
    root: Arc<Value>,
    path: Vec<PathSegment>,
    value: &Value,
    py: Python<'py>,
) -> Bound<'py, PyAny> {
    match value {
        Value::Empty => py.None().into_bound(py).into_any(),

        Value::Unsigned(v) => v.into_bound_py_any(py).unwrap(),
        Value::Signed(v) | Value::Enum(v) => v.into_bound_py_any(py).unwrap(),
        Value::Float(v) => v.into_bound_py_any(py).unwrap(),
        Value::Bool(v) => v.into_bound_py_any(py).unwrap(),

        Value::String(v) => v.0.as_slice().into_bound_py_any(py).unwrap(),
        Value::WString(v) => convert_to_utf16(py, &v.0),

        Value::List(_) => LazyList::new(root, path).into_bound_py_any(py).unwrap(),
        Value::Object(_) => LazyObject::new(root, path).into_bound_py_any(py).unwrap(),

        Value::Color(v) => {
            let Color { r, g, b, a } = *v;
            leaf_types::Color { r, g, b, a }
                .into_bound_py_any(py)
                .unwrap()
        }
        Value::Vec3(v) => {
            let Vec3 { x, y, z } = *v;
            leaf_types::Vec3 { x, y, z }.into_bound_py_any(py).unwrap()
        }
        Value::Quat(v) => {
            let Quaternion { x, y, z, w } = *v;
            leaf_types::Quaternion { x, y, z, w }
                .into_bound_py_any(py)
                .unwrap()
        }
        Value::Euler(v) => {
            let Euler { pitch, roll, yaw } = *v;
            leaf_types::Euler { pitch, roll, yaw }
                .into_bound_py_any(py)
                .unwrap()
        }
        Value::Mat3x3(v) => {
            let Matrix { i, j, k } = **v;
            leaf_types::Matrix { i, j, k }
                .into_bound_py_any(py)
                .unwrap()
        }

        Value::PointInt(v) => {
            let Point { x, y } = *v;
            leaf_types::PointInt { x, y }.into_bound_py_any(py).unwrap()
        }
        Value::PointUChar(v) => {
            let Point { x, y } = *v;
            leaf_types::PointUChar { x, y }
                .into_bound_py_any(py)
                .unwrap()
        }
        Value::PointUInt(v) => {
            let Point { x, y } = *v;
            leaf_types::PointUInt { x, y }
                .into_bound_py_any(py)
                .unwrap()
        }
        Value::PointFloat(v) => {
            let Point { x, y } = *v;
            leaf_types::PointFloat { x, y }
                .into_bound_py_any(py)
                .unwrap()
        }

        Value::SizeInt(v) => {
            let Size { width, height } = *v;
            leaf_types::SizeInt { width, height }
                .into_bound_py_any(py)
                .unwrap()
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
            .into_bound_py_any(py)
            .unwrap()
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
            .into_bound_py_any(py)
            .unwrap()
        }
    }
}
