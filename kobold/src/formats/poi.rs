use std::{collections::HashMap, ops::Deref};

use binrw::{
    binread,
    io::{Read, Seek},
    BinRead, BinReaderExt, BinResult, ReadOptions,
};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use super::utils;

/// A zone ID as referenced in the POI format.
#[binread]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ZoneId(pub(crate) u32);

impl ZoneId {
    /// Gets the inner value of the zone ID.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

#[cfg(feature = "python")]
impl IntoPy<PyObject> for ZoneId {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0.into_py(py)
    }
}

#[binread]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SizedString {
    #[br(temp)]
    len: u32,
    #[br(args(len as usize), parse_with = utils::parse_string)]
    #[serde(flatten)]
    pub data: String,
}

impl Deref for SizedString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(feature = "python")]
impl IntoPy<PyObject> for SizedString {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.data.into_py(py)
    }
}

/// A point of interest in a [`Poi`] object.
///
/// Points of Interests describe event sources for
/// interacting with zones.
#[binread]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointOfInterest {
    /// If the quest helper references this point.
    #[br(map = |x: u8| x != 0)]
    pub no_quest_helper: bool,
    /// The [`ZoneId`] identifying the zone this
    /// point is part of.
    pub zone_id: ZoneId,
    /// The template ID for this point of interest.
    pub template_id: u64,
    /// The location of this point.
    pub location: [f32; 3],
    /// Whether this point is an interactable NPC.
    #[br(map = |x: u8| x != 0)]
    pub interactable: bool,
    /// Whether this point is a collectable item.
    #[br(map = |x: u8| x != 0)]
    pub collectable: bool,
}

/// A full POI object with all its points.
#[binread]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Poi {
    #[br(temp)]
    zone_count: u32,

    #[br(count = zone_count)]
    zone_names: Vec<SizedString>,

    #[br(temp)]
    goal_count: u32,

    /// A mapping of goal template IDs to associated
    /// [`PointOfInterest`]s.
    #[br(args(goal_count as usize), parse_with = utils::parse_hashmap)]
    pub goals: HashMap<u64, PointOfInterest>,

    #[br(temp)]
    zone_interactable_goal_count: u32,

    /// A mapping of [`ZoneId`]s to lists of interactable
    /// goal template IDs.
    #[br(args(zone_interactable_goal_count as usize), parse_with = utils::parse_vec_hashmap)]
    pub interactable_goals: HashMap<ZoneId, Vec<u64>>,

    #[br(temp)]
    zone_teleporters_count: u32,

    /// A mapping of [`ZoneId`]s to mappings of destination
    /// zone names to teleporter locations.
    #[br(args(zone_teleporters_count as usize), parse_with = parse_zone_teleporters)]
    pub zone_teleporters: HashMap<ZoneId, HashMap<String, Vec<[f32; 3]>>>,

    #[br(temp)]
    goal_adjective_count: u32,

    /// A mapping of goal template IDs to adjectives.
    #[br(args(goal_adjective_count as usize), parse_with = utils::parse_vec_hashmap)]
    pub goal_adjectives: HashMap<u64, Vec<u32>>,

    #[br(temp)]
    zone_mob_count: u32,

    /// A mapping of [`ZoneId`]s to lists of mob names
    /// in that zone.
    #[br(args(zone_mob_count as usize), parse_with = parse_zone_mobs)]
    pub zone_mobs: HashMap<ZoneId, Vec<String>>,
}

impl Poi {
    /// Attempts to parse a POI file from a given input source.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Gets a zone name given its ID, if exists.
    pub fn zone_name(&self, id: ZoneId) -> Option<&str> {
        self.zone_names.get(id.0 as usize).map(|x| x.as_str())
    }
}

#[allow(clippy::type_complexity)]
fn parse_zone_teleporters<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    (count,): (usize,),
) -> BinResult<HashMap<ZoneId, HashMap<String, Vec<[f32; 3]>>>> {
    let mut map: HashMap<ZoneId, HashMap<String, Vec<[f32; 3]>>> = HashMap::with_capacity(count);
    for _ in 0..count {
        let zone_id = u32::read_options(reader, options, ()).map(ZoneId)?;

        let teleporter_count = u32::read_options(reader, options, ())? as usize;
        for _ in 0..teleporter_count {
            let target_zone = SizedString::read_options(reader, options, ())?;
            let position = <[f32; 3]>::read_options(reader, options, ())?;

            let entry = map.entry(zone_id).or_insert_with(HashMap::new);
            let inner = entry
                .entry(target_zone.data)
                .or_insert_with(|| vec![position]);

            if !inner.contains(&position) {
                inner.push(position);
            }
        }
    }

    Ok(map)
}

fn parse_zone_mobs<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    (count,): (usize,),
) -> BinResult<HashMap<ZoneId, Vec<String>>> {
    let mut map = HashMap::with_capacity(count);
    for _ in 0..count {
        let zone_id = u32::read_options(reader, options, ()).map(ZoneId)?;
        let mob_asset = SizedString::read_options(reader, options, ())?;

        map.entry(zone_id)
            .or_insert_with(Vec::new)
            .push(mob_asset.data);
    }

    Ok(map)
}
