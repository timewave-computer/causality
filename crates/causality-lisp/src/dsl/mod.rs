// crates/causality-lisp/src/dsl/mod.rs
//! # Lisp Domain Specific Language (DSL) for Rust using Higher-Order Functions
//!
//! This module provides a Higher-Order Function (HOF) based DSL for constructing
//! `causality_types::expr::ast::Expr` instances programmatically in Rust.
//! It aims to offer a more functional and Rust-idiomatic way to build Lisp ASTs.
//!
//! The primary interface will be through the `hof` module. The `builders` module
//! provides primitive AST node constructors, which are mainly intended for internal
//! use by the `hof` module.

//-----------------------------------------------------------------------------
// Module Declarations
//-----------------------------------------------------------------------------

// Primitive AST node constructors (used by hof.rs)
pub mod builders;

// HOF-style DSL
pub mod hof;

// Serializer remains relevant
pub mod serializer;

// Value extensions for convenience
pub mod value_ext;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Re-export all from the HOF module for direct use.
pub use hof::*;

// types.rs is currently minimal and might be removed or repurposed.
// If it's not used by `hof` or `serializer`, consider removing it.
pub mod types;

// New fluent builders and HOF-style helpers
// These modules were deleted, removing their declarations.
// pub mod fluent;
// pub mod helpers;

// Re-export core types and builders for easier use.
// LispExpr was removed from types.rs
pub use builders::{
    // Arithmetic
    add,
    and_,
    bool_lit,
    // Effect Status
    completed,
    cons,
    defun,
    div,
    eq_,
    first,
    // Data Access
    get_context_value,
    get_field,
    get_map_value,
    // Comparison
    gt,
    gte,
    has_key,
    // Control flow & Equality
    if_,
    int_lit,
    keyword_lit,
    len,
    list,
    lt,
    lte,
    // Map/Struct Operations
    make_map,
    mul,
    nil,
    not_,
    // List Operations
    nth,
    or_,
    rest,
    str_lit,
    sub,
    sym,
};
// The lisp_list! macro is exported via #[macro_export] in builders.rs and can be imported with use path.

// Re-export all from original builders for now, then decide on a more curated set.
pub use builders::*;

// Re-export all from fluent and helpers
// These modules were deleted, removing their re-exports.
// pub use fluent::*;
// pub use helpers::*;
