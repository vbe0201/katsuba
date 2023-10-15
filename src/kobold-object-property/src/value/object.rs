use std::{
    collections::BTreeMap,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr,
};

use smartstring::alias::String;

use super::{drop, Value};

/// Representation of an object in the ObjectProperty system.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    /// A mapping of class member names to their values.
    pub inner: BTreeMap<String, Value>,
}

impl Drop for Object {
    fn drop(&mut self) {
        for (_, child) in mem::take(&mut self.inner) {
            drop::safely(child);
        }
    }
}

impl Deref for Object {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for Object {
    type Item = (String, Value);
    type IntoIter = <BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let this = ManuallyDrop::new(self);
        unsafe { ptr::read(&this.inner).into_iter() }
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a String, &'a Value);
    type IntoIter = <&'a BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Object {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = <&'a mut BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
