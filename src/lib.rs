// Causality - A Unified Resource Management System
//
// This library provides a unified resource management system with lifecycle management,
// relationship tracking, capability-based authorization, and effect templates.

// Core modules
pub mod types;
pub mod error;
pub mod address;
pub mod time;
pub mod effect;
pub mod resource;

// Make the key types available directly
pub use types::{ResourceId, DomainId, TraceId, Timestamp, Metadata};
pub use error::{Error, Result};
pub use address::Address;
pub use time::TimeMapSnapshot;
pub use effect::{Effect, EffectContext, EffectOutcome, EffectResult, EffectError};
pub use resource::{
    ResourceRegister, 
    RegisterState, 
    ResourceRegisterLifecycleManager,
    RelationshipTracker,
    RelationshipType,
    ResourceTimeMapIntegration,
    StorageStrategy,
};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Log a message to the console
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}

// Main library file
//
// This file brings together all the modules and provides a unified public API
// for the hashing, commitment, SMT, and database interfaces.

// Re-export public modules and types
pub use types::*;

// Crypto primitives
pub use crypto::hash::{
    HashFunction, HashOutput, Hasher, HashFactory, 
    HashAlgorithm, HashError
};
pub use crypto::merkle::{
    Commitment, CommitmentScheme, CommitmentFactory, 
    CommitmentType, CommitmentError, MerkleTreeCommitmentScheme,
    MerkleProof, H256
};
pub use crypto::signature::{
    Signature, SignatureScheme, SignatureError, 
    SignatureVerificationResult, SignatureFactory
};
pub use crypto::zk::{
    ZkProof, ZkVerifier, ZkProver, ZkError, ZkFactory
};

// SMT implementation
pub use smt::{
    SmtFactory, SmtConfig, SmtError, Key, 
    SmtKeyValue, MerkleSmt
};

// Database interfaces
pub use db::{Database, DbConfig, DbError, DbFactory};

// Verification services
pub mod verification;
pub use verification::{
    VerificationContext, VerificationResult, VerificationError,
    VerificationProvider, VerificationProviderRegistry, 
    VerificationType, VerificationStatus, VerificationOptions,
    Verifiable, UnifiedProof,
};

// Public modules
pub mod crypto;
pub mod smt;
pub mod db;

// Legacy modules (to be removed after migration)
pub mod hash;
// commitment module has been removed and replaced by crypto::merkle

// Domain-specific modules
pub mod domain;
pub mod actor;
pub mod operation;

// Test utilities (only for tests)
#[cfg(test)]
pub mod test_utils {
    // Export utilities for testing here
}

/// Feature flags
#[cfg(feature = "rocksdb")]
pub const HAS_ROCKSDB: bool = true;

#[cfg(not(feature = "rocksdb"))]
pub const HAS_ROCKSDB: bool = false;

// Make the boundary module public 
pub mod boundary;

// Make the examples module public
pub mod examples;

// Add boundary exports
pub use boundary::{BoundaryType, CrossingType, BoundarySafe};