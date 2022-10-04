//! Library for parsing the Wizard101 Binary Collision Data
//! (BCD) format.
//!
//! As the name suggests, this format describes collision
//! types in game zones and is used for physics.

use std::mem;

use binrw::{
    binread,
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use bitflags::bitflags;

use super::utils;

mod sealed {
    use binrw::binread;

    use super::Face;

    #[binread]
    #[derive(Clone, Debug, PartialEq)]
    pub struct MeshShapeTheSadWay {
        #[br(temp)]
        vertex_count: u32,
        #[br(temp)]
        face_count: u32,
        #[br(count = face_count)]
        pub vertices: Vec<[f32; 3]>,
        #[br(count = face_count)]
        pub faces: Vec<Face>,
    }
}

bitflags! {
    /// Attribute flags encoded in [`Geometry`] objects.
    #[derive(BinRead)]
    #[br(map = CollisionFlags::from_bits_truncate)]
    pub struct CollisionFlags: u32 {
        const OBJECT = 1 << 0;
        const WALKABLE = 1 << 1;
        const HITSCAN = 1 << 3;
        const LOCAL_PLAYER = 1 << 4;
        const WATER = 1 << 6;
        const CLIENT_OBJECT = 1 << 7;
        const TRIGGER = 1 << 8;
        const FOG = 1 << 9;
        const GOO = 1 << 10;
        const FISH = 1 << 11;
    }
}

/// A face used to describe mesh [`ShapeData`].
#[derive(Clone, Debug, PartialEq, BinRead)]
pub struct Face {
    /// The face vector.
    pub face: [u32; 3],
    /// The normal vector.
    pub normal: [f32; 3],
}

/// Data describing a geometric [`Shape`].
#[derive(Clone, Debug, PartialEq, BinRead)]
#[br(import(mesh: Option<sealed::MeshShapeTheSadWay>))]
pub enum ShapeData {
    /// A box shape.
    #[br(pre_assert(mesh.is_none()), magic = 0_u32)]
    Box { length: f32, width: f32, depth: f32 },
    /// A ray shape.
    #[br(pre_assert(mesh.is_none()), magic = 1_u32)]
    Ray {
        position: f32,
        direction: f32,
        length: f32,
    },
    /// A sphere shape.
    #[br(pre_assert(mesh.is_none()), magic = 2_u32)]
    Sphere { radius: f32 },
    /// A cylinder shape.
    #[br(pre_assert(mesh.is_none()), magic = 3_u32)]
    Cylinder { radius: f32, length: f32 },
    /// A tube shape.
    #[br(pre_assert(mesh.is_none()), magic = 4_u32)]
    Tube { radius: f32, length: f32 },
    /// A plane shape.
    #[br(pre_assert(mesh.is_none()), magic = 5_u32)]
    Plane { normal: [f32; 3], distance: f32 },
    /// A mesh shape.
    #[br(pre_assert(mesh.is_some()), magic = 6_u32)]
    // SAFETY: `mesh` is Some so we can safely unwrap without checking.
    Mesh {
        #[br(calc = unsafe { mem::take(mesh.as_mut().map(|v| &mut v.vertices).unwrap_unchecked()) })]
        vertices: Vec<[f32; 3]>,
        #[br(calc = unsafe { mem::take(mesh.as_mut().map(|v| &mut v.faces).unwrap_unchecked()) })]
        faces: Vec<Face>,
    },
}

impl From<sealed::MeshShapeTheSadWay> for ShapeData {
    fn from(value: sealed::MeshShapeTheSadWay) -> Self {
        ShapeData::Mesh {
            vertices: value.vertices,
            faces: value.faces,
        }
    }
}

/// The shape described by a [`Geometry`].
#[binread]
#[derive(Clone, Debug, PartialEq)]
#[br(import(mesh: Option<sealed::MeshShapeTheSadWay>))]
pub struct Shape {
    #[br(temp)]
    name_len: u32,
    /// The name of the shape.
    #[br(args(name_len as usize), parse_with = utils::parse_string)]
    pub name: String,
    /// The rotation matrix of the shape.
    pub rotation: [f32; 9],
    /// The location vector of the shape.
    pub location: [f32; 3],
    /// The scale value of the shape.
    pub scale: f32,
    #[br(temp)]
    material_len: u32,
    /// The material name for the shape.
    #[br(args(material_len as usize), parse_with = utils::parse_string)]
    pub material: String,
    /// The shape data.
    #[br(args(mesh))]
    pub data: ShapeData,
}

/// Geometric data describing a collision.
#[binread]
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    #[br(map = |x: u32| x == 6, temp)]
    is_mesh: bool,

    /// The category flags for the shape.
    pub category_flags: CollisionFlags,
    /// The collision flags for the shape.
    pub collision_flags: CollisionFlags,

    #[br(if(is_mesh), temp)]
    mesh_helper: Option<sealed::MeshShapeTheSadWay>,

    /// The actual shape layout.
    #[br(args(mesh_helper.take()))]
    pub shape: Shape,
}

/// A full BCD object with all its shapes.
#[binread]
#[derive(Clone, Debug, PartialEq)]
pub struct Bcd {
    #[br(temp)]
    geometry_count: u32,
    /// The geometric collisions described by the format.
    #[br(count = geometry_count)]
    pub geometry: Vec<Geometry>,
}

impl Bcd {
    /// Attempts to parse a BCD file from a given input source.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }
}
