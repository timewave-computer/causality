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
use uuid;
use thiserror::Error;

use crate::error::{Error, Result};
use crate::capabilities::{CapabilityId, Right};
use crate::types::{ResourceId, DomainId};

// Module declarations
pub mod boundary;
pub mod constraints;
pub mod continuation;
pub mod factory;
pub mod handler;
pub mod storage;
pub mod templates;
pub mod three_layer;
pub mod transfer_effect;
pub mod types;
pub mod outcome;
pub mod empty;
pub mod random;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export common types
pub use self::boundary::ExecutionBoundary;
pub use self::handler::{EffectHandler, HandlerResult};
pub use self::types::ResourceChange;
pub use self::continuation::StatusToken;

/// Unique identifier for an effect
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectId(String);

impl EffectId {
    /// Create a new unique ID
    pub fn new_unique() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create an ID from a string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
    pub execution_id: Option<uuid::Uuid>,
    
    /// Resource changes resulting from the effect
    pub resource_changes: Vec<ResourceChange>,
    
    /// Metadata about the execution
    pub metadata: HashMap<String, String>,
}

impl EffectOutcome {
    /// Create a new successful outcome
    pub fn success(id: EffectId) -> Self {
        Self {
            id,
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: Some(uuid::Uuid::new_v4()),
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new failed outcome
    pub fn failure(id: EffectId, error: impl Into<String>) -> Self {
        Self {
            id,
            success: false,
            data: HashMap::new(),
            error: Some(error.into()),
            execution_id: Some(uuid::Uuid::new_v4()),
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a key-value pair to the output data
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
    
    /// Add a resource change
    pub fn with_resource_change(mut self, change: ResourceChange) -> Self {
        self.resource_changes.push(change);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Execution status of an async effect
#[derive(Debug)]
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

/// Core trait for effects
pub trait Effect: Send + Sync + fmt::Debug {
    /// Get the unique ID of this effect
    fn id(&self) -> &EffectId;
    
    /// Get the name of the effect
    fn name(&self) -> &str;
    
    /// Get a display name for this effect
    fn display_name(&self) -> String;
    
    /// Get the description of the effect
    fn description(&self) -> String;
    
    /// Execute the effect
    fn execute(&self, context: &EffectContext) -> Result<EffectOutcome>;
    
    /// Check if this effect requires authorization
    fn requires_authorization(&self) -> bool {
        true
    }
    
    /// Get the required capabilities for this effect
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
        Vec::new()
    }
    
    /// Get the dependencies of this effect
    fn dependencies(&self) -> Vec<EffectId> {
        Vec::new()
    }
    
    /// Check if this effect can execute in the given boundary
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool;
    
    /// Get the preferred execution boundary for this effect
    fn preferred_boundary(&self) -> ExecutionBoundary;
    
    /// Get display parameters that describe what this effect does
    fn display_parameters(&self) -> HashMap<String, String>;
    
    /// Get the fact dependencies for this effect
    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        Vec::new()
    }
    
    /// Get the fact snapshot for this effect
    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        None
    }
    
    /// Validate fact dependencies for this effect
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        Ok(())
    }
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
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

#[async_trait]
impl AsyncEffect for EmptyEffect {
    async fn execute_async(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        Ok(EffectOutcome::success(self.id.clone()))
    }
}

// Re-exports
pub use self::types::*;
pub use self::outcome::*;
pub use self::boundary::*;
pub use self::empty::*;

pub mod templates;
pub mod effect_id;
pub mod empty_effect;

// Re-exports
pub use effect_id::EffectId;
pub use empty_effect::EmptyEffect;

/// Outcome of an effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectOutcome {
    /// ID of the effect
    pub id: String,
    
    /// Whether the effect executed successfully
    pub success: bool,
    
    /// Result data from the effect
    pub data: HashMap<String, serde_json::Value>,
    
    /// Error message if the effect failed
    pub error: Option<String>,
    
    /// Execution ID for tracing
    pub execution_id: Option<String>,
    
    /// Changes to resources
    pub resource_changes: Vec<ResourceChange>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Change to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChange {
    /// Resource ID
    pub resource_id: String,
    
    /// Type of change
    pub change_type: ResourceChangeType,
    
    /// New value if applicable
    pub new_value: Option<serde_json::Value>,
}

/// Type of resource change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceChangeType {
    Created,
    Updated,
    Deleted,
    StateChanged,
}

/// Error that can occur during effect execution
#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("Effect not found: {0}")]
    NotFound(String),
    
    #[error("Effect execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Required service not available: {0}")]
    ServiceNotAvailable(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for effect operations
pub type EffectResult<T> = std::result::Result<T, EffectError>;

/// Context for effect execution
#[derive(Debug, Clone, Default)]
pub struct EffectContext {
    /// Execution ID for tracing
    pub execution_id: Option<String>,
    
    /// Invoker of the effect
    pub invoker: Option<crate::address::Address>,
    
    /// Domains in scope
    pub domains: Vec<crate::types::DomainId>,
    
    /// Capabilities available
    pub capabilities: Vec<String>,
    
    /// Resource manager
    pub resource_manager: Option<Arc<crate::resource::manager::ResourceManager>>,
    
    /// Authorization service
    pub authorization_service: Option<Arc<dyn std::any::Any + Send + Sync>>,
    
    /// Time service
    pub time_service: Option<Arc<dyn std::any::Any + Send + Sync>>,
    
    /// Boundary manager
    pub boundary_manager: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl EffectContext {
    /// Get the authorization service
    pub fn get_authorization_service<T: 'static>(&self) -> EffectResult<&T> {
        self.authorization_service
            .as_ref()
            .and_then(|service| service.downcast_ref::<T>())
            .ok_or_else(|| EffectError::ServiceNotAvailable("Authorization service not available".to_string()))
    }
    
    /// Get the time service
    pub fn get_time_service<T: 'static>(&self) -> EffectResult<&T> {
        self.time_service
            .as_ref()
            .and_then(|service| service.downcast_ref::<T>())
            .ok_or_else(|| EffectError::ServiceNotAvailable("Time service not available".to_string()))
    }
    
    /// Get the boundary manager
    pub fn get_boundary_manager<T: 'static>(&self) -> EffectResult<&T> {
        self.boundary_manager
            .as_ref()
            .and_then(|manager| manager.downcast_ref::<T>())
            .ok_or_else(|| EffectError::ServiceNotAvailable("Boundary manager not available".to_string()))
    }
}

/// Effect trait for stateful transformations
#[async_trait]
pub trait Effect: Send + Sync {
    /// Get the ID of this effect
    fn id(&self) -> &str;
    
    /// Get the display name of this effect
    fn display_name(&self) -> String;
    
    /// Get a description of this effect
    fn description(&self) -> String;
    
    /// Execute this effect
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome>;
}

/// Create a composite effect from multiple effects
pub fn create_composite_effect(
    effects: Vec<Arc<dyn Effect>>,
    description: String,
) -> Arc<dyn Effect> {
    // Implementation would create a composite effect
    // that executes all the given effects in sequence
    // For now, return a simple empty effect
    Arc::new(EmptyEffect::with_description(description))
}

/// Execution boundary for effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBoundary {
    /// Boundary ID
    pub id: String,
    
    /// Boundary type
    pub boundary_type: String,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionBoundary {
    /// Create a new execution boundary
    pub fn new(id: String) -> Self {
        Self {
            id,
            boundary_type: "default".to_string(),
            metadata: HashMap::new(),
        }
    }
} 