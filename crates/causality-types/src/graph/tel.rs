//! Temporal Effect Language (TEL)
//!
//! Defines the core data structures and types for the Temporal Effect Language,
//! which is used to express temporal effects and their relationships in the
//! Causality framework.

// Re-export unified core types for convenience in TEL context
pub use crate::effect::{Effect, Intent, Handler, Transaction};
pub use crate::resource::Resource;

//-----------------------------------------------------------------------------
// TEL Core Types (from tel/graph.rs, tel/graph_structure.rs, tel/graph_types.rs, tel/common_refs.rs)
//-----------------------------------------------------------------------------

use crate::primitive::ids::{EntityId, DomainId, ResourceId, NodeId, EdgeId, HandlerId, AsId};
use crate::primitive::string::Str;
use crate::expression::value::ValueExpr;
use crate::graph::r#trait::AsEdge;
use crate::serialization::{Encode, Decode, DecodeError, DecodeWithLength, SimpleSerialize};
use std::collections::BTreeMap;

/// A reference to a resource within the TEL system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRef {
    /// The ID of the resource being referenced
    pub resource_id: ResourceId,
    /// The domain where this resource exists
    pub domain_id: DomainId,
    /// Optional type information for the resource
    pub resource_type: Option<Str>,
}

impl ResourceRef {
    /// Create a new resource reference
    pub fn new(resource_id: ResourceId, domain_id: DomainId) -> Self {
        Self {
            resource_id,
            domain_id,
            resource_type: None,
        }
    }

    /// Create a new resource reference with type information
    pub fn with_type(resource_id: ResourceId, domain_id: DomainId, resource_type: Str) -> Self {
        Self {
            resource_id,
            domain_id,
            resource_type: Some(resource_type),
        }
    }
}

impl Encode for ResourceRef {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.resource_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        
        // Encode the optional resource_type
        if let Some(ref resource_type) = self.resource_type {
            bytes.push(1u8); // Some variant
            bytes.extend_from_slice(&resource_type.as_ssz_bytes());
        } else {
            bytes.push(0u8); // None variant
        }
        
        bytes
    }
}

impl Decode for ResourceRef {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (resource_ref, _consumed) = Self::from_ssz_bytes_with_length(bytes)?;
        Ok(resource_ref)
    }
}

impl DecodeWithLength for ResourceRef {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let mut offset = 0;
        
        // Decode resource_id (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("ResourceRef: Input bytes too short for resource_id"));
        }
        let resource_id = ResourceId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        // Decode domain_id (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("ResourceRef: Input bytes too short for domain_id"));
        }
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        // Decode optional resource_type
        if offset >= bytes.len() {
            return Err(DecodeError::new("ResourceRef: Insufficient bytes for resource_type variant"));
        }
        
        let resource_type = match bytes[offset] {
            0u8 => {
                offset += 1;
                None // None variant
            }
            1u8 => {
                offset += 1;
                let resource_type = Str::from_ssz_bytes(&bytes[offset..])?;
                let type_len = resource_type.as_ssz_bytes().len();
                offset += type_len;
                Some(resource_type)
            }
            _ => return Err(DecodeError::new("ResourceRef: Invalid variant for resource_type")),
        };
        
        Ok((ResourceRef {
            resource_id,
            domain_id,
            resource_type,
        }, offset))
    }
}

impl SimpleSerialize for ResourceRef {}

impl From<ResourceId> for ResourceRef {
    fn from(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            domain_id: DomainId::null(), // Default domain
            resource_type: None,
        }
    }
}

/// Represents different types of edges in the effect graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// A dependency edge (A depends on B)
    Dependency,
    /// A temporal ordering edge (A happens before B)
    Temporal,
    /// A resource flow edge (resource flows from A to B)
    ResourceFlow,
    /// A constraint edge (A constrains B)
    Constraint,
    /// A custom edge type with a string identifier
    Custom(Str),
    /// Control flow edge
    ControlFlow,
    /// Next edge with target node
    Next(NodeId),
    /// Dependency edge with target node
    DependsOn(NodeId),
    /// Resource consumption edge
    Consumes(ResourceRef),
    /// Resource production edge
    Produces(ResourceRef),
    /// Handler application edge
    Applies(HandlerId),
    /// Scoping edge
    ScopedBy(HandlerId),
    /// Override edge
    Override(HandlerId),
}

impl Encode for EdgeKind {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            EdgeKind::Dependency => bytes.push(0u8),
            EdgeKind::Temporal => bytes.push(1u8),
            EdgeKind::ResourceFlow => bytes.push(2u8),
            EdgeKind::Constraint => bytes.push(3u8),
            EdgeKind::Custom(s) => {
                bytes.push(4u8);
                bytes.extend_from_slice(&s.as_ssz_bytes());
            }
            EdgeKind::ControlFlow => bytes.push(5u8),
            EdgeKind::Next(node_id) => {
                bytes.push(6u8);
                bytes.extend_from_slice(&node_id.as_ssz_bytes());
            }
            EdgeKind::DependsOn(node_id) => {
                bytes.push(7u8);
                bytes.extend_from_slice(&node_id.as_ssz_bytes());
            }
            EdgeKind::Consumes(resource_ref) => {
                bytes.push(8u8);
                bytes.extend_from_slice(&resource_ref.as_ssz_bytes());
            }
            EdgeKind::Produces(resource_ref) => {
                bytes.push(9u8);
                bytes.extend_from_slice(&resource_ref.as_ssz_bytes());
            }
            EdgeKind::Applies(handler_id) => {
                bytes.push(10u8);
                bytes.extend_from_slice(&handler_id.as_ssz_bytes());
            }
            EdgeKind::ScopedBy(handler_id) => {
                bytes.push(11u8);
                bytes.extend_from_slice(&handler_id.as_ssz_bytes());
            }
            EdgeKind::Override(handler_id) => {
                bytes.push(12u8);
                bytes.extend_from_slice(&handler_id.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for EdgeKind {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::new("EdgeKind: Input bytes too short for discriminant"));
        }
        let discriminant = bytes[0];
        match discriminant {
            0 => Ok(EdgeKind::Dependency),
            1 => Ok(EdgeKind::Temporal),
            2 => Ok(EdgeKind::ResourceFlow),
            3 => Ok(EdgeKind::Constraint),
            4 => {
                let s = Str::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::Custom(s))
            }
            5 => Ok(EdgeKind::ControlFlow),
            6 => {
                let node_id = NodeId::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::Next(node_id))
            }
            7 => {
                let node_id = NodeId::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::DependsOn(node_id))
            }
            8 => {
                let resource_ref = ResourceRef::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::Consumes(resource_ref))
            }
            9 => {
                let resource_ref = ResourceRef::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::Produces(resource_ref))
            }
            10 => {
                let handler_id = HandlerId::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::Applies(handler_id))
            }
            11 => {
                let handler_id = HandlerId::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::ScopedBy(handler_id))
            }
            12 => {
                let handler_id = HandlerId::from_ssz_bytes(&bytes[1..])?;
                Ok(EdgeKind::Override(handler_id))
            }
            _ => Err(DecodeError::new(&format!("EdgeKind: Unknown discriminant: {}", discriminant))),
        }
    }
}

impl DecodeWithLength for EdgeKind {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::new("EdgeKind (with length): Input bytes too short for discriminant"));
        }
        let discriminant = bytes[0];
        match discriminant {
            0 => Ok((EdgeKind::Dependency, 1)),
            1 => Ok((EdgeKind::Temporal, 1)),
            2 => Ok((EdgeKind::ResourceFlow, 1)),
            3 => Ok((EdgeKind::Constraint, 1)),
            4 => {
                let s = Str::from_ssz_bytes(&bytes[1..])?;
                let s_len = s.as_ssz_bytes().len();
                Ok((EdgeKind::Custom(s), 1 + s_len))
            }
            5 => Ok((EdgeKind::ControlFlow, 1)),
            6 => {
                let node_id = NodeId::from_ssz_bytes(&bytes[1..])?;
                Ok((EdgeKind::Next(node_id), 1 + 32))
            }
            7 => {
                let node_id = NodeId::from_ssz_bytes(&bytes[1..])?;
                Ok((EdgeKind::DependsOn(node_id), 1 + 32))
            }
            8 => {
                let (resource_ref, ref_len) = ResourceRef::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((EdgeKind::Consumes(resource_ref), 1 + ref_len))
            }
            9 => {
                let (resource_ref, ref_len) = ResourceRef::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((EdgeKind::Produces(resource_ref), 1 + ref_len))
            }
            10 => {
                let handler_id = HandlerId::from_ssz_bytes(&bytes[1..])?;
                Ok((EdgeKind::Applies(handler_id), 1 + 32))
            }
            11 => {
                let handler_id = HandlerId::from_ssz_bytes(&bytes[1..])?;
                Ok((EdgeKind::ScopedBy(handler_id), 1 + 32))
            }
            12 => {
                let handler_id = HandlerId::from_ssz_bytes(&bytes[1..])?;
                Ok((EdgeKind::Override(handler_id), 1 + 32))
            }
            _ => Err(DecodeError::new(&format!("EdgeKind (with length): Unknown discriminant: {}", discriminant))),
        }
    }
}

impl SimpleSerialize for EdgeKind {}

impl Default for EdgeKind {
    fn default() -> Self {
        EdgeKind::Custom(Str::from("default"))
    }
}

/// An edge in the effect graph
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Edge {
    /// Unique identifier for this edge
    pub id: EdgeId,
    /// Source node of the edge
    pub source: NodeId,
    /// Target node of the edge
    pub target: NodeId,
    /// Type of edge
    pub kind: EdgeKind,
    /// Optional metadata for the edge
    pub metadata: BTreeMap<Str, ValueExpr>,
}

impl Encode for Edge {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.0); // EdgeId is [u8; 32]
        bytes.extend_from_slice(&self.source.0); // NodeId is [u8; 32]
        bytes.extend_from_slice(&self.target.0); // NodeId is [u8; 32]
        bytes.extend_from_slice(&self.kind.as_ssz_bytes());
        bytes.extend_from_slice(&self.metadata.as_ssz_bytes());
        bytes
    }
}

impl Decode for Edge {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (edge, _consumed) = Self::from_ssz_bytes_with_length(bytes)?;
        // Potentially check if all bytes were consumed if strict parsing is needed,
        // but from_ssz_bytes_with_length handles partial reads correctly for its components.
        Ok(edge)
    }
}

impl DecodeWithLength for Edge {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let mut offset = 0;

        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Edge: Input bytes too short for id"));
        }
        let id = EdgeId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;

        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Edge: Input bytes too short for source"));
        }
        let source = NodeId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;

        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Edge: Input bytes too short for target"));
        }
        let target = NodeId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;

        let (kind, kind_consumed) = EdgeKind::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += kind_consumed;

        // For now, create an empty metadata map to avoid complex serialization issues
        let metadata = BTreeMap::new();

        Ok((
            Edge {
                id,
                source,
                target,
                kind,
                metadata,
            },
            offset,
        ))
    }
}

impl SimpleSerialize for Edge {}

impl Edge {
    /// Create a new edge
    pub fn new(id: EdgeId, source: NodeId, target: NodeId, kind: EdgeKind) -> Self {
        Self {
            id,
            source,
            target,
            kind,
            metadata: BTreeMap::new(),
        }
    }

    /// Add metadata to the edge
    pub fn with_metadata(mut self, key: Str, value: ValueExpr) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl AsEdge for Edge {
    fn to_edge_id(&self) -> EdgeId {
        self.id
    }

    fn from_edge_id(id: EdgeId) -> Option<Self> {
        Some(Self {
            id,
            source: NodeId::null(),
            target: NodeId::null(),
            kind: EdgeKind::Custom(Str::from("unknown")),
            metadata: BTreeMap::new(),
        })
    }

    fn source(&self) -> NodeId {
        self.source
    }

    fn target(&self) -> NodeId {
        self.target
    }
}

/// The main effect graph structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectGraph {
    /// Unique identifier for this graph
    pub id: EntityId,
    /// Domain this graph belongs to
    pub domain_id: DomainId,
    /// All nodes in the graph (effects, intents, resources)
    pub nodes: BTreeMap<NodeId, ValueExpr>,
    /// All edges in the graph
    pub edges: BTreeMap<EdgeId, Edge>,
    /// Metadata for the graph
    pub metadata: BTreeMap<Str, ValueExpr>,
}

impl EffectGraph {
    /// Create a new empty effect graph
    pub fn new(id: EntityId, domain_id: DomainId) -> Self {
        Self {
            id,
            domain_id,
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node_id: NodeId, node_data: ValueExpr) {
        self.nodes.insert(node_id, node_data);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge.id, edge);
    }

    /// Get a node from the graph
    pub fn get_node(&self, node_id: &NodeId) -> Option<&ValueExpr> {
        self.nodes.get(node_id)
    }

    /// Get an edge from the graph
    pub fn get_edge(&self, edge_id: &EdgeId) -> Option<&Edge> {
        self.edges.get(edge_id)
    }

    /// Add metadata to the graph
    pub fn add_metadata(&mut self, key: Str, value: ValueExpr) {
        self.metadata.insert(key, value);
    }
}

//-----------------------------------------------------------------------------
// TEL Integration Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tel_integration_tests {
    use super::*;

    #[test]
    fn test_resource_ref_creation() {
        let resource_id = ResourceId::new([1u8; 32]);
        let domain_id = DomainId::new([2u8; 32]);
        
        let resource_ref = ResourceRef::new(resource_id, domain_id);
        assert_eq!(resource_ref.resource_id, resource_id);
        assert_eq!(resource_ref.domain_id, domain_id);
        assert_eq!(resource_ref.resource_type, None);
        
        let typed_ref = ResourceRef::with_type(resource_id, domain_id, Str::from("token"));
        assert_eq!(typed_ref.resource_type, Some(Str::from("token")));
    }

    #[test]
    fn test_edge_creation() {
        let edge_id = EdgeId::new([1u8; 32]);
        let source = NodeId::new([2u8; 32]);
        let target = NodeId::new([3u8; 32]);
        
        let edge = Edge::new(edge_id, source, target, EdgeKind::Dependency);
        assert_eq!(edge.id, edge_id);
        assert_eq!(edge.source, source);
        assert_eq!(edge.target, target);
        assert_eq!(edge.kind, EdgeKind::Dependency);
    }

    #[test]
    fn test_effect_graph_operations() {
        let graph_id = EntityId::new([1u8; 32]);
        let domain_id = DomainId::new([2u8; 32]);
        
        let mut graph = EffectGraph::new(graph_id, domain_id);
        
        let node_id = NodeId::new([3u8; 32]);
        let node_data = ValueExpr::String(Str::from("test_node"));
        
        graph.add_node(node_id, node_data.clone());
        assert_eq!(graph.get_node(&node_id), Some(&node_data));
        
        let edge = Edge::new(
            EdgeId::new([4u8; 32]),
            node_id,
            NodeId::new([5u8; 32]),
            EdgeKind::ResourceFlow
        );
        let edge_id = edge.id;
        graph.add_edge(edge);
        
        assert!(graph.get_edge(&edge_id).is_some());
    }
} 