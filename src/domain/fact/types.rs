// Fact Types Module for Causality
//
// This module defines the types used for facts in Causality.

// Re-export FactQuery
pub use crate::domain::adapter::FactQuery;

// Re-export the FactType from the log module
pub use crate::log::fact_types::{FactType, RegisterFact, ZKProofFact};
