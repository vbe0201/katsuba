//! Crate for parsing and writing Binary Collision Data (BCD) files.
//!
//! As the name suggests, this format describes geometric collision
//! shapes for zones and is used for physics.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use bitflags::bitflags;
use kobold_utils::{
    binrw::{
        self, binrw,
        io::{Read, Seek, Write},
        BinReaderExt, BinResult, BinWriterExt,
    },
    binrw_ext::{read_prefixed_string, write_prefixed_string},
};
use serde::{Deserialize, Serialize};

bitflags! {
    /// Attribute flags encoded in [`Geometry`] objects.
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = Self::bits)]
    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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
        const MUCK = 1 << 12;
    }
}

/// A face used to describe mesh [`ShapeData`].
#[binrw]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Face {
    /// The face vector.
    pub face: [u32; 3],
    /// The normal vector.
    pub normal: [f32; 3],
}

/// Extra parameters for the encoded geometric shape.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GeomParams {
    /// Box-shaped geometry.
    #[brw(magic = 0_u32)]
    Box { length: f32, width: f32, depth: f32 },

    /// Ray-shaped geometry.
    #[brw(magic = 1_u32)]
    Ray {
        position: f32,
        direction: f32,
        length: f32,
    },

    /// Sphere-shaped geometry.
    #[brw(magic = 2_u32)]
    Sphere { radius: f32 },

    /// Cylinder-shaped geometry.
    #[brw(magic = 3_u32)]
    Cylinder { radius: f32, length: f32 },

    /// Tube-shaped geometry.
    #[brw(magic = 4_u32)]
    Tube { radius: f32, length: f32 },

    /// Plane-shaped geometry.
    #[brw(magic = 5_u32)]
    Plane { normal: [f32; 3], distance: f32 },

    /// Mesh geometry.
    #[brw(magic = 6_u32)]
    Mesh,
}

/// Representation of any geometric shape.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyGeometry {
    #[br(temp)]
    #[bw(calc = name.len() as u32)]
    name_len: u32,

    /// The name of the shape.
    #[br(args(name_len as usize, false), parse_with = read_prefixed_string)]
    #[bw(args(false), write_with = write_prefixed_string)]
    pub name: String,

    /// The rotation matrix of the shape.
    pub rotation: [[f32; 3]; 3],

    /// The location vector of the shape.
    pub location: [f32; 3],

    /// The scaling factor of the shape.
    pub scale: f32,

    #[br(temp)]
    #[bw(calc = material.len() as u32)]
    material_len: u32,

    /// The material name for the shape.
    #[br(args(material_len as usize, false), parse_with = read_prefixed_string)]
    #[bw(args(false), write_with = write_prefixed_string)]
    pub material: String,

    /// Geometric shape parameters.
    pub params: GeomParams,
}

impl ProxyGeometry {
    #[inline]
    fn params_type(&self) -> u32 {
        match self.params {
            GeomParams::Box { .. } => 0,
            GeomParams::Ray { .. } => 1,
            GeomParams::Sphere { .. } => 2,
            GeomParams::Cylinder { .. } => 3,
            GeomParams::Tube { .. } => 4,
            GeomParams::Plane { .. } => 5,
            GeomParams::Mesh => 6,
        }
    }
}

/// Representation of an arbitrary mesh shape.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyMesh {
    #[br(temp)]
    #[bw(calc = self.vertices.len() as u32)]
    vertex_count: u32,

    #[br(temp)]
    #[bw(calc = self.faces.len() as u32)]
    face_count: u32,

    /// A dynamic list of vertices in the mesh.
    #[br(count = vertex_count)]
    pub vertices: Vec<[f32; 3]>,

    /// A dynamic list of faces in the mesh.
    #[br(count = face_count)]
    pub faces: Vec<Face>,
}

/// Representation of an individual collision entry.
///
/// Describes a geometric shape and the associated metadata.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collision {
    #[br(temp)]
    #[bw(calc = self.geometry.params_type())]
    geometry_type: u32,

    /// The category flags for the shape.
    pub category_flags: CollisionFlags,
    /// The collision flags for the shape.
    pub collision_flags: CollisionFlags,

    /// Additional data for mesh-based collisions, if any.
    #[brw(if(geometry_type == 6))]
    pub mesh: Option<ProxyMesh>,

    /// Universal geometric data for the collision shape.
    pub geometry: ProxyGeometry,
}

/// Representation of a BCD file.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bcd {
    #[br(temp)]
    #[bw(calc = self.collisions.len() as u32)]
    collision_count: u32,

    /// A list of all [`Collision`] objects in the file.
    #[br(count = collision_count)]
    pub collisions: Vec<Collision>,
}

impl Bcd {
    /// Attempts to parse a BCD file from a given [`Read`]er.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes the BCD data to the given [`Write`]r.
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()> {
        writer.write_le(self).map_err(Into::into)
    }
}
