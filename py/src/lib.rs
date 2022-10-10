use kobold::object_property as kobold;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyNotImplementedError},
    prelude::*,
};

create_exception!(kobold_py, KoboldError, PyException);

#[derive(Clone)]
#[pyclass(module = "kobold_py")]
pub struct TypeList {
    inner: kobold::TypeList,
}

#[pymethods]
impl TypeList {
    #[new]
    pub fn new(data: &str) -> PyResult<Self> {
        kobold::TypeList::from_str(data)
            .map(|inner| Self { inner })
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }
}

#[pyclass(module = "kobold_py", subclass)]
pub struct Deserializer;

#[pymethods]
impl Deserializer {
    #[new]
    pub fn new(_options: kobold::DeserializerOptions, _types: PyRef<TypeList>) -> Self {
        Self
    }

    pub fn deserialize(&mut self, _data: &[u8]) -> PyResult<kobold::Value> {
        Err(PyNotImplementedError::new_err(
            "use a Deserializer subclass",
        ))
    }
}

#[pyclass(module = "kobold_py", extends = Deserializer, subclass)]
pub struct BinaryDeserializer {
    inner: kobold::Deserializer<kobold::PropertyClass>,
}

#[pymethods]
impl BinaryDeserializer {
    #[new]
    pub fn new(
        options: kobold::DeserializerOptions,
        types: PyRef<TypeList>,
    ) -> (Self, Deserializer) {
        (
            Self {
                inner: kobold::Deserializer::new(options, types.inner.clone()),
            },
            Deserializer,
        )
    }

    pub fn deserialize(&mut self, data: &[u8]) -> PyResult<kobold::Value> {
        self.inner
            .deserialize(data)
            .map_err(|e| KoboldError::new_err(e.to_string()))
    }
}

#[pymodule]
fn kobold_py(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("KoboldError", py.get_type::<KoboldError>())?;
    m.add_class::<kobold::DeserializerOptions>()?;
    m.add_class::<TypeList>()?;
    m.add_class::<Deserializer>()?;
    m.add_class::<BinaryDeserializer>()?;

    Ok(())
}
