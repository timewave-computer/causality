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
pub mod relationship {
    //! Relationship management system
    
    // Re-export core types from the resource relationship module
    pub use crate::resource::relationship::*;
    
    // Import the cross-domain relationship query
    pub mod cross_domain_query;
}

// Exposed language modules
pub mod ast;
pub mod capabilities;

// Concurrency and execution frameworks
pub mod concurrency;
pub mod execution;

// Domain and adapter modules
pub mod domain;
pub mod domain_adapters;
pub mod committee;

// Integration modules
pub mod integration;

// Workflow modules
pub mod invocation;
pub mod snapshot;
pub mod zk;

// Make the key types available directly
pub use types::{ResourceId, DomainId, TraceId, Timestamp, Metadata};
pub use error::{Error, Result};
pub use address::Address;
pub use time::TimeMapSnapshot;
pub use effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    ContentHash, CodeContent, CodeDefinition, 
    ContentAddressableExecutor, CodeRepository,
    ExecutionContext, SecuritySandbox, Value,
    CodeEntry, CodeMetadata
};
pub use resource::{
    ResourceRegister, 
    RegisterState, 
    ResourceRegisterLifecycleManager,
    RelationshipTracker,
    RelationshipType,
    ResourceTemporalConsistency,
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
pub use crypto::smt::{
    SmtKeyValue, SmtFactory, MerkleSmt, SmtError
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
pub mod db;
pub mod actor;
pub mod operation;
pub mod log;
pub mod boundary;
pub mod examples;

// Concurrency exports
pub use concurrency::{
    TaskId, ResourceGuard, ResourceManager, SharedResourceManager,
    WaitQueue, SharedWaitQueue, TaskScheduler
};

// Execution exports
pub use execution::{
    ExecutionContext as ExecContext, CallFrame as ExecCallFrame, 
    ExecutionEvent as ExecEvent, ContextId as ExecContextId,
    ContentAddressableExecutor as ExecCodeExecutor, ExecutionError, ExecutionTracer
};

// Feature management
pub mod features;

/// Feature flags
#[cfg(feature = "rocksdb")]
pub const HAS_ROCKSDB: bool = true;

#[cfg(not(feature = "rocksdb"))]
pub const HAS_ROCKSDB: bool = false;

// Add boundary exports
pub use boundary::{BoundaryType, CrossingType, BoundarySafe};