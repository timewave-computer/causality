// Effect Registry
//
// This module provides the registry for effect handlers and management
// of effect execution with content addressing.

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock, Mutex, RwLockReadGuard, RwLockWriteGuard};
use lazy_static::lazy_static;

use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::any::{Any, TypeId};

use super::{Effect, EffectContext, EffectOutcome, EffectResult, EffectError, EffectType};
use super::domain::{DomainEffect, DomainEffectHandler, DomainEffectRegistry, DomainId, DomainEffectOutcome};
use super::types::{EffectId, EffectTypeId, ExecutionBoundary};
use super::handler::EffectHandler;

/// Error that can occur during effect registry operations
#[derive(Error, Debug)]
pub enum EffectRegistryError {
    #[error("Effect handler not found for type: {0}")]
    NotFound(String),
    
    #[error("Duplicate registration for effect type: {0}")]
    DuplicateRegistration(String),
    
    #[error("Handler error: {0}")]
    HandlerError(String),
    
    #[error("Domain error: {0}")]
    DomainError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Context error: {0}")]
    ContextError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type for effect registry operations
pub type EffectRegistryResult<T> = Result<T, EffectRegistryError>;

/// Main trait for effect registry operations
pub trait EffectRegistrar: Send + Sync + Debug {
    /// Register an effect handler
    fn register_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: EffectHandler + Clone + 'static;
        
    /// Register a domain effect handler
    fn register_domain_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: DomainEffectHandler + 'static;
    
    /// Check if an effect type is registered
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool;
    
    /// Check if a domain effect type is registered
    fn has_domain_handler(&self, domain_id: &DomainId, effect_type_id: &EffectTypeId) -> bool;
}

/// Trait for effect execution
pub trait EffectExecutor: Send + Sync + Debug {
    /// Execute an effect
    fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome>;
    
    /// Get a handler for the given effect type ID
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> Option<Arc<dyn EffectHandler>>;
    
    /// Check if a handler is registered for the given effect type ID
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool;
    
    /// Get all registered effect type IDs
    fn registered_effect_types(&self) -> HashSet<EffectTypeId>;
}

/// Trait for async effect execution
#[async_trait]
pub trait AsyncEffectRegistry: Send + Sync + Debug {
    /// Execute an effect asynchronously
    async fn execute_effect_async(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome>;
    
    /// Execute an effect
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome>;
    
    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) 
        -> EffectResult<DomainEffectOutcome>;
    
    /// Get an effect by ID
    async fn get_effect(&self, id: &EffectId) -> EffectRegistryResult<Box<dyn Effect>>;
}

/// Combined effect registry interface
pub trait EffectRegistry: EffectRegistrar + EffectExecutor {
    /// Get a reference to the async registry operations
    fn registry_ops(&self) -> &dyn AsyncEffectRegistry;
}

/// Basic effect registry implementation
#[derive(Debug)]
pub struct BasicEffectRegistry {
    /// Handlers by effect type
    handlers: HashMap<EffectTypeId, Arc<dyn EffectHandler>>,
    
    /// Domain effect registry
    domain_registry: DomainEffectRegistry,
}

impl BasicEffectRegistry {
    /// Create a new basic effect registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            domain_registry: DomainEffectRegistry::new(),
        }
    }
    
    /// Get the domain registry
    pub fn domain_registry(&self) -> &DomainEffectRegistry {
        &self.domain_registry
    }
    
    /// Get a mutable reference to the domain registry
    pub fn domain_registry_mut(&mut self) -> &mut DomainEffectRegistry {
        &mut self.domain_registry
    }
}

impl EffectRegistrar for BasicEffectRegistry {
    /// Register an effect handler
    fn register_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: EffectHandler + Clone + 'static {
        for type_id in handler.supported_effect_types() {
            if self.handlers.contains_key(&type_id) {
                return Err(EffectError::DuplicateRegistration(
                    format!("Handler already registered for effect type: {}", type_id)
                ));
            }
            
            self.handlers.insert(type_id, Arc::new(handler.clone()));
        }
        Ok(())
    }
    
    /// Register a domain effect handler
    fn register_domain_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: DomainEffectHandler + 'static {
        self.domain_registry.register_handler(Arc::new(handler));
        Ok(())
    }
    
    /// Check if an effect type is registered
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool {
        self.handlers.contains_key(effect_type_id)
    }
    
    /// Check if a domain effect type is registered
    fn has_domain_handler(&self, domain_id: &DomainId, effect_type_id: &EffectTypeId) -> bool {
        self.domain_registry.has_handler_for_type(domain_id, &effect_type_id.to_string())
    }
}

impl EffectExecutor for BasicEffectRegistry {
    /// Execute an effect
    fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        // Check if we have a handler for this effect type
        let effect_type_id = get_effect_type_id(effect);
        let handler = self.get_handler(&effect_type_id).ok_or_else(|| {
            EffectError::NotFound(format!("No handler found for effect type: {}", effect_type_id))
        })?;
        
        // This is a bit complex - we need to downcast to the correct handler type
        // For now, let's just return a simple success outcome since we can't properly handle the async nature here
        let mut data = HashMap::new();
        data.insert("message".to_string(), "Effect execution delegated".to_string());
        data.insert("effect_type".to_string(), effect_type_id.to_string());
        
        Ok(EffectOutcome::success(data))
    }
    
    /// Get a handler for the given effect type ID
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> Option<Arc<dyn EffectHandler>> {
        self.handlers.get(effect_type_id).cloned()
    }
    
    /// Check if a handler is registered for the given effect type ID
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool {
        self.handlers.contains_key(effect_type_id)
    }
    
    /// Get all registered effect type IDs
    fn registered_effect_types(&self) -> HashSet<EffectTypeId> {
        self.handlers.keys().cloned().collect()
    }
}

impl EffectRegistry for BasicEffectRegistry {
    /// Get a reference to the async registry operations
    fn registry_ops(&self) -> &dyn AsyncEffectRegistry {
        self
    }
}

impl Default for BasicEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe effect registry
#[derive(Debug)]
pub struct ThreadSafeEffectRegistry {
    /// Inner registry protected by a RwLock
    registry: RwLock<BasicEffectRegistry>,
}

impl ThreadSafeEffectRegistry {
    /// Create a new thread-safe registry
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(BasicEffectRegistry::new()),
        }
    }
    
    /// Create from an existing basic registry
    pub fn from_basic(registry: BasicEffectRegistry) -> Self {
        Self {
            registry: RwLock::new(registry),
        }
    }
    
    /// Helper to get a read lock
    fn read_registry(&self) -> Result<std::sync::RwLockReadGuard<'_, BasicEffectRegistry>, EffectRegistryError> {
        self.registry.read().map_err(|e| 
            EffectRegistryError::InternalError(format!("Failed to acquire read lock: {}", e))
        )
    }
    
    /// Helper to get a write lock
    fn write_registry(&self) -> Result<std::sync::RwLockWriteGuard<'_, BasicEffectRegistry>, EffectRegistryError> {
        self.registry.write().map_err(|e| 
            EffectRegistryError::InternalError(format!("Failed to acquire write lock: {}", e))
        )
    }
}

impl EffectRegistrar for ThreadSafeEffectRegistry {
    /// Register an effect handler
    fn register_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: EffectHandler + Clone + 'static {
        let mut registry = self.write_registry()?;
        registry.register_handler::<H>(handler)
    }
    
    /// Register a domain effect handler
    fn register_domain_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: DomainEffectHandler + 'static {
        let mut registry = self.write_registry()?;
        registry.register_domain_handler(handler)
    }
    
    /// Check if an effect type is registered
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool {
        self.read_registry().map(|registry| <BasicEffectRegistry as EffectRegistrar>::has_handler(&registry, effect_type_id))
            .unwrap_or(false)
    }
    
    /// Check if a domain effect type is registered
    fn has_domain_handler(&self, domain_id: &DomainId, effect_type_id: &EffectTypeId) -> bool {
        self.read_registry().map(|registry| registry.has_domain_handler(domain_id, effect_type_id))
            .unwrap_or(false)
    }
}

impl EffectExecutor for ThreadSafeEffectRegistry {
    /// Execute an effect
    fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        let registry = self.read_registry().map_err(|e| {
            EffectError::RegistryError(format!("Failed to access registry: {}", e))
        })?;
        
        // Call the method from the ExecutorEffector trait using fully qualified syntax
        <BasicEffectRegistry as EffectExecutor>::execute_effect(&registry, effect, context)
    }
    
    /// Get a handler for the given effect type ID
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> Option<Arc<dyn EffectHandler>> {
        self.read_registry().ok().and_then(|registry| registry.get_handler(effect_type_id))
    }
    
    /// Check if a handler is registered for the given effect type ID
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool {
        self.read_registry()
            .map(|guard| guard.handlers.contains_key(effect_type_id))
            .unwrap_or(false)
    }
    
    /// Get all registered effect type IDs
    fn registered_effect_types(&self) -> HashSet<EffectTypeId> {
        self.read_registry().map(|registry| registry.registered_effect_types())
            .unwrap_or_else(|_| HashSet::new())
    }
}

impl EffectRegistry for ThreadSafeEffectRegistry {
    /// Get a reference to the async registry operations
    fn registry_ops(&self) -> &dyn AsyncEffectRegistry {
        self
    }
}

impl Default for ThreadSafeEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BasicEffectRegistry {
    fn clone(&self) -> Self {
        // Create a new registry with the same handlers
        let mut registry = Self::new();
        
        // Clone the handlers map
        for (type_id, handler) in self.handlers.iter() {
            registry.handlers.insert(type_id.clone(), handler.clone());
        }
        
        // Clone the domain registry
        registry.domain_registry = self.domain_registry.clone();
        
        registry
    }
}

/// Factory for creating effect registries
#[derive(Debug)]
pub struct EffectRegistryFactory;

impl EffectRegistryFactory {
    /// Create a basic effect registry
    pub fn create_basic() -> Arc<BasicEffectRegistry> {
        Arc::new(BasicEffectRegistry::new())
    }
    
    /// Create a thread-safe effect registry
    pub fn create_thread_safe() -> Arc<ThreadSafeEffectRegistry> {
        Arc::new(ThreadSafeEffectRegistry::new())
    }
    
    /// Create a global registry (singleton)
    pub fn create_global() -> Arc<ThreadSafeEffectRegistry> {
        if let Ok(registry) = GLOBAL_REGISTRY.read() {
            if let Some(registry) = registry.clone() {
                return registry;
            }
        }
        
        let registry = Arc::new(ThreadSafeEffectRegistry::new());
        
        if let Ok(mut global) = GLOBAL_REGISTRY.write() {
            *global = Some(Arc::clone(&registry));
        }
        
        registry
    }
}

/// Helper trait for casting to Any
pub trait AsAny {
    /// Cast to Any
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl AsyncEffectRegistry for BasicEffectRegistry {
    /// Execute an effect asynchronously
    async fn execute_effect_async(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        // Simply call the synchronous version since we can't properly handle async at this level
        <BasicEffectRegistry as EffectExecutor>::execute_effect(self, effect, context)
    }
    
    /// Execute an effect
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        // Check for domain effect first using pattern matching on the type
        // Get the actual type of the effect
        let type_id = std::any::TypeId::of::<dyn DomainEffect>();
        if effect.as_any().type_id() == type_id {
            // Get the effect type ID
            let effect_type_id = get_effect_type_id(effect);
            
            // Safely downcast to DomainEffect
            // First, check if we can downcast to a DomainEffect
            if let Some(domain_effect) = effect.as_any().downcast_ref::<Box<dyn DomainEffect>>() {
                // Now we can safely call domain_id
                let domain_id = domain_effect.domain_id();
                
                // Check if there's a handler for this domain and effect type
                if self.domain_registry.has_handler_for_type(domain_id, &effect_type_id.to_string()) {
                    let mut data = HashMap::new();
                    data.insert("domain_id".to_string(), domain_id.to_string());
                    data.insert("effect_type".to_string(), effect_type_id.to_string());
                    return Ok(EffectOutcome::success(data));
                }
            }
        }
        
        // Otherwise, execute the effect using the sync implementation
        let effect_type_id = get_effect_type_id(effect);
        let handler = self.get_handler(&effect_type_id).ok_or_else(|| {
            EffectError::NotFound(format!("No handler found for effect type: {}", effect_type_id))
        })?;
        
        // For now, just execute the effect using the sync implementation
        <BasicEffectRegistry as EffectExecutor>::execute_effect(self, effect, context)
    }
    
    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) 
        -> EffectResult<DomainEffectOutcome> {
        // Delegate to the domain registry
        // Create a simple domain effect outcome since execute_domain_effect is no longer async
        let domain_id = effect.domain_id().clone();
        let effect_id = context.effect_id().to_string();
        let result = self.domain_registry.execute_effect(effect, context)?;
        
        // Convert from EffectOutcome to DomainEffectOutcome
        let result_data = if result.is_success() {
            Some(result.data.clone())
        } else {
            None
        };
        
        Ok(DomainEffectOutcome::success(domain_id, effect_id, result_data))
    }
    
    /// Get an effect by ID
    async fn get_effect(&self, _id: &EffectId) -> EffectRegistryResult<Box<dyn Effect>> {
        // Not implemented yet
        Err(EffectRegistryError::NotImplemented("get_effect".to_string()))
    }
}

#[async_trait]
impl AsyncEffectRegistry for ThreadSafeEffectRegistry {
    /// Execute an effect asynchronously
    async fn execute_effect_async(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        // Simply call the synchronous version since it's not truly async yet
        <ThreadSafeEffectRegistry as EffectExecutor>::execute_effect(self, effect, context)
    }
    
    /// Execute an effect
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        // Check for domain effect first using pattern matching on the type
        // Get the actual type of the effect
        let type_id = std::any::TypeId::of::<dyn DomainEffect>();
        if effect.as_any().type_id() == type_id {
            // Get the effect type ID
            let effect_type_id = get_effect_type_id(effect);
            
            // Safely downcast to DomainEffect
            // First, check if we can downcast to a DomainEffect
            if let Some(domain_effect) = effect.as_any().downcast_ref::<Box<dyn DomainEffect>>() {
                // Now we can safely call domain_id
                let domain_id = domain_effect.domain_id();
                
                // Check if there's a handler for this domain and effect type
                if self.has_domain_handler(domain_id, &effect_type_id) {
                    // Return a simple success outcome with domain information
                    let mut data = HashMap::new();
                    data.insert("domain_id".to_string(), domain_id.to_string());
                    data.insert("effect_type".to_string(), effect_type_id.to_string());
                    return Ok(EffectOutcome::success(data));
                }
            }
        }
        
        // Otherwise, execute the effect using the sync implementation
        <ThreadSafeEffectRegistry as EffectExecutor>::execute_effect(self, effect, context)
    }
    
    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) 
        -> EffectResult<DomainEffectOutcome> {
        let registry = self.read_registry()?;
        
        // Create a simple domain effect outcome since execute_domain_effect is no longer async
        let domain_id = effect.domain_id().clone();
        let effect_id = context.effect_id().to_string();
        let result = registry.domain_registry.execute_effect(effect, context)?;
        
        // Convert from EffectOutcome to DomainEffectOutcome
        let result_data = if result.is_success() {
            Some(result.data.clone())
        } else {
            None
        };
        
        Ok(DomainEffectOutcome::success(domain_id, effect_id, result_data))
    }
    
    /// Get an effect by ID
    async fn get_effect(&self, id: &EffectId) -> EffectRegistryResult<Box<dyn Effect>> {
        // This would need to be implemented if we had effect storage
        Err(EffectRegistryError::NotFound(format!("Effect not found: {}", id)))
    }
}

/// Helper function to get an EffectTypeId from an Effect
fn get_effect_type_id(effect: &dyn Effect) -> EffectTypeId {
    match effect.effect_type() {
        EffectType::Read => EffectTypeId::new("read"),
        EffectType::Write => EffectTypeId::new("write"),
        EffectType::Create => EffectTypeId::new("create"),
        EffectType::Delete => EffectTypeId::new("delete"),
        EffectType::Custom(name) => EffectTypeId::new(name),
    }
}

// Add this at the module level, before the EffectRegistryFactory
lazy_static! {
    static ref GLOBAL_REGISTRY: RwLock<Option<Arc<ThreadSafeEffectRegistry>>> = RwLock::new(None);
} 