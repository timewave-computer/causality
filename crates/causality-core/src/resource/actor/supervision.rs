// Resource actor supervision
//
// This module provides supervision for resource actors,
// including error handling, recovery, and monitoring.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};

use super::{ActorError, ActorResult, ResourceActor};

/// Actor supervision strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    /// Stop the actor
    Stop,
    
    /// Restart the actor
    Restart,
    
    /// Escalate the error
    Escalate,
    
    /// Resume the actor
    Resume,
}

/// Actor supervision decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionDecision {
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Error
    pub error: ActorError,
    
    /// Strategy
    pub strategy: SupervisionStrategy,
    
    /// Decision metadata
    pub metadata: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Actor supervision context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionContext {
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Actor type
    pub actor_type: String,
    
    /// Error count
    pub error_count: usize,
    
    /// Last error
    pub last_error: Option<ActorError>,
    
    /// Last decision
    pub last_decision: Option<SupervisionDecision>,
    
    /// Context metadata
    pub metadata: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Actor supervisor
#[async_trait]
pub trait Supervisor: Send + Sync + Debug {
    /// Handle actor error
    async fn handle_error(
        &self,
        actor: Arc<dyn ResourceActor>,
        error: ActorError,
    ) -> ActorResult<SupervisionDecision>;
    
    /// Get supervision context
    async fn get_context(&self, actor_id: &ContentId) -> ActorResult<SupervisionContext>;
    
    /// Update supervision context
    async fn update_context(
        &self,
        actor_id: &ContentId,
        context: SupervisionContext,
    ) -> ActorResult<()>;
    
    /// Get supervision history
    async fn get_history(
        &self,
        actor_id: &ContentId,
        limit: Option<usize>,
    ) -> ActorResult<Vec<SupervisionDecision>>;
}

/// Supervision policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionPolicy {
    /// Default strategy
    pub default_strategy: SupervisionStrategy,
    
    /// Error-specific strategies
    pub error_strategies: HashMap<String, SupervisionStrategy>,
    
    /// Maximum restart count
    pub max_restarts: usize,
    
    /// Restart backoff
    pub restart_backoff: Duration,
    
    /// Maximum backoff
    pub max_backoff: Duration,
}

impl Default for SupervisionPolicy {
    fn default() -> Self {
        Self {
            default_strategy: SupervisionStrategy::Restart,
            error_strategies: HashMap::new(),
            max_restarts: 10,
            restart_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }
}

/// Supervision event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionEvent {
    /// Event ID
    pub id: String,
    
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Event type
    pub event_type: SupervisionEventType,
    
    /// Error (if any)
    pub error: Option<String>,
    
    /// Strategy applied
    pub strategy: SupervisionStrategy,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Supervision event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupervisionEventType {
    /// Actor started
    Started,
    
    /// Actor stopped
    Stopped,
    
    /// Actor restarted
    Restarted,
    
    /// Actor failed
    Failed,
    
    /// Actor recovered
    Recovered,
    
    /// Actor escalated
    Escalated,
}

/// Actor supervisor
#[async_trait]
pub trait ActorSupervisor: Send + Sync + Debug {
    /// Handle actor failure
    async fn handle_failure(
        &self,
        actor: Arc<dyn ResourceActor>,
        error: ActorError,
    ) -> Result<SupervisionStrategy, ActorError>;
    
    /// Start supervising an actor
    async fn supervise(&self, actor: Arc<dyn ResourceActor>) -> Result<(), ActorError>;
    
    /// Stop supervising an actor
    async fn stop_supervising(&self, actor_id: &ContentId) -> Result<(), ActorError>;
    
    /// Get the supervision policy
    fn get_policy(&self) -> &SupervisionPolicy;
    
    /// Set the supervision policy
    async fn set_policy(&self, policy: SupervisionPolicy) -> Result<(), ActorError>;
    
    /// Get supervision events for an actor
    async fn get_events(&self, actor_id: &ContentId) -> Result<Vec<SupervisionEvent>, ActorError>;
    
    /// Get all supervised actors
    async fn get_supervised_actors(&self) -> Result<Vec<Arc<dyn ResourceActor>>, ActorError>;
}

/// Creates a new actor supervisor with the given policy
pub fn create_supervisor(policy: SupervisionPolicy) -> Arc<dyn ActorSupervisor> {
    // Implementation would go here in a real system
    unimplemented!("Supervisor implementation not available")
}

/// Actor supervision error
#[derive(Debug, thiserror::Error)]
pub enum SupervisionError {
    /// Invalid supervision strategy
    #[error("Invalid supervision strategy for actor {0}")]
    InvalidStrategy(ContentId),
    
    /// Supervision context not found
    #[error("Supervision context not found for actor {0}")]
    ContextNotFound(ContentId),
    
    /// Supervision policy error
    #[error("Supervision policy error: {0}")]
    PolicyError(String),
    
    /// Supervision decision error
    #[error("Supervision decision error: {0}")]
    DecisionError(String),
}

impl From<SupervisionError> for ActorError {
    fn from(error: SupervisionError) -> Self {
        ActorError::SupervisionError(error)
    }
} 