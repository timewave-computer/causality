// Crypto module for timewave
//
// This module provides cryptographic primitives used throughout the system, 
// including hashing, signatures, merkle trees, and zero-knowledge proofs.

pub mod hash;
pub mod merkle;
pub mod signature;
pub mod smt;
pub mod zk;

pub use hash::{HashFunction, HashOutput, Hasher, HashFactory, HashAlgorithm, HashError};
pub use merkle::{Commitment, CommitmentScheme, CommitmentFactory, CommitmentType, CommitmentError, MerkleTreeCommitmentScheme, MerkleProof, H256};
pub use signature::{Signature, SignatureScheme, SignatureError, SignatureVerificationResult, SignatureFactory};
pub use zk::{ZkProof, ZkVerifier, ZkProver, ZkError, ZkFactory};
pub use smt::{SmtKeyValue, SmtFactory, MerkleSmt, SmtError};
pub use hash::{ChecksumOutput, ChecksumFunction, Checksum, ChecksumFactory, ChecksumAlgorithm, Md5ChecksumFunction}; 