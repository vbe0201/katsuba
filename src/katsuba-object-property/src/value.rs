//! Value representations for dynamic ObjectProperty serialization.
//!
//! Values have dynamic types and can be composed, at the cost of
//! incurring memory and performance overhead.

pub use smartstring::alias::String;

mod color;
pub use color::*;

mod drop;

mod math;
pub use math::*;

mod list;
pub use list::*;

mod object;
pub use object::*;

mod strings;
pub use strings::*;

// TODO: Evaluate optimizations.

/// A runtime value from the ObjectProperty system.
///
/// Its type is dynamically assigned at runtime, which mandates
/// appropriate checks for interpreting its contents.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
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
    String(CxxStr),
    /// A wide string of code points, not null-terminated.
    WString(CxxWStr),

    /// An enum variant or bitflags.
    Enum(i64),

    /// A homogenous list of elements.
    List(List),
    /// An object which maps field names to values.
    Object {
        #[cfg_attr(feature = "serde", serde(rename = "$__type"))]
        hash: u32,

        #[cfg_attr(feature = "serde", serde(flatten))]
        obj: Object,
    },

    /// Representation of an RGBA color.
    Color(Color),
    Vec3(Vec3),
    Quat(Quaternion),
    Euler(Euler),
    Mat3x3(Box<Matrix>),

    /// A 2D point with integer coordinates.
    PointInt(Point<i32>),
    /// A 2D point with floating-point coordinates.
    PointFloat(Point<f32>),

    /// A size description with integer measures.
    SizeInt(Size<i32>),

    /// A rectangle described by integer edges.
    RectInt(Rect<i32>),
    /// A rectangle described by floating-point edges.
    RectFloat(Rect<f32>),
}
