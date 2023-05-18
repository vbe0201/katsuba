use std::{fs, path::PathBuf, sync::Arc};

use kobold::object_property as kobold;
use pyo3::{
    exceptions::{PyNotImplementedError, PyOSError},
    prelude::*,
};

use crate::KoboldError;

#[derive(Clone)]
#[pyclass]
struct TypeList {
    inner: Arc<kobold::TypeList>,
}

#[pymethods]
impl TypeList {
    #[new]
    pub fn new(data: &str) -> PyResult<Self> {
        kobold::TypeList::from_str(data)
            .map(|inner| Self {
                inner: Arc::new(inner),
            })
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }
}

#[pyclass(subclass)]
struct Deserializer;

#[pymethods]
impl Deserializer {
    #[new]
    pub fn new(_options: kobold::DeserializerOptions, _types: &TypeList) -> Self {
        Self
    }

    pub fn deserialize(&mut self, _data: &[u8]) -> PyResult<kobold::Value> {
        Err(PyNotImplementedError::new_err(
            "use a Deserializer subclass",
        ))
    }

    pub fn deserialize_from_path(&mut self, _path: PathBuf) -> PyResult<kobold::Value> {
        Err(PyNotImplementedError::new_err(
            "use a Deserializer subclass",
        ))
    }
}

#[pyclass(extends = Deserializer, subclass)]
struct BinaryDeserializer {
    inner: kobold::Deserializer<kobold::PropertyClass>,
}

#[pyclass(extends = Deserializer, subclass)]
struct CoreObjectDeserializer {
    inner: kobold::Deserializer<kobold::CoreObject>,
}

#[pymethods]
impl BinaryDeserializer {
    #[new]
    pub fn new(
        options: kobold::DeserializerOptions,
        types: &TypeList,
    ) -> PyResult<(Self, Deserializer)> {
        Ok((
            Self {
                inner: kobold::Deserializer::new(options, Arc::clone(&types.inner))
                    .map_err(|e| KoboldError::new_err(e.to_string()))?,
            },
            Deserializer,
        ))
    }

    pub fn deserialize(&mut self, data: &[u8]) -> PyResult<kobold::Value> {
        self.inner
            .deserialize(data)
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }

    pub fn deserialize_from_path(&mut self, path: PathBuf) -> PyResult<kobold::Value> {
        let data = fs::read(path).map_err(PyOSError::new_err)?;

        let mut data = &data[..];
        if let (b"BINd", actual) = data.split_at(4) {
            data = actual;
        }

        self.inner
            .deserialize(data)
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }
}

#[pymethods]
impl CoreObjectDeserializer {
    #[new]
    pub fn new(
        options: kobold::DeserializerOptions,
        types: &TypeList,
    ) -> PyResult<(Self, Deserializer)> {
        Ok((
            Self {
                inner: kobold::Deserializer::new(options, Arc::clone(&types.inner))
                    .map_err(|e| KoboldError::new_err(e.to_string()))?,
            },
            Deserializer,
        ))
    }

    pub fn deserialize(&mut self, data: &[u8]) -> PyResult<kobold::Value> {
        self.inner
            .deserialize(data)
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }

    pub fn deserialize_from_path(&mut self, path: PathBuf) -> PyResult<kobold::Value> {
        let data = fs::read(path).map_err(PyOSError::new_err)?;

        let mut data = &data[..];
        if let (b"BINd", actual) = data.split_at(4) {
            data = actual;
        }

        self.inner
            .deserialize(data)
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }
}

pub fn kobold_op(m: &PyModule) -> PyResult<()> {
    m.add_class::<kobold::DeserializerOptions>()?;
    m.add_class::<TypeList>()?;
    m.add_class::<Deserializer>()?;
    m.add_class::<BinaryDeserializer>()?;
    m.add_class::<CoreObjectDeserializer>()?;

    // Define serializer flag constants.
    m.add("STATEFUL_FLAGS", 1_u32 << 0)?;
    m.add("COMPACT_LENGTH_PREFIXES", 1_u32 << 1)?;
    m.add("HUMAN_READABLE_ENUMS", 1_u32 << 2)?;
    m.add("COMPRESS", 1_u32 << 3)?;
    m.add("FORBID_DELTA_ENCODE", 1_u32 << 4)?;

    Ok(())
}
