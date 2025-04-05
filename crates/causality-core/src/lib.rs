// Copyright 2023 Causality Ltd
// Licensed under the Apache License, Version 2.0 (the "License");

//! # Causality Core Library
//!
//! This crate provides the core functionality for the Causality system,
//! including resource management, effect handling, concurrency control,
//! and temporal reasoning primitives.

// Module exports
pub mod capability;
// pub mod crypto; // No dedicated crypto module in core
pub mod effect;
pub mod identity; // Add identity module
pub mod resource;
pub mod resource_types; // Add resource_types module for compatibility
// pub mod role; // Role module doesn't exist yet
pub mod time; // Re-enable the time module
pub mod concurrency;
pub mod serialization;
pub mod id_utils;
pub mod utils; // Module with utility functions

// Re-export key types from submodules
pub use capability::{Capability, ResourceGuard};
pub use effect::{Effect, EffectContext, EffectOutcome, EffectId, EffectType};
pub use identity::IdentityId; // Re-export IdentityId
pub use resource::Resource;
pub use crate::resource::types::ResourceId;

// Re-export utilities for easier access
pub use utils::content_addressing::{
    content_hash_to_id,
    content_id_to_hash,
    hash_bytes,
    hash_string,
    default_content_hash,
    hash_object,
};

// Re-export core types from causality_types
pub use causality_types::{ContentAddressed, ContentHash, ContentId};
pub use causality_crypto::hash::ContentHasher;

// Error types
pub use causality_error::{CausalityError, ErrorCode, ErrorDomain, Result};

// Standard library dependencies

// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Top-level initialization function (example)
pub fn initialize() {
    // Perform any necessary setup for the core library
    println!("Causality Core library initialized (version {})", VERSION);
}

// Example usage or test function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        initialize();
        assert!(true); // Simple test to ensure initialization runs
    }
}
