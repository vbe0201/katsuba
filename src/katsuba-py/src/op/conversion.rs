use katsuba_object_property::value::*;
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyBytes, PyDict, PyList},
};

use super::Object;

fn convert_to_utf16<'py>(py: Python<'py>, x: &[u16]) -> Bound<'py, PyAny> {
    match String::from_utf16(x) {
        Ok(s) => s.into_bound_py_any(py).unwrap(),
        Err(_) => PyBytes::new(py, bytemuck::cast_slice(x)).into_any(),
    }
}

/// Converts a [`Value`] to a native Python object, recursively resolving all nested values.
///
/// - [`Value::Object`] becomes an [`ObjectProperties`] dict subclass keyed by field name.
/// - [`Value::List`] becomes a plain Python `list`.
/// - All leaf types (Vec3, Color, …) become a `dict` with named fields.
/// - Primitives and strings map to their natural Python equivalents.
pub fn value_to_python<'py>(value: &Value, py: Python<'py>) -> Bound<'py, PyAny> {
    match value {
        Value::Empty => py.None().into_bound(py).into_any(),

        Value::Unsigned(v) => v.into_bound_py_any(py).unwrap(),
        Value::Signed(v) | Value::Enum(v) => v.into_bound_py_any(py).unwrap(),
        Value::Float(v) => v.into_bound_py_any(py).unwrap(),
        Value::Bool(v) => v.into_bound_py_any(py).unwrap(),

        Value::String(v) => v.0.as_slice().into_bound_py_any(py).unwrap(),
        Value::WString(v) => convert_to_utf16(py, &v.0),

        Value::List(list) => {
            let items: Vec<Bound<'py, PyAny>> =
                list.inner.iter().map(|v| value_to_python(v, py)).collect();
            PyList::new(py, items).unwrap().into_any()
        }

        Value::Object(obj) => {
            let bound = Py::new(py, Object::new(obj.type_hash))
                .unwrap()
                .into_bound(py);
            {
                let dict = bound.as_any().cast::<PyDict>().unwrap();
                for (k, v) in obj.iter() {
                    dict.set_item(k.as_ref(), value_to_python(v, py)).unwrap();
                }
            }
            bound.into_any()
        }

        Value::Color(v) => {
            let Color { r, g, b, a } = *v;
            let d = PyDict::new(py);
            d.set_item("r", r).unwrap();
            d.set_item("g", g).unwrap();
            d.set_item("b", b).unwrap();
            d.set_item("a", a).unwrap();
            d.into_any()
        }

        Value::Vec3(v) => {
            let Vec3 { x, y, z } = *v;
            let d = PyDict::new(py);
            d.set_item("x", x).unwrap();
            d.set_item("y", y).unwrap();
            d.set_item("z", z).unwrap();
            d.into_any()
        }

        Value::Quat(v) => {
            let Quaternion { x, y, z, w } = *v;
            let d = PyDict::new(py);
            d.set_item("x", x).unwrap();
            d.set_item("y", y).unwrap();
            d.set_item("z", z).unwrap();
            d.set_item("w", w).unwrap();
            d.into_any()
        }

        Value::Euler(v) => {
            let Euler { pitch, roll, yaw } = *v;
            let d = PyDict::new(py);
            d.set_item("pitch", pitch).unwrap();
            d.set_item("roll", roll).unwrap();
            d.set_item("yaw", yaw).unwrap();
            d.into_any()
        }

        Value::Mat3x3(v) => {
            let Matrix { i, j, k } = **v;
            let d = PyDict::new(py);
            d.set_item("i", PyList::new(py, i).unwrap()).unwrap();
            d.set_item("j", PyList::new(py, j).unwrap()).unwrap();
            d.set_item("k", PyList::new(py, k).unwrap()).unwrap();
            d.into_any()
        }

        Value::PointInt(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item("x", x).unwrap();
            d.set_item("y", y).unwrap();
            d.into_any()
        }

        Value::PointUChar(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item("x", x).unwrap();
            d.set_item("y", y).unwrap();
            d.into_any()
        }

        Value::PointUInt(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item("x", x).unwrap();
            d.set_item("y", y).unwrap();
            d.into_any()
        }

        Value::PointFloat(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item("x", x).unwrap();
            d.set_item("y", y).unwrap();
            d.into_any()
        }

        Value::SizeInt(v) => {
            let Size { width, height } = *v;
            let d = PyDict::new(py);
            d.set_item("width", width).unwrap();
            d.set_item("height", height).unwrap();
            d.into_any()
        }

        Value::RectInt(v) => {
            let Rect {
                left,
                top,
                right,
                bottom,
            } = *v;
            let d = PyDict::new(py);
            d.set_item("left", left).unwrap();
            d.set_item("top", top).unwrap();
            d.set_item("right", right).unwrap();
            d.set_item("bottom", bottom).unwrap();
            d.into_any()
        }

        Value::RectFloat(v) => {
            let Rect {
                left,
                top,
                right,
                bottom,
            } = *v;
            let d = PyDict::new(py);
            d.set_item("left", left).unwrap();
            d.set_item("top", top).unwrap();
            d.set_item("right", right).unwrap();
            d.set_item("bottom", bottom).unwrap();
            d.into_any()
        }
    }
}
