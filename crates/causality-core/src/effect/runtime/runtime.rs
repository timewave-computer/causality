//! Effect runtime interface
//!
//! This module defines the interfaces for executing effects in the system.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use super::context::Context;
use super::core::handler::EffectHandler;
use super::error::EffectResult;
use super::types::{Effect, EffectTypeId};

/// Non-generic interface for effect runtime operations
pub trait EffectRuntimeBase: Debug + Send + Sync {
    /// Register an effect handler
    fn register_handler(
        &mut self,
        effect_type: EffectTypeId,
        handler: Arc<dyn EffectHandler>,
    );
    
    /// Check if a handler exists for the given effect type
    fn has_handler(&self, effect_type: &EffectTypeId) -> bool;
    
    /// Get all registered effect types
    fn registered_effect_types(&self) -> Vec<EffectTypeId>;
}

/// Interface for runtime that executes effects
#[async_trait]
pub trait EffectRuntime: EffectRuntimeBase {
    /// Execute an effect
    async fn execute<E: Effect>(
        &self,
        effect: &E,
        param: E::Param,
        context: &dyn Context,
    ) -> EffectResult<E::Outcome>;
}

/// Interface for verifying capabilities
pub trait CapabilityVerifier: Debug + Send + Sync {
    /// Verify that the context has the required capabilities for the effect
    fn verify_capabilities<E: Effect>(
        &self,
        effect: &E,
        context: &dyn Context,
    ) -> EffectResult<()>;
}

/// Factory for creating effect runtimes
pub trait EffectRuntimeFactory: Debug + Send + Sync {
    /// Create a new effect runtime
    fn create_runtime(&self) -> Arc<dyn EffectRuntimeBase>;
} 