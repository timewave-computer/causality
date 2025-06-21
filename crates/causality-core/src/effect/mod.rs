//! Effect system for the Causality framework
//!
//! This module provides Layer 2 effect system including:
//! - Intent-based declarative effect specification
//! - Effect synthesis and compilation
//! - Temporal Effect Graph (TEG) execution
//! - Effect patterns and operations
//! - Effect resource management
//! - Execution tracing
//! - Record capability effects for Layer 2
//! - Capability system and object model
//! - Row/record type operations
//! - Transform-based constraint system
//! - Automatic protocol derivation from row operations
//! - Unified transform-based effect system

/// Core effect types and expressions
pub mod core;

/// Effect algebra operations
pub mod operations;

/// Resource algebra
pub mod resource;

/// Causality tracking
pub mod causality;

/// Pattern matching
pub mod pattern;

/// Intent-based programming system
pub mod intent;

/// Flow synthesis engine
// pub mod synthesis; // Temporarily disabled due to API incompatibilities

/// Temporal Effect Graph (TEG) for dynamic orchestration
// pub mod teg; // Temporarily disabled due to API incompatibilities

/// Execution tracing
pub mod trace;

/// Interface to Layer 1
pub mod interface;

/// Record capability effects for Layer 2
pub mod record;

/// Capability system (moved from Layer 1)
pub mod capability;

/// Object system with configurable linearity (moved from Layer 1)
pub mod object;

/// Row types for extensible records (moved from Layer 1)
pub mod row;

/// Location-aware row types for unified computation and communication
pub mod location_row;

/// Automatic protocol derivation from row operations
pub mod protocol_derivation;

/// Handler registry for effect handlers
pub mod handler_registry;

/// Intent evaluator for effect handlers
// pub mod intent_evaluator; // Temporarily disabled due to API incompatibilities

/// ZK proof integration for effects
pub mod zk_integration;

/// Storage proof effects for blockchain state verification
pub mod storage_proof;

/// Cross-chain effect coordination for atomic operations across blockchains
pub mod cross_chain;

/// Session registry for global session management
pub mod session_registry;

/// Transform-based constraint system for unified Layer 2 operations
pub mod transform_constraint;

/// Unified transform-based effect system
pub mod transform;

/// Re-export key types for convenience
pub use handler_registry::{EffectHandler, EffectHandlerRegistry, EffectResult};
pub use zk_integration::{EffectHash, ZkProof, ZkVerifiedEffectHandler, ZkEffectRegistry};
pub use storage_proof::{
    StorageProofEffect, StorageDependency, StorageKeySpec, StorageSlot,
    StorageProofRequirements, StorageProofResult, ProofData, EffectPriority,
    StorageValueConstraint, StorageCachePolicy, ZkCircuitConfig, ProofAggregationStrategy,
};
pub use cross_chain::{
    CrossChainEffect, CrossChainCoordinator, CrossChainTxState, CrossChainExecutionResult,
    StorageProofRequirement, ProofType, VerificationConstraint, ConstraintType,
    CrossChainStatistics, BlockchainDomain,
};
// pub use core::SessionBranch; // Temporarily disabled
pub use session_registry::{
    SessionRegistry, Choreography, ChoreographyProtocol, RegistryStats,
};

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Core types
pub use core::{
    EffectExpr, EffectExprKind,
    Span, Position,
};

// Machine types needed for intents
// pub use crate::machine::instruction::{ConstraintExpr, Hint};

// Operations
pub use operations::{
    pure, bind, perform, handle, parallel, race,
    seq, map, join, handler, simple_handler,
    transact, atomic, commit,
};

// Resource algebra
pub use resource::{
    produce, transform, combine, split,
    transfer, split_fungible, merge,
    has_capability, grant_capability, revoke_capability,
    assert_conservation, check_resource,
};

// Causality
pub use causality::{
    check, depend, sequence, verify,
    causal_chain, assert_causality_preserved,
    happens_before, concurrent, causal_barrier,
    verify_causal_consistency, causal_snapshot, restore_snapshot,
};

// Pattern matching
pub use pattern::{
    Pattern, PatternKind, FieldPattern,
};

// Intent system
pub use intent::{
    Intent, IntentId, ResourceBinding, IntentError,
    // Constraint, ValueExpr, // Temporarily disabled - these types don't exist in new intent system
    ResourceRef,
};

// Flow synthesis
// pub use synthesis::{
//     FlowSynthesizer, EffectLibrary, EffectTemplate, ConstraintSolver,
//     SynthesisError, ValidationError, SynthesisStrategy, ResourcePattern,
//     ResourceInfo, ResourceTransformation,
// };

// Temporal Effect Graph (TEG)
// pub use teg::{
//     TemporalEffectGraph, EffectNode, EffectEdge, NodeStatus, NodeId,
//     TegMetadata, TegResult, ExecutionStats, TegError,
// };

// Execution tracing
pub use trace::{
    ExecutionTrace, EffectStep, ExecutionStatus, StepStatus,
};

// Layer 1 interface
pub use interface::{compile_effect, compile_transaction, EffectCompileError};

// Record capability effects
pub use record::{
    RecordEffect, CapabilityToken, RecordOperationResult,
    access_field, update_field, project_record, extend_record, restrict_record,
    create_record, delete_record, require_capability, grant_capability as grant_record_capability,
    record_transaction, validate_capabilities, record_capability_handlers,
};

// Capability system (moved from Layer 1)
pub use capability::{
    Capability, CapabilityLevel, CapabilitySet, RecordCapability, RecordSchema, FieldName,
};

// Object system (moved from Layer 1)
pub use object::{
    Object, CapabilityError,
    LinearObject, AffineObject, RelevantObject, UnrestrictedObject,
};

// Row types (moved from Layer 1) 
pub use row::{
    RowType, RowVariable, RowConstraint, RecordType, RowOpResult, 
    FieldType, FieldAccess, LocationConstraint,
    row, open_row, record, location_row,
};

// Location-aware row types
pub use location_row::{
    LocationAwareRowType, AccessProtocol, SyncProtocol, ConsistencyModel, ConflictResolution,
    RowOpResult as LocationRowOpResult, GeneratedProtocol, ProtocolType, MigrationSpec, MigrationStrategy,
    LocationConstraint as LocationRowConstraint, LocationRequirement, PerformanceRequirement,
    LocationRowError,
};

// Protocol derivation
pub use protocol_derivation::{
    ProtocolDerivationEngine, OptimizationPattern, AccessPattern, MultiPartyTemplate,
    ParticipantRole, ProtocolTemplate, CoordinationStep, ResponsePattern, PeerInteraction,
    PeerInteractionType, CoordinationProtocol, NetworkTopology, ProtocolDerivationError,
};

// Re-export main types
// pub use teg::*;
pub use handler_registry::*;
// pub use intent_evaluator::*;

// Transform constraint system
pub use transform_constraint::{
    TransformConstraintSystem, TransformDefinition, RecordSchema as TransformRecordSchema,
    FieldDefinition, TransformConstraint, SchemaConstraint, TransformConstraintError,
};

// Effect system error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    /// Type error in effect expression
    TypeError(String),
    
    /// Capability error
    CapabilityError(String),
    
    /// Intent error
    IntentError(String),
    
    /// Location error
    LocationError(String),
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::TypeError(msg) => write!(f, "Type error: {}", msg),
            EffectError::CapabilityError(msg) => write!(f, "Capability error: {}", msg),
            EffectError::IntentError(msg) => write!(f, "Intent error: {}", msg),
            EffectError::LocationError(msg) => write!(f, "Location error: {}", msg),
        }
    }
}

impl std::error::Error for EffectError {}

// Re-exports for convenience
pub use core::*;
pub use operations::*;
pub use resource::*;
pub use causality::*;
pub use pattern::*;
pub use intent::*;
// pub use synthesis::*;
// pub use teg::*;
pub use interface::*;
pub use capability::*;
pub use row::*;
pub use location_row::*;
pub use session_registry::*;
// pub use intent_evaluator::*;
pub use transform_constraint::*;
pub use protocol_derivation::*;
pub use transform::*; 