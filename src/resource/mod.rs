// Resource system module
//
// This module provides the register-based resource model implementation
// including one-time use registers, lifecycle management, and ZK integration.

// Core resource modules
pub mod register;
pub mod lifecycle;
pub mod transition;
pub mod nullifier;
pub mod versioning;
pub mod epoch;
pub mod summarization;
pub mod archival;
pub mod garbage_collection;
pub mod register_service;
pub mod zk_integration;
pub mod one_time_register;
pub mod tel;
pub mod allocator;
pub mod manager;
pub mod fact_observer;
pub mod capability;
pub mod capability_api;
pub mod capability_chain;
pub mod register_migration;

#[cfg(test)]
pub mod tests;
#[cfg(test)]
pub mod register_tests;

// Re-export core components
pub use register::{Register, RegisterId, RegisterContents, RegisterState, TimeRange, RegisterNullifier, Metadata};
pub use lifecycle::{RegisterState, StateTransition, TransitionReason};
pub use nullifier::{RegisterNullifier, NullifierRegistry, SharedNullifierRegistry};
pub use transition::{TransitionSystem, TransitionObserver, LoggingTransitionObserver, TransitionResult};
pub use versioning::{SchemaVersion, VersionMigration, MigrationRegistry, VersionError};
pub use epoch::{EpochId, EpochManager, SharedEpochManager, EpochConfig, EpochSummary};
pub use summarization::{SummaryManager, SharedSummaryManager, SummarizationStrategy, RegisterSummary};
pub use archival::{ArchiveStorage, FileSystemStorage, InMemoryStorage, CompressionFormat, RegisterArchive, ArchiveManager, SharedArchiveManager};
pub use garbage_collection::{GarbageCollectionConfig, GarbageCollectionManager, SharedGarbageCollectionManager, GarbageCollectionResult, GarbageCollectionStats, CollectionEligibility};
pub use register_service::{RegisterService, InMemoryRegisterService};
pub use zk_integration::{ZkProofData, ProofVerifier, SharedProofVerifier};
pub use one_time_register::{OneTimeRegisterSystem, OneTimeRegisterConfig, RegisterResult, RegisterError, RegisterOperation};
pub use tel::{TelResourceAdapter, TelResourceMapping};
pub use allocator::{ResourceAllocator, ResourceRequest, ResourceGrant, GrantId, ResourceUsage};
pub use manager::{ResourceManager, ResourceConfig};
pub use capability::{CapabilityId, Right, Restrictions, ResourceCapability, CapabilityError, CapabilityRegistry};
pub use capability_api::{ResourceAPI, ResourceIntent, ResourceOperation, ResourceApiError, ResourceApiResult};
pub use capability_chain::{CapabilityChain, CapabilityExt, ComposedIntent, ChainedIntent, ConditionalIntent, MultiTransferIntent};
pub use register_migration::{RegisterMigrationAdapter, TelRegisterAdapter};

// Re-export domain adapter integration types
pub use crate::domain::DomainId; 

// Re-export time model integration types
pub use crate::domain::map::map::{TimeMap, TimeMapEntry, SharedTimeMap}; 

// Export the fact observer
pub use fact_observer::RegisterFactObserver; 