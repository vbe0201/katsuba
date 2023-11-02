//! Utilities for iterating over a subset of archive files chosen
//! by a UNIX glob pattern.

pub use globset::Error as GlobError;

use std::collections::btree_map::Iter;

use globset::{Glob, GlobMatcher};

use crate::{types::File, Archive};

/// A glob matcher for checking archive file strings.
pub struct Matcher {
    inner: GlobMatcher,
}

impl Matcher {
    /// Creates a new glob matcher over the given pattern.
    pub fn new(pattern: &str) -> Result<Self, GlobError> {
        let inner = Glob::new(pattern)?.compile_matcher();
        Ok(Self { inner })
    }

    /// Checks if a given path is a match to the glob pattern.
    #[inline]
    pub fn is_match(&self, path: &str) -> bool {
        self.inner.is_match(path)
    }
}

/// An iterator that only yields [`Archive`] elements which match
/// a specified UNIX glob pattern.
pub struct GlobIter<'a> {
    archive: Iter<'a, String, File>,
    matcher: Matcher,
}

impl<'a> GlobIter<'a> {
    /// Creates a new glob iterator that yields [`Archive`] files
    /// matching the given pattern.
    ///
    /// Errors on failure to compile the provided glob pattern.
    pub fn new(archive: &'a Archive, pattern: &str) -> Result<Self, GlobError> {
        Matcher::new(pattern).map(move |matcher| Self {
            archive: archive.files().iter(),
            matcher,
        })
    }
}

impl<'a> Iterator for GlobIter<'a> {
    type Item = (&'a String, &'a File);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.archive.next() {
                Some((path, file)) if self.matcher.is_match(path) => break Some((path, file)),
                Some(..) => continue,
                None => break None,
            }
        }
    }
}
