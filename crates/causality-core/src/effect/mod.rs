use std::any::Any;
use std::fmt::Debug;
use thiserror::Error;
use async_trait::async_trait;

// Sub-modules
pub mod resource;
pub mod capability;
pub mod info;
pub mod utils;
pub mod handler;
pub mod types;
pub mod context;
pub mod outcome;
pub mod error;

// Import from domain for convenience 
pub use types::{EffectId, EffectTypeId, ExecutionBoundary, Right};
pub use handler::EffectHandler;
pub use handler::HandlerResult;
pub use context::EffectContext;
pub use context::Capability;
pub use outcome::{EffectOutcome, EffectResult};

/// Error type for effect operations
#[derive(Debug, Error, Clone)]
pub enum EffectError {
    #[error("Missing required capability: {0}")]
    MissingCapability(String),

    #[error("Missing required resource: {0}")]
    MissingResource(String),

    #[error("Execution error: {0}")]
    ExecutionFailed(String),

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
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Object already exists: {0}")]
    AlreadyExists(String),
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

impl std::str::FromStr for EffectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(EffectType::Read),
            "write" => Ok(EffectType::Write),
            "create" => Ok(EffectType::Create),
            "delete" => Ok(EffectType::Delete),
            _ => Ok(EffectType::Custom(s.to_string())),
        }
    }
}

impl From<&str> for EffectType {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_else(|_| EffectType::Custom(s.to_string()))
    }
}

#[async_trait]
pub trait DomainEffectHandler: EffectHandler + Debug + Send + Sync {
    /// Handle the domain effect
    async fn handle_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
}

// Placeholder trait definition for DomainEffect
pub trait DomainEffect: Effect {}

// Re-export system errors but not EffectError which is defined above
pub use error::EffectSystemError;
pub use error::EffectSystemResult;

// Re-export resource module types
pub use resource::*;