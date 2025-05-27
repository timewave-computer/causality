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

// Re-exports for convenience (new structure)
pub use element::*;
pub use subgraph::*;
pub use registry::*;
pub use r#trait::*;
pub use tel::*;
pub use execution::*;
pub use optimization::*;
pub use dataflow::*;
