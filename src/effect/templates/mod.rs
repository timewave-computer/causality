// Effect Templates Module
//
// This module provides various effect templates for common operations.

// Define the relationship validation module
pub mod relationship_validation;

// Define the state transition helper module
pub mod state_transition;

// Re-export the effect validation trait
pub use relationship_validation::RelationshipStateValidationEffect;
pub use state_transition::ResourceStateTransitionHelper; 