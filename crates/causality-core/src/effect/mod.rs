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
pub mod synthesis;

/// Temporal Effect Graph (TEG) for dynamic orchestration
pub mod teg;

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

/// Handler registry for effect handlers
pub mod handler_registry;

/// Intent evaluator for effect handlers
pub mod intent_evaluator;

/// ZK proof integration for effects
pub mod zk_integration;

/// Re-export key types for convenience
pub use handler_registry::{EffectHandler, EffectHandlerRegistry, EffectResult};
pub use intent_evaluator::{IntentEvaluator, IntentEvaluationConfig, EvaluationContext};
pub use zk_integration::{EffectHash, ZkProof, ZkVerifiedEffectHandler, ZkEffectRegistry};

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Core types
pub use core::{
    EffectExpr, EffectExprKind,
    Span, Position,
};

// Machine types needed for intents
pub use crate::machine::instruction::{ConstraintExpr, Hint};

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
    Constraint, ValueExpr, ResourceRef,
};

// Flow synthesis
pub use synthesis::{
    FlowSynthesizer, EffectLibrary, EffectTemplate, ConstraintSolver,
    SynthesisError, ValidationError, SynthesisStrategy, ResourcePattern,
    ResourceInfo, ResourceTransformation,
};

// Temporal Effect Graph (TEG)
pub use teg::{
    TemporalEffectGraph, EffectNode, EffectEdge, NodeStatus, NodeId,
    TegMetadata, TegResult, ExecutionStats, TegError,
};

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
    row, open_row, record,
};

// Re-export main types
pub use teg::*;
pub use handler_registry::*;
pub use intent_evaluator::*; 