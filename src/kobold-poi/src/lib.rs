//! Crate for parsing and writing Point of Interest (POI) files.
//!
//! This format describes interactive events at zone coordinates.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use kobold_utils::{
    binrw::{
        self, binrw,
        io::{Read, Seek, Write},
        BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, VecArgs,
    },
    binrw_ext::*,
};
use serde::{Deserialize, Serialize};

/// An event point inside a [`Poi`] object.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Whether the quest helper references this point.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub no_quest_helper: bool,
    /// The ID of the zone this point is part of.
    pub zone_id: u16,
    /// The template ID associated with this point.
    pub template_id: u64,
    /// The location of this point.
    pub location: [f32; 3],
    /// Whether this point is an interactable NPC.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub interactable: bool,
    /// Whether this point is a collectable item.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub collectable: bool,
}

/// Representation of a teleporter entry in [`Poi`] files.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Teleporter {
    #[br(temp)]
    #[bw(calc = self.destination.len() as u32)]
    len: u32,

    /// The destination zone for the teleport.
    #[br(args(len as _, false), parse_with = read_prefixed_string)]
    #[bw(args(false), write_with = write_prefixed_string)]
    pub destination: String,

    /// The exact teleport position in the zone.
    pub position: [f32; 3],
}

/// Representation of a POI file.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Poi {
    #[br(temp)]
    #[bw(calc = self.zone_names.len() as u32)]
    zone_count: u32,

    /// A list of all zone names described by this file.
    #[br(args(zone_count as _, false), parse_with = read_string_list)]
    #[bw(args(false), write_with = write_string_list)]
    pub zone_names: Vec<String>,

    #[br(temp)]
    #[bw(calc = self.goals.len() as u32)]
    goal_count: u32,

    /// A mapping of goal IDs to the respective [`Point`]s.
    #[br(args(goal_count as _, |()| ()), parse_with = read_map)]
    #[bw(args(|_| ()), write_with = write_map)]
    pub goals: HashMap<u64, Point>,

    #[br(temp)]
    #[bw(calc = self.interactive_goals.len() as u32)]
    interactive_goal_count: u32,

    /// A mapping of zone IDs to lists of interactable template IDs.
    #[br(
        args(interactive_goal_count as _, |x: u32| VecArgs { count: x as _, inner: () }),
        parse_with = read_map,
    )]
    #[bw(args(|v| v.len() as u32), write_with = write_map)]
    pub interactive_goals: HashMap<u32, Vec<u64>>,

    #[br(temp)]
    #[bw(calc = self.teleporters.len() as u32)]
    teleporters_count: u32,

    /// Teleporter entries between zones in this file.
    #[br(
        args(teleporters_count as _, |x: u32| VecArgs { count: x as _, inner: () }),
        parse_with = read_map,
    )]
    #[bw(args(|v| v.len() as u32), write_with = write_map)]
    pub teleporters: HashMap<u32, Vec<Teleporter>>,

    #[br(temp)]
    #[bw(calc = self.goal_adjectives.len() as u32)]
    goal_adjective_count: u32,

    /// A mapping of goal IDs to goal adjectives.
    #[br(
        args(goal_adjective_count as _, |x: u32| VecArgs { count: x as _, inner: () }),
        parse_with = read_map,
    )]
    #[bw(args(|v| v.len() as u32), write_with = write_map)]
    pub goal_adjectives: HashMap<u64, Vec<u32>>,

    #[br(temp)]
    #[bw(calc = self.zone_mobs.values().map(|v| v.len()).sum::<usize>() as u32)]
    zone_mob_count: u32,

    /// A list of zone mobs for each zone ID in the file.
    #[br(args(zone_mob_count as _), parse_with = read_zone_mobs)]
    #[bw(write_with = write_zone_mobs)]
    pub zone_mobs: HashMap<u32, Vec<String>>,
}

impl Poi {
    /// Attempts to parse a BCD file from a given [`Read`]er.
    pub fn parse<R: Read + Seek>(mut reader: R) -> BinResult<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes the BCD data to the given [`Write`]r.
    pub fn write<W: Write + Seek>(&self, mut writer: W) -> BinResult<()> {
        writer.write_le(self).map_err(Into::into)
    }
}

#[binrw::parser(reader, endian)]
fn read_zone_mobs(count: usize) -> BinResult<HashMap<u32, Vec<String>>> {
    let mut map: HashMap<u32, Vec<String>> = HashMap::with_capacity(count);

    for _ in 0..count {
        let zone_id = u32::read_options(reader, endian, ())?;

        let len = u32::read_options(reader, endian, ())? as usize;
        let mob_asset = read_prefixed_string(reader, endian, (len, false))?;

        map.entry(zone_id).or_default().push(mob_asset);
    }

    Ok(map)
}

#[binrw::writer(writer, endian)]
fn write_zone_mobs(mobs: &HashMap<u32, Vec<String>>) -> BinResult<()> {
    for (zone_id, mob_assets) in mobs {
        for mob_asset in mob_assets {
            zone_id.write_options(writer, endian, ())?;

            (mob_asset.len() as u32).write_options(writer, endian, ())?;
            write_prefixed_string(mob_asset, writer, endian, (false,))?;
        }
    }

    Ok(())
}
