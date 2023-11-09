use std::{fs, io, path::PathBuf, sync::Arc};

use katsuba_object_property::{
    serde::{self, SerializerFlags},
    Value,
};
use pyo3::{prelude::*, types::PyType};

use crate::{error, KatsubaError};

mod conversion;

mod lazy;
pub use lazy::*;

mod leaf_types;
pub use leaf_types::*;

#[derive(Clone)]
#[pyclass(module = "katsuba.op")]
pub struct TypeList(Arc<katsuba_types::TypeList>);

#[pymethods]
impl TypeList {
    #[new]
    pub fn new(data: &str) -> PyResult<Self> {
        katsuba_types::TypeList::from_str(data)
            .map(|v| Self(Arc::new(v)))
            .map_err(|e| KatsubaError::new_err(e.to_string()))
    }

    #[classmethod]
    pub fn open(_cls: &PyType, path: PathBuf) -> PyResult<Self> {
        let file = fs::File::open(path)?;
        katsuba_types::TypeList::from_reader(io::BufReader::new(file))
            .map(|v| Self(Arc::new(v)))
            .map_err(|e| KatsubaError::new_err(e.to_string()))
    }
}

#[derive(Clone, Copy, Default)]
#[pyclass(module = "katsuba.op")]
pub struct SerializerOptions(serde::SerializerOptions);

#[pymethods]
impl SerializerOptions {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[getter]
    pub fn get_flags(&self) -> u32 {
        self.0.flags.bits()
    }

    #[setter]
    pub fn set_flags(&mut self, new: u32) {
        self.0.flags = serde::SerializerFlags::from_bits_truncate(new);
    }

    #[getter]
    pub fn get_property_mask(&self) -> u32 {
        self.0.property_mask.bits()
    }

    #[setter]
    pub fn set_property_mask(&mut self, new: u32) {
        self.0.property_mask = katsuba_types::PropertyFlags::from_bits_truncate(new);
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

#[pyclass(module = "katsuba.op")]
pub struct Serializer(pub(crate) serde::Serializer);

#[pymethods]
impl Serializer {
    #[new]
    pub fn new(options: SerializerOptions, types: &TypeList) -> PyResult<Self> {
        serde::Serializer::new(options.0, Arc::clone(&types.0))
            .map(Self)
            .map_err(error::op_to_py_err)
    }

    pub fn deserialize(&mut self, data: &[u8]) -> PyResult<LazyObject> {
        self.0
            .deserialize::<serde::PropertyClass>(data)
            .map(|v| {
                let value = Arc::new(v);
                let (hash, obj) = match &*value {
                    Value::Object { hash, obj } => (*hash, obj),
                    _ => unreachable!(),
                };

                unsafe { LazyObject::new(value.clone(), hash, obj) }
            })
            .map_err(error::op_to_py_err)
    }
}

pub fn katsuba_op(m: &PyModule) -> PyResult<()> {
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

    m.add_class::<LazyList>()?;
    m.add_class::<LazyObject>()?;

    m.add_class::<Vec3>()?;
    m.add_class::<Quaternion>()?;
    m.add_class::<Matrix>()?;
    m.add_class::<Euler>()?;
    m.add_class::<PointInt>()?;
    m.add_class::<PointFloat>()?;
    m.add_class::<SizeInt>()?;
    m.add_class::<RectInt>()?;
    m.add_class::<RectFloat>()?;
    m.add_class::<Color>()?;

    Ok(())
}
