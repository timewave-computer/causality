// Causality Core Library
//
// This library provides core functionality for the Causality framework.

// =================================================================
// Core component modules - these form the stable API of this crate
// =================================================================

// Re-export crypto module from causality-crypto
pub use causality_crypto as crypto;

// Re-export important types from causality_types
pub use causality_types::{
    ContentId, ContentAddressed, ContentHash, HashOutput, HashAlgorithm, HashError,
    content_addressing,
};

// Time Management
// Provides abstractions for temporal operations, logical time tracking,
// clock synchronization, and timestamp validation
pub mod time;

// Concurrency Primitives
// Thread-safe data structures, task scheduling abstractions,
// and event notification systems
pub mod concurrency;

// Verification Framework
// Common verification interfaces, proof validation utilities,
// verification context management, and pluggable verification strategies
pub mod verification;

// Resource System
// Content-addressed resources, state management, capability-based access control,
// and resource operations
pub mod resource;

// Effect System
// Core effect definitions, context management, and registry interfaces
// for implementing side effects and state changes
pub mod effect;

// Observation System
// External chain indexing, fact extraction, data provider interfaces,
// and log reconstruction primitives
pub mod observation;

// Error Handling
// Unified error types, error context tracking, and result extension utilities
pub mod error;

// Serialization Helpers
// Common serialization utilities, schema management, and content-addressing support
pub mod serialization;

// Capability Model
// Content addressing, capability-based security, and access control
pub mod capability;

// SMT module
pub mod smt;

// Integration Module
// Integration components for connecting different parts of the system
pub mod integration;

// ID Utilities
// Helper functions for generating content-addressed identifiers
pub mod id_utils {
    use crate::ContentId;
    
    /// Generate a unique content-addressed identifier for an operation
    pub fn generate_operation_id(prefix: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        // Create a content hash from the timestamp and a random component
        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_be_bytes());
        
        // Add some randomness (using thread_rng since it's for local identifiers)
        let rand_bytes: [u8; 8] = rand::random();
        data.extend_from_slice(&rand_bytes);
        
        // Create a ContentId using the causality-types functionality
        let content_id = ContentId::from(&data[..]);
        
        // Format with the specified prefix
        format!("{}-{}", prefix, content_id.to_string())
    }
    
    /// Generate a unique content-addressed identifier for a decision
    pub fn generate_decision_id() -> String {
        generate_operation_id("decision")
    }
    
    /// Generate a unique content-addressed identifier for a system operation
    pub fn generate_system_operation_id() -> String {
        generate_operation_id("operation")
    }
    
    /// Generate a unique content-addressed identifier for a maintenance window
    pub fn generate_maintenance_window_id() -> String {
        generate_operation_id("window")
    }
    
    /// Generate a unique content-addressed identifier for a transfer
    pub fn generate_transfer_id() -> String {
        generate_operation_id("transfer")
    }
}

// =================================================================
// Legacy modules - these are being evaluated for refactoring or removal
// =================================================================

// Zero-Knowledge Proof System
// This module may be moved to a specialized crate
// TODO: Evaluate if this should remain in core or be moved
#[cfg(feature = "zk")]
pub mod zk;

// Re-export important types for easier access
pub use error::{Error as CoreError, Result as CoreResult};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Return the version of the crate
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Return the name of the crate
pub fn name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Return the description of the crate
pub fn description() -> &'static str {
    env!("CARGO_PKG_DESCRIPTION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        assert!(!super::version().is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(super::name(), "causality-core");
    }

    #[test]
    fn test_description() {
        assert!(!super::description().is_empty());
    }
}

// Re-export integration components
pub use integration::{
    domain::{
        DomainIntegrationError, DomainIntegrationResult, DomainAdapterFactory,
        DomainEffectRouter, DomainResourceRouter, create_domain_integration_layer,
    },
};

// Common re-exports from resource module
pub use resource::types as resource_types;
pub use resource::storage as resource_storage;
pub use resource::validation as resource_validation;
pub use resource::agent::registry as agent_registry;  // Use agent registry instead of domain registry

// Public types
pub type CausalityResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>; 