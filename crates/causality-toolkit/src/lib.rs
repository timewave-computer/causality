//! Developer toolkit for the Causality Resource Model framework
//!
//! This crate provides high-level utilities and tools for developers working with
//! the Causality Resource Model, including testing infrastructure.

// Core modules
pub mod core;
pub mod effects;
pub mod meta;
pub mod control_flow;
pub mod registry;

// Testing utilities for developers
#[cfg(feature = "testing")]
pub mod testing;

// Re-export core functionality
pub use core::{
    ToolkitEffect, ToolkitTelEffectData, ToTelEffect, DefaultAsResourceConvertible,
    ResourceState, TypedResource, ConsumedResource, CloneableEffectBox, EffectExpr,
};
pub use effects::*;
pub use meta::*;
pub use control_flow::{
    ControlFlowInput, ControlFlowOutput, IfEffect, SequenceEffect, WhileEffect,
    HandleableEffect,
};
pub use registry::{EffectRegistry, EffectAny, SimpleEffectHandler, Handles};

// Re-export testing utilities when testing feature is enabled
#[cfg(feature = "testing")]
pub use testing::{
    fixtures::*,
    assertions::*,
    async_utils::*,
};

// Import necessary types for trait definitions
use causality_types::expression::r#type::TypeExpr;
use causality_types::primitive::ids::TypeExprId;
use sha2::{Digest, Sha256};

/// Trait for types that can provide a schema representation
pub trait AsTypeSchema {
    /// Get the type schema for this type
    fn type_schema(&self) -> TypeExpr;
    
    /// Get the effect type name
    fn effect_type_name(&self) -> &'static str;
    
    /// Generate a schema ID based on the effect type name
    fn schema_id(&self) -> TypeExprId {
        let mut hasher = Sha256::new();
        hasher.update(self.effect_type_name().as_bytes());
        let hash = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        TypeExprId::new(hash_array)
    }
}

/// Common result type for toolkit operations
pub type ToolkitResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Test configuration for consistent test setup
#[cfg(feature = "testing")]
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Enable debug logging in tests
    pub debug_logging: bool,
    /// Test timeout in seconds
    pub timeout_secs: u64,
    /// Use deterministic randomness
    pub deterministic: bool,
}

#[cfg(feature = "testing")]
impl Default for TestConfig {
    fn default() -> Self {
        Self {
            debug_logging: false,
            timeout_secs: 30,
            deterministic: true,
        }
    }
}

/// Initialize test environment with common setup
#[cfg(feature = "testing")]
pub fn init_test_env() -> TestConfig {
    TestConfig::default()
}

/// Initialize test environment with custom configuration
#[cfg(feature = "testing")]
pub fn init_test_env_with_config(config: TestConfig) -> TestConfig {
    config
}
