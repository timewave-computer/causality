// Fact Module for Causality
//
// This module defines the types and functionality for
// working with facts in the Causality system.

// Sub-modules
mod types;
mod verification;
mod verifiers;
mod observer;
mod zkproof_observer;

// Re-exports
pub use verification::{VerificationResult};
pub use types::{FactType, RegisterFact, ZKProofFact};
pub use verifiers::{
    MerkleProofVerifier, SignatureVerifier, ConsensusVerifier,
    VerifierRegistry
};
