use pyo3::prelude::*;

#[pyclass(module = "katsuba.op")]
pub struct Vec3 {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
    #[pyo3(get, set)]
    pub z: f32,
}

#[pyclass(module = "katsuba.op")]
pub struct Quaternion {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
    #[pyo3(get, set)]
    pub z: f32,
    #[pyo3(get, set)]
    pub w: f32,
}

#[pyclass(module = "katsuba.op")]
pub struct Matrix {
    #[pyo3(get, set)]
    pub i: [f32; 3],
    #[pyo3(get, set)]
    pub j: [f32; 3],
    #[pyo3(get, set)]
    pub k: [f32; 3],
}

#[pyclass(module = "katsuba.op")]
pub struct Euler {
    #[pyo3(get, set)]
    pub pitch: f32,
    #[pyo3(get, set)]
    pub yaw: f32,
    #[pyo3(get, set)]
    pub roll: f32,
}

#[pyclass(module = "katsuba.op")]
pub struct PointInt {
    #[pyo3(get, set)]
    pub x: i32,
    #[pyo3(get, set)]
    pub y: i32,
}

#[pyclass(module = "katsuba.op")]
pub struct PointFloat {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
}

#[pyclass(module = "katsuba.op")]
pub struct SizeInt {
    #[pyo3(get, set)]
    pub width: i32,
    #[pyo3(get, set)]
    pub height: i32,
}

#[pyclass(module = "katsuba.op")]
pub struct RectInt {
    #[pyo3(get, set)]
    pub left: i32,
    #[pyo3(get, set)]
    pub top: i32,
    #[pyo3(get, set)]
    pub right: i32,
    #[pyo3(get, set)]
    pub bottom: i32,
}

#[pyclass(module = "katsuba.op")]
pub struct RectFloat {
    #[pyo3(get, set)]
    pub left: f32,
    #[pyo3(get, set)]
    pub top: f32,
    #[pyo3(get, set)]
    pub right: f32,
    #[pyo3(get, set)]
    pub bottom: f32,
}

#[pyclass(module = "katsuba.op")]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    #[pyo3(get, set)]
    pub g: u8,
    #[pyo3(get, set)]
    pub b: u8,
    #[pyo3(get, set)]
    pub a: u8,
}
