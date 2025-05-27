// Purpose: Defines the Subgraph structure, a collection of nodes and edges.

use crate::primitive::ids::{EdgeId, NodeId, SubgraphId};
use crate::expr::value::ValueExpr;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::collections::{BTreeMap, HashSet};

/// Represents a subgraph, a collection of nodes and edges within a larger graph.
/// Subgraphs are the primary units of compilation and circuit generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subgraph {
    /// Unique identifier for this subgraph.
    pub id: SubgraphId,

    /// Set of nodes that are part of this subgraph.
    /// These NodeIds should correspond to Node instances stored elsewhere (e.g., in a NodeRegistry).
    pub nodes: HashSet<NodeId>,

    /// Set of edges that are part of this subgraph.
    /// These EdgeIds should correspond to Edge instances stored elsewhere (e.g., in an EdgeRegistry).
    /// It's typically assumed that edges connect nodes within this subgraph's node set,
    /// or define connections at the boundary.
    pub edges: HashSet<EdgeId>,

    /// Optional: Ordered list of NodeIds representing the entry points into this subgraph.
    /// The order might be significant for execution or data flow.
    pub entry_nodes: Vec<NodeId>,

    /// Optional: Ordered list of NodeIds representing the exit points from this subgraph.
    /// The order might be significant for execution or data flow.
    pub exit_nodes: Vec<NodeId>,

    /// Arbitrary metadata associated with the subgraph.
    /// This can be used for properties like subgraph name, version, or compilation hints.
    pub metadata: BTreeMap<String, ValueExpr>,
}

impl Subgraph {
    /// Creates a new Subgraph with the given ID.
    pub fn new(id: SubgraphId) -> Self {
        Self {
            id,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            entry_nodes: Vec::new(),
            exit_nodes: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

impl SimpleSerialize for Subgraph {}

impl Encode for Subgraph {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Serialize SubgraphId
        let id_bytes = self.id.as_ssz_bytes();
        result.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&id_bytes);
        
        // Serialize nodes HashSet
        let nodes_vec: Vec<NodeId> = self.nodes.iter().cloned().collect();
        let nodes_bytes = nodes_vec.as_ssz_bytes();
        result.extend_from_slice(&(nodes_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&nodes_bytes);
        
        // Serialize edges HashSet
        let edges_vec: Vec<EdgeId> = self.edges.iter().cloned().collect();
        let edges_bytes = edges_vec.as_ssz_bytes();
        result.extend_from_slice(&(edges_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&edges_bytes);
        
        // Serialize entry_nodes Vec
        let entry_bytes = self.entry_nodes.as_ssz_bytes();
        result.extend_from_slice(&(entry_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&entry_bytes);
        
        // Serialize exit_nodes Vec
        let exit_bytes = self.exit_nodes.as_ssz_bytes();
        result.extend_from_slice(&(exit_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&exit_bytes);
        
        // Serialize metadata BTreeMap
        let metadata_bytes = self.metadata.as_ssz_bytes();
        result.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&metadata_bytes);
        
        result
    }
}

impl Decode for Subgraph {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Deserialize SubgraphId
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Truncated subgraph ID length"));
        }
        let id_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        if offset + id_len > bytes.len() {
            return Err(DecodeError::new("Truncated subgraph ID data"));
        }
        let id = SubgraphId::from_ssz_bytes(&bytes[offset..offset+id_len])
            .map_err(|_| DecodeError::new("Failed to decode SubgraphId"))?;
        offset += id_len;
        
        // Deserialize nodes
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Truncated nodes length"));
        }
        let nodes_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        if offset + nodes_len > bytes.len() {
            return Err(DecodeError::new("Truncated nodes data"));
        }
        let nodes_vec = Vec::<NodeId>::from_ssz_bytes(&bytes[offset..offset+nodes_len])
            .map_err(|_| DecodeError::new("Failed to decode nodes"))?;
        let nodes: HashSet<NodeId> = nodes_vec.into_iter().collect();
        offset += nodes_len;
        
        // Deserialize edges
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Truncated edges length"));
        }
        let edges_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        if offset + edges_len > bytes.len() {
            return Err(DecodeError::new("Truncated edges data"));
        }
        let edges_vec = Vec::<EdgeId>::from_ssz_bytes(&bytes[offset..offset+edges_len])
            .map_err(|_| DecodeError::new("Failed to decode edges"))?;
        let edges: HashSet<EdgeId> = edges_vec.into_iter().collect();
        offset += edges_len;
        
        // Deserialize entry_nodes
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Truncated entry nodes length"));
        }
        let entry_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        if offset + entry_len > bytes.len() {
            return Err(DecodeError::new("Truncated entry nodes data"));
        }
        let entry_nodes = Vec::<NodeId>::from_ssz_bytes(&bytes[offset..offset+entry_len])
            .map_err(|_| DecodeError::new("Failed to decode entry nodes"))?;
        offset += entry_len;
        
        // Deserialize exit_nodes
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Truncated exit nodes length"));
        }
        let exit_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        if offset + exit_len > bytes.len() {
            return Err(DecodeError::new("Truncated exit nodes data"));
        }
        let exit_nodes = Vec::<NodeId>::from_ssz_bytes(&bytes[offset..offset+exit_len])
            .map_err(|_| DecodeError::new("Failed to decode exit nodes"))?;
        offset += exit_len;
        
        // Deserialize metadata
        if offset + 4 > bytes.len() {
            return Err(DecodeError::new("Truncated metadata length"));
        }
        let metadata_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        if offset + metadata_len > bytes.len() {
            return Err(DecodeError::new("Truncated metadata data"));
        }
        let metadata = BTreeMap::<String, ValueExpr>::from_ssz_bytes(&bytes[offset..offset+metadata_len])
            .map_err(|_| DecodeError::new("Failed to decode metadata"))?;
        
        Ok(Subgraph {
            id,
            nodes,
            edges,
            entry_nodes,
            exit_nodes,
            metadata,
        })
    }
}
