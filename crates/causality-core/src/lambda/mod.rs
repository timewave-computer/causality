//! Layer 1: Linear Lambda Calculus
//!
//! This module implements the core linear lambda calculus with exactly 11 primitives (Layer 1 Linear Lambda Calculus).
//! Complex features like capabilities, objects, and record operations have been 
//! moved to Layer 2 (effect module) for better architectural separation.

/// Base types and core type definitions
pub mod base;

/// Linear type system and linearity tracking
pub mod linear;

/// Tensor product types (A ⊗ B)
pub mod tensor;

/// Sum types (A ⊕ B)
pub mod sum;

/// Linear function types (A ⊸ B)
pub mod function;

/// Symbol type
pub mod symbol;

/// Term representation for Layer 1
pub mod term;

/// Type checker for session types and linear types
pub mod type_checker;

/// Interface to Layer 0
pub mod interface;

/// Location type system for unified computation and communication
pub mod location;

/// Session types fully integrated with linear types
pub mod session_linear;

// Removed rational module - causes compilation errors with missing dashu dependencies
// /// Rational number arithmetic using dashu-ratio
// pub mod rational;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

pub use base::{
    BaseType, Type, TypeInner, Value, TypeRegistry,
    Linear, Affine, Relevant, Unrestricted,
};

pub use linear::{
    Linearity, LinearResource,
    SingleUse, Droppable, Copyable, MustUse, LinearityCheck,
};

// Type constructors
pub use tensor::tensor as product;
pub use sum::sum;
pub use function::linear_function;

// Value types and traits from individual modules
pub use tensor::{TensorValue as ProductValue, TensorIntro as ProductIntro, TensorElim as ProductElim};
pub use sum::{SumValue, SumIntro, SumElim};
pub use function::{LinearFunctionValue, LinearFunctionIntro, LinearFunctionElim, UnitValue, UnitIntro, UnitElim};

pub use symbol::Symbol;

// Term language
pub use term::{Term, TermKind, Literal};

// Type checking
pub use type_checker::{
    type_check, TypeContext, TypeCheckError,
    infer_session_types, solve_constraints, SessionTypeConstraint, SessionOperation
};

// Layer 0 interface
pub use interface::{compile_term, CompileError, CompilationContext};

// Re-export error type from system
pub use crate::system::error::LinearityError;

// Location system
pub use location::{
    Location, LocationUnification, LocationConstraint, LocationUnifier, LocationContext, LocationError,
};

// Session-linear integration
pub use session_linear::{
    LinearSessionEnvironment, SessionOperationResult, SessionLinearError, LinearResourceStats,
}; 