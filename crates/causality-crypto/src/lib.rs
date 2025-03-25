// Cryptographic primitives and utilities
// Original file: src/crypto/mod.rs

// Crypto module for timewave
//
// This module provides cryptographic primitives used throughout the system, 
// including hashing, signatures, merkle trees, and zero-knowledge proofs.

pub mod hash;
pub mod merkle;
pub mod signature;
pub mod smt;
pub mod zk;
pub mod deferred;
pub mod content_addressed_storage;

pub use hash::{HashFunction, HashOutput, Hasher, HashFactory, HashAlgorithm, HashError};
pub use merkle::{Commitment, CommitmentScheme, CommitmentFactory, CommitmentType, CommitmentError, MerkleTreeCommitmentScheme, MerkleProof, H256};
pub use signature::{Signature, SignatureScheme, SignatureError, SignatureVerificationResult, SignatureFactory};
pub use zk::{ZkProof, ZkVerifier, ZkProver, ZkError, ZkFactory, VerificationCircuit, GenericCircuit};
pub use smt::{SmtKeyValue, SmtFactory, MerkleSmt, SmtError, SmtProof, ContentAddressedSmt};
pub use hash::{ChecksumOutput, ChecksumFunction, Checksum, ChecksumFactory, ChecksumAlgorithm, Md5ChecksumFunction}; 
pub use hash::{ContentAddressed, ContentId, DeferredHashing, DeferredHashId};
pub use deferred::{DeferredHashingContext, DeferredHashBatchProcessor, DeferredHashInput}; 
pub use content_addressed_storage::{ContentAddressedStorage, StorageError, InMemoryStorage, StorageFactory, StorageType}; 