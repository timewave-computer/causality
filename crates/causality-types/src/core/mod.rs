//! Core type definitions for the Causality framework.
//!
//! This module contains fundamental types and traits used throughout the system.

// Core module: Common types and utilities

// Bring necessary types and traits into scope
pub mod domain;
pub mod id;

// Unified type definitions
pub mod effect;
pub mod intent;
pub mod handler;
pub mod transaction;
pub mod resource;
pub mod resource_conversion;

/// Contextual error handling implementation
pub mod contextual_error;
pub mod error;
pub mod error_context;

pub use error::{
    expr_error, resource_error, serialization_error, storage_error,
    time_error, type_error, validation_error, ErrorCategory,
};
// Re-export error handling traits and types
pub use contextual_error::ContextualError;
pub use contextual_error::DefaultErrorContext;
pub use error_context::{
    AsErrorContext, BoundedString, ErrorMetadata, SourceLocation,
};

// Re-export types
pub use effect::Effect;
pub use intent::Intent;
pub use handler::Handler;
pub use transaction::Transaction;
pub use resource::{Resource, ResourceFlow, ResourcePattern, Nullifier};
pub use resource_conversion::{ToValueExpr, FromValueExpr, AsResourceData, ConversionError};

// Re-export all public items from each module
pub use domain::*;
pub use id::*;
