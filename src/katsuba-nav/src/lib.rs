//! Crate for parsing and writing Zone Navigation Graph (NAV) files.
//!
//! As the name suggests, this format describes an interconnected network
//! between zones as edges and their travel paths as vertices.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinResult, BinWriterExt,
};
use katsuba_utils::binrw_ext::{read_string_list, write_string_list};
use serde::{Deserialize, Serialize};

/// A navigation node in the zone.
#[binrw]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationNode {
    /// The location of the node.
    pub location: [f32; 3],
    /// The unique identifier of the node.
    pub id: u16,
}

/// A link between two [`NavigationNode`]s in the graph.
#[binrw]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationLink {
    /// The first [`NavigationNode`] identifier.
    pub first: u16,
    /// The second [`NavigationNode`] identifier.
    pub second: u16,
}

/// A graph of navigation nodes and their interconnections.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationGraph {
    #[br(temp)]
    #[bw(calc = self.nodes.iter().map(|n| n.id).max().unwrap_or(0))]
    #[serde(skip)]
    _last_id: u16,

    #[br(temp)]
    #[bw(calc = self.nodes.len() as u32)]
    #[serde(skip)]
    node_count: u32,

    /// The navigation nodes, representing the edges of the graph.
    #[br(count = node_count)]
    pub nodes: Vec<NavigationNode>,

    #[br(temp)]
    #[bw(calc = self.links.len() as u32)]
    link_count: u32,

    /// The links between the nodes, representing the vertices of
    /// the graph.
    #[br(count = link_count)]
    pub links: Vec<NavigationLink>,
}

impl NavigationGraph {
    /// Attempts to parse a NAV graph from a given [`Read`]er.
    pub fn parse<R: Read + Seek>(mut reader: R) -> BinResult<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes the NAV graph to the given [`Write`]r.
    pub fn write<W: Write + Seek>(&self, mut writer: W) -> BinResult<()> {
        writer.write_le(self).map_err(Into::into)
    }
}

/// A navigation graph across zones.
#[binrw]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneNavigationGraph {
    /// The raw [`NavigationGraph`].
    pub graph: NavigationGraph,

    #[br(temp)]
    #[bw(calc = self.zone_names.len() as u32)]
    zone_count: u32,

    /// A list of zone names covered by the graph.
    ///
    /// Each index corresponds to a [`NavigationNode`]'s identifier.
    #[br(args(zone_count as _, false), parse_with = read_string_list)]
    #[bw(args(false), write_with = write_string_list)]
    pub zone_names: Vec<String>,
}

impl ZoneNavigationGraph {
    /// Attempts to parse a zonenav graph from a given [`Read`]er.
    pub fn parse<R: Read + Seek>(mut reader: R) -> BinResult<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes the zonenav graph to the given [`Write`]r.
    pub fn write<W: Write + Seek>(&self, mut writer: W) -> BinResult<()> {
        writer.write_le(self).map_err(Into::into)
    }
}
