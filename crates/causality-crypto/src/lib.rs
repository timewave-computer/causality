// Cryptographic utilities for Causality
//
// This crate provides cryptographic primitives and utilities for the Causality system.
// It includes implementations and abstractions for hashing, signatures, and various
// proof systems used throughout the Causality ecosystem.

//! Cryptographic utilities for Causality
//!
//! This crate provides cryptographic primitives and utilities for the Causality system.
//! It includes:
//! - Hashing functionality with various algorithms
//! - Digital signature generation and verification
//! - Proof systems and verification
//! - Merkle trees and other cryptographic data structures
//! - Zero-knowledge proof utilities

// Core crypto modules
pub mod hash;       // Core hashing functionality 
pub mod deferred;   // Deferred/batched hashing for optimization
pub mod signatures; // Digital signatures
pub mod merkle;     // Merkle trees and related structures
pub mod nullifier;  // Nullifier generation for anonymity
pub mod zk;         // Zero-knowledge proofs

// Proof system implementations
pub mod proof;      // Complex proof structures and data types
pub mod proofs;     // Proof generation and verification abstractions
pub mod extensions; // Extensions to crypto functionality
pub mod utils;      // Utility functions for cryptography
pub mod signature;  // Signature implementation
pub mod traits;     // Common trait definitions

// Re-exports for convenient access to core types
pub use causality_types::crypto_primitives::{
    HashAlgorithm, 
    HashOutput, 
    HashError, 
    ContentHash, 
    ContentId
};

// Trait and type re-exports 
pub use hash::{
    HashFunction, 
    ContentHasher, 
    HashFactory,
    ChecksumOutput
};
pub use signatures::{
    Signature, 
    Signer, 
    Verifier
}; 
pub use proof::UnifiedProof;
pub use proofs::{
    Proof, 
    Prover, 
    ProofVerifier,
    ProofWithInput
};

#[cfg(test)]
mod tests {
    // Basic test to verify the module structure
    #[test]
    fn test_crypto_module() {
        // Simple sanity check
        assert!(true);
    }
} 