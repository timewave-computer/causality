//! OCaml FFI bindings for the Causality framework
//!
//! This module provides Foreign Function Interface (FFI) bindings to expose
//! core Causality functionality to OCaml applications. It focuses on:
//!
//! - Core types like ResourceId, ExprId, LispValue
//! - Layer 1 (Causality Lisp) interaction
//! - Layer 2 (Intents and Effects) 
//! - Safe memory management between Rust and OCaml

pub mod core_types;
pub mod layer1;
pub mod layer2;
pub mod memory_management;
pub mod error_handling;
pub mod runtime;

// Re-exports
pub use core_types::*;
pub use layer1::*;
pub use layer2::*;
pub use memory_management::*;
pub use error_handling::*;
pub use runtime::*; 