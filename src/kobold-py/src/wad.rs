use std::{borrow::Cow, collections::btree_map, path::PathBuf};

use pyo3::{exceptions::PyKeyError, prelude::*, types::PyType};

use crate::{error, op, KoboldError};

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
                .map_err(|e| KoboldError::new_err(format!("{e:?}")))?;

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
        let iter = slf.0.files().clone().into_keys();
        Py::new(slf.py(), ArchiveIter { iter })
    }

    #[classmethod]
    #[pyo3(signature = (path, verify_crcs=true, /))]
    pub fn heap(_cls: &PyType, path: PathBuf, verify_crcs: bool) -> PyResult<Self> {
        kobold_wad::Archive::open_heap(path, verify_crcs)
            .map(Self)
            .map_err(error::wad_to_py_err)
    }

    #[classmethod]
    #[pyo3(signature = (path, verify_crcs=true, /))]
    pub fn mmap(_cls: &PyType, path: PathBuf, verify_crcs: bool) -> PyResult<Self> {
        kobold_wad::Archive::open_mmap(path, verify_crcs)
            .map(Self)
            .map_err(error::wad_to_py_err)
    }

    pub fn deserialize(
        &self,
        file: &str,
        serializer: &mut op::Serializer,
    ) -> PyResult<op::LazyObject> {
        let raw = self.__getitem__(file)?;
        serializer.deserialize(&raw)
    }
}

#[pyclass]
pub struct ArchiveIter {
    iter: btree_map::IntoKeys<String, kobold_wad::types::File>,
}

#[pymethods]
impl ArchiveIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.iter.next()
    }
}

pub fn kobold_wad(m: &PyModule) -> PyResult<()> {
    m.add_class::<Archive>()?;
    m.add_class::<ArchiveIter>()?;

    Ok(())
}
