//! Core computational substrate for the Causality framework.
//!
//! This crate provides the fundamental types, traits, and implementations
//! for the Causality linear resource language, organized as a three-layer architecture.
//!
//! ## Architecture
//!
//! The crate is organized into three distinct layers:
//!
//! - **`machine/`** - Layer 0: Register Machine (11 instructions, minimal verifiable execution)
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
    // Content addressing and core types
    EntityId, ResourceId, ExprId, RowTypeId, HandlerId, TransactionId, IntentId, DomainId, NullifierId,
    Timestamp, Str, ContentAddressable,
    encode_fixed_bytes, decode_fixed_bytes, DecodeWithRemainder,
    encode_with_length, decode_with_length, encode_enum_variant, decode_enum_variant,
    // Causality and domain system
    CausalProof, Domain,
};

// SMT re-exports from valence-coprocessor and our hasher
pub use valence_coprocessor::{
    Smt, Hash, HASH_LEN, 
    DataBackend, MemoryBackend, Hasher, SmtChildren, Opening,
};

// SHA256 hasher implementation
use sha2::{Sha256, Digest};

/// SHA256-based hasher implementation
#[derive(Clone)]
pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn hash(data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }

    fn key(domain: &str, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(domain.as_bytes());
        hasher.update(b":");
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }

    fn merge(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }

    fn digest<'a>(data: impl IntoIterator<Item = &'a [u8]>) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for chunk in data {
            hasher.update(chunk);
        }
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }
}

// An in-memory SMT implementation with SHA256 hashing
pub type MemorySmt = Smt<MemoryBackend, Sha256Hasher>;

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
    RegisterValue, Resource, Effect, Constraint,
    ReductionEngine,
    Nullifier, NullifierSet, NullifierError,
    ResourceHeap, ResourceManager,
    Metering, ComputeBudget, InstructionCosts,
};

// Layer 2: Effect Algebra components
pub use effect::{
    EffectExpr, EffectExprKind, EffectHandler, Span, Position,
    Pattern as AstPattern, PatternKind, FieldPattern,
};
