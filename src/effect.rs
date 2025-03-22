// Effect system for Causality
//
// The effect system allows for algebraic effects to be used in the Causality
// system, providing a way to inject side effects and requirements.

use std::fmt;
use crate::types::{ResourceId, DomainId};
use crate::log::fact_snapshot::{FactSnapshot, FactId, FactDependency, FactDependencyType};
use std::collections::HashMap;
use std::sync::Arc;
use crate::error::{Error, Result};
use crate::log::fact_types::FactType;
#[cfg(feature = "code-repo")]
use crate::effect_adapters::{hash::Hash as CodeHash, repository::CodeRepository, definition::CodeDefinition};
use crate::execution::context::{ExecutionContext, ExecutionEvent, Value};
use crate::execution::trace::ExecutionTracer;
use crate::execution::security::SecuritySandbox;

// Module declarations
pub mod handler;
pub mod continuation;
pub mod types;
pub mod factory;
pub mod dependency;
pub mod snapshot;

// Re-export core types
pub use types::EffectType;
pub use handler::{EffectHandler, SharedHandler, NoopHandler, shared, compose};
pub use continuation::{Continuation, map, and_then, constant, identity};
pub use factory::{deposit, withdrawal, observation};
pub use dependency::{FactDependency as DependencyFactDependency, EffectDependency, DependencySet};
pub use snapshot::{FactSnapshot as SnapshotFactSnapshot, SystemSnapshot, SnapshotManager};

// Marker trait for effect serialize/deserialize
pub trait SerializableEffect: Effect {}

// Base Effect trait that can be used as an object
pub trait Effect: Send + Sync {
    /// The output type of this effect
    type Output;
    
    /// Get the type of this effect
    fn get_type(&self) -> EffectType;
    
    /// Get a debug representation of this effect
    fn as_debug(&self) -> &dyn std::fmt::Debug;
    
    /// Clone this effect
    fn clone_box(&self) -> Box<dyn Effect<Output = Self::Output>>;
    
    /// Get the resources affected by this effect
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get the domains involved in this effect
    fn domains(&self) -> Vec<DomainId>;
    
    /// Execute this effect using the given handler
    fn execute(self, handler: &dyn EffectHandler) -> Self::Output;
    
    /// Get the fact dependencies for this effect (default implementation)
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        Vec::new()
    }
    
    /// Get the fact snapshot for this effect (default implementation)
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        None
    }
}

// Implement Clone for Box<dyn Effect> via clone_box
impl<R> Clone for Box<dyn Effect<Output = R>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Implement Debug for Box<dyn Effect> via as_debug
impl<R> fmt::Debug for Box<dyn Effect<Output = R>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_debug().fmt(f)
    }
}

/// Trait extension for effects with fact dependencies
pub trait EffectWithFactDependencies: Effect {
    /// Add a fact dependency to this effect
    fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    );
    
    /// Add multiple fact dependencies to this effect
    fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>);
    
    /// Set the entire fact snapshot for this effect
    fn with_fact_snapshot(&mut self, snapshot: FactSnapshot);
    
    /// Validate that all required fact dependencies are present
    fn validate_fact_dependencies(&self) -> Result<()>;
}

// Direct API for content-addressed effects (moved from integration module)
#[cfg(feature = "code-repo")]
/// Represents a content-addressed effect
#[derive(Debug, Clone)]
pub struct ContentAddressedEffect {
    /// The content hash of the effect
    pub hash: CodeHash,
    /// The effect type/name
    pub effect_type: String,
    /// Parameter schema for the effect
    pub parameter_schema: HashMap<String, String>,
    /// Result type schema
    pub result_schema: String,
    /// Is this effect pure (no side effects)
    pub is_pure: bool,
    /// Resource requirements for this effect
    pub resource_requirements: EffectResourceRequirements,
    /// Fact dependencies for this effect
    pub fact_dependencies: Vec<FactDependency>,
    /// Fact snapshot this effect depends on
    pub fact_snapshot: Option<FactSnapshot>,
}

#[cfg(feature = "code-repo")]
/// Resource requirements for effect execution
#[derive(Debug, Clone)]
pub struct EffectResourceRequirements {
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// CPU usage in milliseconds
    pub cpu_millis: usize,
    /// I/O operations count
    pub io_operations: usize,
    /// Secondary effects that might be triggered
    pub secondary_effects: usize,
}

#[cfg(feature = "code-repo")]
impl ContentAddressedEffect {
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(
        mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) -> Self {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.fact_dependencies.push(dependency);
        self
    }
    
    /// Set the fact snapshot for this effect
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.fact_snapshot = Some(snapshot);
        self
    }
}

#[cfg(feature = "code-repo")]
/// Interface for effect integration
pub trait EffectIntegrator: Send + Sync {
    /// Register a content-addressed effect
    fn register_effect(
        &self, 
        effect: ContentAddressedEffect,
        handler_hash: CodeHash,
    ) -> Result<()>;
    
    /// Apply an effect in a content-addressed manner
    fn apply_effect(
        &self,
        context: &mut ExecutionContext,
        effect_type: &str,
        parameters: HashMap<String, Value>,
    ) -> Result<Value>;
    
    /// Get the content hash for an effect
    fn get_effect_hash(
        &self,
        effect_type: &str,
    ) -> Result<CodeHash>;
    
    /// List all registered effects
    fn list_effects(&self) -> Result<Vec<ContentAddressedEffect>>;
    
    /// Validate an effect against security sandbox
    fn validate_effect(
        &self,
        effect: &ContentAddressedEffect,
        sandbox: &SecuritySandbox,
    ) -> Result<bool>;
}

#[cfg(feature = "code-repo")]
/// Implementation of effect integration
pub struct ContentAddressedEffectIntegrator {
    /// The code repository
    repository: Arc<CodeRepository>,
    /// Registered effects
    effects: RwLock<HashMap<String, ContentAddressedEffect>>,
    /// Effect handlers
    handlers: RwLock<HashMap<String, CodeHash>>,
    /// Execution tracer
    tracer: Arc<ExecutionTracer>,
}

#[cfg(feature = "code-repo")]
impl ContentAddressedEffectIntegrator {
    /// Create a new effect integrator
    pub fn new(
        repository: Arc<CodeRepository>,
        tracer: Arc<ExecutionTracer>,
    ) -> Self {
        ContentAddressedEffectIntegrator {
            repository,
            effects: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
            tracer,
        }
    }
}

#[cfg(feature = "code-repo")]
// Implementation of EffectIntegrator for ContentAddressedEffectIntegrator
impl EffectIntegrator for ContentAddressedEffectIntegrator {
    fn register_effect(
        &self, 
        effect: ContentAddressedEffect,
        handler_hash: CodeHash,
    ) -> Result<()> {
        let mut effects = self.effects.write().map_err(|_| 
            Error::LockError("Failed to acquire effects write lock".to_string()))?;
        
        let mut handlers = self.handlers.write().map_err(|_| 
            Error::LockError("Failed to acquire handlers write lock".to_string()))?;
        
        effects.insert(effect.effect_type.clone(), effect);
        handlers.insert(effect.effect_type.clone(), handler_hash);
        
        Ok(())
    }
    
    fn apply_effect(
        &self,
        context: &mut ExecutionContext,
        effect_type: &str,
        parameters: HashMap<String, Value>,
    ) -> Result<Value> {
        // Get the effect handler
        let handler_hash = {
            let handlers = self.handlers.read().map_err(|_| 
                Error::LockError("Failed to acquire handlers read lock".to_string()))?;
            
            handlers.get(effect_type).cloned().ok_or_else(|| 
                Error::NotFound(format!("Effect handler not found: {}", effect_type)))?
        };
        
        // Get the handler code
        let handler_code = self.repository.get_code(&handler_hash)?;
        
        // Log the effect application - simplified for compatibility
        self.tracer.trace_event(
            context.id(),
            ExecutionEvent::EffectApplied(effect_type.to_string())
        )?;
        
        // Execute the handler code with the parameters
        // This is simplified and would actually involve more complex execution
        let result = Value::Null; // Placeholder for actual execution
        
        Ok(result)
    }
    
    fn get_effect_hash(
        &self,
        effect_type: &str,
    ) -> Result<CodeHash> {
        let effects = self.effects.read().map_err(|_| 
            Error::LockError("Failed to acquire effects read lock".to_string()))?;
        
        let effect = effects.get(effect_type).ok_or_else(|| 
            Error::NotFound(format!("Effect not found: {}", effect_type)))?;
        
        Ok(effect.hash.clone())
    }
    
    fn list_effects(&self) -> Result<Vec<ContentAddressedEffect>> {
        let effects = self.effects.read().map_err(|_| 
            Error::LockError("Failed to acquire effects read lock".to_string()))?;
        
        Ok(effects.values().cloned().collect())
    }
    
    fn validate_effect(
        &self,
        effect: &ContentAddressedEffect,
        sandbox: &SecuritySandbox,
    ) -> Result<bool> {
        // In a real implementation, this would validate the effect against security constraints
        // For now, we'll just return true as a placeholder
        Ok(true)
    }
}

/// Validates fact dependencies for effects
pub struct FactDependencyValidator {
    /// Map of fact IDs to presence status
    fact_cache: HashMap<FactId, bool>,
}

impl FactDependencyValidator {
    /// Create a new fact dependency validator
    pub fn new() -> Self {
        FactDependencyValidator {
            fact_cache: HashMap::new(),
        }
    }
    
    /// Add a fact to the validator cache
    pub fn add_fact(&mut self, fact_id: FactId) {
        self.fact_cache.insert(fact_id, true);
    }
    
    /// Check if all required facts are present for an effect
    pub fn validate_required_facts(&self, effect: &dyn Effect) -> Result<()> {
        let dependencies = effect.fact_dependencies();
        
        for dependency in dependencies {
            if dependency.is_required() && !self.fact_cache.contains_key(&dependency.fact_id) {
                return Err(Error::ValidationError(format!(
                    "Required fact dependency not found: {}",
                    dependency.fact_id.0
                )));
            }
        }
        
        Ok(())
    }
    
    /// Validate an entire fact snapshot
    pub fn validate_snapshot(&self, snapshot: &FactSnapshot) -> Result<()> {
        for fact_id in &snapshot.observed_facts {
            if !self.fact_cache.contains_key(fact_id) {
                return Err(Error::ValidationError(format!(
                    "Fact in snapshot not found: {}",
                    fact_id.0
                )));
            }
        }
        
        Ok(())
    }
}
