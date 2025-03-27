// Core Effect System
//
// This module provides a type-driven algebraic effect system for Causality,
// providing abstractions for managing state changes and side effects.

use std::fmt::Debug;

use causality_types::ContentId;
use crate::resource_types::ResourceId;
use crate::capability::Right;

pub mod context;
pub mod domain;
pub mod outcome;
pub mod registry;
pub mod resource;
pub mod orchestration;
pub mod storage;
pub mod types;
#[cfg(test)]
mod tests;

pub use context::{
    EffectContext, EffectContextError, EffectContextResult, 
    BasicEffectContext, EffectContextBuilder,
};

pub use domain::{
    DomainEffect, DomainEffectHandler, DomainId, DomainCapabilityMapping,
    DomainEffectError, DomainEffectResult, DomainEffectOutcome,
    ParameterValidationResult, DomainParameterValidator,
    EnhancedDomainContextAdapter, EnhancedDomainEffectHandler,
    DomainEffectExt,
};

pub use outcome::{EffectOutcome, EffectError, EffectResult};

pub use registry::{
    EffectHandler, EffectRegistry, EffectRegistryError, EffectRegistryResult,
    BasicEffectRegistry,
};

pub use resource::{
    ResourceEffect, ResourceOperation, ResourceEffectOutcome,
    ResourceEffectError, ResourceEffectResult,
};

pub use orchestration::{
    OrchestrationStatus, OrchestrationRef, OrchestrationStep, 
    OrchestrationPlan, OrchestrationBuilder, EffectOrchestrator,
    BasicEffectOrchestrator, OrchestrationFactory, BasicOrchestrationFactory,
};

pub use storage::{
    EffectStorage, EffectStorageError, EffectStorageResult,
    EffectExecutionRecord, EffectOutcomeRecord,
    ContentAddressedEffectStorage, InMemoryEffectStorage,
    EffectStorageConfig, create_effect_storage,
};

pub use types::{EffectId, EffectTypeId, ExecutionBoundary};

/// Trait for effects that can be executed
#[async_trait::async_trait]
pub trait Effect: Debug + Send + Sync {
    /// Get the ID of this effect
    fn id(&self) -> &EffectId;
    
    /// Get the type ID of this effect
    fn type_id(&self) -> EffectTypeId;
    
    /// Get the execution boundary for this effect
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Any
    }
    
    /// Get a display name for this effect
    fn name(&self) -> String {
        format!("{:?}", self)
    }
    
    /// Check if this effect is valid in the given context
    fn is_valid(&self, _context: &dyn EffectContext) -> bool {
        true
    }
    
    /// Get the resources this effect depends on
    fn dependencies(&self) -> Vec<ResourceId> {
        Vec::new()
    }
    
    /// Get the resources this effect modifies
    fn modifications(&self) -> Vec<ResourceId> {
        Vec::new()
    }
    
    /// Clone this effect into a boxed effect
    fn clone_effect(&self) -> Box<dyn Effect>;
    
    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Get a human-readable name for this effect
    fn display_name(&self) -> String;
    
    /// Get a human-readable description of this effect
    fn description(&self) -> String;
    
    /// Execute the effect with the given context
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Get the capabilities required to execute this effect
    fn required_capabilities(&self) -> Vec<(ContentId, Right)> {
        Vec::new()
    }
    
    /// Check if this effect can execute in the given boundary
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        matches!(boundary, ExecutionBoundary::Any)
    }
    
    /// Get the preferred execution boundary for this effect
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Inside
    }
    
    /// Get a map of parameters for display purposes
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
} 