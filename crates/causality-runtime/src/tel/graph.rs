//! TEL Graph Structure
//!
//! Defines the specialized graph structure for the Temporal Effect Language (TEL).

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use crate::tel::types::{TelEdgeTypes, TelNodeTypes};
use causality_core::{NodeRegistry, EdgeRegistry};

// Import the EffectGraph type for conversion purposes

/// Represents the graph structure for the Temporal Effect Language.
/// It uses generic NodeRegistry and EdgeRegistry from causality-core
/// specialized with TEL-specific node and edge type lists.
#[derive(Debug)]
pub struct TelGraph {
    pub nodes: NodeRegistry<TelNodeTypes>,
    pub edges: EdgeRegistry<TelEdgeTypes>,
    // TODO: Add other graph-level metadata if needed, e.g., GraphId.
}

impl TelGraph {
    /// Creates a new, empty TEL graph.
    pub fn new() -> Self {
        Self {
            nodes: NodeRegistry::new(),
            edges: EdgeRegistry::new(),
        }
    }

    // TODO: Add methods for graph manipulation specific to TEL,
    // e.g., adding effects, resources, edges, and querying them.
    // These will wrap the underlying registry methods but provide
    // TEL-specific type safety or convenience.
}

impl Default for TelGraph {
    fn default() -> Self {
        Self::new()
    }
}
