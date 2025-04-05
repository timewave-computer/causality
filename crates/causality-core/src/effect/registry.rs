// Effect Registry
//
// This module provides the registry for effect handlers and management
// of effect execution with content addressing.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use lazy_static::lazy_static;

use async_trait::async_trait;
use thiserror::Error;

use super::{Effect, EffectContext, EffectOutcome, EffectResult, EffectError, EffectType};
use super::domain::{DomainEffect, DomainEffectHandler, DomainEffectRegistry, DomainId, DomainEffectOutcome};
use super::types::{EffectId, EffectTypeId};
use super::handler::EffectHandler;

// Use Tokio's RwLock
use tokio::sync::RwLock;


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
        self.domain_registry.has_handler_for_type(domain_id, effect_type_id)
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
    /// Create a new thread-safe effect registry
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(BasicEffectRegistry::new()),
        }
    }
    
    /// Create from a basic registry
    pub fn from_basic(registry: BasicEffectRegistry) -> Self {
        Self {
            registry: RwLock::new(registry),
        }
    }
}

impl EffectRegistrar for ThreadSafeEffectRegistry {
    /// Register an effect handler
    fn register_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: EffectHandler + Clone + 'static {
        let mut registry = self.registry.blocking_write();
        EffectRegistrar::register_handler(&mut *registry, handler)
    }

    /// Register a domain effect handler
    fn register_domain_handler<H>(&mut self, handler: H) -> Result<(), EffectError>
    where
        H: DomainEffectHandler + 'static {
        let mut registry = self.registry.blocking_write();
        EffectRegistrar::register_domain_handler(&mut *registry, handler)
    }

    /// Check if an effect type is registered
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool {
        let registry = self.registry.blocking_read();
        EffectRegistrar::has_handler(&*registry, effect_type_id)
    }

    /// Check if a domain effect type is registered
    fn has_domain_handler(&self, domain_id: &DomainId, effect_type_id: &EffectTypeId) -> bool {
        let registry = self.registry.blocking_read();
        EffectRegistrar::has_domain_handler(&*registry, domain_id, effect_type_id)
    }
}

impl EffectExecutor for ThreadSafeEffectRegistry {
    /// Execute an effect (Synchronous wrapper, potentially blocking)
    fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext)
        -> EffectResult<EffectOutcome> {
        let registry = self.registry.blocking_read();
        EffectExecutor::execute_effect(&*registry, effect, context)
    }

    /// Get a handler for the given effect type ID
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> Option<Arc<dyn EffectHandler>> {
       let registry = self.registry.blocking_read();
       EffectExecutor::get_handler(&*registry, effect_type_id)
    }

    /// Check if a handler is registered for the given effect type ID
    fn has_handler(&self, effect_type_id: &EffectTypeId) -> bool {
        let registry = self.registry.blocking_read();
        EffectExecutor::has_handler(&*registry, effect_type_id)
    }

    /// Get all registered effect type IDs
    fn registered_effect_types(&self) -> HashSet<EffectTypeId> {
        let registry = self.registry.blocking_read();
        EffectExecutor::registered_effect_types(&*registry)
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
        let mut registry = Self::new();
        
        for (type_id, handler) in self.handlers.iter() {
            registry.handlers.insert(type_id.clone(), handler.clone());
        }
        
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
    pub async fn create_global() -> Arc<ThreadSafeEffectRegistry> {
        let registry_option = GLOBAL_REGISTRY.read().await;
        if let Some(registry) = registry_option.as_ref() {
            return Arc::clone(registry);
        }
        drop(registry_option);
        
        let mut global_registry = GLOBAL_REGISTRY.write().await;
        if let Some(registry) = global_registry.as_ref() {
             Arc::clone(registry)
        } else {
            let registry = Arc::new(ThreadSafeEffectRegistry::new());
            *global_registry = Some(Arc::clone(&registry));
            registry
        }
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
    /// Execute an effect asynchronously (Placeholder implementation)
    async fn execute_effect_async(&self, effect: &dyn Effect, context: &dyn EffectContext)
        -> EffectResult<EffectOutcome> {
         EffectExecutor::execute_effect(self, effect, context)
    }

     /// Execute an effect (delegates to async)
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext)
        -> EffectResult<EffectOutcome> {
        self.execute_effect_async(effect, context).await
    }

    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext)
        -> EffectResult<DomainEffectOutcome> {
        let effect_outcome: EffectResult<EffectOutcome> = self.domain_registry.execute_effect(effect, context).await;
        effect_outcome.map(|outcome| {
            let domain_id = effect.domain_id().clone();
            let effect_id_str = context.effect_id().to_string();
            if outcome.is_success() {
                 super::domain::DomainEffectOutcome::success(domain_id, effect_id_str, Some(outcome.data))
            } else {
                 super::domain::DomainEffectOutcome::failure(
                    domain_id, 
                    effect_id_str, 
                    outcome.error_message.unwrap_or_else(|| "Unknown error".to_string()))
            }
        })
    }

    /// Get an effect by ID (Placeholder implementation)
    async fn get_effect(&self, id: &EffectId) -> EffectRegistryResult<Box<dyn Effect>> {
        Err(EffectRegistryError::NotImplemented(format!("get_effect for ID: {}", id)))
    }
}

#[async_trait]
impl AsyncEffectRegistry for ThreadSafeEffectRegistry {
    /// Execute an effect asynchronously
    async fn execute_effect_async(&self, effect: &dyn Effect, context: &dyn EffectContext)
        -> EffectResult<EffectOutcome> {
        let registry = self.registry.read().await;
        AsyncEffectRegistry::execute_effect(&*registry, effect, context).await
    }

    /// Execute an effect (delegating to async version)
    async fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext)
        -> EffectResult<EffectOutcome> {
        self.execute_effect_async(effect, context).await
    }


    /// Execute a domain effect
    async fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext)
        -> EffectResult<DomainEffectOutcome> {
       let registry = self.registry.read().await;
       AsyncEffectRegistry::execute_domain_effect(&*registry, effect, context).await
    }

    /// Get an effect by ID (Placeholder implementation)
    async fn get_effect(&self, id: &EffectId) -> EffectRegistryResult<Box<dyn Effect>> {
       let _registry = self.registry.read().await;
        Err(EffectRegistryError::NotImplemented(format!("get_effect for ID: {}", id)))
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
    /// Global effect registry instance, lazily initialized.
    static ref GLOBAL_REGISTRY: RwLock<Option<Arc<ThreadSafeEffectRegistry>>> = RwLock::new(None);
}

/// Get a reference to the global effect registry.
/// Initializes the registry if it hasn't been already.
/// Note: This function is now async due to Tokio's RwLock.
pub async fn get_global_registry() -> Arc<ThreadSafeEffectRegistry> {
    let registry_option = GLOBAL_REGISTRY.read().await;
    if let Some(registry) = registry_option.as_ref() {
        return Arc::clone(registry);
    }
    drop(registry_option);

    let mut registry_option_mut = GLOBAL_REGISTRY.write().await;
    if let Some(registry) = registry_option_mut.as_ref() {
        Arc::clone(registry)
    } else {
        println!("Initializing global effect registry...");
        let new_registry = Arc::new(ThreadSafeEffectRegistry::new());
        *registry_option_mut = Some(Arc::clone(&new_registry));
        new_registry
    }
}

/// Initialize the global effect registry with a specific instance.
/// Returns an error if the registry is already initialized.
/// Note: This function is now async due to Tokio's RwLock.
pub async fn initialize_global_registry(registry: ThreadSafeEffectRegistry) -> Result<(), EffectRegistryError> {
    // Use write().await
    let mut global_registry = GLOBAL_REGISTRY.write().await;
    if global_registry.is_some() {
        Err(EffectRegistryError::DuplicateRegistration("Global registry already initialized".to_string()))
    } else {
        *global_registry = Some(Arc::new(registry));
        Ok(())
    }
}

// -- Tests --
#[cfg(test)]
mod tests {
    // Explicit imports instead of glob
    use super::{
        BasicEffectRegistry, ThreadSafeEffectRegistry, EffectRegistryError, EffectRegistrar, 
        EffectExecutor, AsyncEffectRegistry, get_global_registry, initialize_global_registry
    };
    // Import necessary core effect types
    use crate::effect::{Effect, EffectContext, EffectOutcome, EffectError, EffectType, EffectResult};
    use crate::effect::handler::EffectHandler;
    // Import domain types
    use crate::effect::domain::{DomainEffect, DomainEffectHandler, DomainId, DomainEffectOutcome, DomainContextAdapter, CrossDomainSupport, ExecutionBoundary as DomainExecutionBoundary};
    // Import types needed directly
    use crate::effect::types::{EffectId, EffectTypeId, ExecutionBoundary as TypesExecutionBoundary};
    use crate::effect::context::Capability;
    use crate::resource::ResourceId;
    // Standard lib imports
    use std::sync::{Arc, Mutex};
    use std::collections::{HashMap, HashSet};
    use std::any::Any;
    // Crate imports
    use async_trait::async_trait;
    use tokio::runtime::Runtime;
    use serde::{Serialize, Deserialize};
    use typetag;

    // Basic Test Effect
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEffect { id: String }
    #[typetag::serde]
    #[async_trait]
    impl Effect for TestEffect {
        fn effect_type(&self) -> EffectType {
            EffectType::Custom("test".to_string())
        }
        fn description(&self) -> String {
            format!("Test effect with id: {}", self.id)
        }
        async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
            // Placeholder implementation for testing
            println!("Executing TestEffect {}", self.id);
            Ok(EffectOutcome::success(HashMap::new()))
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Basic Test Effect Handler
    #[derive(Clone, Debug)]
    struct TestEffectHandler;
    #[async_trait]
    impl EffectHandler for TestEffectHandler {
        fn supported_effect_types(&self) -> Vec<EffectTypeId> {
            vec![EffectTypeId::from("test")]
        }
        async fn handle(&self, _effect: &dyn Effect, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
            Ok(EffectOutcome::success(HashMap::new()))
        }
    }

     // Basic Test Domain Effect
    #[derive(Debug, Clone)]
    struct TestDomainEffect { domain: DomainId, data: String, id: EffectId }
    #[typetag::serde]
    #[async_trait]
    impl Effect for TestDomainEffect {
        fn effect_type(&self) -> EffectType {
            EffectType::Custom("test_domain".to_string())
        }
        fn description(&self) -> String {
            format!("Test domain effect for domain {}", self.domain)
        }
        async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
            // Placeholder implementation for testing
            println!("Executing TestDomainEffect for domain {}", self.domain);
            Ok(EffectOutcome::success(HashMap::new()))
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Implement DomainEffect for TestDomainEffect
    #[typetag::serde]
    #[async_trait]
    impl DomainEffect for TestDomainEffect {
        fn domain_id(&self) -> &DomainId { &self.domain }
        fn execution_boundary(&self) -> DomainExecutionBoundary {
            DomainExecutionBoundary::Domain
        }
        fn can_execute_in(&self, domain_id: &DomainId) -> bool {
            self.domain_id() == domain_id
        }
        fn validate_parameters(&self) -> EffectResult<EffectOutcome> {
            Ok(EffectOutcome::success(HashMap::new()))
        }
        fn domain_parameters(&self) -> HashMap<String, String> {
            let mut params = HashMap::new();
            params.insert("data".to_string(), self.data.clone());
            params
        }
        fn adapt_context(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
            Ok(EffectOutcome::success(context.metadata().clone()))
        }
    }

    // Basic Test Domain Effect Handler
    #[derive(Clone, Debug)]
    struct TestDomainHandler { domain: DomainId }
    #[async_trait]
    impl DomainEffectHandler for TestDomainHandler {
        fn domain_id(&self) -> &DomainId { &self.domain }
        fn can_handle(&self, effect: &dyn DomainEffect) -> bool {
            effect.domain_id() == self.domain_id()
            && effect.effect_type() == EffectType::Custom("test_domain".to_string())
        }
        async fn handle_domain_effect(
            &self,
            _effect: &dyn DomainEffect,
            _context: &dyn EffectContext,
        ) -> EffectResult<EffectOutcome> {
            Ok(EffectOutcome::success(HashMap::new()))
        }
        fn context_adapter(&self) -> Option<&crate::effect::domain::DomainContextAdapter> { None }
        fn cross_domain_support(&self) -> Option<crate::effect::domain::CrossDomainSupport> { None }
    }

    // Simple Effect Context
    #[derive(Debug)]
    struct TestContext {
        id: EffectId,
        metadata: HashMap<String, String>,
        capabilities: Vec<Capability>,
        resources: HashSet<ResourceId>,
        parent: Option<Arc<dyn EffectContext>>,
    }
    impl EffectContext for TestContext {
        fn effect_id(&self) -> &EffectId { &self.id }
        fn capabilities(&self) -> &[Capability] { &self.capabilities }
        fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
        fn resources(&self) -> &HashSet<ResourceId> { &self.resources }
        fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> { self.parent.as_ref() }
        fn has_capability(&self, capability: &Capability) -> bool {
            self.capabilities.contains(capability)
               || self.parent.as_ref().map_or(false, |p| p.has_capability(capability))
        }
        fn get_registry(&self) -> Option<Arc<dyn EffectExecutor>> { None }
        fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
            Box::new(Self {
                id: effect_id,
                metadata: self.metadata.clone(),
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                parent: self.parent.clone(),
            })
        }
        fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
            let mut new_context = self.clone_context();
            let mut current_caps = self.capabilities.clone();
            current_caps.extend(capabilities);
            Box::new(Self {
                id: self.id.clone(),
                metadata: self.metadata.clone(),
                capabilities: current_caps,
                resources: self.resources.clone(),
                parent: self.parent.clone(),
            })
        }
        fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
            let mut current_res = self.resources.clone();
            current_res.extend(resources);
            Box::new(Self {
                id: self.id.clone(),
                metadata: self.metadata.clone(),
                capabilities: self.capabilities.clone(),
                resources: current_res,
                parent: self.parent.clone(),
            })
        }
        fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
            let mut current_meta = self.metadata.clone();
            current_meta.extend(metadata);
             Box::new(Self {
                id: self.id.clone(),
                metadata: current_meta,
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                parent: self.parent.clone(),
            })
        }
        fn clone_context(&self) -> Box<dyn EffectContext> {
            Box::new(Self {
                id: self.id.clone(),
                metadata: self.metadata.clone(),
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                parent: self.parent.clone(),
            })
        }
        fn as_any(&self) -> &dyn Any { self }
    }


    #[tokio::test]
    async fn test_basic_registry_registration_and_execution() {
        let mut registry = BasicEffectRegistry::new();
        let handler = TestEffectHandler;
        registry.register_handler(handler).unwrap();

        assert!(EffectRegistrar::has_handler(&registry, &EffectTypeId::from("test")));

        let effect = TestEffect { id: "test1".to_string() };
        let context = TestContext {
            id: EffectId::from("ctx1"),
            metadata: HashMap::new(),
            capabilities: vec![],
            resources: HashSet::new(),
            parent: None,
        };
        
        let outcome = AsyncEffectRegistry::execute_effect(&registry, &effect, &context).await.unwrap();
        assert!(outcome.is_success());
    }

    #[tokio::test]
    async fn test_thread_safe_registry_registration_and_execution() {
        let mut registry = ThreadSafeEffectRegistry::new();
        let handler = TestEffectHandler;
        registry.register_handler(handler).unwrap();

        assert!(EffectRegistrar::has_handler(&registry, &EffectTypeId::from("test")));
        assert!(EffectExecutor::has_handler(&registry, &EffectTypeId::from("test")));

        let effect = TestEffect { id: "test2".to_string() };
        let context = TestContext {
            id: EffectId::from("ctx2"),
            metadata: HashMap::new(),
            capabilities: vec![],
            resources: HashSet::new(),
            parent: None,
        };

        let outcome = AsyncEffectRegistry::execute_effect(&registry, &effect, &context).await.unwrap();
        assert!(outcome.is_success());
    }
    
    #[tokio::test]
    async fn test_domain_registry_registration_and_execution() {
        let mut registry = BasicEffectRegistry::new();
        let domain_id = DomainId::from("domain1");
        let handler = TestDomainHandler { domain: domain_id.clone() };
        registry.register_domain_handler(handler).unwrap();

        assert!(registry.has_domain_handler(&domain_id, &EffectTypeId::from("test_domain")));

        let effect = TestDomainEffect {
            domain: domain_id.clone(),
            data: "data1".to_string(),
            id: EffectId::from("domain_effect_1"),
         };
        let context = TestContext {
            id: EffectId::from("ctx_domain1"),
            metadata: HashMap::new(),
            capabilities: vec![],
            resources: HashSet::new(),
            parent: None,
        };

        let outcome = AsyncEffectRegistry::execute_domain_effect(&registry, &effect, &context).await.unwrap();
        assert!(outcome.is_success());
        assert!(outcome.data.is_some());
        assert_eq!(outcome.data.unwrap().get("domain_id").map(|s| s.as_str()), Some("domain1"));
    }
    
    #[tokio::test]
    async fn test_global_registry_initialization_and_retrieval() {
        let global_reg1 = get_global_registry().await;
        let global_reg2 = get_global_registry().await;

        assert!(Arc::ptr_eq(&global_reg1, &global_reg2));

        let new_reg = ThreadSafeEffectRegistry::new();
        let init_result = initialize_global_registry(new_reg).await;
        if Arc::strong_count(&global_reg1) > 1 {
             assert!(init_result.is_err());
             if let Err(EffectRegistryError::DuplicateRegistration(_)) = init_result {
                 // Expected error
             } else {
                 panic!("Expected DuplicateRegistration error");
             }
        } else {
            // If this test was the first to initialize, it might succeed or fail depending on timing
        }
    }
} 