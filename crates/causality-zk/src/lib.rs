//! ZK proof generation for the Causality system.
//!
//! This crate provides functionality for generating and verifying zero-knowledge proofs
//! for Causality graph executions, integrated with the Valence Coprocessor for ZK proving.
//!
//! Architecture based on the Valence Coprocessor dual-target system as described in
//! [Valence Coprocessor Interaction](../../../docs/valence-coprocessor-interaction.md).
//!
//! Expression verification uses static compilation to Rust+RISC-V with minimal interpreter
//! fallback as described in:
//! - [Expression Compilation](../../../docs/expression_compilation.md)
//! - [Expression Interpretation](../../../docs/expression_interpretation.md)

//-----------------------------------------------------------------------------
// Core modules - shared between WASM and RISC-V targets
//-----------------------------------------------------------------------------

pub mod circuit;
pub mod compiler;
pub mod core;
pub mod deployment;
pub mod interpreter;
pub mod models;
pub mod runtime;
pub mod verification;
pub mod witness;

//-----------------------------------------------------------------------------
// Target-specific Modules
//-----------------------------------------------------------------------------

// SP1-specific module
#[cfg(feature = "sp1")]
pub mod sp1;

//-----------------------------------------------------------------------------
// Target-specific modules
//-----------------------------------------------------------------------------

// SP1 support is temporarily disabled during refactoring

//-----------------------------------------------------------------------------
// Re-exports for user convenience

//-----------------------------------------------------------------------------

// Essential core type

pub use crate::core::{CircuitId, Error as ZkError, ProofId, WitnessId};

// Backward compatibility re-exports
pub use crate::interpreter::core::{
    interpret_expr, interpret_expr_with_step_limit, InterpreterError,
};

// Exposing interpreter components directly to maintain the same API
pub mod expr_interpreter {
    pub use crate::interpreter::core::validate_constraints;
}

pub mod combinator_interpreter {
    pub use crate::interpreter::core::{
        interpret_expr_with_step_limit, InterpreterError,
    };
}

// Witness building utilitie
pub use crate::witness::AsWitness;
pub use crate::witness::PublicInputs;
pub use crate::witness::WitnessData;

// Error types and handling
pub use crate::core::Error;

// Model type
pub use crate::runtime::core::ZkEffect;
pub use crate::runtime::core::ZkResource;
pub use causality_types::resource::state::ResourceState;

// Compiler integration
pub use crate::compiler::generate_circuit_id;
#[cfg(feature = "host")]
pub use crate::compiler::register_circuit;
pub use crate::compiler::CircuitTemplate;

// Runtime integration
pub use crate::runtime::convert_effects;
pub use crate::runtime::convert_resources;
pub use crate::runtime::process_execution_trace;
pub use crate::runtime::VerificationResult;

// Deployment infrastructure
#[cfg(feature = "host")]
pub use crate::deployment::DeploymentManager;
#[cfg(feature = "host")]
pub use crate::deployment::KeyStore;
pub use crate::deployment::ProgramRegistration;
pub use crate::deployment::VerificationKey;

// Runtime API
#[cfg(feature = "host")]
pub use crate::runtime::ProofRepository;
pub use crate::runtime::StoredProof;
#[cfg(feature = "host")]
pub use crate::runtime::ZkRuntimeApi;

// Expression interpretation function
pub use crate::interpreter::combinators::all_constraints_satisfied;
pub use crate::interpreter::core::validate_constraints;

// Re-export the minimal interpreter and validator for direct use
pub use causality_lisp::core::ExprContextual;
pub use causality_lisp::Interpreter;
pub use causality_types::system::provider::AsExprContext;

// Make ssz available for consistency
pub use ssz;

/// Macro to define witness types using frunk's HList
///
/// # Examples
///
/// ```
/// # use causality_zk::define_witness_types;
/// # use std::marker::PhantomData;
/// # struct MyWitness1; struct MyWitness2;
/// define_witness_types!(MyWitnessTypes, MyWitness1, MyWitness2);
/// ```
#[macro_export]
macro_rules! define_witness_types {
    ($name:ident, $($t:ty),* $(,)?) => {
        pub type $name = frunk::HList![$($t,)* frunk::HNil];
    };
}

/// Macro to create a witness types registry
///
/// # Examples
///
/// ```
/// # use causality_zk::{define_witness_types, create_witness_registry};
/// # struct MyWitness1; struct MyWitness2;
/// # define_witness_types!(MyWitnessTypes, MyWitness1, MyWitness2);
/// create_witness_registry!(MyRegistry, MyWitnessTypes);
/// ```
#[macro_export]
macro_rules! create_witness_registry {
    ($name:ident, $types:ty) => {
        pub type $name = $crate::witness::WitnessRegistry<$types>;
    };
}

// Re-export key types for convenience
pub use crate::core::ZkCombinatorInterpreter;
pub use crate::runtime::core::ZkExecutionResult;
