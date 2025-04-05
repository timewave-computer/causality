// TEL effect system and interfaces
// Original file: src/tel/effect.rs

//! TEL effect system
//!
//! This module defines the core effect abstractions for the Transaction
//! Effect Language (TEL) system, including basic effect traits and context.

use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::resource::Quantity;

/// Base trait for all effects
#[async_trait]
pub trait Effect: Debug + Send + Sync {
    /// Get the effect type
    fn effect_type(&self) -> &'static str;
    
    /// Apply the effect in the given context
    async fn apply(&self, context: &EffectContext) -> Result<EffectOutcome, EffectError>;
}

/// Context for effect execution
#[derive(Debug, Clone, Default)]
pub struct EffectContext {
    /// Domain-specific parameters
    pub parameters: std::collections::HashMap<String, JsonValue>,
    
    /// Authorization context
    pub authorization: Option<AuthorizationContext>,
}

impl EffectContext {
    /// Create a new effect context
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a parameter to the context
    pub fn with_parameter(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }
    
    /// Set the authorization context
    pub fn with_authorization(mut self, auth: AuthorizationContext) -> Self {
        self.authorization = Some(auth);
        self
    }
    
    /// Get a parameter from the context
    pub fn get_parameter(&self, key: &str) -> Option<&JsonValue> {
        self.parameters.get(key)
    }
}

/// Authorization context for effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationContext {
    /// The identity executing the effect
    pub identity: String,
    
    /// Authorization tokens
    pub tokens: Vec<String>,
}

/// Outcome of an effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectOutcome {
    /// Effect type
    pub effect_type: String,
    
    /// Status of the effect
    pub status: EffectStatus,
    
    /// Output data from the effect
    pub output: Option<JsonValue>,
    
    /// Error message if the effect failed
    pub error: Option<String>,
}

impl EffectOutcome {
    /// Create a new successful effect outcome
    pub fn success(effect_type: impl Into<String>, output: Option<JsonValue>) -> Self {
        Self {
            effect_type: effect_type.into(),
            status: EffectStatus::Success,
            output,
            error: None,
        }
    }
    
    /// Create a new failed effect outcome
    pub fn failure(effect_type: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            effect_type: effect_type.into(),
            status: EffectStatus::Failed,
            output: None,
            error: Some(error.into()),
        }
    }
    
    /// Check if the effect succeeded
    pub fn is_success(&self) -> bool {
        matches!(self.status, EffectStatus::Success)
    }
    
    /// Check if the effect failed
    pub fn is_failure(&self) -> bool {
        matches!(self.status, EffectStatus::Failed)
    }
}

/// Status of an effect
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectStatus {
    /// Effect succeeded
    Success,
    /// Effect failed
    Failed,
    /// Effect is pending
    Pending,
}

/// Error that can occur during effect execution
#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("Effect application failed: {0}")]
    ApplicationFailed(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result of an effect execution
pub type EffectResult<T> = Result<T, EffectError>;

// Note: TransferEffect, StorageEffect, and QueryEffect traits are defined in the handlers module
// and are not duplicated here to avoid conflicts.

// Implement basic effect types

/// Effect for transferring assets between accounts
#[async_trait]
pub trait TransferEffect: Effect {
    /// Get the source address
    fn from(&self) -> &str;
    
    /// Get the destination address
    fn to(&self) -> &str;
    
    /// Get the asset identifier
    fn asset(&self) -> &str;
    
    /// Get the amount
    fn amount(&self) -> &Quantity;
}

/// Effect for storing data
#[async_trait]
pub trait StorageEffect: Effect {
    /// Get the storage key
    fn key(&self) -> &str;
    
    /// Get the data being stored
    fn data(&self) -> &JsonValue;
}

/// Effect for querying data
#[async_trait]
pub trait QueryEffect: Effect {
    /// Get the query function
    fn function(&self) -> &str;
    
    /// Get the query parameters
    fn parameters(&self) -> &JsonValue;
}

/// Base implementation for effects
#[derive(Debug)]
pub struct BaseEffect {
    /// The type of effect
    effect_type: &'static str,
}

impl BaseEffect {
    /// Create a new base effect
    pub fn new(effect_type: &'static str) -> Self {
        Self { effect_type }
    }
}

#[async_trait]
impl Effect for BaseEffect {
    fn effect_type(&self) -> &'static str {
        self.effect_type
    }
    
    async fn apply(&self, _context: &EffectContext) -> Result<EffectOutcome, EffectError> {
        Err(EffectError::ApplicationFailed(
            "BaseEffect cannot be applied directly".to_string()
        ))
    }
} 