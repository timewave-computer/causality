// Resource actor state management
//
// This module provides state management for resource actors,
// including state transitions, snapshots, and persistence.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};

use super::{ActorError, ActorResult, ResourceActor};
use crate::resource::interface::ResourceState;

/// Actor state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Old state
    pub old_state: ResourceState,
    
    /// New state
    pub new_state: ResourceState,
    
    /// Transition reason
    pub reason: String,
    
    /// Transition metadata
    pub metadata: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Actor state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Current state
    pub state: ResourceState,
    
    /// State metadata
    pub metadata: HashMap<String, String>,
    
    /// Last transition
    pub last_transition: Option<StateTransition>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Actor state manager
#[async_trait]
pub trait StateManager: Send + Sync + Debug {
    /// Get current state
    async fn get_state(&self, actor_id: &ContentId) -> ActorResult<StateSnapshot>;
    
    /// Transition state
    async fn transition_state(
        &self,
        actor_id: &ContentId,
        new_state: ResourceState,
        reason: String,
        metadata: HashMap<String, String>,
    ) -> ActorResult<StateTransition>;
    
    /// Get state history
    async fn get_state_history(
        &self,
        actor_id: &ContentId,
        limit: Option<usize>,
    ) -> ActorResult<Vec<StateTransition>>;
    
    /// Get state transitions by reason
    async fn get_transitions_by_reason(
        &self,
        actor_id: &ContentId,
        reason: &str,
    ) -> ActorResult<Vec<StateTransition>>;
}

/// Actor state persistence
#[async_trait]
pub trait StatePersistence: Send + Sync + Debug {
    /// Save state snapshot
    async fn save_snapshot(&self, snapshot: &StateSnapshot) -> ActorResult<()>;
    
    /// Load state snapshot
    async fn load_snapshot(&self, actor_id: &ContentId) -> ActorResult<Option<StateSnapshot>>;
    
    /// Save state transition
    async fn save_transition(&self, transition: &StateTransition) -> ActorResult<()>;
    
    /// Load state transitions
    async fn load_transitions(
        &self,
        actor_id: &ContentId,
        limit: Option<usize>,
    ) -> ActorResult<Vec<StateTransition>>;
}

/// Actor state factory
#[async_trait]
pub trait StateFactory: Send + Sync + Debug {
    /// Create state manager
    async fn create_state_manager(&self) -> ActorResult<Arc<dyn StateManager>>;
    
    /// Create state persistence
    async fn create_state_persistence(&self) -> ActorResult<Arc<dyn StatePersistence>>;
}

/// State configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Snapshot interval in seconds
    pub snapshot_interval: u64,
    
    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,
    
    /// State validation rules
    pub validation_rules: Vec<StateValidationRule>,
    
    /// State metadata
    pub metadata: HashMap<String, String>,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            snapshot_interval: 300,
            max_snapshots: 10,
            validation_rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// State validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValidationRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Rule description
    pub description: String,
    
    /// Rule expression (JSON Path or similar)
    pub expression: String,
    
    /// Expected result (for comparison)
    pub expected_result: Option<serde_json::Value>,
}

/// State error types
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// State not found
    #[error("State not found: {0}")]
    NotFound(ContentId),
    
    /// State already exists
    #[error("State already exists: {0}")]
    AlreadyExists(ContentId),
    
    /// State validation error
    #[error("State validation error: {0}")]
    ValidationError(String),
    
    /// State error
    #[error("State error: {0}")]
    StateError(String),
}

/// State result type
pub type StateResult<T> = Result<T, StateError>;

/// State snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Snapshot ID
    pub id: ContentId,
    
    /// Actor ID
    pub actor_id: ContentId,
    
    /// State values
    pub state: HashMap<String, serde_json::Value>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ContentAddressed for StateSnapshot {
    fn content_id(&self) -> ContentId {
        // Implementation would hash the state contents
        self.id
    }
}

/// State validation context
#[derive(Debug, Clone)]
pub struct StateValidationContext {
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Previous state
    pub previous_state: Option<HashMap<String, serde_json::Value>>,
    
    /// New state
    pub new_state: HashMap<String, serde_json::Value>,
    
    /// Validation rules
    pub rules: Vec<StateValidationRule>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// State validator
#[async_trait]
pub trait StateValidator: Send + Sync + Debug {
    /// Validate state transition
    async fn validate(
        &self,
        context: StateValidationContext,
    ) -> StateResult<Vec<StateValidationResult>>;
}

/// State validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValidationResult {
    /// Rule ID
    pub rule_id: String,
    
    /// Is valid
    pub is_valid: bool,
    
    /// Error message (if any)
    pub error: Option<String>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// State store
#[async_trait]
pub trait StateStore: Send + Sync + Debug {
    /// Get state
    async fn get_state(&self, actor_id: &ContentId) -> StateResult<HashMap<String, serde_json::Value>>;
    
    /// Set state
    async fn set_state(
        &self,
        actor_id: &ContentId,
        state: HashMap<String, serde_json::Value>,
    ) -> StateResult<()>;
    
    /// Get state snapshot
    async fn get_snapshot(&self, snapshot_id: &ContentId) -> StateResult<StateSnapshot>;
    
    /// Create state snapshot
    async fn create_snapshot(&self, snapshot: StateSnapshot) -> StateResult<ContentId>;
    
    /// Get all snapshots for an actor
    async fn get_snapshots(&self, actor_id: &ContentId) -> StateResult<Vec<StateSnapshot>>;
    
    /// Delete a snapshot
    async fn delete_snapshot(&self, snapshot_id: &ContentId) -> StateResult<()>;
    
    /// Clean up old snapshots
    async fn cleanup_snapshots(&self, actor_id: &ContentId, max_snapshots: usize) -> StateResult<usize>;
}

impl From<StateError> for ActorError {
    fn from(error: StateError) -> Self {
        ActorError::StateError(error.to_string())
    }
} 