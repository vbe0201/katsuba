//! Crate for parsing and writing Point of Interest (POI) files.
//!
//! This format describes interactive events at zone coordinates.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::{collections::HashMap, io};

use katsuba_utils::binary;
use serde::{Deserialize, Serialize};

/// An event point inside a [`Poi`] object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Whether the quest helper references this point.
    pub no_quest_helper: bool,
    /// The ID of the zone this point is part of.
    pub zone_id: u16,
    /// The template ID associated with this point.
    pub template_id: u64,
    /// The location of this point.
    pub location: [f32; 3],
    /// Whether this point is an interactable NPC.
    pub interactable: bool,
    /// Whether this point is a collectable item.
    pub collectable: bool,
}

impl Point {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            no_quest_helper: binary::boolean(reader)?,
            zone_id: binary::uint16(reader)?,
            template_id: binary::uint64(reader)?,
            location: [
                binary::float32(reader)?,
                binary::float32(reader)?,
                binary::float32(reader)?,
            ],
            interactable: binary::boolean(reader)?,
            collectable: binary::boolean(reader)?,
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        binary::write_boolean(writer, self.no_quest_helper)?;
        binary::write_uint16(writer, self.zone_id)?;
        binary::write_uint64(writer, self.template_id)?;
        for v in self.location {
            binary::write_float32(writer, v)?;
        }
        binary::write_boolean(writer, self.interactable)?;
        binary::write_boolean(writer, self.collectable)?;

        Ok(())
    }
}

/// Representation of a teleporter entry in [`Poi`] files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Teleporter {
    /// The destination zone for the teleport.
    pub destination: String,
    /// The exact teleport position in the zone.
    pub position: [f32; 3],
}

impl Teleporter {
    fn parse<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            destination: binary::uint32(reader).and_then(|len| binary::str(reader, len, false))?,
            position: [
                binary::float32(reader)?,
                binary::float32(reader)?,
                binary::float32(reader)?,
            ],
        })
    }

    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        binary::write_str(writer, &self.destination, false)?;
        for v in self.position {
            binary::write_float32(writer, v)?;
        }

        Ok(())
    }
}

/// Representation of a POI file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Poi {
    /// A list of all zone names described by this file.
    pub zone_names: Vec<String>,
    /// A mapping of goal IDs to the respective [`Point`]s.
    pub goals: HashMap<u64, Point>,
    /// A mapping of zone IDs to lists of interactable template IDs.
    pub interactive_goals: HashMap<u32, Vec<u64>>,
    /// Teleporter entries between zones in this file.
    pub teleporters: HashMap<u32, Vec<Teleporter>>,
    /// A mapping of goal IDs to goal adjectives.
    pub goal_adjectives: HashMap<u64, Vec<u32>>,
    /// A list of zone mobs for each zone ID in the file.
    pub zone_mobs: HashMap<u32, Vec<String>>,
}

impl Poi {
    /// Attempts to parse a BCD file from a given [`Read`]er.
    pub fn parse<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Ok(Self {
            zone_names: binary::uint32(&mut reader).and_then(|len| {
                binary::seq(&mut reader, len, |r| {
                    let len = binary::uint32(r)?;
                    binary::str(r, len, false)
                })
            })?,
            goals: binary::uint32(&mut reader).and_then(|len| {
                binary::map(&mut reader, len, |r| binary::uint64(r), Point::parse)
            })?,
            interactive_goals: binary::uint32(&mut reader).and_then(|len| {
                binary::map(&mut reader, len, binary::uint32, |r| {
                    let len = binary::uint32(r)?;
                    binary::seq(r, len, binary::uint64)
                })
            })?,
            teleporters: binary::uint32(&mut reader).and_then(|len| {
                binary::map(&mut reader, len, binary::uint32, |r| {
                    let len = binary::uint32(r)?;
                    binary::seq(r, len, Teleporter::parse)
                })
            })?,
            goal_adjectives: binary::uint32(&mut reader).and_then(|len| {
                binary::map(&mut reader, len, binary::uint64, |r| {
                    let len = binary::uint32(r)?;
                    binary::seq(r, len, binary::uint32)
                })
            })?,
            zone_mobs: binary::uint32(&mut reader).and_then(|len| {
                binary::map(&mut reader, len, binary::uint32, |r| {
                    let len = binary::uint32(r)?;
                    binary::seq(r, len, |r| {
                        let len = binary::uint32(r)?;
                        binary::str(r, len, false)
                    })
                })
            })?,
        })
    }

    /// Writes the BCD data to the given [`Write`]r.
    pub fn write<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        /*
        pub goal_adjectives: HashMap<u64, Vec<u32>>,
        /// A list of zone mobs for each zone ID in the file.
        pub zone_mobs: HashMap<u32, Vec<String>>,
             */
        binary::write_seq(&mut writer, true, &self.zone_names, |v, w| {
            binary::write_str(w, v, false)
        })?;
        binary::write_map(
            &mut writer,
            true,
            &self.goals,
            |&v, w| binary::write_uint64(w, v),
            Point::write,
        )?;
        binary::write_map(
            &mut writer,
            true,
            &self.interactive_goals,
            |&v, w| binary::write_uint32(w, v),
            |v, w| binary::write_seq(w, true, v, |&v, w| binary::write_uint64(w, v)),
        )?;
        binary::write_map(
            &mut writer,
            true,
            &self.teleporters,
            |&v, w| binary::write_uint32(w, v),
            |v, w| binary::write_seq(w, true, v, Teleporter::write),
        )?;
        binary::write_map(
            &mut writer,
            true,
            &self.goal_adjectives,
            |&v, w| binary::write_uint64(w, v),
            |v, w| binary::write_seq(w, true, v, |&v, w| binary::write_uint32(w, v)),
        )?;
        binary::write_map(
            &mut writer,
            true,
            &self.zone_mobs,
            |&v, w| binary::write_uint32(w, v),
            |v, w| binary::write_seq(w, true, v, |v, w| binary::write_str(w, v, false)),
        )?;

        Ok(())
    }
}
