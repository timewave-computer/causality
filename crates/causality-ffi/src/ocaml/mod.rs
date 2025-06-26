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
pub mod error_handling;
pub mod layer1;
pub mod layer2;
pub mod memory_management;
pub mod runtime;
pub mod unified_types;

// Re-exports
pub use core_types::{LispValue, ResourceId, ExprId};
pub use error_handling::result_to_ocaml;
pub use runtime::{causality_init, causality_version, causality_cleanup, with_runtime_state};
pub use unified_types::{OcamlLocation, OcamlTypeInner, OcamlSessionType};
