use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use async_trait::async_trait;

// Sub-modules
pub mod resource;
pub mod capability;
pub mod info;
pub mod utils;
pub mod domain;
pub mod handler;
pub mod types;
pub mod registry;
pub mod context;
pub mod outcome;
pub mod runtime;

// Import from domain for convenience 
pub use domain::DomainEffect;
pub use types::{EffectId, EffectTypeId, ExecutionBoundary, Right};
pub use handler::{EffectHandler, EffectHandlerRegistry, HandlerResult};
pub use registry::{EffectRegistry, BasicEffectRegistry, ThreadSafeEffectRegistry, EffectRegistryFactory, EffectRegistrar, EffectExecutor, AsyncEffectRegistry};
pub use context::{EffectContext, BasicEffectContext, EffectContextBuilder, Capability, CapabilityError, CapabilityGrants, EffectContextError, EffectContextResult};
pub use domain::SimpleEffectContext;
pub use outcome::{EffectOutcome, EffectResult};

/// Error type for effect operations
#[derive(Debug, Error, Clone)]
pub enum EffectError {
    #[error("Missing required capability: {0}")]
    MissingCapability(String),

    #[error("Missing required resource: {0}")]
    MissingResource(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Handler not found for effect type: {0}")]
    HandlerNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Resource access denied: {0}")]
    ResourceAccessDenied(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Other error: {0}")]
    Other(String),

    #[error("Resource or object not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Duplicate registration: {0}")]
    DuplicateRegistration(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Registry error: {0}")]
    RegistryError(String),
}

impl From<registry::EffectRegistryError> for EffectError {
    fn from(err: registry::EffectRegistryError) -> Self {
        match err {
            registry::EffectRegistryError::NotFound(msg) => Self::NotFound(msg),
            registry::EffectRegistryError::DuplicateRegistration(msg) => Self::DuplicateRegistration(msg),
            registry::EffectRegistryError::HandlerError(msg) => Self::ExecutionError(msg),
            registry::EffectRegistryError::DomainError(msg) => Self::ExecutionError(msg),
            registry::EffectRegistryError::ValidationError(msg) => Self::ValidationError(msg),
            registry::EffectRegistryError::ContextError(msg) => Self::ExecutionError(msg),
            registry::EffectRegistryError::InternalError(msg) => Self::SystemError(msg),
            registry::EffectRegistryError::NotImplemented(msg) => Self::InvalidOperation(format!("Not implemented: {}", msg)),
        }
    }
}

/// The core Effect trait that all effects must implement
#[async_trait]
pub trait Effect: Debug + Send + Sync {
    /// Returns the type of this effect
    fn effect_type(&self) -> EffectType;
    
    /// Returns a human-readable description of this effect
    fn description(&self) -> String;
    
    /// Executes this effect in the provided context
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Allows downcasting to concrete effect types
    fn as_any(&self) -> &dyn Any;
}

/// Trait for effects that can be downcast to concrete types
pub trait DowncastEffect: Effect {
    /// Helper method to downcast to a specific effect type
    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

// Blanket implementation of DowncastEffect for all types that implement Effect
impl<T: Effect + ?Sized> DowncastEffect for T {}

/// Describes the type of an effect for handler registration and discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectType {
    Read,
    Write,
    Create,
    Delete,
    Custom(String),
}

impl std::fmt::Display for EffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectType::Read => write!(f, "read"),
            EffectType::Write => write!(f, "write"),
            EffectType::Create => write!(f, "create"),
            EffectType::Delete => write!(f, "delete"),
            EffectType::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[async_trait]
pub trait DomainEffectHandler: EffectHandler + Debug + Send + Sync {
    /// Handle the domain effect
    async fn handle_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
}

// REMOVE local EffectRegistry trait definition
/*
/// Registry for effect handlers
pub trait EffectRegistry: Debug + Send + Sync {
    /// Register a domain effect handler
    fn register_domain_handler<T>(&mut self, handler: Arc<T>) -> Result<(), EffectError>
    where 
        T: EffectHandler + Debug + Send + Sync + 'static;
    
    /// Register a regular effect handler
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>);
    
    /// Get the effect executor
    fn executor(&self) -> Arc<dyn EffectExecutorBase>; // Note: EffectExecutorBase was removed
}
*/

// REMOVE local DefaultEffectRegistry struct and impl block
/*
/// Default implementation of the effect registry
#[derive(Debug)]
pub struct DefaultEffectRegistry {
    // Use a mutable DefaultEffectExecutor to allow handler registration
    handlers: Vec<Arc<dyn EffectHandler>>,
}

impl DefaultEffectRegistry {
    /// Creates a new effect registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl EffectRegistry for DefaultEffectRegistry {
    fn register_domain_handler<T>(&mut self, handler: Arc<T>) -> Result<(), EffectError>
    where 
        T: EffectHandler + Debug + Send + Sync + 'static 
    {
        // In the future, this might do domain-specific registration
        // For now, just register as a regular handler
        self.register_handler(handler);
        Ok(())
    }
    
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) {
        // Add the handler to our internal collection
        self.handlers.push(handler);
    }
    
    fn executor(&self) -> Arc<dyn EffectExecutorBase> { // Note: EffectExecutorBase was removed
        // Create a new executor with the current handlers
        let mut executor = DefaultEffectExecutor::new(); // Note: DefaultEffectExecutor was removed
        for handler in &self.handlers {
            // executor.register_handler(handler.clone()); // This logic needs the removed types
        }
        Arc::new(executor)
    }
}
*/

// Keep the re-exports
// Remove this duplicate export
// pub use registry::EffectExecutor;
// ... rest of file ... 