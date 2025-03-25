// Resource locking management (LEGACY VERSION)
//
// This module contains the deprecated implementation of resource locking
// management. Use the ResourceLocking trait implementations in
// causality-effects::resource::locking instead.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use causality_common::identity::ContentId;
use thiserror::Error;

use crate::interface::deprecation::messages;
use crate::deprecated_warning;
use crate::deprecated_error;

/// Types of locks for resources
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LOCKING_DEPRECATED
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockType {
    /// Exclusive lock (only one holder can access the resource)
    Exclusive,
    
    /// Shared lock (multiple readers can access the resource)
    Shared,
    
    /// Intent lock (signaling intent to acquire an exclusive lock soon)
    Intent,
}

/// Resource lock information
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LOCKING_DEPRECATED
)]
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// Type of lock
    pub lock_type: LockType,
    
    /// ID of the lock holder
    pub holder_id: ContentId,
    
    /// Optional ID of the domain that the lock is valid in
    pub domain_id: Option<ContentId>,
    
    /// When the lock was acquired
    pub acquired_at: SystemTime,
    
    /// Optional expiration time
    pub expires_at: Option<SystemTime>,
}

/// Errors that can occur during locking operations
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LOCKING_DEPRECATED
)]
#[derive(Debug, Error)]
pub enum LockError {
    /// Resource already locked by another entity
    #[error("Resource {0} is locked by {1}")]
    AlreadyLocked(ContentId, ContentId),
    
    /// Lock has expired
    #[error("Lock on resource {0} has expired")]
    LockExpired(ContentId),
    
    /// Not the lock holder
    #[error("Cannot release lock on {0}: not the lock holder")]
    NotLockHolder(ContentId),
    
    /// Incompatible lock type
    #[error("Incompatible lock type requested for resource {0}")]
    IncompatibleLockType(ContentId),
    
    /// Generic lock error
    #[error("Lock error: {0}")]
    Other(String),
}

/// Result type for locking operations
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::LOCKING_DEPRECATED
)]
pub type LockResult<T> = Result<T, LockError>;

/// Legacy resource lock manager
#[deprecated_error(
    since = messages::SINCE_VERSION,
    note = messages::LOCKING_DEPRECATED
)]
pub struct ResourceLockManager {
    /// Map of resource ID to lock information
    locks: RwLock<HashMap<ContentId, LockInfo>>,
    
    /// Map of resource ID to shared lock holders
    shared_locks: RwLock<HashMap<ContentId, Vec<ContentId>>>,
    
    /// Default lock timeout
    default_timeout: Option<Duration>,
}

impl ResourceLockManager {
    /// Create a new resource lock manager
    pub fn new() -> Self {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::new",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        Self {
            locks: RwLock::new(HashMap::new()),
            shared_locks: RwLock::new(HashMap::new()),
            default_timeout: None,
        }
    }
    
    /// Create a new resource lock manager with a default timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::with_timeout",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        Self {
            locks: RwLock::new(HashMap::new()),
            shared_locks: RwLock::new(HashMap::new()),
            default_timeout: Some(timeout),
        }
    }
    
    /// Acquire a lock on a resource
    pub fn acquire_lock(
        &self,
        resource_id: &ContentId,
        lock_type: LockType,
        holder_id: &ContentId,
        domain_id: Option<ContentId>,
        timeout: Option<Duration>,
    ) -> LockResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::acquire_lock",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        // Calculate expiration time if timeout provided
        let expires_at = timeout.or(self.default_timeout).map(|t| {
            SystemTime::now().checked_add(t).unwrap_or_else(|| {
                // Handle potential overflow
                SystemTime::now().checked_add(Duration::from_secs(u64::MAX / 2))
                    .unwrap_or(SystemTime::now())
            })
        });
        
        match lock_type {
            LockType::Exclusive => self.acquire_exclusive_lock(resource_id, holder_id, domain_id, expires_at),
            LockType::Shared => self.acquire_shared_lock(resource_id, holder_id, domain_id, expires_at),
            LockType::Intent => self.acquire_intent_lock(resource_id, holder_id, domain_id, expires_at),
        }
    }
    
    /// Release a lock on a resource
    pub fn release_lock(
        &self,
        resource_id: &ContentId,
        holder_id: &ContentId,
    ) -> LockResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::release_lock",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        // Check exclusive locks
        let mut locks = self.locks.write().unwrap();
        if let Some(lock_info) = locks.get(resource_id) {
            if &lock_info.holder_id != holder_id {
                return Err(LockError::NotLockHolder(resource_id.clone()));
            }
            
            // Remove the lock
            locks.remove(resource_id);
        }
        
        // Check shared locks
        let mut shared_locks = self.shared_locks.write().unwrap();
        if let Some(holders) = shared_locks.get_mut(resource_id) {
            // Remove this holder
            holders.retain(|id| id != holder_id);
            
            // Remove the entry if no more holders
            if holders.is_empty() {
                shared_locks.remove(resource_id);
            }
        }
        
        Ok(())
    }
    
    /// Check if a resource is locked
    pub fn is_locked(&self, resource_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::is_locked",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        let locks = self.locks.read().unwrap();
        let shared_locks = self.shared_locks.read().unwrap();
        
        locks.contains_key(resource_id) || shared_locks.contains_key(resource_id)
    }
    
    /// Get information about a lock
    pub fn get_lock_info(&self, resource_id: &ContentId) -> Option<LockInfo> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::get_lock_info",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        let locks = self.locks.read().unwrap();
        locks.get(resource_id).cloned()
    }
    
    /// Check if a resource is locked by a specific holder
    pub fn is_locked_by(&self, resource_id: &ContentId, holder_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::is_locked_by",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        // Check exclusive locks
        let locks = self.locks.read().unwrap();
        if let Some(lock_info) = locks.get(resource_id) {
            if &lock_info.holder_id == holder_id {
                return true;
            }
        }
        
        // Check shared locks
        let shared_locks = self.shared_locks.read().unwrap();
        if let Some(holders) = shared_locks.get(resource_id) {
            return holders.contains(holder_id);
        }
        
        false
    }
    
    /// Get all resources locked by a specific holder
    pub fn get_locked_resources_by(&self, holder_id: &ContentId) -> Vec<ContentId> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceLockManager::get_locked_resources_by",
            messages::SINCE_VERSION,
            messages::LOCKING_DEPRECATED
        );
        
        let mut result = Vec::new();
        
        // Check exclusive locks
        let locks = self.locks.read().unwrap();
        for (resource_id, lock_info) in locks.iter() {
            if &lock_info.holder_id == holder_id {
                result.push(resource_id.clone());
            }
        }
        
        // Check shared locks
        let shared_locks = self.shared_locks.read().unwrap();
        for (resource_id, holders) in shared_locks.iter() {
            if holders.contains(holder_id) {
                result.push(resource_id.clone());
            }
        }
        
        result
    }
    
    // Private helper methods
    
    fn acquire_exclusive_lock(
        &self,
        resource_id: &ContentId,
        holder_id: &ContentId,
        domain_id: Option<ContentId>,
        expires_at: Option<SystemTime>,
    ) -> LockResult<()> {
        // Check if already locked
        let mut locks = self.locks.write().unwrap();
        if let Some(lock_info) = locks.get(resource_id) {
            // Check if lock has expired
            if let Some(expiry) = lock_info.expires_at {
                if SystemTime::now() > expiry {
                    // Lock has expired, allow re-locking
                } else {
                    return Err(LockError::AlreadyLocked(
                        resource_id.clone(),
                        lock_info.holder_id.clone(),
                    ));
                }
            } else {
                return Err(LockError::AlreadyLocked(
                    resource_id.clone(),
                    lock_info.holder_id.clone(),
                ));
            }
        }
        
        // Check for shared locks
        let shared_locks = self.shared_locks.read().unwrap();
        if shared_locks.contains_key(resource_id) {
            return Err(LockError::IncompatibleLockType(resource_id.clone()));
        }
        
        // Create lock info
        let lock_info = LockInfo {
            lock_type: LockType::Exclusive,
            holder_id: holder_id.clone(),
            domain_id,
            acquired_at: SystemTime::now(),
            expires_at,
        };
        
        // Store the lock
        locks.insert(resource_id.clone(), lock_info);
        
        Ok(())
    }
    
    fn acquire_shared_lock(
        &self,
        resource_id: &ContentId,
        holder_id: &ContentId,
        domain_id: Option<ContentId>,
        expires_at: Option<SystemTime>,
    ) -> LockResult<()> {
        // Check for exclusive lock
        let locks = self.locks.read().unwrap();
        if let Some(lock_info) = locks.get(resource_id) {
            if lock_info.lock_type == LockType::Exclusive || lock_info.lock_type == LockType::Intent {
                return Err(LockError::AlreadyLocked(
                    resource_id.clone(),
                    lock_info.holder_id.clone(),
                ));
            }
        }
        drop(locks);
        
        // Add shared lock
        let mut shared_locks = self.shared_locks.write().unwrap();
        let holders = shared_locks.entry(resource_id.clone()).or_insert_with(Vec::new);
        
        // Add holder if not already present
        if !holders.contains(holder_id) {
            holders.push(holder_id.clone());
        }
        
        // Store metadata in exclusive lock table if this is the first shared lock
        if holders.len() == 1 {
            let mut locks = self.locks.write().unwrap();
            let lock_info = LockInfo {
                lock_type: LockType::Shared,
                holder_id: holder_id.clone(), // First holder as representative
                domain_id,
                acquired_at: SystemTime::now(),
                expires_at,
            };
            locks.insert(resource_id.clone(), lock_info);
        }
        
        Ok(())
    }
    
    fn acquire_intent_lock(
        &self,
        resource_id: &ContentId,
        holder_id: &ContentId,
        domain_id: Option<ContentId>,
        expires_at: Option<SystemTime>,
    ) -> LockResult<()> {
        // Intent locks can coexist with shared locks but not exclusive locks
        let locks = self.locks.read().unwrap();
        if let Some(lock_info) = locks.get(resource_id) {
            if lock_info.lock_type == LockType::Exclusive || lock_info.lock_type == LockType::Intent {
                return Err(LockError::AlreadyLocked(
                    resource_id.clone(),
                    lock_info.holder_id.clone(),
                ));
            }
        }
        drop(locks);
        
        // Create lock info
        let lock_info = LockInfo {
            lock_type: LockType::Intent,
            holder_id: holder_id.clone(),
            domain_id,
            acquired_at: SystemTime::now(),
            expires_at,
        };
        
        // Store the lock
        let mut locks = self.locks.write().unwrap();
        locks.insert(resource_id.clone(), lock_info);
        
        Ok(())
    }
}

impl Default for ResourceLockManager {
    fn default() -> Self {
        Self::new()
    }
} 