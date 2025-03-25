// Effect constraints system
// Original file: src/effect/constraints/mod.rs

//! Effect constraint system
//!
//! This module defines constraints that can be applied to effects to verify their behavior.
//! Constraints represent behavioral contracts that effects must adhere to.

mod validation;

pub use validation::{
    ConstraintVerifier,
    ConstraintVerificationResult,
    ConstraintCondition,
    EffectConstraint,
    create_constraint,
};

// Re-export the constraint traits
pub use crate::effect::constraints::{
    TransferEffect,
    QueryEffect,
    StorageEffect,
};

// Re-export constraint utility functions
pub use crate::effect::constraints::check_transfer_effect;

// Re-export key validation components
pub use validation::{EffectValidator, EffectOrchestrator}; 