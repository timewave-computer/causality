// Resource actor registry
//
// This module provides the registry for resource actors, 
// allowing actor lookup, registration, and lifecycle management.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, oneshot};

use crate::crypto::{ContentId, ContentAddressed};

use super::{
    ActorError, ActorResult, ResourceActor, ResourceActorFactory, MessageBus,
    message::Message, state::StateTransition, InitialActorState
};

/// Registry for resource actors
///
/// The registry keeps track of all active resource actors in the system,
/// provides lookup functionality, and manages actor lifecycles.
#[async_trait]
pub trait ActorRegistry: Send + Sync + Debug {
    /// Check if an actor is registered
    async fn is_registered(&self, actor_id: &ContentId) -> ActorResult<bool>;
    
    /// Get an actor by ID
    async fn get_actor(&self, actor_id: &ContentId) -> ActorResult<Arc<dyn ResourceActor>>;
    
    /// Register an actor
    async fn register_actor(&self, actor: Arc<dyn ResourceActor>) -> ActorResult<()>;
    
    /// Unregister an actor
    async fn unregister_actor(&self, actor_id: &ContentId) -> ActorResult<()>;
    
    /// Create a new actor
    async fn create_actor(
        &self,
        actor_type: &str,
        initial_state: InitialActorState,
    ) -> ActorResult<ContentId>;
    
    /// Get all actors by type
    async fn get_actors_by_type(&self, actor_type: &str) -> ActorResult<Vec<Arc<dyn ResourceActor>>>;
    
    /// Get all registered actors
    async fn get_all_actors(&self) -> ActorResult<Vec<Arc<dyn ResourceActor>>>;
}

/// Registry factory for creating actor registries
#[async_trait]
pub trait ActorRegistryFactory: Send + Sync + Debug {
    /// Create a new actor registry
    async fn create_registry(&self) -> ActorResult<Arc<dyn ActorRegistry>>;
    
    /// Get the types of registries this factory can create
    fn supported_types(&self) -> Vec<String>;
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Maximum number of actors allowed
    pub max_actors: usize,
    
    /// Maximum number of actors per type
    pub max_actors_per_type: usize,
    
    /// Actor cleanup interval
    pub cleanup_interval: std::time::Duration,
    
    /// Registry metadata
    pub metadata: HashMap<String, String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_actors: 1000,
            max_actors_per_type: 100,
            cleanup_interval: std::time::Duration::from_secs(3600),
            metadata: HashMap::new(),
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of registered actors
    pub total_actors: usize,
    
    /// Number of actors by type
    pub actors_by_type: HashMap<String, usize>,
    
    /// Number of active actors
    pub active_actors: usize,
    
    /// Number of inactive actors
    pub inactive_actors: usize,
    
    /// Last cleanup timestamp
    pub last_cleanup: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for RegistryStats {
    fn default() -> Self {
        Self {
            total_actors: 0,
            actors_by_type: HashMap::new(),
            active_actors: 0,
            inactive_actors: 0,
            last_cleanup: None,
        }
    }
}

/// Registry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryEvent {
    /// Actor registered
    ActorRegistered {
        /// Actor ID
        actor_id: ContentId,
        /// Actor type
        actor_type: String,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Actor unregistered
    ActorUnregistered {
        /// Actor ID
        actor_id: ContentId,
        /// Actor type
        actor_type: String,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Actor state changed
    ActorStateChanged {
        /// Actor ID
        actor_id: ContentId,
        /// Actor type
        actor_type: String,
        /// Old state
        old_state: crate::resource::interface::ResourceState,
        /// New state
        new_state: crate::resource::interface::ResourceState,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Registry cleaned up
    RegistryCleanedUp {
        /// Number of actors removed
        actors_removed: usize,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Registry event handler
#[async_trait]
pub trait RegistryEventHandler: Send + Sync + Debug {
    /// Handle a registry event
    async fn handle_event(&self, event: RegistryEvent) -> ActorResult<()>;
}

/// Registry event bus
#[async_trait]
pub trait RegistryEventBus: Send + Sync + Debug {
    /// Publish a registry event
    async fn publish_event(&self, event: RegistryEvent) -> ActorResult<()>;
    
    /// Subscribe to registry events
    async fn subscribe(&self, handler: Arc<dyn RegistryEventHandler>) -> ActorResult<()>;
    
    /// Unsubscribe from registry events
    async fn unsubscribe(&self, handler: Arc<dyn RegistryEventHandler>) -> ActorResult<()>;
}

/// Actor registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorRegistration {
    /// Actor ID
    pub id: ContentId,
    
    /// Actor type
    pub actor_type: String,
    
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    
    /// Last active timestamp
    pub last_active: chrono::DateTime<chrono::Utc>,
    
    /// Actor status
    pub status: ActorStatus,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Actor status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorStatus {
    /// Actor is initializing
    Initializing,
    
    /// Actor is active
    Active,
    
    /// Actor is inactive (but still exists)
    Inactive,
    
    /// Actor is shutting down
    ShuttingDown,
    
    /// Actor has an error
    Error,
}

/// Actor registry
#[async_trait]
pub trait ActorRegistry: Send + Sync + Debug {
    /// Get an actor by ID
    async fn get_actor(&self, id: &ContentId) -> Result<Arc<dyn ResourceActor>, ActorError>;
    
    /// Register an actor
    async fn register_actor(&self, actor: Arc<dyn ResourceActor>) -> Result<(), ActorError>;
    
    /// Unregister an actor
    async fn unregister_actor(&self, id: &ContentId) -> Result<(), ActorError>;
    
    /// Get all actors
    async fn get_all_actors(&self) -> Result<Vec<Arc<dyn ResourceActor>>, ActorError>;
    
    /// Get actors by type
    async fn get_actors_by_type(&self, actor_type: &str) -> Result<Vec<Arc<dyn ResourceActor>>, ActorError>;
    
    /// Get actor registration
    async fn get_registration(&self, id: &ContentId) -> Result<ActorRegistration, ActorError>;
    
    /// Update actor status
    async fn update_status(&self, id: &ContentId, status: ActorStatus) -> Result<(), ActorError>;
    
    /// Clean up inactive actors
    async fn cleanup(&self) -> Result<usize, ActorError>;
    
    /// Get registry stats
    async fn get_stats(&self) -> Result<RegistryStats, ActorError>;
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of actors
    pub total_actors: usize,
    
    /// Number of active actors
    pub active_actors: usize,
    
    /// Number of inactive actors
    pub inactive_actors: usize,
    
    /// Number of actors with errors
    pub error_actors: usize,
    
    /// Actor counts by type
    pub actors_by_type: HashMap<String, usize>,
    
    /// Last cleanup timestamp
    pub last_cleanup: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Registry metadata
    pub metadata: HashMap<String, String>,
} 