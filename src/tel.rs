// Temporal Effect Language Module
//
// This module provides the core functionality for the Temporal Effect
// Language, including type definitions, resource management, and
// effect handling.

// Core submodules
pub mod types;
pub mod error;
pub mod resource;
pub mod effect;
pub mod adapter;
pub mod builder;

// Re-export core components
pub use types::{ResourceId, Address, Domain, OperationId, Proof, Timestamp};
pub use error::{TelError, TelResult};
pub use resource::{
    Register, RegisterId, RegisterState, RegisterContents,
    Resource, ResourceGuard, ResourceManager, AccessMode,
    ResourceOperation, ResourceOperationType, ResourceTracker,
    SnapshotManager, SnapshotId, 
    VersionManager, VersionId,
    ZkVerifier, VerificationResult,
};
pub use effect::{
    ResourceEffect,
    EffectResult,
    ResourceEffectAdapter,
    EffectComposer,
    RepeatingEffect,
    RepeatSchedule,
    RepeatConfig,
    proof::{
        EffectProofGenerator,
        EffectProofVerifier,
        EffectProofFormat,
        EffectProofMetadata,
    },
};
pub use builder::TelBuilder;
pub use adapter::AdapterRegistry; 