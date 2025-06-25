//! Causality FFI - Foreign Function Interface for OCaml integration
//!
//! This crate provides FFI bindings to integrate the Causality unified 5-instruction
//! system with OCaml implementations.

#![warn(missing_docs)]

// OCaml FFI modules
#[cfg(feature = "ocaml-ffi")]
pub mod ocaml;

// Re-export key types
pub use causality_core::{Value, TypeInner, BaseType};

#[cfg(feature = "ocaml-ffi")]
pub use ocaml::{
    causality_init,
    causality_version,
    causality_cleanup,
};

/// FFI error type
#[derive(Debug, thiserror::Error)]
pub enum FfiError {
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
} 