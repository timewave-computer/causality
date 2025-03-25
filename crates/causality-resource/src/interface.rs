// Resource Management Core Interfaces
//
// This module defines the core trait interfaces for the unified resource management system.
// These interfaces provide a common foundation for resource operations across both domain
// and effect boundaries.

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use causality_common::identity::{ContentId, IdentitySource};

/// Represents the state of a resource in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    /// Resource has been created but not activated
    Created,
    /// Resource is active and available for use
    Active,
    /// Resource is locked for exclusive use
    Locked,
    /// Resource is frozen and cannot be modified
    Frozen,
    /// Resource has been consumed and is no longer available
    Consumed,
    /// Resource has been archived
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

/// Types of access to a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceAccessType {
    /// Read access to the resource
    Read,
    /// Write access to the resource
    Write,
    /// Execute access to the resource
    Execute,
    /// Lock access to the resource
    Lock,
    /// Transfer access to the resource
    Transfer,
}

impl Display for ResourceAccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceAccessType::Read => write!(f, "Read"),
            ResourceAccessType::Write => write!(f, "Write"),
            ResourceAccessType::Execute => write!(f, "Execute"),
            ResourceAccessType::Lock => write!(f, "Lock"),
            ResourceAccessType::Transfer => write!(f, "Transfer"),
        }
    }
}

/// Types of locks that can be placed on a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LockType {
    /// Exclusive lock prevents any other access
    Exclusive,
    /// Shared lock allows other shared locks but no exclusive locks
    Shared,
    /// Intent lock signals intention to lock in the future
    Intent,
}

impl Display for LockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockType::Exclusive => write!(f, "Exclusive"),
            LockType::Shared => write!(f, "Shared"),
            LockType::Intent => write!(f, "Intent"),
        }
    }
}

/// Types of dependencies between resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Strong dependency means the dependent resource cannot exist without the dependency
    Strong,
    /// Weak dependency means the dependent resource can exist without the dependency
    Weak,
    /// Temporal dependency means the dependency must be processed before the dependent
    Temporal,
    /// Data dependency means the dependent resource depends on data from the dependency
    Data,
    /// Identity dependency means the dependent resource's identity depends on the dependency
    Identity,
}

impl Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Strong => write!(f, "Strong"),
            DependencyType::Weak => write!(f, "Weak"),
            DependencyType::Temporal => write!(f, "Temporal"),
            DependencyType::Data => write!(f, "Data"),
            DependencyType::Identity => write!(f, "Identity"),
        }
    }
}

/// Outcome of a lock request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockStatus {
    /// Lock was successfully acquired
    Acquired,
    /// Lock was already held by the requester
    AlreadyHeld,
    /// Lock could not be acquired because it is held by another entity
    Unavailable,
    /// Lock request timed out
    TimedOut,
}

impl Display for LockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockStatus::Acquired => write!(f, "Acquired"),
            LockStatus::AlreadyHeld => write!(f, "AlreadyHeld"),
            LockStatus::Unavailable => write!(f, "Unavailable"),
            LockStatus::TimedOut => write!(f, "TimedOut"),
        }
    }
}

/// Core trait for resource identity management
pub trait ResourceIdentity {
    /// Get the content ID of the resource
    fn resource_id(&self) -> &ContentId;
    
    /// Check if this resource matches the given content ID
    fn is_resource(&self, id: &ContentId) -> bool {
        self.resource_id() == id
    }
    
    /// Generate a new resource ID based on the current resource
    fn derive_resource_id(&self, source: &impl IdentitySource) -> Result<ContentId>;
}

/// Core trait for resource access control
#[async_trait]
pub trait ResourceAccess {
    /// Check if the given access type is allowed for the resource
    async fn is_access_allowed(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<bool>;
    
    /// Record an access to a resource
    async fn record_access(
        &self, 
        resource_id: &ContentId, 
        access_type: ResourceAccessType, 
        context: &dyn ResourceContext
    ) -> Result<()>;
    
    /// Get all recorded accesses for a resource
    async fn get_resource_accesses(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceAccessRecord>>;
}

/// Record of a resource access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccessRecord {
    /// ID of the resource being accessed
    pub resource_id: ContentId,
    /// Type of access
    pub access_type: ResourceAccessType,
    /// ID of the domain containing the resource (if applicable)
    pub domain_id: Option<ContentId>,
    /// ID of the effect accessing the resource (if applicable)
    pub effect_id: Option<ContentId>,
    /// Whether the access was granted
    pub granted: bool,
    /// Time of the access
    pub timestamp: SystemTime,
    /// Additional metadata about the access
    pub metadata: HashMap<String, String>,
}

/// Core trait for resource lifecycle management
#[async_trait]
pub trait ResourceLifecycle {
    /// Register a new resource with the system
    async fn register_resource(
        &self, 
        resource_id: ContentId, 
        initial_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()>;
    
    /// Update the state of a resource
    async fn update_resource_state(
        &self, 
        resource_id: &ContentId, 
        new_state: ResourceState, 
        context: &dyn ResourceContext
    ) -> Result<()>;
    
    /// Get the current state of a resource
    async fn get_resource_state(
        &self, 
        resource_id: &ContentId
    ) -> Result<ResourceState>;
    
    /// Check if a resource exists in the system
    async fn resource_exists(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool>;
    
    /// Activate a resource (transition to Active state)
    async fn activate_resource(
        &self, 
        resource_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        self.update_resource_state(resource_id, ResourceState::Active, context).await
    }
    
    /// Consume a resource (transition to Consumed state)
    async fn consume_resource(
        &self, 
        resource_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        self.update_resource_state(resource_id, ResourceState::Consumed, context).await
    }
    
    /// Archive a resource (transition to Archived state)
    async fn archive_resource(
        &self, 
        resource_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<()> {
        self.update_resource_state(resource_id, ResourceState::Archived, context).await
    }
}

/// Core trait for resource locking
#[async_trait]
pub trait ResourceLocking {
    /// Acquire a lock on a resource
    async fn acquire_lock(
        &self, 
        resource_id: &ContentId, 
        lock_type: LockType, 
        holder_id: &ContentId, 
        timeout: Option<Duration>, 
        context: &dyn ResourceContext
    ) -> Result<LockStatus>;
    
    /// Release a lock on a resource
    async fn release_lock(
        &self, 
        resource_id: &ContentId, 
        holder_id: &ContentId, 
        context: &dyn ResourceContext
    ) -> Result<bool>;
    
    /// Check if a resource is locked
    async fn is_locked(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool>;
    
    /// Get information about the lock on a resource, if any
    async fn get_lock_info(
        &self, 
        resource_id: &ContentId
    ) -> Result<Option<ResourceLockInfo>>;
}

/// Information about a lock on a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLockInfo {
    /// ID of the resource being locked
    pub resource_id: ContentId,
    /// Type of lock
    pub lock_type: LockType,
    /// ID of the entity holding the lock
    pub holder_id: ContentId,
    /// Time when the lock was acquired
    pub acquired_at: SystemTime,
    /// Time when the lock expires, if any
    pub expires_at: Option<SystemTime>,
    /// ID of the transaction associated with the lock, if any
    pub transaction_id: Option<ContentId>,
}

/// Core trait for resource dependency tracking
#[async_trait]
pub trait ResourceDependency {
    /// Add a dependency from source to target
    async fn add_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<()>;
    
    /// Remove a dependency from source to target
    async fn remove_dependency(
        &self, 
        source_id: &ContentId, 
        target_id: &ContentId, 
        dependency_type: DependencyType, 
        context: &dyn ResourceContext
    ) -> Result<bool>;
    
    /// Get all dependencies of a resource
    async fn get_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>>;
    
    /// Get all resources that depend on a resource
    async fn get_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<Vec<ResourceDependencyInfo>>;
    
    /// Check if a resource has any dependencies
    async fn has_dependencies(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool>;
    
    /// Check if a resource has any dependents
    async fn has_dependents(
        &self, 
        resource_id: &ContentId
    ) -> Result<bool>;
}

/// Information about a dependency between resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependencyInfo {
    /// ID of the source resource (the dependent)
    pub source_id: ContentId,
    /// ID of the target resource (the dependency)
    pub target_id: ContentId,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// ID of the domain containing the source resource, if applicable
    pub source_domain_id: Option<ContentId>,
    /// ID of the domain containing the target resource, if applicable
    pub target_domain_id: Option<ContentId>,
    /// ID of the effect that created the dependency, if applicable
    pub creator_effect_id: Option<ContentId>,
    /// Time when the dependency was created
    pub created_at: SystemTime,
    /// Additional metadata about the dependency
    pub metadata: HashMap<String, String>,
}

/// Context for resource operations
pub trait ResourceContext: Send + Sync {
    /// Get the ID of the resource operation context
    fn context_id(&self) -> ContentId;
    
    /// Get the ID of the domain, if applicable
    fn domain_id(&self) -> Option<&ContentId>;
    
    /// Get the ID of the effect, if applicable
    fn effect_id(&self) -> Option<&ContentId>;
    
    /// Get the timestamp of the operation
    fn timestamp(&self) -> SystemTime;
    
    /// Get contextual metadata
    fn metadata(&self) -> &HashMap<String, String>;
}

/// Basic implementation of ResourceContext
#[derive(Debug, Clone)]
pub struct BasicResourceContext {
    /// ID of the context
    pub context_id: ContentId,
    /// ID of the domain
    pub domain_id: Option<ContentId>,
    /// ID of the effect
    pub effect_id: Option<ContentId>,
    /// Timestamp of the operation
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl BasicResourceContext {
    /// Create a new basic resource context
    pub fn new(context_id: ContentId) -> Self {
        Self {
            context_id,
            domain_id: None,
            effect_id: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new context with domain ID
    pub fn with_domain(mut self, domain_id: ContentId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Create a new context with effect ID
    pub fn with_effect(mut self, effect_id: ContentId) -> Self {
        self.effect_id = Some(effect_id);
        self
    }
    
    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl ResourceContext for BasicResourceContext {
    fn context_id(&self) -> ContentId {
        self.context_id.clone()
    }
    
    fn domain_id(&self) -> Option<&ContentId> {
        self.domain_id.as_ref()
    }
    
    fn effect_id(&self) -> Option<&ContentId> {
        self.effect_id.as_ref()
    }
    
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
} 