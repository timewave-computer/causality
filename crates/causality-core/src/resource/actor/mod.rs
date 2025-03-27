// Resource-based actor system
//
// This module provides a complete actor system built on top of resources,
// implementing the architecture described in ADR-032.

pub mod message;
pub mod reference;
pub mod registry;
pub mod state;
pub mod supervision;
pub mod impl_basic;
pub mod smt_integration;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};
use crate::resource::interface::ResourceState;

pub use message::*;
pub use reference::*;
pub use registry::*;
pub use state::*;
pub use supervision::*;
pub use impl_basic::*;
pub use smt_integration::*;

/// Actor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorConfig {
    /// Maximum number of actors
    pub max_actors: usize,
    
    /// Message queue capacity
    pub message_queue_capacity: usize,
    
    /// Actor supervision policy
    pub supervision_policy: supervision::SupervisionPolicy,
    
    /// Registry configuration
    pub registry_config: registry::RegistryConfig,
    
    /// State configuration
    pub state_config: state::StateConfig,
    
    /// Actor metadata
    pub metadata: HashMap<String, String>,
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            max_actors: 1000,
            message_queue_capacity: 1000,
            supervision_policy: supervision::SupervisionPolicy::default(),
            registry_config: registry::RegistryConfig::default(),
            state_config: state::StateConfig::default(),
            metadata: HashMap::new(),
        }
    }
}

/// Actor error types
#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    /// Actor not found
    #[error("Actor not found: {0}")]
    NotFound(ContentId),
    
    /// Actor already exists
    #[error("Actor already exists: {0}")]
    AlreadyExists(ContentId),
    
    /// Actor state error
    #[error("Actor state error: {0}")]
    StateError(#[from] state::StateError),
    
    /// Actor supervision error
    #[error("Actor supervision error: {0}")]
    SupervisionError(String),
    
    /// Message error
    #[error("Message error: {0}")]
    MessageError(String),
    
    /// SMT error
    #[error("SMT error: {0}")]
    SmtError(#[from] smt_integration::ActorStateSmtError),
    
    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    /// Actor error
    #[error("Actor error: {0}")]
    ActorError(String),
}

/// Actor result type
pub type ActorResult<T> = Result<T, ActorError>;

/// Response status for actor messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Message was processed successfully
    Success,
    
    /// Message processing failed with an error
    Error,
    
    /// Message was not processed (e.g., actor was busy)
    NotProcessed,
    
    /// Timeout
    Timeout,
    
    /// Cancelled
    Cancelled,
}

/// Response to an actor message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    /// Response ID
    pub id: String,
    
    /// Request ID
    pub request_id: String,
    
    /// Status
    pub status: ResponseStatus,
    
    /// Payload
    pub payload: serde_json::Value,
    
    /// Error
    pub error: Option<String>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Actor message bus interface
#[async_trait]
pub trait MessageBus: Send + Sync + Debug {
    /// Send a message to an actor
    async fn send(&self, message: Message) -> ActorResult<()>;
    
    /// Receive a message addressed to this actor
    async fn receive(&self) -> ActorResult<Message>;
    
    /// Send a response to a message
    async fn send_response(&self, response: MessageResponse) -> ActorResult<()>;
    
    /// Receive a response to a previously sent message
    async fn receive_response(&self, request_id: &str) -> ActorResult<MessageResponse>;
}

/// Resource actor interface
#[async_trait]
pub trait ResourceActor: Send + Sync + Debug {
    /// Get the actor's unique ID
    fn id(&self) -> ContentId;
    
    /// Get the actor's type
    fn actor_type(&self) -> &str;
    
    /// Get the actor's current state
    async fn get_state(&self) -> ActorResult<HashMap<String, serde_json::Value>>;
    
    /// Set the actor's state
    async fn set_state(&self, state: HashMap<String, serde_json::Value>) -> ActorResult<()>;
    
    /// Get the actor's metadata
    async fn get_metadata(&self) -> ActorResult<HashMap<String, String>>;
    
    /// Set the actor's metadata
    async fn set_metadata(&self, metadata: HashMap<String, String>) -> ActorResult<()>;
    
    /// Handle a message sent to this actor
    async fn handle_message(&self, message: Message) -> ActorResult<()>;
    
    /// Start the actor
    async fn start(&self) -> ActorResult<()>;
    
    /// Stop the actor
    async fn stop(&self) -> ActorResult<()>;
}

/// Initial state for a new actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialActorState {
    /// Actor identifier
    pub id: String,
    
    /// Initial state values
    pub state: HashMap<String, serde_json::Value>,
    
    /// Initial metadata
    pub metadata: HashMap<String, String>,
}

/// Factory for creating resource actors
#[async_trait]
pub trait ResourceActorFactory: Send + Sync + Debug {
    /// Create a new actor
    async fn create_actor(
        &self,
        actor_type: &str,
        initial_state: InitialActorState,
    ) -> ActorResult<Arc<dyn ResourceActor>>;
    
    /// Get the types of actors this factory can create
    fn supported_types(&self) -> Vec<String>;
} 