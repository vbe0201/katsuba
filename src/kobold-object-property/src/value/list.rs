use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr,
};

use super::{drop, Value};

/// A list of values with a non-recursive drop impl.
///
/// A list can store arbitrary values in the ObjectProperty
/// system, not necessarily being homogenous.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    /// The inner [`Value`]s of the list.
    pub inner: Vec<Value>,
}

impl Drop for List {
    fn drop(&mut self) {
        self.inner.drain(..).for_each(drop::safely);
    }
}

impl Deref for List {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for List {
    type Item = Value;
    type IntoIter = <Vec<Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let this = ManuallyDrop::new(self);
        unsafe { ptr::read(&this.inner).into_iter() }
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a Value;
    type IntoIter = <&'a Vec<Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut List {
    type Item = &'a mut Value;
    type IntoIter = <&'a mut Vec<Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
