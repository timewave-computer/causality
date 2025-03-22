// Causality System
//
// A Rust implementation of the Causality system for cross-Domain 
// programming with algebraic effects, RISC-V compilation for 
// zero-knowledge virtual machine execution, and content-addressed code.

// Feature flags
pub mod features;

// Core modules (always included)
pub mod error;
pub mod types;
pub mod actor;
pub mod address;
pub mod effect_adapters;
pub mod time;

// Effect system (core components always included)
#[path = "effect.rs"]
pub mod effect;

// Temporal Effect Language (TEL) system
#[path = "tel.rs"]
pub mod tel;

// Optional modules based on features
#[cfg(feature = "domain")]
pub mod domain;

#[cfg(feature = "domain")]
pub mod domain_adapters;

#[cfg(feature = "code-repo")]
pub mod code;

// Resource system
#[path = "resource.rs"]
pub mod resource;

// Abstract Syntax Tree (AST) system
pub mod ast;

// Execution system
pub mod execution;

// Snapshot system (includes time-travel debugging functionality)
pub mod snapshot;

// Zero-knowledge proof system
pub mod zk;

// Logging system
#[path = "log.rs"]
pub mod log;

// Optional invocation system
pub mod invocation;

// Concurrency primitives
pub mod concurrency;

// Interpreter for executing code
pub mod interpreter;

// Builder patterns
pub mod builder;

// Program Account Layer
#[path = "program_account.rs"]
pub mod program_account;

// System boundaries
pub mod boundary;

// Re-exports from effect system
pub use effect::{Effect, EffectType, SerializableEffect};
pub use effect::handler::{EffectHandler, SharedHandler, NoopHandler, shared, compose};
pub use effect::continuation::{Continuation, map, and_then, constant, identity};
pub use effect::dependency::{FactDependency as DependencyFactDependency, EffectDependency, DependencySet};
pub use effect::snapshot::{FactSnapshot as SnapshotFactSnapshot, SystemSnapshot, SnapshotManager};

// Re-exports from TEL system
pub use tel::{
    Effect as TelEffect,
    DomainId,
    AssetId,
    Amount,
    Address as TelAddress,
    ResourceId,
    Authorization as TelAuthorization,
    AuthorizedEffect,
    ConditionalEffect,
    TimedEffect,
    ResourceContents,
    FactType,
    Condition,
    Predicate,
    CircuitType,
};

// Re-exports from resource system
pub use resource::{
    ResourceManager, 
    ResourceGuard,
    ResourceAllocator, 
    ResourceRequest, 
    ResourceGrant, 
    ResourceUsage, 
    StaticAllocator
};

// Re-exports from AST system
pub use ast::{
    AstNodeId,
    AstNodeType,
    AstContext,
    GraphCorrelation,
    CorrelationTracker
};

// Re-exports from code system
#[cfg(feature = "code-repo")]
pub use crate::effect_adapters::repository::{CodeRepository, CodeEntry, CodeMetadata};

// Re-exports from log system
pub use log::{LogEntry, LogStorage, ReplayEngine};
pub use log::sync::{SyncManager, SyncConfig, PeerInfo, SyncProtocol, HttpSyncProtocol};
pub use log::visualization::{LogVisualizer, VisualizationFilter, VisualizationFormat};

// Re-exports from domain system
#[cfg(feature = "domain")]
pub use domain::{
    DomainAdapter, DomainRegistry, DomainId, DomainInfo, DomainType, DomainStatus,
};

// Re-exports from domain time system
#[cfg(feature = "domain")]
pub use domain::map::{
    TimeMap, TimeMapEntry, TimeMapHistory, TimeMapNotifier, SharedTimeMap,
    TimePoint, TimeRange, TimeSyncConfig, SyncStatus, SyncResult,
    TimeSyncManager
};

// Re-export from time module
pub use time::LamportTime;

// Re-exports from ZK system
pub use zk::{
    ZkVirtualMachine,
    ZkAdapter,
    Witness,
    StateTransition,
    MemoryAccess,
    VmState,
    Proof,
    RiscVProgram,
    RiscVSection,
    RiscVSectionType
};

// Re-exports from invocation system
pub use invocation::{
    InvocationSystem,
    context::InvocationContext,
    registry::{EffectRegistry, HandlerRegistration, HandlerInput, HandlerOutput},
    patterns::{
        InvocationPattern,
        DirectInvocation,
        CallbackInvocation,
        ContinuationInvocation,
        PromiseInvocation,
        StreamingInvocation,
        BatchInvocation,
    }
}; 

// Re-exports from snapshot system
pub use snapshot::{
    ExecutionSnapshot, 
    SnapshotId, 
    SnapshotError, 
    FileSystemSnapshotManager,
    IncrementalSnapshotManager,
    SnapshotDiff,
    CheckpointManager,
    CheckpointConfig,
    // Time-travel functionality
    navigator::TimeTravel,
    navigator::TimeTravelNavigator,
    inspector::StateInspector,
    inspector::ContextStateInspector,
    diff::StateDiffer,
    diff::StateComparer
};

// Re-exports from modules directly (avoiding integration modules)
#[cfg(feature = "code-repo")]
pub use effect::{EffectIntegrator, ContentAddressedEffect};
#[cfg(feature = "code-repo")]
pub use code::{TelIntegrator, ContentAddressedTelCompiler};

// Re-exports from program account system
pub use program_account::{
    ProgramAccount,
    ProgramAccountRegistry,
    AssetProgramAccount,
    UtilityProgramAccount,
    DomainBridgeProgramAccount,
    ProgramAccountResource,
    AvailableEffect,
    EffectResult,
};

// Export tel model
pub use tel::resource::model;