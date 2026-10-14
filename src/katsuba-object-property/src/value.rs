//! Value representations for dynamic ObjectProperty serialization.
//!
//! Values have dynamic types and can be composed, at the cost of
//! incurring memory and performance overhead.

mod color;
pub use color::*;

#[cfg(feature = "serde")]
mod de;

mod drop;

mod math;
pub use math::*;

mod list;
pub use list::*;

mod object;
pub use object::*;

mod strings;
pub use strings::*;

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
    Object(Box<Object>),

    /// Representation of an RGBA color.
    Color(Color),
    Vec3(Vec3),
    Quat(Quaternion),
    Euler(Euler),
    Mat3x3(Box<Matrix>),

    /// A 2D point with integer coordinates.
    PointInt(Point<i32>),
    /// A 2D point with byte coordinates.
    PointUChar(Point<u8>),
    /// A 2D point with unsigned integer coordinates.
    PointUInt(Point<u32>),
    /// A 2D point with floating-point coordinates.
    PointFloat(Point<f32>),

    /// A size description with integer measures.
    SizeInt(Size<i32>),

    /// A rectangle described by integer edges.
    RectInt(Rect<i32>),
    /// A rectangle described by floating-point edges.
    RectFloat(Rect<f32>),
}

impl Value {
    /// Attempts to deserialize an object from the given deserializer.
    ///
    /// # A Note on Data Validation
    ///
    /// Note that no data validation is very sporadic and only ensures
    /// that the input data can be represented by a designated variant
    /// of [`Value`].
    ///
    /// What is NOT done are more nuanced range checks on the concrete
    /// values to ensure that they're valid in the context they were
    /// provided for (e.g. a `char` value being in 8-bit range). The
    /// responsibility for this falls on the user that consumes the
    /// [`Value`].
    #[cfg(feature = "serde")]
    pub fn deserialize_obj<'de, D>(
        deserializer: D,
        types: &katsuba_types::TypeList,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de::deserialize(deserializer, types)
    }
}
