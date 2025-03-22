// Fact Module for Causality
//
// This module defines the types and functionality for
// working with facts in the Causality system.

// Sub-modules
mod types;
mod verification;
mod verifiers;
mod observer;
mod bridge;   // Compatibility layer for legacy code
mod register_observer;
mod zkproof_observer;

// Re-exports
pub use verification::{VerificationResult};
pub use types::{FactType, RegisterFact, ZKProofFact};
pub use verifiers::{
    MerkleProofVerifier, SignatureVerifier, ConsensusVerifier,
    VerifierRegistry
};

// Legacy compatibility exports - marked for deprecation in future versions
// @deprecated - Use new FactType system instead
pub use bridge::{ObservedFact, VerifiedFact, FactProof, ProofType};
