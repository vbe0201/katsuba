//! Library for parsing the Wizard101 Navigation Graph
//! (NAV) format.
//!
//! This format allows for pathfinding within zones and
//! between zones.

use std::collections::HashMap;

use binrw::{
    binread,
    io::{Read, Seek},
    BinRead, BinReaderExt, BinResult,
};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use super::utils;

/// A navigation node in the zone.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct NavigationNode {
    /// The location of the node.
    pub location: [f32; 3],
    /// The unique ID of the node.
    pub id: u16,
}

/// A link between two [`NavigationNode`]s.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, PartialEq, Eq, BinRead, Serialize, Deserialize)]
pub struct NavigationLink {
    first: u16,
    second: u16,
}

impl NavigationLink {
    /// The ID of the first linked node.
    pub fn first<'g>(&self, graph: &'g NavigationGraph) -> Option<&'g NavigationNode> {
        graph.find_node(self.first)
    }

    /// The ID of the second linked node.
    pub fn second<'g>(&self, graph: &'g NavigationGraph) -> Option<&'g NavigationNode> {
        graph.find_node(self.second)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl NavigationLink {
    pub fn find_first(&self, graph: PyRef<NavigationGraph>) -> PyResult<Option<NavigationNode>> {
        Ok(graph.find_node(self.first).copied())
    }

    pub fn find_second(&self, graph: PyRef<NavigationGraph>) -> PyResult<Option<NavigationNode>> {
        Ok(graph.find_node(self.second).copied())
    }
}

/// A graph of navigation nodes and their links between
/// each other within a zone.
#[binread]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationGraph {
    #[br(temp)]
    last_id: u16,

    #[br(temp)]
    node_count: u32,
    /// The [`NavigationNode`]s in the graph.
    #[br(count = node_count)]
    pub nodes: Vec<NavigationNode>,

    #[br(calc = Self::build_node_map(&nodes))]
    #[serde(skip)]
    nodes_map: HashMap<u16, usize>,

    #[br(temp)]
    link_count: u32,
    /// The links between the [`NavigationNode`]s.
    #[br(count = link_count)]
    pub links: Vec<NavigationLink>,
}

impl NavigationGraph {
    fn build_node_map(nodes: &[NavigationNode]) -> HashMap<u16, usize> {
        let mut map = HashMap::with_capacity(nodes.len());
        for (idx, node) in nodes.iter().enumerate() {
            map.insert(node.id, idx);
        }
        map
    }

    /// Attempts to parse a NAV file from a given input source.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }

    /// Gets a [`NavigationNode`] given its ID, if present.
    pub fn find_node(&self, id: u16) -> Option<&NavigationNode> {
        self.nodes_map.get(&id).map(|&value| &self.nodes[value])
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl NavigationGraph {
    pub fn get_node(&self, id: u16) -> PyResult<Option<NavigationNode>> {
        Ok(self.find_node(id).copied())
    }
}

/// A navigation graph across zones.
#[binread]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneNavigationGraph {
    /// The raw [`NavigationGraph`].
    pub graph: NavigationGraph,

    #[br(temp)]
    zone_count: u32,
    /// A list of zone names covered by the graph.
    #[br(args(zone_count as usize), parse_with = utils::parse_string_vec::<u32, _>)]
    pub zone_names: Vec<String>,
}

impl ZoneNavigationGraph {
    /// Attempts to parse a zonenav file from a given input source.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }
}
