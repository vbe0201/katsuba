use kobold_types::{Property, TypeDef};

use super::Deserializer;
use crate::{value::Object, Value};

/// Defines common handlers for diagnostic events during
/// the deserialization routine.
///
/// Implementations can customize how they want to receive
/// and process debug information
pub trait Diagnostics: Sized {
    /// Called when a new object is being deserialized.
    ///
    /// When `info` is [`None`], the object was an empty
    /// null pointer value.
    fn object(&mut self, info: Option<&TypeDef>);

    /// Called when an object is done being deserialized
    /// and its [`Object`] value can be reported.
    ///
    /// `remaining` is the number of bytes still left in
    /// the reader afterwards.
    fn object_finished(&mut self, value: &Object, remaining: usize);

    /// Called when an object with no known type information
    /// was encountered.
    ///
    /// This is only invoked when skipping objects is allowed
    /// in the deserializer. Implementation may perform further
    /// examination of the raw byte slice.
    fn unknown_object(&mut self, de: &mut Deserializer<Self>, raw: &[u8]);

    /// Called when a property in an object is being deserialized.
    fn property(&mut self, info: &Property);

    /// Called when a property is done being deserialized and its
    /// [`Value`] can be reported.
    fn property_finished(&mut self, value: &Value);
}

/// Quiet receiver which does not produce any output.
pub struct Quiet;

impl Diagnostics for Quiet {
    fn object(&mut self, _info: Option<&TypeDef>) {}

    fn object_finished(&mut self, _value: &Object, _remaining: usize) {}

    fn unknown_object(&mut self, _de: &mut Deserializer<Self>, _raw: &[u8]) {}

    fn property(&mut self, _info: &Property) {}

    fn property_finished(&mut self, _value: &Value) {}
}
