// Purpose: Defines HList-based type lists for TEL graph nodes and edges.

// use crate::primitive::ids::{EdgeId, NodeId}; // Removed as unused
// Removed unused graph traits based on compiler warnings
// use crate::graph::traits::{AsNodeTypesList, AsEdgeTypesList, AsContainsNodeType, AsContainsEdgeType};
// Removed: use crate::resource::base::Resource; // No longer needed due to FQN
// Removed: use crate::tel::{Effect, Intent}; // No longer needed due to FQN
use crate::tel::graph::Edge as TelEdge; // Specific import for tel::Edge
// use frunk::{HCons, HNil}; // Ensure HCons and HNil are imported from frunk directly
use crate::graph::traits::{HCons, HNil}; // Use HCons and HNil from local graph::traits

// --- Node Types for TEL ---
// Defines the HList of allowed node types in a TelGraph.
// Using fully qualified paths to be absolutely sure.
pub type TelNodeTypes = HCons<crate::core::Effect, HCons<crate::core::Resource, HCons<crate::core::Intent, HNil>>>;

// // REMOVED: These specific impls conflict with generic HList impls in graph::traits.rs
// impl AsNodeTypesList for TelNodeTypes {}
// impl AsContainsNodeType<Effect> for TelNodeTypes {}
// impl AsContainsNodeType<Resource> for TelNodeTypes {}
// impl AsContainsNodeType<Intent> for TelNodeTypes {}

// --- Edge Types for TEL ---
// Defines the HList of allowed edge types in a TelGraph.
pub type TelEdgeTypes = HCons<TelEdge, HNil>; // Only one edge type for now, using local HNil

// // REMOVED: These specific impls conflict with generic HList impls in graph::traits.rs
// impl AsEdgeTypesList for TelEdgeTypes {}
// impl AsContainsEdgeType<TelEdge> for TelEdgeTypes {}
