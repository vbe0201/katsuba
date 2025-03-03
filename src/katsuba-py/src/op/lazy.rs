use std::{ptr::NonNull, sync::Arc};

use katsuba_object_property::value::{List, Object, Value};
use pyo3::{
    exceptions::{PyIndexError, PyKeyError},
    prelude::*,
    types::PyTuple,
};

use super::{conversion::value_to_python, TypeList};

#[derive(Clone)]
#[pyclass(module = "katsuba.op")]
pub struct LazyList(Arc<Value>, NonNull<List>);

impl LazyList {
    // SAFETY: `current` must be derived from `base` in some way.
    pub unsafe fn new(base: Arc<Value>, current: &List) -> Self {
        Self(base, NonNull::from(current))
    }

    #[inline(always)]
    fn get_ref(&self) -> &List {
        // SAFETY: Constructor ensures our list is fine and we never get a mut ref.
        unsafe { self.1.as_ref() }
    }
}

// SAFETY: Raw pointers are never exposed for mutation.
unsafe impl Send for LazyList {}
unsafe impl Sync for LazyList {}

#[pymethods]
impl LazyList {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<LazyListIter>> {
        let iter = LazyListIter {
            list: slf.clone(),
            idx: 0,
        };

        Py::new(slf.py(), iter)
    }

    pub fn __len__(&self) -> usize {
        let list = self.get_ref();
        list.len()
    }

    pub fn __getitem__(&self, py: Python<'_>, idx: usize) -> PyResult<PyObject> {
        let list = self.get_ref();

        list.get(idx)
            .map(|v| unsafe { value_to_python(self.0.clone(), v, py) })
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

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        let idx = slf.idx;
        slf.idx += 1;

        slf.list.__getitem__(slf.py(), idx).ok()
    }
}

#[derive(Clone)]
#[pyclass(module = "katsuba.op")]
pub struct LazyObject(Arc<Value>, u32, NonNull<Object>);

impl LazyObject {
    // SAFETY: `current` must be derived from `base` in some way.
    pub unsafe fn new(base: Arc<Value>, hash: u32, current: &Object) -> Self {
        Self(base, hash, NonNull::from(current))
    }

    #[inline(always)]
    fn get_ref(&self) -> &Object {
        // SAFETY: Constructor ensures our list is fine and we never get a mut ref.
        unsafe { self.2.as_ref() }
    }
}

#[pymethods]
impl LazyObject {
    #[getter]
    pub fn type_hash(&self) -> u32 {
        self.1
    }

    pub fn __len__(&self) -> usize {
        let obj = self.get_ref();
        obj.len()
    }

    pub fn __contains__(&self, key: &str) -> bool {
        let obj = self.get_ref();
        obj.contains_key(key)
    }

    pub fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<PyObject> {
        self.get(py, key)
            .ok_or_else(|| PyKeyError::new_err(key.to_string()))
    }

    pub fn get(&self, py: Python<'_>, key: &str) -> Option<PyObject> {
        let obj = self.get_ref();

        obj.get(key)
            .map(|v| unsafe { value_to_python(self.0.clone(), v, py) })
    }

    pub fn items(&self, py: Python<'_>, types: &TypeList) -> PyResult<Py<LazyObjectIter>> {
        let iter = LazyObjectIter {
            object: self.clone(),
            entry: types.find(self.1)?,
            idx: 0,
        };

        Py::new(py, iter)
    }
}

#[pyclass(module = "katsuba.op")]
pub struct LazyObjectIter {
    object: LazyObject,
    entry: katsuba_types::TypeDef,
    idx: usize,
}

#[pymethods]
impl LazyObjectIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Bound<'_, PyTuple>> {
        loop {
            let idx = slf.idx;
            slf.idx += 1;

            // If this returns None, there are no valid properties anymore.
            let property = slf.entry.properties.get(idx)?;

            // There is a property, but it might not be in the object due to
            // selective deserialization with flag masks.
            if let Ok(v) = slf.object.__getitem__(slf.py(), &property.name) {
                let name = property.name.into_pyobject(slf.py()).unwrap();
                return (name, v).into_pyobject(slf.py()).ok();
            }
        }
    }
}

// SAFETY: Raw pointers are never exposed for mutation.
unsafe impl Send for LazyObject {}
unsafe impl Sync for LazyObject {}
