use std::{borrow::Cow, collections::btree_map, path::PathBuf};

use pyo3::{
    exceptions::{PyIOError, PyKeyError, PyOSError},
    prelude::*,
    types::PyType,
};

fn extract_file_contents<'a>(
    archive: &'a kobold_wad::Archive,
    file: &'a kobold_wad::types::File,
) -> PyResult<Cow<'a, [u8]>> {
    let contents = archive.file_contents(file);
    let contents = match file.compressed {
        true => {
            // We trade some efficiency for a nicer and error-resilient Python API
            // by doing a new memory allocation for every decompressed file.
            let mut inflater = kobold_wad::Inflater::new();
            inflater
                .decompress(contents, file.uncompressed_size as _)
                .map_err(|e| PyIOError::new_err(e.to_string()))?;

            Cow::Owned(inflater.into_inner())
        }

        false => Cow::Borrowed(contents),
    };

    Ok(contents)
}

#[pyclass]
struct Archive(kobold_wad::Archive);

#[pymethods]
impl Archive {
    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    pub fn __contains__(&self, file: &str) -> bool {
        self.0.files().contains_key(file)
    }

    pub fn __getitem__(&self, file: &str) -> PyResult<Cow<'_, [u8]>> {
        if let Some(file) = self.0.file_raw(file) {
            extract_file_contents(&self.0, file)
        } else {
            Err(PyKeyError::new_err(file.to_string()))
        }
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<ArchiveIter>> {
        let inner = slf.0.files().clone().into_keys();
        Py::new(slf.py(), ArchiveIter { inner })
    }

    #[classmethod]
    #[pyo3(signature = (path, verify_crcs=true, /))]
    pub fn heap(_cls: &PyType, path: PathBuf, verify_crcs: bool) -> PyResult<Self> {
        kobold_wad::Archive::heap(path, verify_crcs)
            .map(Self)
            .map_err(|e| PyOSError::new_err(e.to_string()))
    }

    #[classmethod]
    #[pyo3(signature = (path, verify_crcs=true, /))]
    pub fn mmap(_cls: &PyType, path: PathBuf, verify_crcs: bool) -> PyResult<Self> {
        kobold_wad::Archive::mmap(path, verify_crcs)
            .map(Self)
            .map_err(|e| PyOSError::new_err(e.to_string()))
    }
}

#[pyclass]
pub struct ArchiveIter {
    inner: btree_map::IntoKeys<String, kobold_wad::types::File>,
}

#[pymethods]
impl ArchiveIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.inner.next()
    }
}

pub fn kobold_wad(m: &PyModule) -> PyResult<()> {
    m.add_class::<Archive>()?;
    m.add_class::<ArchiveIter>()?;

    Ok(())
}
