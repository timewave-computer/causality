//! Traits for graph nodes and edges.
//!
//! This module defines the core traits required for graph operations,
//! including node and edge type conversions and type lists.
//! Also includes TEL-specific traits.

use std::any::TypeId;
use std::fmt::Debug;

use crate::primitive::ids::{EdgeId, NodeId, DomainId};

// Type-level heterogeneous list implementation for type lists
pub struct HNil;
pub struct HCons<H, T> {
    pub head: H,
    pub tail: T,
}

/// Converts domain types to graph nodes.
pub trait AsNode: Sized + 'static {
    /// Converts this domain type to a node ID.
    fn to_node_id(&self) -> NodeId;

    /// Attempts to convert a node ID back to this domain type.
    fn from_node_id(id: NodeId) -> Option<Self>;

    /// Returns the type ID for this node type.
    fn node_type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Converts domain relationships to graph edges.
pub trait AsEdge: Sized + 'static {
    /// Converts this domain relationship to an edge ID.
    fn to_edge_id(&self) -> EdgeId;

    /// Attempts to convert an edge ID back to this domain relationship.
    fn from_edge_id(id: EdgeId) -> Option<Self>;

    /// Returns the source node of this edge.
    fn source(&self) -> NodeId;

    /// Returns the target node of this edge.
    fn target(&self) -> NodeId;

    /// Returns the type ID for this edge type.
    fn edge_type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Marker trait for a list of node types (e.g., a tuple `(NodeTypeA, NodeTypeB)`).
/// Used for type-safe graph configurations.
pub trait AsNodeTypesList {}

/// Marker trait for a list of edge types.
/// Marker trait for HLists that can be used as node type lists
/// Implementations are provided in registry.rs
pub trait AsEdgeTypesList {}

// Note: Implementations for AsNodeTypesList and AsEdgeTypesList 
// have been moved to registry.rs to avoid duplicate implementations

/// Type-level function to check if a node type is in an HList.
pub trait AsContainsNodeType<Needle: AsNode>: AsNodeTypesList {
    fn is_present() -> bool;
}

/// Type-level function to check if an edge type is in an HList.
pub trait AsContainsEdgeType<Needle: AsEdge>: AsEdgeTypesList {
    fn is_present() -> bool;
}

/// Simple error type for graph operations
#[derive(Debug, Clone)]
pub struct GraphError {
    /// Error message
    pub message: String,
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GraphError {}

/// Result type for graph operations
pub type GraphResult<T> = Result<T, GraphError>;

//-----------------------------------------------------------------------------
// Test
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::{NodeId, AsId};

    // Define a test node type
    #[derive(Debug, Clone, PartialEq)]
    struct TestNode {
        id: u64,
    }

    impl AsNode for TestNode {
        fn to_node_id(&self) -> NodeId {
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&self.id.to_be_bytes());
            NodeId::new(bytes)
        }

        fn from_node_id(_id: NodeId) -> Option<Self> {
            Some(TestNode { id: 1 })
        }
    }

    #[test]
    fn test_as_node() {
        let node = TestNode { id: 42 };
        let node_id = node.to_node_id();
        let node2 = TestNode::from_node_id(node_id).unwrap();

        assert_eq!(node2.id, 1); // In our mock implementation, always returns 1
    }

    // Define type lists
    type EmptyList = HNil;
    type NodeList = HCons<TestNode, EmptyList>;
    type NodeListNonHead = HCons<AnotherTestNode, HCons<TestNode, HNil>>;

    // Define another test node type for recursive checks
    #[derive(Debug, Clone, PartialEq)]
    struct AnotherTestNode {
        id: u32,
    }
    impl AsNode for AnotherTestNode {
        fn to_node_id(&self) -> NodeId {
            NodeId::null()
        } // Dummy
        fn from_node_id(_id: NodeId) -> Option<Self> {
            Some(Self { id: 0 })
        } // Dummy
    }

    #[derive(Debug, Clone, PartialEq)]
    struct UnrelatedNode {
        id: u32,
    }
    impl AsNode for UnrelatedNode {
        fn to_node_id(&self) -> NodeId {
            NodeId::null()
        }
        fn from_node_id(_id: NodeId) -> Option<Self> {
            Some(Self { id: 0 })
        }
    }

    #[test]
    fn test_contains_node_type() {
        assert!(!<EmptyList as AsContainsNodeType<TestNode>>::is_present());
        assert!(<NodeList as AsContainsNodeType<TestNode>>::is_present());
        assert!(<NodeListNonHead as AsContainsNodeType<TestNode>>::is_present());
        assert!(
            <NodeListNonHead as AsContainsNodeType<AnotherTestNode>>::is_present()
        );
        assert!(
            !<NodeListNonHead as AsContainsNodeType<UnrelatedNode>>::is_present()
        );
    }
}

//-----------------------------------------------------------------------------
// TEL Traits (from tel/traits.rs)
//-----------------------------------------------------------------------------

/// Trait for TEL components that have an associated DomainId.
pub trait HasDomainId {
    fn domain_id(&self) -> DomainId;
}
