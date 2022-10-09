//! Implementation of KingsIsle's ObjectProperty serialization
//! system through dynamic type info from the client.

use std::{
    collections::BTreeMap,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr,
};

mod drop;

mod reader;

mod serialization;
pub use serialization::*;

mod type_list;
pub use type_list::*;

mod type_tag;
pub use type_tag::*;

/// A list of values with a non-recursive drop impl.
#[derive(Clone, Debug)]
pub struct List {
    pub inner: Vec<Value>,
}

impl Drop for List {
    fn drop(&mut self) {
        self.inner.drain(..).for_each(drop::safely);
    }
}

/// A mapping of object members to values with a non-recursive
/// drop impl.
#[derive(Clone, Debug)]
pub struct Object {
    pub inner: BTreeMap<String, Value>,
}

impl Drop for Object {
    fn drop(&mut self) {
        for (_, child) in mem::take(&mut self.inner) {
            drop::safely(child);
        }
    }
}

/// A dynamically composed, deserialized value.
#[derive(Clone, Debug)]
pub enum Value {
    /// Any unsigned integer value.
    Unsigned(u64),
    /// Any signed integer value.
    Signed(i64),
    /// Any floating-point value.
    Float(f64),
    /// Any boolean value.
    Bool(bool),
    /// Any string value.
    String(Vec<u8>),
    /// Any wide string value.
    WString(Vec<u16>),
    /// A homogenous list of elements.
    List(List),
    /// A readable string of encoded bitflags.
    Bits(String),
    /// An RGBA color.
    Color { b: u8, g: u8, r: u8, a: u8 },
    /// A vector in 3D space.
    Vec3 { x: f32, y: f32, z: f32 },
    /// A quaternion.
    Quat { x: f32, y: f32, z: f32, w: f32 },
    /// An Euler angle.
    Euler { pitch: f32, roll: f32, yaw: f32 },
    /// A 3x3 matrix.
    Mat3x3 {
        i: [f32; 3],
        j: [f32; 3],
        k: [f32; 3],
    },
    /// A 2D point.
    Point {
        /// (x, y) tuple.
        xy: Box<(Value, Value)>,
    },
    /// A size description.
    Size {
        /// (width, height) tuple.
        wh: Box<(Value, Value)>,
    },
    /// A rectangle described by its edges.
    Rect {
        /// (left, top, right, bottom) tuple.
        inner: Box<(Value, Value, Value, Value)>,
    },
    /// An object that maps field names to values.
    Object(Object),
    /// An empty value with no further information.
    Empty,
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
