use std::{borrow::Cow, collections::btree_map, path::PathBuf};

use katsuba_object_property::serde;
use pyo3::{exceptions::PyKeyError, prelude::*, types::PyType};

use crate::{error, op, KatsubaError};

fn extract_file_contents<'a>(
    archive: &'a katsuba_wad::Archive,
    file: &katsuba_wad::types::File,
) -> PyResult<Cow<'a, [u8]>> {
    let contents = archive
        .file_contents(file)
        .ok_or_else(|| KatsubaError::new_err("file contents missing from archive"))?;

    let contents = match file.compressed {
        true => {
            // We trade some efficiency for a nicer and error-resilient Python API
            // by doing a new memory allocation for every decompressed file.
            let mut inflater = katsuba_wad::Inflater::new();
            inflater
                .decompress(contents, file.uncompressed_size as _)
                .map_err(|e| KatsubaError::new_err(format!("{e:?}")))?;

            Cow::Owned(inflater.into_inner())
        }

        false => Cow::Borrowed(contents),
    };

    Ok(contents)
}

#[pyclass(module = "katsuba.wad")]
struct Archive(katsuba_wad::Archive);

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

    pub fn iter_glob(slf: PyRef<'_, Self>, pattern: &str) -> PyResult<Py<GlobArchiveIter>> {
        let matcher = katsuba_wad::glob::Matcher::new(pattern)
            .map_err(|e| KatsubaError::new_err(format!("{e:?}")))?;
        let iter = slf.0.files().clone().into_keys();

        Py::new(
            slf.py(),
            GlobArchiveIter {
                matcher,
                iter: ArchiveIter { iter },
            },
        )
    }

    #[classmethod]
    pub fn heap(_cls: &Bound<'_, PyType>, path: PathBuf) -> PyResult<Self> {
        katsuba_wad::Archive::open_heap(path)
            .map(Self)
            .map_err(error::wad_to_py_err)
    }

    #[classmethod]
    pub fn mmap(_cls: &Bound<'_, PyType>, path: PathBuf) -> PyResult<Self> {
        katsuba_wad::Archive::open_mmap(path)
            .map(Self)
            .map_err(error::wad_to_py_err)
    }

    pub fn deserialize(
        &self,
        file: &str,
        serializer: &mut op::Serializer,
    ) -> PyResult<op::LazyObject> {
        let raw = self.__getitem__(file)?;
        let mut raw: &[u8] = &raw;

        // Set generic configuration for game files if this is one.
        if raw.get(0..4) == Some(serde::BIND_MAGIC) {
            serializer.0.parts.options.flags |= serde::SerializerFlags::STATEFUL_FLAGS;
            serializer.0.parts.options.shallow = false;

            raw = raw.get(4..).unwrap();
        }

        serializer.deserialize(raw)
    }
}

#[pyclass(module = "katsuba.wad")]
pub struct ArchiveIter {
    iter: btree_map::IntoKeys<String, katsuba_wad::types::File>,
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

#[pyclass(module = "katsuba.wad")]
pub struct GlobArchiveIter {
    matcher: katsuba_wad::glob::Matcher,
    iter: ArchiveIter,
}

#[pymethods]
impl GlobArchiveIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        loop {
            match slf.iter.iter.next() {
                Some(path) if slf.matcher.is_match(&path) => break Some(path),
                Some(..) => continue,
                None => break None,
            }
        }
    }
}

pub fn katsuba_wad(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Archive>()?;
    m.add_class::<ArchiveIter>()?;
    m.add_class::<GlobArchiveIter>()?;

    Ok(())
}
