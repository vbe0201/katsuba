use katsuba_object_property::value::*;
use katsuba_types::TypeList;
use katsuba_types::indexmap::IndexMap;
use pyo3::{
    FromPyObject, IntoPyObjectExt,
    exceptions::{PyTypeError, PyValueError},
    intern,
    prelude::*,
    types::{PyBytes, PyDict, PyList, PyString},
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
/// - [`Value::Object`] becomes a dict subclass keyed by field name.
/// - [`Value::List`] becomes a plain Python `list`.
/// - All leaf types (Vec3, Color, ...) become a `dict` with named fields.
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
            d.set_item(intern!(py, "r"), r).unwrap();
            d.set_item(intern!(py, "g"), g).unwrap();
            d.set_item(intern!(py, "b"), b).unwrap();
            d.set_item(intern!(py, "a"), a).unwrap();
            d.into_any()
        }

        Value::Vec3(v) => {
            let Vec3 { x, y, z } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "x"), x).unwrap();
            d.set_item(intern!(py, "y"), y).unwrap();
            d.set_item(intern!(py, "z"), z).unwrap();
            d.into_any()
        }

        Value::Quat(v) => {
            let Quaternion { x, y, z, w } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "x"), x).unwrap();
            d.set_item(intern!(py, "y"), y).unwrap();
            d.set_item(intern!(py, "z"), z).unwrap();
            d.set_item(intern!(py, "w"), w).unwrap();
            d.into_any()
        }

        Value::Euler(v) => {
            let Euler { pitch, roll, yaw } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "pitch"), pitch).unwrap();
            d.set_item(intern!(py, "roll"), roll).unwrap();
            d.set_item(intern!(py, "yaw"), yaw).unwrap();
            d.into_any()
        }

        Value::Mat3x3(v) => {
            let Matrix { i, j, k } = **v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "i"), PyList::new(py, i).unwrap())
                .unwrap();
            d.set_item(intern!(py, "j"), PyList::new(py, j).unwrap())
                .unwrap();
            d.set_item(intern!(py, "k"), PyList::new(py, k).unwrap())
                .unwrap();
            d.into_any()
        }

        Value::PointInt(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "x"), x).unwrap();
            d.set_item(intern!(py, "y"), y).unwrap();
            d.into_any()
        }

        Value::PointUChar(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "x"), x).unwrap();
            d.set_item(intern!(py, "y"), y).unwrap();
            d.into_any()
        }

        Value::PointUInt(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "x"), x).unwrap();
            d.set_item(intern!(py, "y"), y).unwrap();
            d.into_any()
        }

        Value::PointFloat(v) => {
            let Point { x, y } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "x"), x).unwrap();
            d.set_item(intern!(py, "y"), y).unwrap();
            d.into_any()
        }

        Value::SizeInt(v) => {
            let Size { width, height } = *v;
            let d = PyDict::new(py);
            d.set_item(intern!(py, "width"), width).unwrap();
            d.set_item(intern!(py, "height"), height).unwrap();
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
            d.set_item(intern!(py, "left"), left).unwrap();
            d.set_item(intern!(py, "top"), top).unwrap();
            d.set_item(intern!(py, "right"), right).unwrap();
            d.set_item(intern!(py, "bottom"), bottom).unwrap();
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
            d.set_item(intern!(py, "left"), left).unwrap();
            d.set_item(intern!(py, "top"), top).unwrap();
            d.set_item(intern!(py, "right"), right).unwrap();
            d.set_item(intern!(py, "bottom"), bottom).unwrap();
            d.into_any()
        }
    }
}

/// Converts a `katsuba.op.Object` instance (or `None`) into a [`Value`],
/// resolving each of its entries against `type_def`'s declared properties.
///
/// This is the reverse of [`value_to_python`].
pub(super) fn python_to_object(obj: &Bound<'_, PyAny>, types: &TypeList) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Empty);
    }

    let py_obj = obj
        .cast::<Object>()
        .map_err(|_| PyTypeError::new_err("expected None or a katsuba.op.Object instance"))?;

    let type_hash = py_obj.borrow().type_hash();
    let type_def = types
        .0
        .get(&type_hash)
        .ok_or_else(|| PyValueError::new_err(format!("unknown type hash '{type_hash}'")))?;

    let dict = py_obj.as_any().cast::<PyDict>().unwrap();

    let mut inner = IndexMap::with_capacity(dict.len());
    for (key, value) in dict.iter() {
        let key: String = key.extract()?;
        let property = type_def.property_by_name(&key).ok_or_else(|| {
            PyValueError::new_err(format!(
                "type '{}' has no property named \"{key}\"",
                type_def.name
            ))
        })?;

        let v = python_to_value(&value, property, types)?;
        inner.insert(property.name.clone(), v);
    }

    Ok(Value::Object(Box::new(
        katsuba_object_property::value::Object { type_hash, inner },
    )))
}

fn python_to_value(
    obj: &Bound<'_, PyAny>,
    property: &katsuba_types::Property,
    types: &TypeList,
) -> PyResult<Value> {
    if property.dynamic {
        let list = obj.cast::<PyList>().map_err(|_| {
            PyTypeError::new_err(format!("property '{}' expects a list", property.name))
        })?;

        let mut inner = Vec::with_capacity(list.len());
        for item in list.try_iter()? {
            inner.push(python_to_scalar(&item?, property, types)?);
        }

        Ok(Value::List(List { inner }))
    } else {
        python_to_scalar(obj, property, types)
    }
}

fn python_to_scalar(
    obj: &Bound<'_, PyAny>,
    property: &katsuba_types::Property,
    types: &TypeList,
) -> PyResult<Value> {
    if property.is_enum() {
        return python_to_enum(obj);
    }

    match &*property.r#type {
        "bool" => obj.extract::<bool>().map(Value::Bool),
        "char" | "unsigned char" | "short" | "unsigned short" | "wchar_t" | "int" | "long"
        | "unsigned int" | "unsigned long" | "unsigned __int64" | "gid" | "union gid" | "bi2"
        | "bui2" | "bi3" | "bui3" | "bi4" | "bui4" | "bi5" | "bui5" | "bi6" | "bui6" | "bi7"
        | "bui7" | "s24" | "u24" => python_to_int(obj),
        "float" | "double" => obj.extract::<f64>().map(Value::Float),
        "std::string" => python_to_str(obj).map(Value::String),
        "std::wstring" => python_to_wstr(obj).map(Value::WString),
        "class Color" => python_to_color(obj).map(Value::Color),
        "class Vector3D" => python_to_vec3(obj).map(Value::Vec3),
        "class Quaternion" => python_to_quat(obj).map(Value::Quat),
        "class Euler" => python_to_euler(obj).map(Value::Euler),
        "class Matrix3x3" => python_to_matrix(obj).map(|m| Value::Mat3x3(Box::new(m))),
        "class Size<int>" => python_to_size(obj).map(Value::SizeInt),
        "class Point<int>" => python_to_point::<i32>(obj).map(Value::PointInt),
        "class Point<unsigned char>" => python_to_point::<u8>(obj).map(Value::PointUChar),
        "class Point<unsigned int>" => python_to_point::<u32>(obj).map(Value::PointUInt),
        "class Point<float>" => python_to_point::<f32>(obj).map(Value::PointFloat),
        "class Rect<int>" => python_to_rect::<i32>(obj).map(Value::RectInt),
        "class Rect<float>" => python_to_rect::<f32>(obj).map(Value::RectFloat),

        // Anything else is assumed to be a nested PropertyClass.
        _ => python_to_object(obj, types),
    }
}

fn python_to_int(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if let Ok(v) = obj.extract::<i64>() {
        Ok(Value::Signed(v))
    } else if let Ok(v) = obj.extract::<u64>() {
        Ok(Value::Unsigned(v))
    } else {
        Err(PyTypeError::new_err("expected an integer"))
    }
}

fn python_to_enum(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if let Ok(v) = obj.extract::<i64>() {
        Ok(Value::Signed(v))
    } else if let Ok(v) = obj.extract::<u64>() {
        Ok(Value::Unsigned(v))
    } else if let Ok(s) = obj.cast::<PyString>() {
        Ok(Value::String(CxxStr(s.to_str()?.as_bytes().to_vec())))
    } else {
        Err(PyTypeError::new_err(
            "expected an integer or a string naming an enum variant",
        ))
    }
}

/// Accepts raw `bytes` directly, or a `str` encoded as UTF-8; unlike the
/// JSON input path, there is no need for a lossless escape scheme here
/// since Python natively distinguishes `bytes` from `str`.
fn python_to_str(obj: &Bound<'_, PyAny>) -> PyResult<CxxStr> {
    if let Ok(b) = obj.cast::<PyBytes>() {
        Ok(CxxStr(b.as_bytes().to_vec()))
    } else if let Ok(s) = obj.cast::<PyString>() {
        Ok(CxxStr(s.to_str()?.as_bytes().to_vec()))
    } else {
        Err(PyTypeError::new_err("expected str or bytes"))
    }
}

/// Accepts a `str` (encoded to UTF-16), or raw `bytes` interpreted as
/// little-endian `u16` units directly - the latter is the only way to
/// express an unpaired surrogate, since Rust's `String` cannot hold one.
fn python_to_wstr(obj: &Bound<'_, PyAny>) -> PyResult<CxxWStr> {
    if let Ok(b) = obj.cast::<PyBytes>() {
        let bytes = b.as_bytes();
        if bytes.len() % 2 != 0 {
            return Err(PyValueError::new_err(
                "wstring bytes must have an even length",
            ));
        }

        let (chunks, _) = bytes.as_chunks::<2>();
        let units = chunks.iter().map(|&c| u16::from_le_bytes(c)).collect();
        Ok(CxxWStr(units))
    } else if let Ok(s) = obj.cast::<PyString>() {
        Ok(CxxWStr(s.to_str()?.encode_utf16().collect()))
    } else {
        Err(PyTypeError::new_err("expected str or bytes"))
    }
}

fn as_dict<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    obj.cast::<PyDict>()
        .map_err(|_| PyTypeError::new_err("expected a dict"))
        .cloned()
}

fn field<'py, T>(dict: &Bound<'py, PyDict>, key: &Bound<'py, PyString>) -> PyResult<T>
where
    T: for<'a> FromPyObject<'a, 'py, Error = PyErr>,
{
    let value = dict
        .get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing field '{key}'")))?;
    value.extract()
}

fn python_to_color(obj: &Bound<'_, PyAny>) -> PyResult<Color> {
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Color {
        r: field(&d, intern!(py, "r"))?,
        g: field(&d, intern!(py, "g"))?,
        b: field(&d, intern!(py, "b"))?,
        a: field(&d, intern!(py, "a"))?,
    })
}

fn python_to_vec3(obj: &Bound<'_, PyAny>) -> PyResult<Vec3> {
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Vec3 {
        x: field(&d, intern!(py, "x"))?,
        y: field(&d, intern!(py, "y"))?,
        z: field(&d, intern!(py, "z"))?,
    })
}

fn python_to_quat(obj: &Bound<'_, PyAny>) -> PyResult<Quaternion> {
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Quaternion {
        x: field(&d, intern!(py, "x"))?,
        y: field(&d, intern!(py, "y"))?,
        z: field(&d, intern!(py, "z"))?,
        w: field(&d, intern!(py, "w"))?,
    })
}

fn python_to_euler(obj: &Bound<'_, PyAny>) -> PyResult<Euler> {
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Euler {
        pitch: field(&d, intern!(py, "pitch"))?,
        yaw: field(&d, intern!(py, "yaw"))?,
        roll: field(&d, intern!(py, "roll"))?,
    })
}

fn python_to_row<'py>(d: &Bound<'py, PyDict>, key: &Bound<'py, PyString>) -> PyResult<[f32; 3]> {
    let list: Vec<f32> = field(d, key)?;
    list.try_into()
        .map_err(|_: Vec<f32>| PyValueError::new_err(format!("field '{key}' needs 3 elements")))
}

fn python_to_matrix(obj: &Bound<'_, PyAny>) -> PyResult<Matrix> {
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Matrix {
        i: python_to_row(&d, intern!(py, "i"))?,
        j: python_to_row(&d, intern!(py, "j"))?,
        k: python_to_row(&d, intern!(py, "k"))?,
    })
}

fn python_to_size(obj: &Bound<'_, PyAny>) -> PyResult<Size<i32>> {
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Size {
        width: field(&d, intern!(py, "width"))?,
        height: field(&d, intern!(py, "height"))?,
    })
}

fn python_to_point<'py, T>(obj: &Bound<'py, PyAny>) -> PyResult<Point<T>>
where
    T: for<'a> FromPyObject<'a, 'py, Error = PyErr>,
{
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Point {
        x: field(&d, intern!(py, "x"))?,
        y: field(&d, intern!(py, "y"))?,
    })
}

fn python_to_rect<'py, T>(obj: &Bound<'py, PyAny>) -> PyResult<Rect<T>>
where
    T: for<'a> FromPyObject<'a, 'py, Error = PyErr>,
{
    let d = as_dict(obj)?;
    let py = d.py();
    Ok(Rect {
        left: field(&d, intern!(py, "left"))?,
        top: field(&d, intern!(py, "top"))?,
        right: field(&d, intern!(py, "right"))?,
        bottom: field(&d, intern!(py, "bottom"))?,
    })
}
