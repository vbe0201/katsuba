use std::{
    mem,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use indexmap::IndexMap;

use super::{Value, drop};

/// The obligatory header every serialized CoreObject carries.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreHeader {
    /// The core type identifying the type of serialized object.
    pub core_type: u8,
    /// A namespace identifier that groups related types.
    pub namespace: u8,
    /// Template ID identifying the template the object is built from.
    pub template_id: u32,
}

/// Representation of an object in the ObjectProperty system.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    /// The identifying type hash of this object.
    #[cfg_attr(feature = "serde", serde(rename = "$__type"))]
    pub type_hash: u32,
    /// The CoreObject header of this object, if it has one.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "$__core", skip_serializing_if = "Option::is_none")
    )]
    pub core: Option<CoreHeader>,
    /// A mapping of class member names to their values.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub inner: IndexMap<Arc<str>, Value>,
}

impl Drop for Object {
    fn drop(&mut self) {
        for (_, child) in mem::take(&mut self.inner) {
            drop::safely(child);
        }
    }
}

impl Deref for Object {
    type Target = IndexMap<Arc<str>, Value>;

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
    type Item = (Arc<str>, Value);
    type IntoIter = <IndexMap<Arc<str>, Value> as IntoIterator>::IntoIter;

    fn into_iter(mut self) -> Self::IntoIter {
        mem::take(&mut self.inner).into_iter()
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a Arc<str>, &'a Value);
    type IntoIter = <&'a IndexMap<Arc<str>, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Object {
    type Item = (&'a Arc<str>, &'a mut Value);
    type IntoIter = <&'a mut IndexMap<Arc<str>, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
