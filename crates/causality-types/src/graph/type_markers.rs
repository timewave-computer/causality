// Type Markers for Graph Component
//
// This module defines marker types used to identify different graph components.
// These are used in type-level programming to distinguish between different
// kinds of graph elements.

use crate::serialization::{Decode, Encode, SimpleSerialize};
use std::fmt;
use std::hash::Hash;

/// Marker type for Graph
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct GraphTypeMarker;

/// Marker type for Subgraph
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct SubgraphTypeMarker;

/// Marker type for Node
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct NodeTypeMarker;

/// Marker type for Edge
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct EdgeTypeMarker;

//-----------------------------------------------------------------------------
// Display Implementation
//-----------------------------------------------------------------------------

impl fmt::Display for GraphTypeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GraphTypeMarker")
    }
}

impl fmt::Display for SubgraphTypeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubgraphTypeMarker")
    }
}

impl fmt::Display for NodeTypeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeTypeMarker")
    }
}

impl fmt::Display for EdgeTypeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EdgeTypeMarker")
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementation
//-----------------------------------------------------------------------------

// Implement Encode and Decode for marker types
// Since these are unit types, serialization is simple

impl Encode for GraphTypeMarker {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        Vec::new() // Unit type is empty
    }
}

impl Decode for GraphTypeMarker {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
        if !bytes.is_empty() {
            return Err(crate::serialization::DecodeError {
                message: format!("Invalid unit type length {}, expected 0", bytes.len()),
            });
        }
        Ok(GraphTypeMarker)
    }
}

impl SimpleSerialize for GraphTypeMarker {}

impl Encode for SubgraphTypeMarker {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        Vec::new() // Unit type is empty
    }
}

impl Decode for SubgraphTypeMarker {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
        if !bytes.is_empty() {
            return Err(crate::serialization::DecodeError {
                message: format!("Invalid unit type length {}, expected 0", bytes.len()),
            });
        }
        Ok(SubgraphTypeMarker)
    }
}

impl SimpleSerialize for SubgraphTypeMarker {}

impl Encode for NodeTypeMarker {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        Vec::new() // Unit type is empty
    }
}

impl Decode for NodeTypeMarker {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
        if !bytes.is_empty() {
            return Err(crate::serialization::DecodeError {
                message: format!("Invalid unit type length {}, expected 0", bytes.len()),
            });
        }
        Ok(NodeTypeMarker)
    }
}

impl SimpleSerialize for NodeTypeMarker {}

impl Encode for EdgeTypeMarker {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        Vec::new() // Unit type is empty
    }
}

impl Decode for EdgeTypeMarker {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
        if !bytes.is_empty() {
            return Err(crate::serialization::DecodeError {
                message: format!("Invalid unit type length {}, expected 0", bytes.len()),
            });
        }
        Ok(EdgeTypeMarker)
    }
}

impl SimpleSerialize for EdgeTypeMarker {}
