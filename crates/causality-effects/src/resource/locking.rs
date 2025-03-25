// Resource locking mechanism for cross-domain operations
// This file implements resource locking for coordinating operations across domain boundaries

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use causality_domain::domain::{DomainId, DomainAdapter};
use causality_types::{Error, Result, ContentId};
use crate::effect::{Effect, EffectContext, EffectId, EffectResult, EffectError, EffectOutcome};
use crate::resource::access::{ResourceAccessType, ResourceAccessTracker, ResourceAccess};

/// Lock acquisition status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStatus {
    /// Lock is acquired
    Acquired,
    
    /// Lock is already held
    AlreadyHeld,
    
    /// Lock is unavailable
    Unavailable,
    
    /// Lock acquisition timed out
    TimedOut,
}

/// Cross-domain lock type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossDomainLockType {
    /// Exclusive lock (read-write)
    Exclusive,
    
    /// Shared lock (read-only)
    Shared,
    
    /// Intent lock (preparing to lock)
    Intent,
}

/// Resource lock info
#[derive(Debug, Clone)]
pub struct ResourceLock {
    /// Resource ID
    pub resource_id: ContentId,
    
    /// Lock type
    pub lock_type: CrossDomainLockType,
    
    /// Domain ID where the resource is located
    pub domain_id: Option<DomainId>,
    
    /// Effect ID that holds the lock
    pub holder_id: EffectId,
    
    /// When the lock was acquired
    pub acquired_at: chrono::DateTime<chrono::Utc>,
    
    /// Lock timeout (if any)
    pub timeout: Option<Duration>,
    
    /// Transaction ID (if this lock is part of a transaction)
    pub transaction_id: Option<String>,
}

impl ResourceLock {
    /// Create a new resource lock
    pub fn new(
        resource_id: ContentId,
        lock_type: CrossDomainLockType,
        holder_id: EffectId,
    ) -> Self {
        Self {
            resource_id,
            lock_type,
            domain_id: None,
            holder_id,
            acquired_at: chrono::Utc::now(),
            timeout: None,
            transaction_id: None,
        }
    }
    
    /// Create a resource lock with domain
    pub fn with_domain(
        resource_id: ContentId,
        lock_type: CrossDomainLockType,
        domain_id: DomainId,
        holder_id: EffectId,
    ) -> Self {
        Self {
            resource_id,
            lock_type,
            domain_id: Some(domain_id),
            holder_id,
            acquired_at: chrono::Utc::now(),
            timeout: None,
            transaction_id: None,
        }
    }
    
    /// Set a timeout for the lock
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Associate the lock with a transaction
    pub fn with_transaction(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }
    
    /// Check if the lock has expired
    pub fn is_expired(&self) -> bool {
        if let Some(timeout) = self.timeout {
            let elapsed = chrono::Utc::now()
                .signed_duration_since(self.acquired_at)
                .to_std()
                .unwrap_or_else(|_| Duration::from_secs(0));
            
            elapsed > timeout
        } else {
            false
        }
    }
}

/// Lock manager for cross-domain operations
pub struct CrossDomainLockManager {
    /// Locks by resource ID
    locks: RwLock<HashMap<ContentId, Vec<ResourceLock>>>,
    
    /// Locks by domain ID
    locks_by_domain: RwLock<HashMap<DomainId, HashSet<ContentId>>>,
    
    /// Locks by effect ID
    locks_by_effect: RwLock<HashMap<EffectId, HashSet<ContentId>>>,
    
    /// Locks by transaction ID
    locks_by_transaction: RwLock<HashMap<String, HashSet<ContentId>>>,
    
    /// Resource access tracker
    access_tracker: Arc<ResourceAccessTracker>,
    
    /// Lock timeout handlers
    timeout_handlers: Mutex<HashMap<ContentId, Box<dyn Fn(ResourceLock) + Send + Sync>>>,
}

impl CrossDomainLockManager {
    /// Create a new cross-domain lock manager
    pub fn new(access_tracker: Arc<ResourceAccessTracker>) -> Self {
        Self {
            locks: RwLock::new(HashMap::new()),
            locks_by_domain: RwLock::new(HashMap::new()),
            locks_by_effect: RwLock::new(HashMap::new()),
            locks_by_transaction: RwLock::new(HashMap::new()),
            access_tracker,
            timeout_handlers: Mutex::new(HashMap::new()),
        }
    }
    
    /// Try to acquire a lock on a resource
    pub fn try_acquire_lock(
        &self,
        resource_id: &ContentId,
        lock_type: CrossDomainLockType,
        holder_id: &EffectId,
        domain_id: Option<&DomainId>,
    ) -> Result<LockStatus> {
        // Check if we already have this lock
        if self.is_lock_held(resource_id, holder_id) {
            return Ok(LockStatus::AlreadyHeld);
        }
        
        // Check if the lock can be acquired
        if !self.can_acquire_lock(resource_id, lock_type, holder_id) {
            return Ok(LockStatus::Unavailable);
        }
        
        // Create the lock
        let lock = if let Some(domain_id) = domain_id {
            ResourceLock::with_domain(
                resource_id.clone(),
                lock_type,
                domain_id.clone(),
                holder_id.clone(),
            )
        } else {
            ResourceLock::new(
                resource_id.clone(),
                lock_type,
                holder_id.clone(),
            )
        };
        
        // Record the lock
        self.add_lock(lock.clone())?;
        
        // Record in the access tracker
        let mut access = ResourceAccess::new(
            resource_id.clone(),
            ResourceAccessType::Lock,
            holder_id.clone(),
        );
        
        if let Some(domain_id) = domain_id {
            access.domain_id = Some(domain_id.clone());
        }
        
        access.grant();
        self.access_tracker.record_access(access)?;
        
        Ok(LockStatus::Acquired)
    }
    
    /// Acquire a lock with timeout
    pub async fn acquire_lock_with_timeout(
        &self,
        resource_id: &ContentId,
        lock_type: CrossDomainLockType,
        holder_id: &EffectId,
        domain_id: Option<&DomainId>,
        timeout: Duration,
    ) -> Result<LockStatus> {
        let start = Instant::now();
        
        // Try to acquire the lock immediately
        let mut status = self.try_acquire_lock(resource_id, lock_type, holder_id, domain_id)?;
        
        // If not acquired, keep trying until timeout
        while status == LockStatus::Unavailable && start.elapsed() < timeout {
            // Wait a bit before trying again
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Try again
            status = self.try_acquire_lock(resource_id, lock_type, holder_id, domain_id)?;
        }
        
        if status == LockStatus::Unavailable {
            Ok(LockStatus::TimedOut)
        } else {
            // If we acquired the lock, set its timeout
            if status == LockStatus::Acquired {
                if let Some(locks) = self.locks.write().unwrap().get_mut(resource_id) {
                    for lock in locks.iter_mut() {
                        if lock.holder_id == *holder_id {
                            lock.timeout = Some(timeout);
                        }
                    }
                }
            }
            
            Ok(status)
        }
    }
    
    /// Release a lock
    pub fn release_lock(
        &self,
        resource_id: &ContentId,
        holder_id: &EffectId,
    ) -> Result<()> {
        // Remove the lock
        let mut locks = self.locks.write().unwrap();
        
        if let Some(resource_locks) = locks.get_mut(resource_id) {
            // Find the lock with this holder
            let position = resource_locks.iter().position(|lock| lock.holder_id == *holder_id);
            
            if let Some(index) = position {
                // Get the lock for cleanup
                let lock = resource_locks.remove(index);
                
                // If this was the last lock, remove the resource entry
                if resource_locks.is_empty() {
                    locks.remove(resource_id);
                }
                
                // Update indices
                if let Some(domain_id) = &lock.domain_id {
                    let mut domain_locks = self.locks_by_domain.write().unwrap();
                    if let Some(resources) = domain_locks.get_mut(domain_id) {
                        resources.remove(resource_id);
                        if resources.is_empty() {
                            domain_locks.remove(domain_id);
                        }
                    }
                }
                
                // Update effect index
                let mut effect_locks = self.locks_by_effect.write().unwrap();
                if let Some(resources) = effect_locks.get_mut(holder_id) {
                    resources.remove(resource_id);
                    if resources.is_empty() {
                        effect_locks.remove(holder_id);
                    }
                }
                
                // Update transaction index
                if let Some(transaction_id) = &lock.transaction_id {
                    let mut transaction_locks = self.locks_by_transaction.write().unwrap();
                    if let Some(resources) = transaction_locks.get_mut(transaction_id) {
                        resources.remove(resource_id);
                        if resources.is_empty() {
                            transaction_locks.remove(transaction_id);
                        }
                    }
                }
                
                // Release in access tracker
                if let Err(e) = self.access_tracker.release_lock(resource_id, holder_id) {
                    // Ignore not found errors - the tracker might not have this lock
                    if !format!("{}", e).contains("No lock found") {
                        return Err(e);
                    }
                }
                
                return Ok(());
            }
        }
        
        Err(Error::NotFound(format!("No lock found for resource {} held by effect {}", resource_id, holder_id)))
    }
    
    /// Check if a lock is held by a specific effect
    pub fn is_lock_held(
        &self,
        resource_id: &ContentId,
        holder_id: &EffectId,
    ) -> bool {
        let locks = self.locks.read().unwrap();
        
        if let Some(resource_locks) = locks.get(resource_id) {
            resource_locks.iter().any(|lock| lock.holder_id == *holder_id)
        } else {
            false
        }
    }
    
    /// Check if a lock can be acquired
    pub fn can_acquire_lock(
        &self,
        resource_id: &ContentId,
        lock_type: CrossDomainLockType,
        holder_id: &EffectId,
    ) -> bool {
        let locks = self.locks.read().unwrap();
        
        if let Some(resource_locks) = locks.get(resource_id) {
            // If there are no locks, we can acquire
            if resource_locks.is_empty() {
                return true;
            }
            
            // Check compatibility with existing locks
            for lock in resource_locks {
                // Expired locks don't block acquisition
                if lock.is_expired() {
                    continue;
                }
                
                // Same holder can always acquire
                if lock.holder_id == *holder_id {
                    return true;
                }
                
                // Check compatibility based on lock type
                match (lock_type, lock.lock_type) {
                    // Exclusive locks are incompatible with any other lock
                    (CrossDomainLockType::Exclusive, _) => return false,
                    
                    // Other locks are incompatible with exclusive locks
                    (_, CrossDomainLockType::Exclusive) => return false,
                    
                    // Shared locks are compatible with other shared locks
                    (CrossDomainLockType::Shared, CrossDomainLockType::Shared) => continue,
                    
                    // Intent locks are compatible with shared locks
                    (CrossDomainLockType::Intent, CrossDomainLockType::Shared) => continue,
                    (CrossDomainLockType::Shared, CrossDomainLockType::Intent) => continue,
                    
                    // Intent locks are compatible with other intent locks
                    (CrossDomainLockType::Intent, CrossDomainLockType::Intent) => continue,
                }
            }
            
            // If we didn't find any incompatible locks, we can acquire
            true
        } else {
            // No locks at all, we can acquire
            true
        }
    }
    
    /// Get all locks for a resource
    pub fn get_locks(&self, resource_id: &ContentId) -> Vec<ResourceLock> {
        let locks = self.locks.read().unwrap();
        
        locks.get(resource_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all locks held by an effect
    pub fn get_locks_by_effect(&self, effect_id: &EffectId) -> Vec<ResourceLock> {
        let effect_locks = self.locks_by_effect.read().unwrap();
        
        if let Some(resources) = effect_locks.get(effect_id) {
            let locks = self.locks.read().unwrap();
            
            let mut result = Vec::new();
            for resource_id in resources {
                if let Some(resource_locks) = locks.get(resource_id) {
                    for lock in resource_locks {
                        if lock.holder_id == *effect_id {
                            result.push(lock.clone());
                        }
                    }
                }
            }
            
            result
        } else {
            Vec::new()
        }
    }
    
    /// Get all locks for a domain
    pub fn get_locks_by_domain(&self, domain_id: &DomainId) -> Vec<ResourceLock> {
        let domain_locks = self.locks_by_domain.read().unwrap();
        
        if let Some(resources) = domain_locks.get(domain_id) {
            let locks = self.locks.read().unwrap();
            
            let mut result = Vec::new();
            for resource_id in resources {
                if let Some(resource_locks) = locks.get(resource_id) {
                    for lock in resource_locks {
                        if lock.domain_id.as_ref().map_or(false, |id| id == domain_id) {
                            result.push(lock.clone());
                        }
                    }
                }
            }
            
            result
        } else {
            Vec::new()
        }
    }
    
    /// Get all locks for a transaction
    pub fn get_locks_by_transaction(&self, transaction_id: &str) -> Vec<ResourceLock> {
        let transaction_locks = self.locks_by_transaction.read().unwrap();
        
        if let Some(resources) = transaction_locks.get(transaction_id) {
            let locks = self.locks.read().unwrap();
            
            let mut result = Vec::new();
            for resource_id in resources {
                if let Some(resource_locks) = locks.get(resource_id) {
                    for lock in resource_locks {
                        if lock.transaction_id.as_ref().map_or(false, |id| id == transaction_id) {
                            result.push(lock.clone());
                        }
                    }
                }
            }
            
            result
        } else {
            Vec::new()
        }
    }
    
    /// Register a timeout handler
    pub fn register_timeout_handler<F>(
        &self,
        resource_id: ContentId,
        handler: F,
    ) where
        F: Fn(ResourceLock) + Send + Sync + 'static,
    {
        let mut handlers = self.timeout_handlers.lock().unwrap();
        handlers.insert(resource_id, Box::new(handler));
    }
    
    /// Check for expired locks and handle them
    pub fn handle_expired_locks(&self) -> Result<usize> {
        let mut locks_write = self.locks.write().unwrap();
        let mut expired_count = 0;
        
        // Get timeout handlers
        let handlers = self.timeout_handlers.lock().unwrap();
        
        // Check each resource's locks
        for (resource_id, resource_locks) in locks_write.iter_mut() {
            let mut expired_indices = Vec::new();
            
            // Find expired locks
            for (i, lock) in resource_locks.iter().enumerate() {
                if lock.is_expired() {
                    expired_indices.push(i);
                    
                    // Call timeout handler if registered
                    if let Some(handler) = handlers.get(resource_id) {
                        handler(lock.clone());
                    }
                }
            }
            
            // Remove expired locks (in reverse order to preserve indices)
            for &i in expired_indices.iter().rev() {
                let lock = resource_locks.remove(i);
                
                // Update indices
                if let Some(domain_id) = &lock.domain_id {
                    let mut domain_locks = self.locks_by_domain.write().unwrap();
                    if let Some(resources) = domain_locks.get_mut(domain_id) {
                        resources.remove(resource_id);
                        if resources.is_empty() {
                            domain_locks.remove(domain_id);
                        }
                    }
                }
                
                // Update effect index
                let mut effect_locks = self.locks_by_effect.write().unwrap();
                if let Some(resources) = effect_locks.get_mut(&lock.holder_id) {
                    resources.remove(resource_id);
                    if resources.is_empty() {
                        effect_locks.remove(&lock.holder_id);
                    }
                }
                
                // Update transaction index
                if let Some(transaction_id) = &lock.transaction_id {
                    let mut transaction_locks = self.locks_by_transaction.write().unwrap();
                    if let Some(resources) = transaction_locks.get_mut(transaction_id) {
                        resources.remove(resource_id);
                        if resources.is_empty() {
                            transaction_locks.remove(transaction_id);
                        }
                    }
                }
                
                expired_count += 1;
            }
        }
        
        // Remove empty resource entries
        locks_write.retain(|_, locks| !locks.is_empty());
        
        Ok(expired_count)
    }
    
    // Helper function to add a lock to internal structures
    fn add_lock(&self, lock: ResourceLock) -> Result<()> {
        let resource_id = lock.resource_id.clone();
        
        // Add to main locks map
        {
            let mut locks = self.locks.write().unwrap();
            locks.entry(resource_id.clone())
                .or_insert_with(Vec::new)
                .push(lock.clone());
        }
        
        // Add to domain index
        if let Some(domain_id) = &lock.domain_id {
            let mut domain_locks = self.locks_by_domain.write().unwrap();
            domain_locks.entry(domain_id.clone())
                .or_insert_with(HashSet::new)
                .insert(resource_id.clone());
        }
        
        // Add to effect index
        {
            let mut effect_locks = self.locks_by_effect.write().unwrap();
            effect_locks.entry(lock.holder_id.clone())
                .or_insert_with(HashSet::new)
                .insert(resource_id.clone());
        }
        
        // Add to transaction index
        if let Some(transaction_id) = &lock.transaction_id {
            let mut transaction_locks = self.locks_by_transaction.write().unwrap();
            transaction_locks.entry(transaction_id.clone())
                .or_insert_with(HashSet::new)
                .insert(resource_id.clone());
        }
        
        Ok(())
    }
}

/// Effect for acquiring a cross-domain resource lock
#[derive(Debug)]
pub struct AcquireLockEffect {
    /// Effect ID
    id: EffectId,
    
    /// Resource ID
    resource_id: ContentId,
    
    /// Lock type
    lock_type: CrossDomainLockType,
    
    /// Domain ID
    domain_id: Option<DomainId>,
    
    /// Timeout
    timeout: Option<Duration>,
    
    /// Transaction ID
    transaction_id: Option<String>,
}

impl AcquireLockEffect {
    /// Create a new acquire lock effect
    pub fn new(
        resource_id: ContentId,
        lock_type: CrossDomainLockType,
    ) -> Self {
        Self {
            id: EffectId::new(),
            resource_id,
            lock_type,
            domain_id: None,
            timeout: None,
            transaction_id: None,
        }
    }
    
    /// Set the domain ID
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Set a timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Associate with a transaction
    pub fn with_transaction(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    /// Get the lock type
    pub fn lock_type(&self) -> CrossDomainLockType {
        self.lock_type
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> Option<&DomainId> {
        self.domain_id.as_ref()
    }
}

/// Effect for releasing a cross-domain resource lock
#[derive(Debug)]
pub struct ReleaseLockEffect {
    /// Effect ID
    id: EffectId,
    
    /// Resource ID
    resource_id: ContentId,
    
    /// Domain ID
    domain_id: Option<DomainId>,
}

impl ReleaseLockEffect {
    /// Create a new release lock effect
    pub fn new(resource_id: ContentId) -> Self {
        Self {
            id: EffectId::new(),
            resource_id,
            domain_id: None,
        }
    }
    
    /// Set the domain ID
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> Option<&DomainId> {
        self.domain_id.as_ref()
    }
} 