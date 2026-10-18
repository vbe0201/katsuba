use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use katsuba_object_property::serde::{self, SerializerFlags};
use katsuba_types::PropertyFlags;
use pyo3::{
    buffer::PyBuffer,
    exceptions::{PyKeyError, PyValueError},
    intern,
    prelude::*,
    types::{PyByteArray, PyByteArrayMethods, PyBytes, PyDict, PyTuple, PyType},
};

use crate::{KatsubaError, error};

mod conversion;

#[derive(Clone)]
#[pyclass(module = "katsuba.op")]
pub struct TypeList(Arc<katsuba_types::TypeList>);

impl TypeList {
    #[inline]
    fn open_impl<P: AsRef<Path>>(path: P) -> PyResult<katsuba_types::TypeList> {
        let file = fs::File::open(path)?;
        katsuba_types::TypeList::from_reader(io::BufReader::new(file))
            .map_err(|e| KatsubaError::new_err(e.to_string()))
    }

    pub fn find(&self, hash: u32) -> PyResult<katsuba_types::TypeDef> {
        self.0
            .0
            .get(&hash)
            .cloned()
            .ok_or_else(|| PyValueError::new_err(format!("'{hash}' not in type list")))
    }
}

#[pymethods]
impl TypeList {
    #[new]
    pub fn new(data: &str) -> PyResult<Self> {
        katsuba_types::TypeList::from_str(data)
            .map(|v| Self(Arc::new(v)))
            .map_err(|e| KatsubaError::new_err(e.to_string()))
    }

    #[classmethod]
    pub fn open(_cls: &Bound<'_, PyType>, path: PathBuf) -> PyResult<Self> {
        Self::open_impl(path).map(|v| Self(Arc::new(v)))
    }

    #[classmethod]
    pub fn open_many(_cls: &Bound<'_, PyType>, paths: Vec<PathBuf>) -> PyResult<Self> {
        let mut types = katsuba_types::TypeList::default();
        for path in paths {
            types.merge(Self::open_impl(path)?);
        }
        Ok(Self(Arc::new(types)))
    }

    pub fn name_for(&self, type_hash: u32) -> PyResult<&str> {
        let entry = self
            .0
            .0
            .get(&type_hash)
            .ok_or_else(|| PyKeyError::new_err(format!("'{type_hash}'")))?;

        Ok(&entry.name)
    }
}

#[derive(Clone, Copy, Default)]
#[pyclass(module = "katsuba.op", from_py_object)]
pub struct SerializerOptions(serde::SerializerOptions);

#[pymethods]
impl SerializerOptions {
    #[new]
    #[pyo3(signature = (
        *,
        flags = None,
        property_mask = None,
        shallow = None,
        manual_compression = None,
        recursion_limit = None,
        skip_unknown_types = None
    ))]
    pub fn new(
        flags: Option<u32>,
        property_mask: Option<u32>,
        shallow: Option<bool>,
        manual_compression: Option<bool>,
        recursion_limit: Option<i8>,
        skip_unknown_types: Option<bool>,
    ) -> Self {
        let mut this = serde::SerializerOptions::default();

        if let Some(v) = flags {
            this.flags = SerializerFlags::from_bits_truncate(v);
        }
        if let Some(v) = property_mask {
            this.property_mask = PropertyFlags::from_bits_truncate(v);
        }
        if let Some(v) = shallow {
            this.shallow = v;
        }
        if let Some(v) = manual_compression {
            this.manual_compression = v;
        }
        if let Some(v) = recursion_limit {
            this.recursion_limit = v;
        }
        if let Some(v) = skip_unknown_types {
            this.skip_unknown_types = v;
        }

        Self(this)
    }

    #[getter]
    pub fn get_flags(&self) -> u32 {
        self.0.flags.bits()
    }

    #[setter]
    pub fn set_flags(&mut self, new: u32) {
        self.0.flags = SerializerFlags::from_bits_truncate(new);
    }

    #[getter]
    pub fn get_property_mask(&self) -> u32 {
        self.0.property_mask.bits()
    }

    #[setter]
    pub fn set_property_mask(&mut self, new: u32) {
        self.0.property_mask = PropertyFlags::from_bits_truncate(new);
    }

    #[getter]
    pub fn get_shallow(&self) -> bool {
        self.0.shallow
    }

    #[setter]
    pub fn set_shallow(&mut self, new: bool) {
        self.0.shallow = new;
    }

    #[getter]
    pub fn get_manual_compression(&self) -> bool {
        self.0.manual_compression
    }

    #[setter]
    pub fn set_manual_compression(&mut self, new: bool) {
        self.0.manual_compression = new;
    }

    #[getter]
    pub fn get_recursion_limit(&self) -> i8 {
        self.0.recursion_limit
    }

    #[setter]
    pub fn set_recursion_limit(&mut self, new: i8) {
        self.0.recursion_limit = new;
    }

    #[getter]
    pub fn get_skip_unknown_types(&self) -> bool {
        self.0.skip_unknown_types
    }

    #[setter]
    pub fn set_skip_unknown_types(&mut self, new: bool) {
        self.0.skip_unknown_types = new;
    }
}

#[pyclass(module = "katsuba.op", subclass)]
pub struct Serializer {
    pub(crate) inner: serde::Serializer,
    buf: Vec<u8>,
}

impl Serializer {
    fn serialize_impl(&mut self, py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let value = conversion::python_to_object(obj, &self.inner.parts.types)?;
        py.detach(|| self.inner.serialize_into(&value, &mut self.buf))
            .map_err(error::op_to_py_err)
    }
}

#[pymethods]
impl Serializer {
    #[new]
    pub fn new(options: SerializerOptions, types: &TypeList) -> PyResult<Self> {
        serde::Serializer::new(options.0, Arc::clone(&types.0))
            .map(|inner| Self {
                inner,
                buf: Vec::new(),
            })
            .map_err(error::op_to_py_err)
    }

    #[getter]
    pub fn get_options(&self) -> SerializerOptions {
        SerializerOptions(self.inner.parts.options)
    }

    #[setter]
    pub fn set_options(&mut self, new: SerializerOptions) {
        self.inner.parts.options = new.0;
    }

    #[getter]
    pub fn get_flags(&self) -> u32 {
        self.inner.parts.options.flags.bits()
    }

    #[setter]
    pub fn set_flags(&mut self, new: u32) {
        self.inner.parts.options.flags = SerializerFlags::from_bits_truncate(new);
    }

    #[getter]
    pub fn get_property_mask(&self) -> u32 {
        self.inner.parts.options.property_mask.bits()
    }

    #[setter]
    pub fn set_property_mask(&mut self, new: u32) {
        self.inner.parts.options.property_mask = PropertyFlags::from_bits_truncate(new);
    }

    #[getter]
    pub fn get_shallow(&self) -> bool {
        self.inner.parts.options.shallow
    }

    #[setter]
    pub fn set_shallow(&mut self, new: bool) {
        self.inner.parts.options.shallow = new;
    }

    #[getter]
    pub fn get_manual_compression(&self) -> bool {
        self.inner.parts.options.manual_compression
    }

    #[setter]
    pub fn set_manual_compression(&mut self, new: bool) {
        self.inner.parts.options.manual_compression = new;
    }

    #[getter]
    pub fn get_recursion_limit(&self) -> i8 {
        self.inner.parts.options.recursion_limit
    }

    #[setter]
    pub fn set_recursion_limit(&mut self, new: i8) {
        self.inner.parts.options.recursion_limit = new;
    }

    #[getter]
    pub fn get_skip_unknown_types(&self) -> bool {
        self.inner.parts.options.skip_unknown_types
    }

    #[setter]
    pub fn set_skip_unknown_types(&mut self, new: bool) {
        self.inner.parts.options.skip_unknown_types = new;
    }

    pub fn deserialize<'py>(
        &mut self,
        py: Python<'py>,
        data: &[u8],
    ) -> PyResult<Bound<'py, PyAny>> {
        let value = py
            .detach(|| self.inner.deserialize(data))
            .map_err(error::op_to_py_err)?;
        Ok(conversion::value_to_python(&value, py))
    }

    pub fn serialize<'py>(
        &mut self,
        py: Python<'py>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        self.serialize_impl(py, obj)?;
        Ok(PyBytes::new(py, &self.buf))
    }

    pub fn serialize_into(
        &mut self,
        py: Python<'_>,
        obj: &Bound<'_, PyAny>,
        buf: &Bound<'_, PyByteArray>,
    ) -> PyResult<()> {
        self.serialize_impl(py, obj)?;

        buf.resize(self.buf.len())?;
        PyBuffer::<u8>::get(buf.as_any())?.copy_from_slice(py, &self.buf)?;

        Ok(())
    }
}

#[pyclass(extends=PyDict, module = "katsuba.op")]
pub struct Object {
    type_hash: u32,
}

impl Object {
    pub(super) fn new(type_hash: u32) -> Self {
        Self { type_hash }
    }
}

#[pymethods]
impl Object {
    #[new]
    #[pyo3(signature = (type_hash, *_args, **_kwargs))]
    fn __new__(
        type_hash: u32,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> Self {
        Self::new(type_hash)
    }

    #[pyo3(signature = (type_hash, *args, **kwargs))]
    fn __init__(
        slf: &Bound<'_, Self>,
        type_hash: u32,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let _ = type_hash;
        slf.as_any()
            .py_super()?
            .call_method(intern!(slf.py(), "__init__"), args, kwargs)?;

        Ok(())
    }

    #[getter]
    pub fn type_hash(&self) -> u32 {
        self.type_hash
    }
}

pub fn katsuba_op(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TypeList>()?;
    m.add_class::<SerializerOptions>()?;
    m.add_class::<Serializer>()?;

    m.add("STATEFUL_FLAGS", SerializerFlags::STATEFUL_FLAGS.bits())?;
    m.add(
        "COMPACT_LENGTH_PREFIXES",
        SerializerFlags::COMPACT_LENGTH_PREFIXES.bits(),
    )?;
    m.add(
        "HUMAN_READABLE_ENUMS",
        SerializerFlags::HUMAN_READABLE_ENUMS.bits(),
    )?;
    m.add("WITH_COMPRESSION", SerializerFlags::WITH_COMPRESSION.bits())?;
    m.add(
        "FORBID_DELTA_ENCODE",
        SerializerFlags::FORBID_DELTA_ENCODE.bits(),
    )?;

    m.add_class::<Object>()?;

    Ok(())
}
