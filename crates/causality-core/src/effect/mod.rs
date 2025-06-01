//! Layer 2: Effect Algebra
//!
//! This module implements the effect algebra and domain embeddings
//! for resource management, causality tracking, and transaction orchestration.

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

/// Interface to Layer 1
pub mod interface;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Core types
pub use core::{
    EffectExpr, EffectExprKind, EffectHandler,
    Span, Position,
};

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

// Layer 1 interface
pub use interface::{compile_effect, compile_transaction, EffectCompileError}; 