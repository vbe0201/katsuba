/// A three-dimensional vector.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    /// The X coordinate.
    pub x: f32,
    /// The Y coordinate.
    pub y: f32,
    /// The Z coordinate.
    pub z: f32,
}

/// A quaternion representing an orientation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    /// The X coordinate.
    pub x: f32,
    /// The Y coordinate.
    pub y: f32,
    /// The Z coordinate.
    pub z: f32,
    /// The angle.
    pub w: f32,
}

/// A 3x3 matrix.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix {
    /// The first row of the matrix.
    pub i: [f32; 3],
    /// The second row of the matrix.
    pub j: [f32; 3],
    /// The third row of the matrix.
    pub k: [f32; 3],
}

/// A set of Euler angles representing a rotation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Euler {
    /// The angle to apply around the X axis.
    pub pitch: f32,
    /// The angle to apply around the Y axis.
    pub yaw: f32,
    /// The angle to apply around the Z axis.
    pub roll: f32,
}

/// A point in two-dimensional space represented by its
/// coordinates.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point<T> {
    /// The X coordinate.
    pub x: T,
    /// The Y coordinate.
    pub y: T,
}

/// A two-dimensional size defined by its width and height.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size<T> {
    /// The width of the shape.
    pub width: T,
    /// The height of the shape.
    pub height: T,
}

/// A rectangular shape in two-dimensional space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect<T> {
    /// The location of the left edge.
    pub left: T,
    /// The location of the top edge.
    pub top: T,
    /// The location of the right edge.
    pub right: T,
    /// The location of the bottom edge.
    pub bottom: T,
}
