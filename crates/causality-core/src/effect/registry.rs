// Effect Registry
//
// This module provides the registry for effect handlers and management
// of effect execution with content addressing.

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use thiserror::Error;

use super::{Effect, EffectContext, EffectOutcome, EffectResult};
use super::domain::{DomainEffect, DomainEffectHandler, DomainEffectRegistry, DomainId};
use super::types::{EffectId, EffectTypeId, ExecutionBoundary};

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
}

/// Result type for effect registry operations
pub type EffectRegistryResult<T> = Result<T, EffectRegistryError>;

/// Effect handler trait
#[async_trait]
pub trait EffectHandler: Send + Sync + Debug {
    /// Get the effect type ID this handler can process
    fn effect_type_id(&self) -> EffectTypeId;
    
    /// Check if this handler can handle the given effect
    fn can_handle(&self, effect: &dyn Effect) -> bool {
        self.effect_type_id() == effect.type_id()
    }
    
    /// Handle the effect
    async fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
}

/// Registry for effect handlers
pub trait EffectRegistry: Send + Sync + Debug {
    /// Register an effect handler
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) -> EffectRegistryResult<()>;
    
    /// Get a handler for the given effect type
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> EffectRegistryResult<Arc<dyn EffectHandler>>;
    
    /// Execute an effect
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Register a domain effect handler
    fn register_domain_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) -> EffectRegistryResult<()>;
    
    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Clone the registry
    fn clone_registry(&self) -> Arc<dyn EffectRegistry>;
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

impl EffectRegistry for BasicEffectRegistry {
    /// Register an effect handler
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) -> EffectRegistryResult<()> {
        let type_id = handler.effect_type_id();
        
        if self.handlers.contains_key(&type_id) {
            return Err(EffectRegistryError::DuplicateRegistration(
                format!("Handler already registered for effect type: {}", type_id)
            ));
        }
        
        self.handlers.insert(type_id, handler);
        Ok(())
    }
    
    /// Get a handler for the given effect type
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> EffectRegistryResult<Arc<dyn EffectHandler>> {
        self.handlers.get(effect_type_id)
            .cloned()
            .ok_or_else(|| EffectRegistryError::NotFound(
                format!("No handler found for effect type: {}", effect_type_id)
            ))
    }
    
    /// Execute an effect
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Check for domain effect first
        if let Some(domain_effect) = effect.as_any().downcast_ref::<dyn DomainEffect>() {
            return self.execute_domain_effect(domain_effect, context).await;
        }
        
        let handler = self.get_handler(&effect.type_id())
            .map_err(|e| super::EffectError::ExecutionError(
                format!("Handler lookup error: {}", e)
            ))?;
        
        handler.handle(effect, context).await
    }
    
    /// Register a domain effect handler
    fn register_domain_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) -> EffectRegistryResult<()> {
        self.domain_registry.register_handler(handler);
        Ok(())
    }
    
    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        self.domain_registry.execute_domain_effect(effect, context).await
    }
    
    /// Clone the registry
    fn clone_registry(&self) -> Arc<dyn EffectRegistry> {
        let mut new_registry = BasicEffectRegistry::new();
        
        // Clone handlers
        for (type_id, handler) in &self.handlers {
            new_registry.handlers.insert(type_id.clone(), handler.clone());
        }
        
        // TODO: Clone domain registry handlers once they support cloning
        
        Arc::new(new_registry)
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
    /// Create a new thread-safe effect registry
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
    
    /// Get a reference to the inner registry
    fn read_registry(&self) -> Result<std::sync::RwLockReadGuard<'_, BasicEffectRegistry>, EffectRegistryError> {
        self.registry.read()
            .map_err(|e| EffectRegistryError::InternalError(
                format!("Failed to acquire read lock: {}", e)
            ))
    }
    
    /// Get a mutable reference to the inner registry
    fn write_registry(&self) -> Result<std::sync::RwLockWriteGuard<'_, BasicEffectRegistry>, EffectRegistryError> {
        self.registry.write()
            .map_err(|e| EffectRegistryError::InternalError(
                format!("Failed to acquire write lock: {}", e)
            ))
    }
}

impl EffectRegistry for ThreadSafeEffectRegistry {
    /// Register an effect handler
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) -> EffectRegistryResult<()> {
        self.write_registry()?.register_handler(handler)
    }
    
    /// Get a handler for the given effect type
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> EffectRegistryResult<Arc<dyn EffectHandler>> {
        self.read_registry()?.get_handler(effect_type_id)
    }
    
    /// Execute an effect
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // For async execution, we need to get a clone of the handler to avoid holding the lock
        let handler = if let Some(domain_effect) = effect.as_any().downcast_ref::<dyn DomainEffect>() {
            return self.execute_domain_effect(domain_effect, context).await;
        } else {
            self.get_handler(&effect.type_id())
                .map_err(|e| super::EffectError::ExecutionError(
                    format!("Handler lookup error: {}", e)
                ))?
        };
        
        handler.handle(effect, context).await
    }
    
    /// Register a domain effect handler
    fn register_domain_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) -> EffectRegistryResult<()> {
        self.write_registry()?.register_domain_handler(handler)
    }
    
    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        let registry = self.read_registry()?;
        registry.execute_domain_effect(effect, context).await
    }
    
    /// Clone the registry
    fn clone_registry(&self) -> Arc<dyn EffectRegistry> {
        let registry_guard = self.read_registry()
            .expect("Failed to acquire read lock for cloning");
        
        let cloned_basic = registry_guard.clone_registry();
        if let Some(basic) = cloned_basic.as_any().downcast_ref::<BasicEffectRegistry>() {
            let cloned = basic.clone();
            Arc::new(ThreadSafeEffectRegistry::from_basic(cloned))
        } else {
            // Fallback to creating a new empty registry
            Arc::new(ThreadSafeEffectRegistry::new())
        }
    }
}

impl Default for ThreadSafeEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BasicEffectRegistry {
    fn clone(&self) -> Self {
        let mut new_registry = BasicEffectRegistry::new();
        
        // Clone handlers
        for (type_id, handler) in &self.handlers {
            new_registry.handlers.insert(type_id.clone(), handler.clone());
        }
        
        // TODO: Clone domain registry handlers
        
        new_registry
    }
}

/// Factory for creating effect registries
pub struct EffectRegistryFactory;

impl EffectRegistryFactory {
    /// Create a new basic effect registry
    pub fn create_basic() -> Arc<dyn EffectRegistry> {
        Arc::new(BasicEffectRegistry::new())
    }
    
    /// Create a new thread-safe effect registry
    pub fn create_thread_safe() -> Arc<dyn EffectRegistry> {
        Arc::new(ThreadSafeEffectRegistry::new())
    }
    
    /// Create a shared global registry
    pub fn create_global() -> Arc<dyn EffectRegistry> {
        // For a real global registry, we might want to use a singleton pattern
        // or a lazy_static to ensure there's only one instance
        Arc::new(ThreadSafeEffectRegistry::new())
    }
}

/// Extension trait for casting to Any
pub trait AsAny {
    /// Cast to Any
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
} 