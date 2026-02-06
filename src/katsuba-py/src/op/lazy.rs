use std::sync::Arc;

use katsuba_object_property::value::{List, Object, Value};
use pyo3::{
    exceptions::{PyIndexError, PyKeyError},
    prelude::*,
    types::PyTuple,
};

use super::conversion::value_to_python;

/// A path segment for navigating into nested values.
#[derive(Clone)]
pub enum PathSegment {
    Key(Arc<str>),
    Index(usize),
}

/// Navigates through a Value tree following a path.
fn navigate<'a>(root: &'a Value, path: &[PathSegment]) -> &'a Value {
    let mut current = root;
    for segment in path {
        current = match (current, segment) {
            (Value::Object(obj), PathSegment::Key(key)) => obj.get(key.as_ref()).unwrap(),
            (Value::List(list), PathSegment::Index(idx)) => list.get(*idx).unwrap(),
            _ => panic!("invalid path segment for current value type"),
        };
    }
    current
}

#[derive(Clone)]
#[pyclass(module = "katsuba.op", skip_from_py_object)]
pub struct LazyList {
    root: Arc<Value>,
    path: Vec<PathSegment>,
}

impl LazyList {
    pub fn new(root: Arc<Value>, path: Vec<PathSegment>) -> Self {
        Self { root, path }
    }

    #[inline]
    fn get_ref(&self) -> &List {
        match navigate(&self.root, &self.path) {
            Value::List(list) => list,
            _ => unreachable!(),
        }
    }
}

#[pymethods]
impl LazyList {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<LazyListIter>> {
        Py::new(
            slf.py(),
            LazyListIter {
                list: slf.clone(),
                idx: 0,
            },
        )
    }

    pub fn __len__(&self) -> usize {
        self.get_ref().len()
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, idx: usize) -> PyResult<Bound<'py, PyAny>> {
        self.get_ref()
            .get(idx)
            .map(|v| {
                let mut path = self.path.clone();
                path.push(PathSegment::Index(idx));
                value_to_python(Arc::clone(&self.root), path, v, py)
            })
            .ok_or_else(|| PyIndexError::new_err("list index out of range"))
    }
}

#[pyclass(module = "katsuba.op")]
pub struct LazyListIter {
    list: LazyList,
    idx: usize,
}

#[pymethods]
impl LazyListIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Bound<'_, PyAny>> {
        let idx = slf.idx;
        slf.idx += 1;
        slf.list.__getitem__(slf.py(), idx).ok()
    }
}

#[derive(Clone)]
#[pyclass(module = "katsuba.op", skip_from_py_object)]
pub struct LazyObject {
    root: Arc<Value>,
    path: Vec<PathSegment>,
}

impl LazyObject {
    pub fn new(root: Arc<Value>, path: Vec<PathSegment>) -> Self {
        Self { root, path }
    }

    #[inline]
    fn get_ref(&self) -> &Object {
        match navigate(&self.root, &self.path) {
            Value::Object(obj) => obj.as_ref(),
            _ => panic!("path does not lead to an Object"),
        }
    }
}

#[pymethods]
impl LazyObject {
    #[getter]
    pub fn type_hash(&self) -> u32 {
        self.get_ref().type_hash
    }

    pub fn __len__(&self) -> usize {
        self.get_ref().len()
    }

    pub fn __contains__(&self, key: &str) -> bool {
        self.get_ref().contains_key(key)
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, key: &str) -> PyResult<Bound<'py, PyAny>> {
        self.get(py, key)
            .ok_or_else(|| PyKeyError::new_err(key.to_string()))
    }

    pub fn get<'py>(&self, py: Python<'py>, key: &str) -> Option<Bound<'py, PyAny>> {
        let obj = self.get_ref();
        obj.get_key_value(key).map(|(k, v)| {
            let mut path = self.path.clone();
            path.push(PathSegment::Key(Arc::clone(k)));
            value_to_python(Arc::clone(&self.root), path, v, py)
        })
    }

    pub fn get_index<'py>(&self, py: Python<'py>, idx: usize) -> Option<Bound<'py, PyTuple>> {
        let obj = self.get_ref();
        let (key, value) = obj.get_index(idx)?;

        let mut path = self.path.clone();
        path.push(PathSegment::Key(Arc::clone(key)));

        let key = key.as_ref().into_pyobject(py).unwrap();
        let value = value_to_python(Arc::clone(&self.root), path, value, py);
        (key, value).into_pyobject(py).ok()
    }

    pub fn items(&self, py: Python<'_>) -> PyResult<Py<LazyObjectIter>> {
        Py::new(
            py,
            LazyObjectIter {
                object: self.clone(),
                idx: 0,
            },
        )
    }
}

#[pyclass(module = "katsuba.op")]
pub struct LazyObjectIter {
    object: LazyObject,
    idx: usize,
}

#[pymethods]
impl LazyObjectIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Bound<'_, PyTuple>> {
        let idx = slf.idx;
        slf.idx += 1;
        slf.object.get_index(slf.py(), idx)
    }
}
