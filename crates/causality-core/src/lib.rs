//! Core computational substrate for the Causality framework.
//!
//! This crate provides the fundamental types, traits, and implementations
//! for the Causality linear resource language, organized as a three-layer architecture.
//!
//! ## Architecture
//!
//! The crate is organized into three distinct layers:
//!
//! - **`machine/`** - Layer 0: Register Machine (9 instructions, minimal verifiable execution)
//! - **`lambda/`** - Layer 1: Linear Lambda Calculus (type-safe functional programming)
//! - **`effect/`** - Layer 2: Effect Algebra (domain-specific effect management)
//! - **`system/`** - Cross-cutting system utilities (content addressing, errors, serialization)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(missing_docs)]
#![recursion_limit = "256"]

//-----------------------------------------------------------------------------
// Core Modules
//-----------------------------------------------------------------------------

/// System-level utilities
pub mod system;

/// Layer 0: Register Machine - minimal verifiable execution model
pub mod machine;

/// Layer 1: Linear Lambda Calculus - type-safe functional programming
pub mod lambda;

/// Layer 2: Effect Algebra - domain-specific effect management
pub mod effect;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// System utilities
pub use system::{
    // Errors (unified system)
    Error, Result, ErrorKind,
    error::{TypeError, MachineError, ReductionError, LinearityError, ResultExt},
};

// SMT re-exports from valence-coprocessor
pub use valence_coprocessor::{
    Smt, Hash, HASH_LEN, 
    DataBackend, MemoryBackend, Hasher, SmtChildren, Opening,
    Blake3Hasher,
};

// An in-memory SMT implementation with Blake3 hashing
pub type MemorySmt = Smt<MemoryBackend, Blake3Hasher>;

// Layer 1: Linear Lambda Calculus types
pub use lambda::{
    BaseType, Type, TypeInner, Value, TypeRegistry,
    Linear, Affine, Relevant, Unrestricted,
    Linearity, LinearResource,
    SingleUse, Droppable, Copyable, MustUse, LinearityCheck,
    // Type constructors
    product, sum, linear_function,
    // Value types
    ProductValue, SumValue, UnitValue, LinearFunctionValue,
    // Introduction and elimination rules
    ProductIntro, ProductElim, SumIntro, SumElim,
    LinearFunctionIntro, LinearFunctionElim, UnitIntro, UnitElim,
    Symbol,
};

// Layer 0: Register Machine components
pub use machine::{
    Instruction, RegisterId, Pattern, MatchArm, ConstraintExpr, EffectCall, LiteralValue,
    MachineState, MachineValue,
    RegisterValue, ResourceId, Resource, Effect, Constraint,
    ReductionEngine,
};

// Layer 2: Effect Algebra components
pub use effect::{
    EffectExpr, EffectExprKind, EffectHandler, Span, Position,
    Pattern as AstPattern, PatternKind, FieldPattern,
};
