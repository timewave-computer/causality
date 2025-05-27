//! Type-safe registries for graph nodes and edges with standardized trait implementations.
//!
//! This module provides registries for nodes and edges that ensure type safety
//! while conforming to the common registry interfaces used throughout the codebase.

// Renamed to avoid conflict if HashSet is defined locally
// use std::sync::Arc; // Commented out as unused

// Graph-specific IDs, traits, and error types
use crate::graph::r#trait::{
    AsContainsEdgeType, AsContainsNodeType, AsEdgeTypesList, AsNodeTypesList,
    AsEdge, AsNode,
};
use frunk::{HCons, HNil};
use thiserror::Error; // <-- ADD THIS LINE // Import AsNode and AsEdge traits
                                           // use crate::primitive::ids::{EdgeId as TestEdgeId, NodeId as TestNodeId}; // Use aliases for clarity in tests - Commented out as unused
                                           // use crate::graph::traits::{ // Commented out as unused
                                           //     AsContainsEdgeType as TestAsContainsEdgeType, // Commented out as unused
                                           //     AsContainsNodeType as TestAsContainsNodeType, AsEdge as TestAsEdge, // Commented out as unused
                                           //     AsEdgeTypesList as TestAsEdgeTypesList, AsNode as TestAsNode, // Commented out as unused
                                           //     AsNodeTypesList as TestAsNodeTypesList, // Commented out as unused
                                           // }; // Commented out as unused
                                           // use frunk::{HCons, HNil}; // frunk is used in tests - Commented out as unused
                                           // use std::marker::PhantomData as TestPhantomData; // Use alias - Commented out as unused
                                           // ssz has been replaced with SSZ

// Define GraphResult as a type alias
// type GraphResult<T> = Result<T, GraphError>; // Commented out as unused

// Use AsRegistry from the utils module

// Core types (currently unused directly at top level of this file after other changes)
// use crate::primitive::ids::ResourceId;
// use crate::core::logger::AsLogger;
// use crate::expr::expr_type::TypeExpr;
// use crate::expr::value::ValueExpr;
// use crate::resource::Resource;

// Commented out as unused or problematic:
// use crate::anyhow::{anyhow, Result}; // anyhow is unused
// use crate::primitive::ids::{SchemaId, ValueId}; // SchemaId, ValueId are unresolved
// use crate::provider::context::AsContextServices; // AsContextServices is unresolved
// use crate::provider::store::AsKeyValueStore; // AsKeyValueStore is unused
// use crate::store::StoreError; // StoreError is unresolved, using GraphError instead for Result types
// use frunk::{HCons, HNil}; // HCons and HNil are unused at top level

//-----------------------------------------------------------------------------
// HList Type Definitions (These remain in causality-types)
//-----------------------------------------------------------------------------

/// Base case for HList of node types
impl AsNodeTypesList for HNil {}

/// Recursive case for HList of node types
impl<Head, Tail> AsNodeTypesList for HCons<Head, Tail>
where
    Head: AsNode + Send + Sync + 'static,
    Tail: AsNodeTypesList,
{
}

/// Trait implementation for checking if a type is in an HList of node types (base case)
impl<NodeType> AsContainsNodeType<NodeType> for HNil
where
    NodeType: AsNode + Send + Sync + 'static,
{
    fn is_present() -> bool {
        false
    }
}

/// Trait implementation for checking if a type is in an HList of node types (recursive case)
impl<NodeType, Head, Tail> AsContainsNodeType<NodeType> for HCons<Head, Tail>
where
    NodeType: AsNode + Send + Sync + 'static,
    Head: AsNode + Send + Sync + 'static,
    Tail: AsNodeTypesList + AsContainsNodeType<NodeType>,
{
    fn is_present() -> bool {
        std::any::TypeId::of::<NodeType>() == std::any::TypeId::of::<Head>()
            || Tail::is_present()
    }
}

/// Base case for HList of edge types
impl AsEdgeTypesList for HNil {}

/// Recursive case for HList of edge types
impl<Head, Tail> AsEdgeTypesList for HCons<Head, Tail>
where
    Head: AsEdge + Send + Sync + 'static,
    Tail: AsEdgeTypesList,
{
}

/// Trait implementation for checking if a type is in an HList of edge types (base case)
impl<EdgeType> AsContainsEdgeType<EdgeType> for HNil
where
    EdgeType: AsEdge + Send + Sync + 'static,
{
    fn is_present() -> bool {
        false
    }
}

/// Trait implementation for checking if a type is in an HList of edge types (recursive case)
impl<EdgeType, Head, Tail> AsContainsEdgeType<EdgeType> for HCons<Head, Tail>
where
    EdgeType: AsEdge + Send + Sync + 'static,
    Head: AsEdge + Send + Sync + 'static,
    Tail: AsEdgeTypesList + AsContainsEdgeType<EdgeType>,
{
    fn is_present() -> bool {
        std::any::TypeId::of::<EdgeType>() == std::any::TypeId::of::<Head>()
            || Tail::is_present()
    }
}

//-----------------------------------------------------------------------------
// Test module (remains in causality-types)
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::{EdgeId, NodeId}; // Ensure correct Id types are used
    // use frunk::{HCons, HNil}; // Ensure HList types are available for tests

    // Test Node
    #[derive(Debug, Clone, PartialEq, Eq, Hash)] // Added Eq, Hash for potential use in HashMaps if NodeId derived from it
    struct TestNode {
        id: u64,
        data: String,
    }

    impl AsNode for TestNode {
        fn to_node_id(&self) -> NodeId {
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&self.id.to_le_bytes()); // Use to_le_bytes for consistency
            NodeId::new(bytes) // Use NodeId::new instead of content_addressed
        }

        fn from_node_id(id: NodeId) -> Option<Self> {
            // This is a mock. In a real scenario, you might parse the id or look up.
            // For simplicity, let's assume the first 8 bytes are the u64 id.
            let bytes = id.0; // Access the inner byte array using .0
            let mut u64_bytes = [0u8; 8];
            u64_bytes.copy_from_slice(&bytes[0..8]);
            Some(TestNode {
                id: u64::from_le_bytes(u64_bytes),
                data: "TestNodeData".to_string(),
            })
        }
    }

    // Test Edge
    #[derive(Debug, Clone, PartialEq, Eq, Hash)] // Added Eq, Hash
    struct TestEdge {
        id: u64,
        source_node_val: u64, // Store value to create NodeId
        target_node_val: u64, // Store value to create NodeId
        label: String,
    }

    impl AsEdge for TestEdge {
        fn to_edge_id(&self) -> EdgeId {
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&self.id.to_le_bytes());
            EdgeId::new(bytes) // Use EdgeId::new instead of content_addressed
        }

        fn from_edge_id(id: EdgeId) -> Option<Self> {
            let bytes = id.0; // Access the inner byte array using .0
            let mut u64_bytes = [0u8; 8];
            u64_bytes.copy_from_slice(&bytes[0..8]);
            Some(TestEdge {
                id: u64::from_le_bytes(u64_bytes),
                source_node_val: 0, // Placeholder, real logic would parse from ID or context
                target_node_val: 0, // Placeholder
                label: "TestEdgeLabel".to_string(),
            })
        }

        fn source(&self) -> NodeId {
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&self.source_node_val.to_le_bytes());
            NodeId::new(bytes) // Use NodeId::new instead of content_addressed
        }

        fn target(&self) -> NodeId {
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&self.target_node_val.to_le_bytes());
            NodeId::new(bytes) // Use NodeId::new instead of content_addressed
        }
    }

    // Test commented out as part of the refactoring to separate type definitions from implementations
    // #[test]
    // fn test_edge_registry_operations() {
    //     let mut registry = EdgeRegistry::<TestEdgeTypes>::new();
    //     registry.register_type::<TestEdge>().unwrap();
    //
    //     let edge1 = TestEdge {
    //         id: 1,
    //         source_node_val: 10,
    //         target_node_val: 20,
    //         label: "Edge1".to_string(),
    //     };
    //     let edge1_id = registry.register_edge(&edge1).unwrap();
    //     assert_eq!(edge1_id, edge1.to_edge_id());
    //
    //     let retrieved_edge: TestEdge = registry.get_edge(edge1_id).unwrap();
    //     assert_eq!(retrieved_edge, edge1);
    //
    //     let retrieved_edge_mut: &mut TestEdge =
    //         registry.get_edge_mut(edge1_id).unwrap();
    //     retrieved_edge_mut.label = "Modified Edge1".to_string();
    //
    //     let retrieved_edge_after_mut: TestEdge =
    //         registry.get_edge(edge1_id).unwrap();
    //     assert_eq!(retrieved_edge_after_mut.label, "Modified Edge1");
    // }
    //     let source_node_id = edge1.source();
    //     let target_node_id = edge1.target();
    //
    //     let outgoing = registry.get_outgoing_edges(source_node_id);
    //     assert!(outgoing.contains(&edge1_id));
    //
    //     let incoming = registry.get_incoming_edges(target_node_id);
    //     assert!(incoming.contains(&edge1_id));
    //
    //     let between = registry.get_edges_between(source_node_id, target_node_id);
    //     assert!(between.contains(&edge1_id));
    // }

    // Example commented out as part of refactoring to separate type definitions from implementations
    // // Example of using the AsRegistry trait with the specific registry types
    // // This function is not a test itself but demonstrates usage.
    // // It's kept for illustrative purposes if originally intended as such.
    // // If it was a test, it would need #[test] and assertions.
    // #[allow(dead_code)] // Mark as allowed to be dead if not directly called by a test
    // async fn _example_generic_registry_user(
    //     mut node_reg: impl AsRegistry<NodeId, TestNode>,
    //     mut edge_reg: impl AsRegistry<EdgeId, TestEdge>,
    // ) {
    //     let source_node = TestNode {
    //         id: 10,
    //         data: "Source".to_string(),
    //     };
    //     let target_node = TestNode {
    //         id: 20,
    //         data: "Target".to_string(),
    //     };
    //     let source_node_id = source_node.to_node_id();
    //     let target_node_id = target_node.to_node_id();
    //
    //     node_reg.insert(source_node_id, source_node).unwrap();
    //     node_reg.insert(target_node_id, target_node).unwrap();
    //
    //     let edge = TestEdge {
    //         id: 100,
    //         source_node_val: 10,
    //         target_node_val: 20,
    //         label: "Connects".to_string(),
    //     };
    //     let edge_id = edge.to_edge_id();
    //     edge_reg.insert(edge_id, edge).unwrap();
    //
    //     // Example assertions (if this were a test)
    //     // assert!(node_reg.contains(&source_node_id));
    //     // assert!(edge_reg.contains(&edge_id));
    // }
}

/// Defines errors that can occur during graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    /// Indicates that a type being registered already exists in the registry.
    #[error("Type {type_name:?} already registered.")]
    TypeAlreadyRegistered { type_name: String },

    /// Indicates that a type being acted upon has not been registered.
    #[error("Type {type_name:?} not registered.")]
    TypeNotRegistered { type_name: String },

    /// Indicates an attempt to insert an element that already exists.
    #[error("Element with ID {id:?} already exists.")]
    ElementAlreadyExists { id: String }, // Assuming ID is stringified for error

    /// Indicates that an element being accessed does not exist.
    #[error("Element with ID {id:?} not found.")]
    ElementNotFound { id: String },

    /// Indicates an attempt to downcast to an incorrect type.
    #[error("Failed to downcast to type {type_name:?}.")]
    DowncastError { type_name: String },

    /// Propagates errors from serialization or deserialization processes.
    #[error("Serialization/Deserialization error: {0}")]
    SerializationError(String),

    /// Placeholder for other, more specific graph errors.
    #[error("An unspecified graph operation error occurred: {0}")]
    Other(String),
}
