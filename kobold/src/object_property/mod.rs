//! Implementation of KingsIsle's ObjectProperty serialization
//! system through dynamic type info from the client.

use std::{
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr,
};

mod drop;

mod reader;
pub use reader::*;

mod serialization;
pub use serialization::*;

mod type_list;
pub use type_list::*;

mod type_tag;
pub use type_tag::*;

pub(super) type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;

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
    pub name: String,
    pub inner: HashMap<String, Value>,
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
    /// An enum variant or bitflags.
    Enum(String),
    /// A homogenous list of elements.
    List(List),
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

#[cfg(feature = "python")]
impl pyo3::IntoPy<pyo3::PyObject> for Value {
    fn into_py(self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        let mut this = ManuallyDrop::new(self);
        unsafe {
            match ManuallyDrop::take(&mut this) {
                Value::Unsigned(i) => i.into_py(py),
                Value::Signed(i) => i.into_py(py),
                Value::Float(f) => f.into_py(py),
                Value::Bool(b) => b.into_py(py),
                Value::String(str) => str.as_slice().into_py(py),
                Value::WString(wstr) => wstr.into_py(py),
                Value::Enum(str) => str.into_py(py),
                Value::List(list) => {
                    let list = ManuallyDrop::new(list);
                    ptr::read(&list.inner).into_py(py)
                }
                Value::Color { r, g, b, a } => (r, g, b, a).into_py(py),
                Value::Vec3 { x, y, z } => (x, y, z).into_py(py),
                Value::Quat { x, y, z, w } => (x, y, z, w).into_py(py),
                Value::Euler { pitch, roll, yaw } => (pitch, roll, yaw).into_py(py),
                Value::Mat3x3 { i, j, k } => [i, j, k].into_py(py),
                Value::Point { xy } => xy.into_py(py),
                Value::Size { wh } => wh.into_py(py),
                Value::Rect { inner } => inner.into_py(py),
                Value::Object(object) => {
                    let object = ManuallyDrop::new(object);
                    ptr::read(&object.inner).into_py(py)
                }
                Value::Empty => py.None(),
            }
        }
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

impl Deref for Object {
    type Target = HashMap<String, Value>;

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
    type IntoIter = <HashMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let this = ManuallyDrop::new(self);
        unsafe { ptr::read(&this.inner).into_iter() }
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a String, &'a Value);
    type IntoIter = <&'a HashMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Object {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = <&'a mut HashMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
