// Resource lifecycle management (LEGACY VERSION)
//
// This module contains the deprecated implementation of resource lifecycle
// management. Use the ResourceLifecycle trait implementations in
// causality-effects::resource::lifecycle instead.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use causality_common::identity::ContentId;
use thiserror::Error;

use crate::interface::deprecation::messages;
use crate::deprecated_warning;
use crate::deprecated_error;

/// Legacy resource lifecycle states
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LIFECYCLE_DEPRECATED
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceState {
    /// Resource has been created but is not yet active
    Created,
    
    /// Resource is active and can be used
    Active,
    
    /// Resource is locked by an effect or transaction
    Locked,
    
    /// Resource is frozen and cannot be modified
    Frozen,
    
    /// Resource has been consumed and cannot be used
    Consumed,
    
    /// Resource has been archived
    Archived,
}

/// Errors that can occur during lifecycle management
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LIFECYCLE_DEPRECATED
)]
#[derive(Debug, Error)]
pub enum LifecycleError {
    /// Resource does not exist
    #[error("Resource {0} does not exist")]
    ResourceNotFound(ContentId),
    
    /// Invalid state transition
    #[error("Invalid state transition from {0:?} to {1:?} for resource {2}")]
    InvalidStateTransition(ResourceState, ResourceState, ContentId),
    
    /// Resource is locked
    #[error("Resource {0} is locked by {1}")]
    ResourceLocked(ContentId, String),
    
    /// Generic error
    #[error("Lifecycle error: {0}")]
    Other(String),
}

/// Result type for lifecycle operations
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LIFECYCLE_DEPRECATED
)]
pub type LifecycleResult<T> = Result<T, LifecycleError>;

/// Legacy resource lifecycle manager
#[deprecated_error(
    since = messages::SINCE_VERSION,
    note = messages::LIFECYCLE_DEPRECATED
)]
pub struct ResourceLifecycle {
    /// Map of resource ID to current state
    states: RwLock<HashMap<ContentId, ResourceState>>,
    
    /// Map of resource ID to lock holder
    locks: RwLock<HashMap<ContentId, String>>,
    
    /// Set of resources that have been consumed
    consumed: RwLock<HashSet<ContentId>>,
}

impl ResourceLifecycle {
    /// Create a new resource lifecycle manager
    pub fn new() -> Self {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::new",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        Self {
            states: RwLock::new(HashMap::new()),
            locks: RwLock::new(HashMap::new()),
            consumed: RwLock::new(HashSet::new()),
        }
    }
    
    /// Register a new resource
    pub fn register_resource(&self, resource_id: ContentId, initial_state: ResourceState) -> LifecycleResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::register_resource",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        let mut states = self.states.write().unwrap();
        states.insert(resource_id, initial_state);
        Ok(())
    }
    
    /// Get the current state of a resource
    pub fn get_resource_state(&self, resource_id: &ContentId) -> LifecycleResult<ResourceState> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::get_resource_state",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        let states = self.states.read().unwrap();
        states.get(resource_id)
            .copied()
            .ok_or_else(|| LifecycleError::ResourceNotFound(resource_id.clone()))
    }
    
    /// Update the state of a resource
    pub fn update_resource_state(&self, resource_id: &ContentId, new_state: ResourceState) -> LifecycleResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::update_resource_state",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        let mut states = self.states.write().unwrap();
        
        let current_state = states.get(resource_id)
            .copied()
            .ok_or_else(|| LifecycleError::ResourceNotFound(resource_id.clone()))?;
        
        // Check for invalid transitions
        match (current_state, new_state) {
            // Can't transition from consumed
            (ResourceState::Consumed, _) => {
                return Err(LifecycleError::InvalidStateTransition(
                    current_state, new_state, resource_id.clone()
                ));
            },
            
            // Can't transition from archived except to consumed
            (ResourceState::Archived, state) if state != ResourceState::Consumed => {
                return Err(LifecycleError::InvalidStateTransition(
                    current_state, new_state, resource_id.clone()
                ));
            },
            
            // All other transitions are valid
            _ => {}
        }
        
        // Update state
        states.insert(resource_id.clone(), new_state);
        
        // Special handling for consumed state
        if new_state == ResourceState::Consumed {
            let mut consumed = self.consumed.write().unwrap();
            consumed.insert(resource_id.clone());
        }
        
        Ok(())
    }
    
    /// Check if a resource exists
    pub fn resource_exists(&self, resource_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::resource_exists",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        let states = self.states.read().unwrap();
        states.contains_key(resource_id)
    }
    
    /// Check if a resource has been consumed
    pub fn is_consumed(&self, resource_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::is_consumed",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        let consumed = self.consumed.read().unwrap();
        consumed.contains(resource_id)
    }
    
    /// Get all resources in a particular state
    pub fn get_resources_in_state(&self, state: ResourceState) -> Vec<ContentId> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLifecycle::get_resources_in_state",
            messages::SINCE_VERSION,
            messages::LIFECYCLE_DEPRECATED
        );
        
        let states = self.states.read().unwrap();
        states.iter()
            .filter(|(_, &s)| s == state)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for ResourceLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
