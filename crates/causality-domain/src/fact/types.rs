// Fact type definitions for domains
// Original file: src/domain/fact/types.rs

// Fact Types Module for Causality
//
// This module defines the types used for facts in Causality.

// Re-export FactQuery
pub use causality_domain::FactQuery;

// Re-export the FactType from the log module
pub use causality_engine_types::{FactType, RegisterFact, ZKProofFact};
