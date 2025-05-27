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

use crate::primitive::ids::{EntityId, DomainId, ExprId, ResourceId, NodeId, EdgeId, AsId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::expression::value::ValueExpr;
use crate::graph::r#trait::AsEdge;
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
}

/// An edge in the effect graph
#[derive(Debug, Clone, PartialEq, Eq)]
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