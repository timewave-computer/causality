// Re-exports of common types for easy importing
// Original file: src/lib.rs

// This file re-exports key types from the causality-types crate
// for convenient importing by downstream crates.
//
// NOTE: This file has been pruned to only include types that actually
// exist in the current codebase. Many modules from the original file
// that were incomplete or part of future development have been removed.
// The crypto-related types are now defined directly in causality-types.

#![warn(dead_code)]

// Re-export core types
pub use crate::{
    // Basic types
    LamportTime,
    
    // From domain module
    domain::DomainId,
    
    // From block module
    block::{BlockHash, BlockHeight},
    
    // From timestamp module
    timestamp::Timestamp,
    
    // From trace module
    trace::TraceId,
};

// Re-export from address.rs
pub use crate::address::{
    Address,
    AddressPool,
    AddressError,
};

// Re-export from error.rs
pub use crate::error::{
    Error,
    Result,
};

// Re-export from features.rs
pub use crate::features::{
    has_domain_feature,
    has_effect_feature,
    has_code_repo_feature,
};

// Re-export from time_snapshot.rs
pub use crate::time_snapshot::TimeMapSnapshot;

// Re-export crypto primitive types
pub use crate::crypto_primitives::{
    ContentAddressed,
    ContentId,
    ContentHash,
    HashOutput,
    HashAlgorithm,
    HashError,
};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
