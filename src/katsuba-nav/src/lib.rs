//! Crate for parsing and writing Zone Navigation Graph (NAV) files.
//!
//! As the name suggests, this format describes an interconnected network
//! between zones as edges and their travel paths as vertices.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::io;

use katsuba_utils::binary;
use serde::{Deserialize, Serialize};

/// A navigation node in the zone.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationNode {
    /// The location of the node.
    pub location: [f32; 3],
    /// The unique identifier of the node.
    pub id: u16,
}

/// A link between two [`NavigationNode`]s in the graph.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationLink {
    /// The first [`NavigationNode`] identifier.
    pub first: u16,
    /// The second [`NavigationNode`] identifier.
    pub second: u16,
}

/// A graph of navigation nodes and their interconnections.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationGraph {
    /// The navigation nodes, representing the edges of the graph.
    pub nodes: Vec<NavigationNode>,
    /// The links between the nodes, representing the vertices of
    /// the graph.
    pub links: Vec<NavigationLink>,
}

impl NavigationGraph {
    /// Attempts to parse a NAV graph from a given [`Read`]er.
    pub fn parse<R: io::Read>(mut reader: R) -> io::Result<Self> {
        binary::uint16(&mut reader)?;
        Ok(Self {
            nodes: binary::uint32(&mut reader).and_then(|len| {
                binary::seq(&mut reader, len, |r| {
                    Ok(NavigationNode {
                        location: [
                            binary::float32(r)?,
                            binary::float32(r)?,
                            binary::float32(r)?,
                        ],
                        id: binary::uint16(r)?,
                    })
                })
            })?,
            links: binary::uint32(&mut reader).and_then(|len| {
                binary::seq(&mut reader, len, |r| {
                    Ok(NavigationLink {
                        first: binary::uint16(r)?,
                        second: binary::uint16(r)?,
                    })
                })
            })?,
        })
    }

    /// Writes the NAV graph to the given [`Write`]r.
    pub fn write<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        binary::write_uint16(
            &mut writer,
            self.nodes.iter().map(|n| n.id).max().unwrap_or(0),
        )?;
        binary::write_seq(&mut writer, true, &self.nodes, |v, w| {
            for v in v.location {
                binary::write_float32(w, v)?;
            }
            binary::write_uint16(w, v.id)?;

            Ok(())
        })?;
        binary::write_seq(&mut writer, true, &self.links, |v, w| {
            binary::write_uint16(w, v.first)?;
            binary::write_uint16(w, v.second)?;

            Ok(())
        })?;

        Ok(())
    }
}

/// A navigation graph across zones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneNavigationGraph {
    /// The raw [`NavigationGraph`].
    pub graph: NavigationGraph,
    /// A list of zone names covered by the graph.
    ///
    /// Each index corresponds to a [`NavigationNode`]'s identifier.
    pub zone_names: Vec<String>,
}

impl ZoneNavigationGraph {
    /// Attempts to parse a zonenav graph from a given [`Read`]er.
    pub fn parse<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Ok(Self {
            graph: NavigationGraph::parse(&mut reader)?,
            zone_names: binary::uint32(&mut reader).and_then(|len| {
                binary::seq(&mut reader, len, |r| {
                    let len = binary::uint32(r)?;
                    binary::str(r, len, false)
                })
            })?,
        })
    }

    /// Writes the zonenav graph to the given [`Write`]r.
    pub fn write<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.graph.write(&mut writer)?;
        binary::write_seq(&mut writer, true, &self.zone_names, |v, w| {
            binary::write_str(w, v, false)
        })?;

        Ok(())
    }
}
