// Effect System Module
//
// The Effect System provides a way to model and execute operations that may have
// side effects on resources, domains, or external systems.
//
// The core of the system is the `Effect` trait, which represents any operation that 
// can be executed and produces an outcome. Effects can run both synchronously and 
// asynchronously, and they may target different execution boundaries (inside or outside
// the system).
//
// Instead of using generic type parameters, the Effect system uses a uniform 
// `EffectOutcome` structure that can represent any result of an effect execution.
// This simplifies the API and makes it easier to compose and chain effects.
//
// Key components:
// - `Effect` trait: The core interface for all effects
// - `EffectOutcome`: Uniform structure for effect results
// - `EffectContext`: Provides context for effect execution
// - `EffectHandler`: Processes effects and handles their execution
// - `ExecutionBoundary`: Defines where an effect can run (inside/outside system)
// - `EffectRegistry`: Manages registration and resolution of effects

// Effect system for Causality
//
// This module provides the core effect system for Causality, allowing effects
// to be described, composed, and executed.

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::error::{Error, Result};
use crate::resource::CapabilityId;
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::crypto::hash::ContentId;

// Module declarations
pub mod boundary;
pub mod constraints;
pub mod content;
pub mod executor;
pub mod handler;
pub mod repository;
pub mod storage;
pub mod templates;
pub mod three_layer;
pub mod transfer_effect;
pub mod types;
pub mod random;
pub mod effect_id;
pub mod empty_effect;
pub mod content_addressed_effect;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export common types
pub use self::boundary::ExecutionBoundary;
pub use self::content::{ContentHash, CodeContent, CodeDefinition};
pub use self::executor::{ContentAddressableExecutor, ExecutionContext, SecuritySandbox, Value, ContextId, ExecutionEvent, CallFrame};
pub use self::handler::{EffectHandler, HandlerResult};
pub use self::effect_id::EffectId;
pub use self::empty_effect::EmptyEffect;
pub use self::repository::{CodeRepository, CodeEntry, CodeMetadata};
pub use self::types::{ResourceChangeType, ResourceChange};
pub use self::content_addressed_effect::{Effect as ContentAddressedEffect, EffectType, EffectOutcome as ContentAddressedEffectOutcome, EffectRegistry as ContentAddressedEffectRegistry};

/// The result of applying an effect
#[derive(Debug, Clone)]
pub struct EffectOutcome {
    /// ID of the effect
    pub id: EffectId,
    
    /// Whether the effect was successful
    pub success: bool,
    
    /// Output data (key-value pairs)
    pub data: HashMap<String, String>,
    
    /// Error message if not successful
    pub error: Option<String>,
    
    /// The execution context ID
    pub execution_id: Option<ContentId>,
    
    /// Resource changes resulting from the effect
    pub resource_changes: Vec<ResourceChange>,
    
    /// Metadata about the execution
    pub metadata: HashMap<String, String>,
}

impl EffectOutcome {
    /// Create a successful outcome
    pub fn success(id: EffectId) -> Self {
        Self {
            id,
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: None,
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a failure outcome
    pub fn failure(id: EffectId, error: impl Into<String>) -> Self {
        Self {
            id,
            success: false,
            data: HashMap::new(),
            error: Some(error.into()),
            execution_id: None,
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the execution ID
    pub fn with_execution_id(mut self, execution_id: ContentId) -> Self {
        self.execution_id = Some(execution_id);
        self
    }
    
    /// Add data to the outcome
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
    
    /// Add a resource change to the outcome
    pub fn with_resource_change(mut self, change: ResourceChange) -> Self {
        self.resource_changes.push(change);
        self
    }
    
    /// Add metadata to the outcome
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Status of an effect execution
pub enum EffectExecution {
    /// The effect is complete with a result
    Complete(EffectOutcome),
    
    /// The effect is still in progress
    InProgress(StatusToken),
    
    /// The effect encountered an error
    Error(Error),
}

/// Effect context for execution
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// When the effect was started
    pub started_at: DateTime<Utc>,
    
    /// Caller address
    pub caller: Option<String>,
    
    /// Context parameters
    pub params: HashMap<String, String>,
}

impl EffectContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            started_at: Utc::now(),
            caller: None,
            params: HashMap::new(),
        }
    }
    
    /// Create a context with a caller
    pub fn with_caller(caller: String) -> Self {
        let mut context = Self::new();
        context.caller = Some(caller);
        context
    }
    
    /// Add a parameter to the context
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

impl Default for EffectContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type for effect execution
pub type EffectResult<T> = std::result::Result<T, EffectError>;

/// Error type for effect execution
#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Capability error: {0}")]
    CapabilityError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Boundary error: {0}")]
    BoundaryError(#[from] self::boundary::BoundaryError),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("External error: {0}")]
    ExternalError(String),
    
    #[error("Not implemented")]
    NotImplemented,
}

/// Main effect trait - defines something that can be executed and produces an outcome
#[async_trait]
pub trait Effect: Send + Sync + std::fmt::Debug {
    /// Get the ID of this effect
    fn id(&self) -> EffectId;
    
    /// Get the boundary where this effect can be executed
    fn boundary(&self) -> ExecutionBoundary;
    
    /// Execute the effect and produce an outcome
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Get a description of what this effect does
    fn description(&self) -> String;
    
    /// Validate that this effect can be executed
    async fn validate(&self, context: &EffectContext) -> EffectResult<()>;
    
    /// Get any capabilities required to execute this effect
    fn required_capabilities(&self) -> Vec<(CapabilityId, Vec<Right>)> {
        Vec::new()
    }
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Asynchronous execution trait for effects
#[async_trait]
pub trait AsyncEffect: Effect {
    /// Execute the effect asynchronously
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome>;
}

/// A structure to manage effect registrations and executions
#[derive(Debug)]
pub struct EffectRegistry {
    effects: HashMap<String, Arc<dyn Effect>>,
    crossing_registry: boundary::BoundaryCrossingRegistry,
}

impl EffectRegistry {
    /// Create a new effect registry
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            crossing_registry: boundary::BoundaryCrossingRegistry::new(),
        }
    }
    
    /// Register an effect
    pub fn register(&mut self, effect: Arc<dyn Effect>) {
        self.effects.insert(effect.name().to_string(), effect);
    }
    
    /// Get an effect by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Effect>> {
        self.effects.get(name).cloned()
    }
    
    /// Get all registered effects
    pub fn get_all(&self) -> Vec<Arc<dyn Effect>> {
        self.effects.values().cloned().collect()
    }
    
    /// Get effects that can execute in a specific boundary
    pub fn get_for_boundary(&self, boundary: ExecutionBoundary) -> Vec<Arc<dyn Effect>> {
        self.effects
            .values()
            .filter(|e| e.can_execute_in(boundary))
            .cloned()
            .collect()
    }
    
    /// Record a boundary crossing event
    pub fn record_crossing<T>(&mut self, crossing: &boundary::BoundaryCrossing<T>, 
                             direction: boundary::CrossingDirection, 
                             success: bool, 
                             error: Option<String>)
    where
        T: std::any::Any,
    {
        self.crossing_registry.record_crossing(crossing, direction, success, error);
    }
    
    /// Get the boundary crossing registry
    pub fn crossing_registry(&self) -> &boundary::BoundaryCrossingRegistry {
        &self.crossing_registry
    }
}

impl Default for EffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect Manager for executing and managing effects
#[derive(Debug)]
pub struct EffectManager {
    registry: EffectRegistry,
    resource_api: Option<Arc<dyn resource_api>>,
}

// For the resource API, this is just a placeholder until we implement the actual trait
trait resource_api: Send + Sync {}

impl EffectManager {
    /// Create a new effect manager
    pub fn new() -> Self {
        Self {
            registry: EffectRegistry::new(),
            resource_api: None,
        }
    }
    
    /// Create a new effect manager with a resource API
    pub fn with_resource_api(resource_api: Arc<dyn resource_api>) -> Self {
        Self {
            registry: EffectRegistry::new(),
            resource_api: Some(resource_api),
        }
    }
    
    /// Register an effect
    pub fn register_effect(&mut self, effect: Arc<dyn Effect>) {
        self.registry.register(effect);
    }
    
    /// Get the effect registry
    pub fn registry(&self) -> &EffectRegistry {
        &self.registry
    }
    
    /// Get a mutable reference to the effect registry
    pub fn registry_mut(&mut self) -> &mut EffectRegistry {
        &mut self.registry
    }
    
    /// Execute an effect by name
    pub async fn execute_effect(&self, effect_name: &str, context: EffectContext) -> EffectResult<EffectOutcome> {
        let effect = self.registry.get(effect_name)
            .ok_or_else(|| EffectError::NotFound(format!("Effect '{}' not found", effect_name)))?;
        
        // Verify capabilities if needed
        if effect.requires_authorization() {
            self.verify_capabilities(&effect, &context).await?;
        }
        
        // For synchronous execution, use the regular Effect trait
        effect.execute(&context).map_err(|e| {
            match e {
                Error::BoundaryViolation => EffectError::BoundaryError(boundary::BoundaryError::ViolatedBoundary {
                    operation: effect_name.to_string(),
                }),
                _ => EffectError::ExecutionError(format!("Effect execution failed: {}", e)),
            }
        })
    }
    
    /// Execute an effect asynchronously by name
    pub async fn execute_effect_async(&self, effect_name: &str, context: EffectContext) -> EffectResult<EffectOutcome> {
        let effect = self.registry.get(effect_name)
            .ok_or_else(|| EffectError::NotFound(format!("Effect '{}' not found", effect_name)))?;
        
        // Verify capabilities if needed
        if effect.requires_authorization() {
            self.verify_capabilities(&effect, &context).await?;
        }
        
        // Try to downcast to AsyncEffect
        if let Some(async_effect) = effect.as_any().downcast_ref::<dyn AsyncEffect>() {
            // Execute asynchronously
            async_effect.execute_async(&context).await
        } else {
            // Fall back to synchronous execution
            effect.execute(&context).map_err(|e| {
                match e {
                    Error::BoundaryViolation => EffectError::BoundaryError(boundary::BoundaryError::ViolatedBoundary {
                        operation: effect_name.to_string(),
                    }),
                    _ => EffectError::ExecutionError(format!("Effect execution failed: {}", e)),
                }
            })
        }
    }
    
    /// Verify that the caller has the required capabilities for this effect
    async fn verify_capabilities(&self, effect: &Arc<dyn Effect>, context: &EffectContext) -> EffectResult<()> {
        // Skip verification if there's no resource API configured
        if self.resource_api.is_none() {
            return Ok(());
        }
        
        // In a full implementation, we would check each required capability
        // against the caller's capabilities. For now, this is a placeholder.
        let required = effect.required_capabilities();
        if !required.is_empty() {
            // Just a placeholder - in a real implementation, this would verify
            // capabilities against the caller using the resource API
            return Err(EffectError::AuthorizationFailed(
                "Capability verification not implemented".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for effects that can be applied to program accounts
pub trait ProgramAccountEffect: Effect {
    /// Get the program account types this effect can be applied to
    fn applicable_account_types(&self) -> Vec<&'static str>;
    
    /// Check if this effect can be applied to a specific program account
    fn can_apply_to(&self, account: &dyn ProgramAccount) -> bool;
}

// This is a placeholder for the ProgramAccount trait
trait ProgramAccount: Send + Sync {
    fn account_type(&self) -> &str;
}

/// Basic no-op effect implementation
#[derive(Debug, Clone)]
pub struct EmptyEffect {
    id: EffectId,
    name: String,
    description: String,
    boundary: ExecutionBoundary,
}

impl EmptyEffect {
    /// Create a new empty effect
    pub fn new() -> Self {
        Self {
            id: EffectId::new_unique(),
            name: "empty".to_string(),
            description: "No-op effect".to_string(),
            boundary: ExecutionBoundary::InsideSystem,
        }
    }
    
    /// Create a new empty effect with a specific name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            id: EffectId::new_unique(),
            name: name.into(),
            description: "No-op effect".to_string(),
            boundary: ExecutionBoundary::InsideSystem,
        }
    }
    
    /// Create a new empty effect with a specific description
    pub fn with_description(description: impl Into<String>) -> Self {
        Self {
            id: EffectId::new_unique(),
            name: "empty".to_string(),
            description: description.into(),
            boundary: ExecutionBoundary::InsideSystem,
        }
    }
    
    /// Create a new empty effect with a specific boundary
    pub fn with_boundary(boundary: ExecutionBoundary) -> Self {
        Self {
            id: EffectId::new_unique(),
            name: "empty".to_string(),
            description: "No-op effect".to_string(),
            boundary,
        }
    }
}

impl Effect for EmptyEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn display_name(&self) -> String {
        self.name.clone()
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    fn execute(&self, _context: &EffectContext) -> Result<EffectOutcome> {
        Ok(EffectOutcome::success(self.id.clone()))
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == self.boundary
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        self.boundary
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    
    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        Vec::new()
    }
    
    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        None
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        Ok(())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// End of file
