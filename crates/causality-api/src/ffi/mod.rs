//! Foreign Function Interface (FFI) for the Causality framework
//!
//! This module provides Foreign Function Interface (FFI) for the Causality framework,
//! allowing it to be used from other programming languages, particularly OCaml.
//!
//! ## Features
//!
//! - OCaml bindings: Direct FFI for OCaml integration
//! - Cross-language serialization: Uses SSZ (Simple Serialize) for consistent representation
//! - Error handling: Safe handling of errors across language boundaries
//!
//! ## Usage
//!
//! Enable the `ffi` feature in your Cargo.toml:
//!
//! ```toml
//! causality-api = { version = "0.1", features = ["ffi"] }
//! ```

#[cfg(feature = "ffi")]
pub mod ocaml_adapter;

#[cfg(feature = "ffi")]
pub mod ocaml_bindings;

// Re-export commonly used types and functions for FFI
#[cfg(feature = "ffi")]
pub use causality_types::{
    expr::value::ValueExpr,
    resource::Resource,
    tel::{
        Handler, Effect, Intent, Edge, EdgeKind, EffectGraph,
    },
    serialization::{
        Encode, Decode, SimpleSerialize,
        serialize_for_ffi, deserialize_from_ffi,
        serialize_to_hex, deserialize_from_hex,
    },
};

// Re-export FFI functions for convenience
#[cfg(feature = "ffi")]
pub use ocaml_adapter::{
    value_expr_to_ocaml, value_expr_from_ocaml,
    resource_to_ocaml, resource_from_ocaml,
    handler_to_ocaml, handler_from_ocaml,
    effect_to_ocaml, effect_from_ocaml,
    intent_to_ocaml, intent_from_ocaml,
    edge_to_ocaml, edge_from_ocaml,
    hex_from_ocaml, hex_to_ocaml,
};

// Re-export C bindings for direct FFI use
#[cfg(feature = "ffi")]
pub use ocaml_bindings::*; 