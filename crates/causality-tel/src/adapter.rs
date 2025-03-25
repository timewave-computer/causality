// TEL adapter system
// Original file: src/tel/adapter/mod.rs

// Domain adapter framework for the Temporal Effect Language
//
// This module provides traits and utilities for implementing
// domain-specific adapters that compile TEL effects into
// domain-specific formats for execution.

pub mod traits;
pub mod common;
pub mod registry;
pub mod mock;
#[cfg(test)]
mod tests;

// Re-export core components
pub use traits::{EffectCompiler, CompilationResult, CompilerContext, CompilationOptions};
pub use registry::{AdapterRegistry, AdapterConfig, AdapterMetadata, AdapterStatus};
pub use common::{ValidationResult, ValidationError, ValidationErrorType, CommonValidators};

// Re-export the mock adapter for easier access in tests
#[cfg(test)]
pub use mock::{MockAdapter, MockAdapterConfig, MOCK_DOMAIN_ID, MOCK_ASSET}; 