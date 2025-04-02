// Fact system for domains
// Original file: src/domain/fact.rs

// Fact Module for Causality
//
// This module defines the types and functionality for
// working with facts in the Causality system.

// Sub-modules
pub mod observer;
pub mod types;
pub mod verification;
pub mod verifiers;
// zkproof module for verification of zero-knowledge proofs
pub mod zkproof;

// Re-exports
pub use verification::{FactVerifier, VerificationResult as FactVerification};
pub use observer::FactObserver;
pub use types::FactQuery;
