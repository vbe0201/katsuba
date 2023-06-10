//! Value representations for dynamic ObjectProperty serialization.
//!
//! Values have dynamic types and can be composed, at the cost of
//! incurring memory and performance overhead.

mod color;
pub use color::*;

mod drop;

mod math;
pub use math::*;

mod list;
pub use list::*;

mod object;
pub use object::*;

// TODO: Evaluate optimizations.

/// A runtime value from the ObjectProperty system.
///
/// Its type is dynamically assigned at runtime, which mandates
/// appropriate checks for interpreting its contents.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// An empty unit value.
    Empty,

    /// A unsigned integer value.
    Unsigned(u64),
    /// A signed integer value.
    Signed(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),

    /// A string of bytes, not null-terminated.
    String(Vec<u8>),
    /// A wide string of code points, not null-terminated.
    WString(Vec<u16>),

    /// An enum variant or bitflags.
    Enum(u32),

    /// A homogenous list of elements.
    List(List),
    /// An object which maps field names to values.
    Object(Object),

    /// Representation of an RGBA color.
    Color(Color),
    Vec3(Vec3),
    Quat(Quaternion),
    Euler(Euler),
    Mat3x3(Matrix),

    /// A 2D point.
    Point {
        xy: Box<(Value, Value)>,
    },
    /// A size description.
    Size {
        wh: Box<(Value, Value)>,
    },
    /// A rectangle described by its edges.
    Rect {
        inner: Box<(Value, Value, Value, Value)>,
    },
}
