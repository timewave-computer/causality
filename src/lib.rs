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
pub mod capabilities;

// Effect system
pub mod effect;

// TEL (Transaction Effect Language)
pub mod tel;

// Optional modules based on features
#[cfg(feature = "domain")]
pub mod domain;

#[cfg(feature = "domain")]
pub mod domain_adapters;

// Resource system
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
pub mod program_account;

// System boundaries
pub mod boundary;

// Re-exports from effect system
pub use effect::Effect;
pub use effect::EffectContext;
pub use effect::EffectOutcome;
pub use effect::EffectResult;

// Re-exports from TEL system
pub use tel::{
    TelScript, TelOperation, TelOperationType, TelParser,
    TelHandler, TelHandlerRegistry, TelCompiler, StandardTelCompiler,
    TransferParams, StorageParams, QueryParams,
    parse_tel, compile_tel, execute_tel
};

// Re-exports from resource system
pub use resource::{
    ResourceId, Quantity, RegisterId, ResourceRegister, ResourceLogic,
    ResourceState, TransitionReason,
    RelationshipTracker, ResourceRelationship, RelationshipType, RelationshipDirection,
    ResourceRegisterLifecycleManager, RegisterOperationType,
    StorageStrategy, StorageAdapter, CapabilityId
};

// Re-exports from capabilities system
pub use capabilities::{Right, Capability, CapabilityType};

// Re-exports from AST system
pub use ast::{
    AstNodeId,
    AstNodeType,
    AstContext,
    GraphCorrelation,
    CorrelationTracker
};

// Re-exports from log system
pub use log::{LogEntry, LogStorage, ReplayEngine};

// Re-exports from domain system
#[cfg(feature = "domain")]
pub use domain::{
    DomainAdapter, DomainRegistry, DomainId, DomainInfo, DomainType
};

// Re-export from time module
pub use time::LamportTime;

// Re-exports from ZK system
pub use zk::{
    StateTransition,
    ZkVirtualMachine,
    ZkAdapter,
    Witness,
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

// Re-exports from program account system
pub use program_account::base_account::BaseAccount;
pub use program_account::registry::{StandardProgramAccountRegistry, AccountType};
pub use program_account::asset_account::{AssetAccount, AssetType, AssetCollection};
pub use program_account::utility_account::{UtilityAccount, StoredData};
pub use program_account::effect_adapter::{
    ProgramAccountEffectAdapter,
    ProgramAccountEffectAdapterImpl,
    EffectInfo,
    EffectParameterType
};