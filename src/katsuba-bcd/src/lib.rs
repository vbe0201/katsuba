//! Crate for parsing and writing Binary Collision Data (BCD) files.
//!
//! As the name suggests, this format describes geometric collision
//! shapes for zones and is used for physics.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::io;

use bitflags::bitflags;
use katsuba_utils::binary;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Attribute flags encoded in [`Geometry`] objects.
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Face {
    /// The face vector.
    pub face: [u32; 3],
    /// The normal vector.
    pub normal: [f32; 3],
}

impl Face {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            face: [
                binary::uint32(reader)?,
                binary::uint32(reader)?,
                binary::uint32(reader)?,
            ],
            normal: [
                binary::float32(reader)?,
                binary::float32(reader)?,
                binary::float32(reader)?,
            ],
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for v in self.face {
            binary::write_uint32(writer, v)?;
        }
        for v in self.normal {
            binary::write_float32(writer, v)?;
        }
        Ok(())
    }
}

/// Extra parameters for the encoded geometric shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GeomParams {
    /// Box-shaped geometry.
    Box { length: f32, width: f32, depth: f32 },

    /// Ray-shaped geometry.
    Ray {
        position: f32,
        direction: f32,
        length: f32,
    },

    /// Sphere-shaped geometry.
    Sphere { radius: f32 },

    /// Cylinder-shaped geometry.
    Cylinder { radius: f32, length: f32 },

    /// Tube-shaped geometry.
    Tube { radius: f32, length: f32 },

    /// Plane-shaped geometry.
    Plane { normal: [f32; 3], distance: f32 },

    /// Mesh geometry.
    Mesh,
}

impl GeomParams {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        Ok(match binary::uint32(reader)? {
            0 => Self::Box {
                length: binary::float32(reader)?,
                width: binary::float32(reader)?,
                depth: binary::float32(reader)?,
            },
            1 => Self::Ray {
                position: binary::float32(reader)?,
                direction: binary::float32(reader)?,
                length: binary::float32(reader)?,
            },
            2 => Self::Sphere {
                radius: binary::float32(reader)?,
            },
            3 => Self::Cylinder {
                radius: binary::float32(reader)?,
                length: binary::float32(reader)?,
            },
            4 => Self::Tube {
                radius: binary::float32(reader)?,
                length: binary::float32(reader)?,
            },
            5 => Self::Plane {
                normal: [
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                ],
                distance: binary::float32(reader)?,
            },
            6 => Self::Mesh,

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Received invalid GeomParams",
                ))
            }
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            &Self::Box {
                length,
                width,
                depth,
            } => {
                binary::write_uint32(writer, 0)?;
                binary::write_float32(writer, length)?;
                binary::write_float32(writer, width)?;
                binary::write_float32(writer, depth)?;
            }
            &Self::Ray {
                position,
                direction,
                length,
            } => {
                binary::write_uint32(writer, 1)?;
                binary::write_float32(writer, position)?;
                binary::write_float32(writer, direction)?;
                binary::write_float32(writer, length)?;
            }
            &Self::Sphere { radius } => {
                binary::write_uint32(writer, 2)?;
                binary::write_float32(writer, radius)?;
            }
            &Self::Cylinder { radius, length } => {
                binary::write_uint32(writer, 3)?;
                binary::write_float32(writer, radius)?;
                binary::write_float32(writer, length)?;
            }
            &Self::Tube { radius, length } => {
                binary::write_uint32(writer, 4)?;
                binary::write_float32(writer, radius)?;
                binary::write_float32(writer, length)?;
            }
            &Self::Plane { normal, distance } => {
                binary::write_uint32(writer, 5)?;
                for v in normal {
                    binary::write_float32(writer, v)?;
                }
                binary::write_float32(writer, distance)?;
            }
            Self::Mesh => {
                binary::write_uint32(writer, 6)?;
            }
        }

        Ok(())
    }
}

/// Representation of any geometric shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyGeometry {
    /// The name of the shape.
    pub name: String,
    /// The rotation matrix of the shape.
    pub rotation: [[f32; 3]; 3],
    /// The location vector of the shape.
    pub location: [f32; 3],
    /// The scaling factor of the shape.
    pub scale: f32,
    /// The material name for the shape.
    pub material: String,
    /// Geometric shape parameters.
    pub params: GeomParams,
}

impl ProxyGeometry {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: binary::uint32(reader).and_then(|len| binary::str(reader, len, false))?,
            rotation: [
                [
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                ],
                [
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                ],
                [
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                    binary::float32(reader)?,
                ],
            ],
            location: [
                binary::float32(reader)?,
                binary::float32(reader)?,
                binary::float32(reader)?,
            ],
            scale: binary::float32(reader)?,
            material: binary::uint32(reader).and_then(|len| binary::str(reader, len, false))?,
            params: GeomParams::parse(reader)?,
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        binary::write_str(writer, &self.name, false)?;
        for r in self.rotation {
            for v in r {
                binary::write_float32(writer, v)?;
            }
        }
        for v in self.location {
            binary::write_float32(writer, v)?;
        }
        binary::write_float32(writer, self.scale)?;
        binary::write_str(writer, &self.material, false)?;
        self.params.write(writer)?;

        Ok(())
    }

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyMesh {
    /// A dynamic list of vertices in the mesh.
    pub vertices: Vec<[f32; 3]>,
    /// A dynamic list of faces in the mesh.
    pub faces: Vec<Face>,
}

impl ProxyMesh {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let vertex_count = binary::uint32(reader)?;
        let face_count = binary::uint32(reader)?;
        Ok(Self {
            vertices: binary::seq(reader, vertex_count, |r| {
                Ok([
                    binary::float32(r)?,
                    binary::float32(r)?,
                    binary::float32(r)?,
                ])
            })?,
            faces: binary::seq(reader, face_count, Face::parse)?,
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        binary::write_uint32(writer, self.vertices.len() as u32)?;
        binary::write_uint32(writer, self.faces.len() as u32)?;
        binary::write_seq(writer, false, &self.vertices, |&v, w| {
            for v in v {
                binary::write_float32(w, v)?;
            }
            Ok(())
        })?;
        binary::write_seq(writer, false, &self.faces, Face::write)?;

        Ok(())
    }
}

/// Representation of an individual collision entry.
///
/// Describes a geometric shape and the associated metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collision {
    /// The category flags for the shape.
    pub category_flags: CollisionFlags,
    /// The collision flags for the shape.
    pub collision_flags: CollisionFlags,
    /// Additional data for mesh-based collisions, if any.
    pub mesh: Option<ProxyMesh>,
    /// Universal geometric data for the collision shape.
    pub geometry: ProxyGeometry,
}

impl Collision {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let geometry_type = binary::uint32(reader)?;
        Ok(Self {
            category_flags: binary::uint32(reader).map(CollisionFlags::from_bits_truncate)?,
            collision_flags: binary::uint32(reader).map(CollisionFlags::from_bits_truncate)?,
            mesh: if geometry_type == 6 {
                Some(ProxyMesh::parse(reader)?)
            } else {
                None
            },
            geometry: ProxyGeometry::parse(reader)?,
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        binary::write_uint32(writer, self.geometry.params_type())?;
        binary::write_uint32(writer, self.category_flags.bits())?;
        binary::write_uint32(writer, self.collision_flags.bits())?;
        if let Some(mesh) = &self.mesh {
            mesh.write(writer)?;
        }
        self.geometry.write(writer)?;

        Ok(())
    }
}

/// Representation of a BCD file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bcd {
    /// A list of all [`Collision`] objects in the file.
    pub collisions: Vec<Collision>,
}

impl Bcd {
    /// Attempts to parse a BCD file from a given [`Read`]er.
    pub fn parse<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Ok(Self {
            collisions: binary::uint32(&mut reader)
                .and_then(|len| binary::seq(&mut reader, len, Collision::parse))?,
        })
    }

    /// Writes the BCD data to the given [`Write`]r.
    pub fn write<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        binary::write_seq(&mut writer, true, &self.collisions, Collision::write)?;

        Ok(())
    }
}
