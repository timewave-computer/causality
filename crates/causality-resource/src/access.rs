// Resource access management (LEGACY VERSION)
//
// This module contains the deprecated implementation of resource access
// management. Use the ResourceAccess trait implementations in
// causality-effects::resource::access instead.

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use causality_common::identity::ContentId;
use thiserror::Error;

use crate::interface::deprecation::messages;
use crate::deprecated_warning;
use crate::deprecated_error;

/// Resource access types for legacy implementation
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::RESOURCE_ACCESS_DEPRECATED
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceAccessType {
    /// Read access to a resource
    Read,
    
    /// Write access to a resource
    Write,
    
    /// Execute access to a resource (e.g., running code)
    Execute,
    
    /// Admin access to a resource (full control)
    Admin,
}

/// Resource access error types
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::RESOURCE_ACCESS_DEPRECATED
)]
#[derive(Debug, Error)]
pub enum AccessError {
    /// Resource does not exist
    #[error("Resource {0} does not exist")]
    ResourceNotFound(ContentId),
    
    /// Access denied for the requested operation
    #[error("Access denied for {0:?} operation on resource {1}")]
    AccessDenied(ResourceAccessType, ContentId),
    
    /// Resource is locked by another entity
    #[error("Resource {0} is locked by {1}")]
    ResourceLocked(ContentId, String),
    
    /// Generic access error
    #[error("Access error: {0}")]
    Other(String),
}

/// Result type for access operations
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::RESOURCE_ACCESS_DEPRECATED
)]
pub type AccessResult<T> = Result<T, AccessError>;

/// Access record for a resource
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::RESOURCE_ACCESS_DEPRECATED
)]
#[derive(Debug, Clone)]
pub struct ResourceAccess {
    /// Resource being accessed
    pub resource_id: ContentId,
    
    /// Type of access
    pub access_type: ResourceAccessType,
    
    /// Effect or entity accessing the resource
    pub accessor_id: String,
    
    /// Timestamp of access
    pub timestamp: std::time::SystemTime,
}

/// Legacy resource access manager
#[deprecated_error(
    since = messages::SINCE_VERSION,
    note = messages::RESOURCE_ACCESS_DEPRECATED
)]
pub struct ResourceAccessManager {
    /// Map of resource ID to current accesses
    resource_accesses: RwLock<HashMap<ContentId, Vec<ResourceAccess>>>,
    
    /// Map of resource ID to locks
    resource_locks: RwLock<HashMap<ContentId, String>>,
    
    /// Set of resources requiring access control
    protected_resources: RwLock<HashSet<ContentId>>,
}

impl ResourceAccessManager {
    /// Create a new resource access manager
    pub fn new() -> Self {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::new",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        Self {
            resource_accesses: RwLock::new(HashMap::new()),
            resource_locks: RwLock::new(HashMap::new()),
            protected_resources: RwLock::new(HashSet::new()),
        }
    }
    
    /// Record access to a resource
    pub fn record_access(&self, access: ResourceAccess) -> AccessResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::record_access",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        // Check if resource is locked
        let locks = self.resource_locks.read().unwrap();
        if let Some(lock_holder) = locks.get(&access.resource_id) {
            if lock_holder != &access.accessor_id {
                return Err(AccessError::ResourceLocked(
                    access.resource_id.clone(),
                    lock_holder.clone(),
                ));
            }
        }
        
        // Record the access
        let mut accesses = self.resource_accesses.write().unwrap();
        let resource_accesses = accesses
            .entry(access.resource_id.clone())
            .or_insert_with(Vec::new);
        
        resource_accesses.push(access);
        
        Ok(())
    }
    
    /// Check if a resource is locked
    pub fn is_resource_locked(&self, resource_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::is_resource_locked",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        let locks = self.resource_locks.read().unwrap();
        locks.contains_key(resource_id)
    }
    
    /// Lock a resource for exclusive access
    pub fn lock_resource(&self, resource_id: &ContentId, locker_id: &str) -> AccessResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::lock_resource",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        let mut locks = self.resource_locks.write().unwrap();
        
        // Check if already locked
        if let Some(current_locker) = locks.get(resource_id) {
            return Err(AccessError::ResourceLocked(
                resource_id.clone(),
                current_locker.clone(),
            ));
        }
        
        // Lock the resource
        locks.insert(resource_id.clone(), locker_id.to_string());
        
        Ok(())
    }
    
    /// Release a lock on a resource
    pub fn release_lock(&self, resource_id: &ContentId, locker_id: &str) -> AccessResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::release_lock",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        let mut locks = self.resource_locks.write().unwrap();
        
        // Check if locked by someone else
        if let Some(current_locker) = locks.get(resource_id) {
            if current_locker != locker_id {
                return Err(AccessError::ResourceLocked(
                    resource_id.clone(),
                    current_locker.clone(),
                ));
            }
            
            // Release the lock
            locks.remove(resource_id);
            Ok(())
        } else {
            // Not locked
            Ok(())
        }
    }
    
    /// Get all accesses for a resource
    pub fn get_resource_accesses(&self, resource_id: &ContentId) -> Vec<ResourceAccess> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::get_resource_accesses",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        let accesses = self.resource_accesses.read().unwrap();
        accesses
            .get(resource_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Mark a resource as protected (requiring access control)
    pub fn protect_resource(&self, resource_id: &ContentId) {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::protect_resource",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        let mut protected = self.protected_resources.write().unwrap();
        protected.insert(resource_id.clone());
    }
    
    /// Check if a resource is protected
    pub fn is_resource_protected(&self, resource_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceAccessManager::is_resource_protected",
            messages::SINCE_VERSION,
            messages::RESOURCE_ACCESS_DEPRECATED
        );
        
        let protected = self.protected_resources.read().unwrap();
        protected.contains(resource_id)
    }
}

impl Default for ResourceAccessManager {
    fn default() -> Self {
        Self::new()
    }
} 