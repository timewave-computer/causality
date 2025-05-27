//! Graph structures and TEL
//!
//! Graph elements, subgraphs, registries, traits, TEL types,
//! execution contexts, optimization, and dataflow.

//-----------------------------------------------------------------------------
// Module Exports
//-----------------------------------------------------------------------------

// Existing modules (renamed to match new structure)
pub mod element; // was elements
pub mod subgraph;
pub mod registry;
pub mod r#trait; // was traits

// New consolidated modules
pub mod tel;
pub mod execution;
pub mod optimization;
pub mod dataflow;

// Keep existing error module
pub mod error;

//-----------------------------------------------------------------------------
// Type Definitions
//-----------------------------------------------------------------------------

/// Type marker traits for graph component classification and categorization.
pub mod type_markers;

//-----------------------------------------------------------------------------
// Type Re-exports
//-----------------------------------------------------------------------------

// Re-exports from existing modules
pub use error::GraphError;

// Re-exports for convenience (new structure) - explicit to avoid ambiguity
pub use element::{Node, Edge as GraphEdge, TypeId};
pub use subgraph::Subgraph;
pub use r#trait::{AsEdge, AsNode};
pub use tel::{EffectGraph, Edge, EdgeKind, ResourceRef};
pub use execution::{ExecutionContext, ExecutionMode, ResourceRef as ExecutionResourceRef, ProcessDataflowInstanceState};
pub use optimization::{OptimizationStrategy, TypedDomain};
pub use dataflow::{ProcessDataflowDefinition, ProcessDataflowNode, ProcessDataflowEdge, DomainAwareNode, DataflowPort};
