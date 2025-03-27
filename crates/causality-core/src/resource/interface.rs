// Resource Management Core Interfaces
//
// This module defines the core trait interfaces for the unified resource management system.
// These interfaces provide a common foundation for resource operations across both domain
// and effect boundaries.

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::time::{Duration, SystemTime};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};

/// Resource state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceState {
    /// Created
    Created,
    
    /// Active
    Active,
    
    /// Locked
    Locked,
    
    /// Frozen
    Frozen,
    
    /// Consumed
    Consumed,
    
    /// Archived
    Archived,
}

impl Display for ResourceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceState::Created => write!(f, "Created"),
            ResourceState::Active => write!(f, "Active"),
            ResourceState::Locked => write!(f, "Locked"),
            ResourceState::Frozen => write!(f, "Frozen"),
            ResourceState::Consumed => write!(f, "Consumed"),
            ResourceState::Archived => write!(f, "Archived"),
        }
    }
}

/// Resource access type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceAccessType {
    /// Read access
    Read,
    
    /// Write access
    Write,
    
    /// Execute access
    Execute,
    
    /// Admin access
    Admin,
}

/// Resource lock type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LockType {
    /// Exclusive lock
    Exclusive,
    
    /// Shared lock
    Shared,
    
    /// Intent lock
    Intent,
}

/// Resource dependency type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Strong dependency
    Strong,
    
    /// Weak dependency
    Weak,
    
    /// Reference dependency
    Reference,
}

/// Resource error
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(ContentId),
    
    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(ContentId),
    
    /// Invalid state transition
    #[error("Invalid state transition from {0} to {1}")]
    InvalidStateTransition(ResourceState, ResourceState),
    
    /// Access denied
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    /// Lock error
    #[error("Lock error: {0}")]
    LockError(String),
    
    /// Dependency error
    #[error("Dependency error: {0}")]
    DependencyError(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),
}

/// Resource result
pub type ResourceResult<T> = Result<T, ResourceError>;

/// Resource access
#[async_trait]
pub trait ResourceAccess: Send + Sync + Debug {
    /// Check access
    async fn check_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
    ) -> ResourceResult<bool>;
    
    /// Grant access
    async fn grant_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
    ) -> ResourceResult<()>;
    
    /// Revoke access
    async fn revoke_access(
        &self,
        resource_id: &ContentId,
        access_type: ResourceAccessType,
    ) -> ResourceResult<()>;
    
    /// Get access types
    async fn get_access_types(&self, resource_id: &ContentId) -> ResourceResult<Vec<ResourceAccessType>>;
}

/// Resource lifecycle
#[async_trait]
pub trait ResourceLifecycle: Send + Sync + Debug {
    /// Get state
    async fn get_state(&self, resource_id: &ContentId) -> ResourceResult<ResourceState>;
    
    /// Set state
    async fn set_state(
        &self,
        resource_id: &ContentId,
        state: ResourceState,
    ) -> ResourceResult<()>;
    
    /// Get state history
    async fn get_state_history(
        &self,
        resource_id: &ContentId,
        limit: Option<usize>,
    ) -> ResourceResult<Vec<(ResourceState, chrono::DateTime<chrono::Utc>)>>;
}

/// Resource locking
#[async_trait]
pub trait ResourceLocking: Send + Sync + Debug {
    /// Acquire lock
    async fn acquire_lock(
        &self,
        resource_id: &ContentId,
        lock_type: LockType,
    ) -> ResourceResult<()>;
    
    /// Release lock
    async fn release_lock(
        &self,
        resource_id: &ContentId,
        lock_type: LockType,
    ) -> ResourceResult<()>;
    
    /// Get lock type
    async fn get_lock_type(&self, resource_id: &ContentId) -> ResourceResult<Option<LockType>>;
    
    /// Check if locked
    async fn is_locked(&self, resource_id: &ContentId) -> ResourceResult<bool>;
}

/// Resource dependency
#[async_trait]
pub trait ResourceDependency: Send + Sync + Debug {
    /// Add dependency
    async fn add_dependency(
        &self,
        resource_id: &ContentId,
        dependency_id: &ContentId,
        dependency_type: DependencyType,
    ) -> ResourceResult<()>;
    
    /// Remove dependency
    async fn remove_dependency(
        &self,
        resource_id: &ContentId,
        dependency_id: &ContentId,
    ) -> ResourceResult<()>;
    
    /// Get dependencies
    async fn get_dependencies(
        &self,
        resource_id: &ContentId,
    ) -> ResourceResult<Vec<(ContentId, DependencyType)>>;
    
    /// Get dependents
    async fn get_dependents(
        &self,
        resource_id: &ContentId,
    ) -> ResourceResult<Vec<(ContentId, DependencyType)>>;
}

/// Resource interface
#[async_trait]
pub trait ResourceInterface: Send + Sync + Debug {
    /// Get resource access
    async fn get_access(&self) -> ResourceResult<Arc<dyn ResourceAccess>>;
    
    /// Get resource lifecycle
    async fn get_lifecycle(&self) -> ResourceResult<Arc<dyn ResourceLifecycle>>;
    
    /// Get resource locking
    async fn get_locking(&self) -> ResourceResult<Arc<dyn ResourceLocking>>;
    
    /// Get resource dependency
    async fn get_dependency(&self) -> ResourceResult<Arc<dyn ResourceDependency>>;
}

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Maximum number of resources
    pub max_resources: usize,
    
    /// Maximum number of dependencies per resource
    pub max_dependencies: usize,
    
    /// Resource cleanup interval
    pub cleanup_interval: std::time::Duration,
    
    /// Resource metadata
    pub metadata: HashMap<String, String>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_resources: 1000,
            max_dependencies: 100,
            cleanup_interval: std::time::Duration::from_secs(3600),
            metadata: HashMap::new(),
        }
    }
} 