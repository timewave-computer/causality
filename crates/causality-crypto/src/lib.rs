// Cryptographic primitives and utilities
// Original file: src/crypto/mod.rs

// Crypto module for timewave
//
// This module provides cryptographic primitives used throughout the system, 
// including hashing, signatures, merkle trees, and zero-knowledge proofs.

// Export our modules
pub mod hash;
pub mod merkle;
pub mod signature;
pub mod zk;
pub mod deferred;
// Note: The following modules are missing and have been removed:
// pub mod smt;
// pub mod content_addressed_storage;

// Import and re-export types from causality-types
// Only import what actually exists in causality-types
pub use causality_types::{
    HashOutput, HashAlgorithm, HashError, ContentId, ContentAddressed, ContentHash
};

// Re-export our own types from hash.rs
pub use hash::{HashFunction, Hasher, HashFactory};

// Re-export our own types from merkle.rs
pub use merkle::{Commitment, CommitmentScheme, CommitmentFactory, CommitmentType, CommitmentError, MerkleTreeCommitmentScheme, MerkleProof, H256};

// Re-export our own types from signature.rs
pub use signature::{Signature, SignatureScheme, SignatureError, SignatureVerificationResult, SignatureFactory};

// Re-export our own types from zk.rs
pub use zk::{ZkProof, ZkVerifier, ZkProver, ZkError, ZkFactory, VerificationCircuit, GenericCircuit};

// Re-export our own types from deferred.rs
pub use deferred::{DeferredHashingContext, DeferredHashBatchProcessor, DeferredHashInput, DeferredHashId, DeferredHashing}; 