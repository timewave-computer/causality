// Temporal Effect Graph (TEG) - The intermediate representation for Causality
// This crate implements a graph-based intermediate representation that forms
// a categorical adjunction between TEL combinators and algebraic effects.

pub mod theory;
pub mod graph;
pub mod effect_node;
pub mod resource_node;
pub mod builder;
pub mod validation;
pub mod serialization;
pub mod fragment;
pub mod tel;
pub mod optimization;
pub mod code_generation;

// This module contains structures and algorithms for the Temporal Effect Graph (TEG)
// which serves as an intermediate representation for temporal effects.

mod traversal;
mod transformation;

use causality_types::ContentHash;

// Re-export main types for convenient access
pub use effect_node::EffectNode;
pub use resource_node::ResourceNode;
pub use graph::TemporalEffectGraph;
pub use fragment::TEGFragment;
pub use tel::to_teg::ToTEGFragment;
pub use tel::from_teg::ToTELCombinator;
pub use traversal::*;
pub use transformation::*;

// Core identifiers
pub type EffectId = String;
pub type ResourceId = String;
pub type CapabilityId = String;
pub type DomainId = String;
pub type FactId = String;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
