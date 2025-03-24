// Causality - A Unified Resource Management System
//
// This library provides a unified resource management system with lifecycle management,
// relationship tracking, capability-based authorization, and effect templates.

// Causality crate
//
// This crate provides a content-addressed architecture for building
// distributed applications with cryptographic verification.

#![warn(dead_code)]

pub mod crypto;

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
    // Commented out due to missing module, uncomment when implemented
    // pub use crate::resource::relationship::*;
    
    // Import the cross-domain relationship query
    pub mod cross_domain_query;
}

// Re-export key types for convenience
pub use crypto::{
    ContentAddressed, ContentId, HashOutput, HashAlgorithm,
    ContentAddressedSmt, DeferredHashing,
    ContentAddressedStorage, StorageError, StorageFactory
};

// Effect re-exports
pub use effect::{
    EffectType, // Only export non-duplicated types
    ContentAddressedEffect, ContentAddressedEffectOutcome
};

// Resource re-exports
pub use resource::{
    Resource, ResourceRegistry,
    ContentAddressedRegister, ContentAddressedRegisterOperation,
    ContentAddressedRegisterOperationType, ContentAddressedRegisterRegistry,
    StateVisibility,
};

// Domain re-exports
pub use domain::content_addressed_interface::{
    ContentAddressedDomainInterface, ContentAddressedDomainRegistry,
    CommitmentProof as DomainCommitmentProof, CrossDomainError
};
pub use domain::content_addressed_transaction::{
    ContentAddressedTransaction, ContentAddressedTransactionVerifier,
    ContentAddressedTransactionVerifierImpl, TransactionVerificationResult,
    TransactionVerificationError
};

// Exposed language modules
pub mod ast;
pub mod capabilities;

// Concurrency and execution frameworks
pub mod concurrency;
// Use the directory-based module for execution
// and delete the file-based module src/execution.rs
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

// Import specific types from effect to avoid duplicates
pub use effect::{
    EffectContext, EffectResult, EffectError,
    ContentHash, CodeContent, CodeDefinition, 
    ContentAddressableExecutor, CodeRepository,
    ExecutionContext, SecuritySandbox, Value,
    CodeEntry, CodeMetadata
};

// Import specific types from resource with correct paths
pub use resource::resource_register::{
    ResourceRegister, 
    RegisterState, 
    StorageStrategy,
};

// These need proper paths, commented out until properly implemented
// pub use resource::lifecycle_manager::ResourceRegisterLifecycleManager;
// pub use resource::relationship_tracker::RelationshipTracker;
// pub use resource::relationship_tracker::RelationshipType;
// pub use resource::resource_temporal_consistency::ResourceTemporalConsistency;

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

// Crypto primitives - only export non-duplicated types
pub use crypto::hash::{
    HashFunction, Hasher, HashFactory, 
    HashError
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
pub mod db;
pub mod actor;
pub mod operation;
pub mod log {
    //! Log and fact modules for Causality
    
    // Re-export core log types
    pub mod fact_types;
    pub mod fact_replay;
    pub mod content_addressed_fact;
}
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

// Use the domain exports without the duplicate DomainId
pub use domain::{
    DomainAdapter, DomainRegistry,
    // DomainConfig is not defined in domain module
};

// Provider modules
pub mod provider;
pub use provider::{
    Provider, ContentAddressedProviderRegistry, ProviderMetadata, ProviderError,
};

// Component modules
pub mod component;
pub use component::{
    Component, ComponentType, ComponentState, ComponentMetadata,
    ContentAddressedComponent, ContentAddressedComponentRegistry, ComponentError,
};
