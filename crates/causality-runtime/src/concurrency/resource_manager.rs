// Resource manager for safe and deterministic concurrent resource access
// 
// This module provides a comprehensive resource management system that tracks resources
// and controls access to them in a deterministic way, preventing race conditions
// and deadlocks in concurrent environments.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::any::Any;

use anyhow::{Result, Error};
use causality_types::ContentId;

use super::{SharedWaitQueue, WaitQueue};
use super::ResourceGuard;
// FIXME: Uncomment when ResourceRegister is properly defined
// use super::resource_guard::ResourceRegisterGuard;

/// A shared resource manager that can be cloned and shared between components
pub type SharedResourceManager = Arc<ResourceManager>;

/// Create a shared instance of the resource manager
pub fn shared() -> SharedResourceManager {
    Arc::new(ResourceManager::new())
}

/// A resource manager for tracking and controlling access to resources
///
/// The ResourceManager is responsible for tracking resources and their
/// ownership, ensuring that resources are accessed safely in a concurrent
/// environment.
#[derive(Debug)]
pub struct ResourceManager {
    /// Map of resources to their ownership information, protected by a mutex
    resources: Mutex<HashMap<ContentId, ResourceOwnership>>,
    /// Queue for managing resource contention between concurrent requestors
    wait_queue: SharedWaitQueue,
    /// Actual resource values stored as boxed Any traits, protected by a mutex
    values: Mutex<HashMap<ContentId, Box<dyn Any + Send + Sync>>>,
}

/// Resource ownership information
#[derive(Debug, Clone)]
struct ResourceOwnership {
    /// The unique identifier of the resource
    id: ContentId,
    /// The current owner, if any (None if resource is not currently owned)
    owner: Option<String>,
    /// Whether the resource is locked for exclusive access
    locked: bool,
    /// Set of processes or tasks waiting for this resource to become available
    waiters: HashSet<String>,
}

impl ResourceManager {
    /// Create a new resource manager with empty collections
    pub fn new() -> Self {
        ResourceManager {
            resources: Mutex::new(HashMap::new()),
            wait_queue: super::wait_queue::shared(),
            values: Mutex::new(HashMap::new()),
        }
    }
    
    /// Register a new resource with the manager
    ///
    /// Adds a new resource to the manager, making it available for concurrent access.
    /// Returns an error if a resource with the same ID already exists.
    pub fn register_resource<T: Any + Send + Sync>(
        &self,
        id: ContentId,
        initial_value: T,
    ) -> Result<()> {
        let mut resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        if resources.contains_key(&id) {
            return Err(Error::OperationFailed(format!("Resource already registered: {:?}", id)));
        }
        
        // Create new ownership record with no owner and unlocked state
        resources.insert(id.clone(), ResourceOwnership {
            id: id.clone(),
            owner: None,
            locked: false,
            waiters: HashSet::new(),
        });
        
        // Store the initial resource value in the values collection
        let mut values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        values.insert(id, Box::new(initial_value));
        
        Ok(())
    }
    
    /// Register a ResourceRegister with the manager
    ///
    /// Convenience method that extracts the ID from the register and calls register_resource
    pub fn register_resource_register(
        &self,
        register: ResourceRegister,
    ) -> Result<()> {
        let id = register.id.clone();
        
        // Delegate to register_resource to handle the implementation details
        self.register_resource(id, register)
    }
    
    /// Attempt to acquire a resource asynchronously
    ///
    /// Returns a Future that will resolve when the resource becomes available.
    /// If the resource is currently locked, the requestor will be added to the wait queue.
    pub fn acquire_resource<T: Any + Send + Sync>(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> impl Future<Output = Result<ResourceGuard<T>>> + '_ {
        struct AcquireResourceFuture<'a, T: Any + Send + Sync> {
            manager: &'a ResourceManager,
            id: ContentId,
            requestor: String,
            attempts: usize,
            _phantom: std::marker::PhantomData<T>,
        }
        
        impl<'a, T: Any + Send + Sync> Future for AcquireResourceFuture<'a, T> {
            type Output = Result<ResourceGuard<T>>;
            
            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.get_mut();
                
                // First try to acquire the resource immediately without waiting
                match this.manager.try_acquire_resource::<T>(this.id.clone(), &this.requestor) {
                    Ok(Some(guard)) => {
                        // Resource was available and successfully acquired
                        Poll::Ready(Ok(guard))
                    },
                    Ok(None) => {
                        // Resource is locked by another requestor, need to wait
                        // Add to wait queue only on the first attempt to avoid duplicates
                        if this.attempts == 0 {
                            let owned_resources = match this.manager.get_resources_owned_by(&this.requestor) {
                                Ok(owned) => owned,
                                Err(e) => return Poll::Ready(Err(e)),
                            };
                            
                            // Add this requestor to the wait queue for this resource
                            if let Err(e) = this.manager.wait_queue.add_requestor(
                                this.id.clone(),
                                this.requestor.clone(),
                                owned_resources,
                            ) {
                                return Poll::Ready(Err(e));
                            }
                        }
                        
                        // Check if we're the next requestor in line for this resource
                        match this.manager.wait_queue.is_next_requestor(this.id.clone(), &this.requestor) {
                            Ok(true) => {
                                // We're next in line, try to acquire the resource again
                                match this.manager.try_acquire_resource::<T>(this.id.clone(), &this.requestor) {
                                    Ok(Some(guard)) => {
                                        // Successfully acquired the resource
                                        Poll::Ready(Ok(guard))
                                    },
                                    Ok(None) => {
                                        // Resource is still locked, continue waiting
                                        this.attempts += 1;
                                        Poll::Pending
                                    },
                                    Err(e) => Poll::Ready(Err(e)),
                                }
                            },
                            Ok(false) => {
                                // We're in the wait queue but not next in line
                                this.attempts += 1;
                                Poll::Pending
                            },
                            Err(e) => Poll::Ready(Err(e)),
                        }
                    },
                    Err(e) => Poll::Ready(Err(e)),
                }
            }
        }
        
        // Create and return the future that will handle the resource acquisition
        AcquireResourceFuture {
            manager: self,
            id,
            requestor: requestor.to_string(),
            attempts: 0,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Acquire a ResourceRegister asynchronously
    ///
    /// Convenience method that calls acquire_resource with the ResourceRegister type
    pub fn acquire_resource_register(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> impl Future<Output = Result<ResourceRegisterGuard>> + '_ {
        // Delegate to the generic acquire_resource implementation
        self.acquire_resource::<ResourceRegister>(id, requestor)
    }
    
    /// Try to acquire a resource without waiting
    ///
    /// Returns Some(guard) if the resource was successfully acquired,
    /// or None if the resource is currently locked by another requestor.
    pub fn try_acquire_resource<T: Any + Send + Sync>(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> Result<Option<ResourceGuard<T>>> {
        let mut resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get_mut(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        // If resource is already locked, return None without waiting
        if ownership.locked {
            return Ok(None);
        }
        
        // Resource is available - lock it and assign ownership to this requestor
        ownership.locked = true;
        ownership.owner = Some(requestor.to_string());
        
        // Retrieve the actual resource value from storage
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast the generic boxed Any to the requested concrete type
        let value_ref = value.downcast_ref::<T>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        // Clone the value to return it with the guard (value remains in storage)
        let cloned_value = Clone::clone(value_ref);
        
        // Create and return the guard that manages access to this resource
        Ok(Some(ResourceGuard::new(
            id,
            Arc::new(self.clone()),
            requestor.to_string(),
            cloned_value,
        )))
    }
    
    /// Try to acquire a ResourceRegister without waiting
    ///
    /// Convenience method that calls try_acquire_resource with the ResourceRegister type
    pub fn try_acquire_resource_register(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> Result<Option<ResourceRegisterGuard>> {
        // Delegate to the generic try_acquire_resource implementation
        self.try_acquire_resource::<ResourceRegister>(id, requestor)
    }
    
    /// Get all resources currently owned by a specific requestor
    ///
    /// Used internally to track resource ownership and prevent deadlocks
    fn get_resources_owned_by(&self, requestor: &str) -> Result<HashSet<ContentId>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let mut owned = HashSet::new();
        
        // Iterate through all resources and collect those owned by this requestor
        for (id, ownership) in resources.iter() {
            if ownership.owner.as_ref() == Some(&requestor.to_string()) {
                owned.insert(id.clone());
            }
        }
        
        Ok(owned)
    }
    
    /// Check if a resource is available for acquisition
    ///
    /// Returns true if the resource exists and is not currently locked
    pub fn is_resource_available(&self, id: ContentId) -> Result<bool> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        // Resource is available if it's not locked
        Ok(!ownership.locked)
    }
    
    /// Get the current owner of a resource
    ///
    /// Returns the ID of the current owner, or None if the resource is not owned
    pub fn resource_owner(&self, id: ContentId) -> Result<Option<String>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        Ok(ownership.owner.clone())
    }
    
    /// Get all requestors waiting for a resource
    ///
    /// Returns the set of requestor IDs that are waiting to acquire this resource
    pub fn resource_waiters(&self, id: ContentId) -> Result<HashSet<String>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        Ok(ownership.waiters.clone())
    }
    
    /// Get a list of all resources managed by this resource manager
    ///
    /// Returns the ContentIds of all registered resources
    pub fn list_resources(&self) -> Result<Vec<ContentId>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
            
        Ok(resources.keys().cloned().collect())
    }
    
    /// Release a resource previously acquired by a requestor
    ///
    /// This is typically called automatically by the ResourceGuard's drop implementation.
    /// Unlocks the resource, removes the owner, and notifies waiters.
    pub(crate) fn release_resource(&self, id: ContentId, owner: &str) -> Result<()> {
        let mut resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get_mut(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        // Verify that the release request comes from the current owner
        if ownership.owner.as_ref() != Some(&owner.to_string()) {
            return Err(Error::OperationFailed(
                format!("Resource not owned by {}: {:?}", owner, id)));
        }
        
        // Mark the resource as available by unlocking it and clearing the owner
        ownership.locked = false;
        ownership.owner = None;
        
        // Remove the owner from the wait queue for this resource
        let _ = self.wait_queue.remove_requestor(id.clone(), owner);
        
        // Notify the next waiter (if any) that the resource is now available
        if let Some(next_waiter) = self.wait_queue.get_next_requestor(id.clone())? {
            // Remove this waiter from the resource's waiters set
            ownership.waiters.remove(&next_waiter);
        }
        
        Ok(())
    }
    
    /// Update a resource value
    ///
    /// This is used internally by the resource guards to update the stored value
    /// when they release the resource, allowing modifications to persist.
    pub(crate) fn update_resource_value<T: Any + Send + Sync>(
        &self,
        id: ContentId,
        value: T,
    ) -> Result<()> {
        let mut values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        // Verify the resource exists before updating
        if !values.contains_key(&id) {
            return Err(Error::ResourceNotFound(id));
        }
        
        // Replace the existing value with the updated one
        values.insert(id, Box::new(value));
        
        Ok(())
    }
    
    /// Update a ResourceRegister using a provided update function
    ///
    /// Allows for atomic updates to a ResourceRegister by applying a function
    /// that modifies the register in place.
    pub fn update_resource_register<F>(
        &self,
        id: ContentId,
        update_fn: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut ResourceRegister) -> Result<()>,
    {
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast the generic boxed Any to ResourceRegister type
        let value_ref = value.downcast_ref::<ResourceRegister>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        // Clone the value to avoid modifying it while holding the lock
        let mut cloned_value = value_ref.clone();
        
        // Apply the update function to the cloned value
        update_fn(&mut cloned_value)?;
        
        // Release the read lock before acquiring the write lock
        drop(values);
        
        // Acquire write lock and store the updated value
        let mut values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        values.insert(id.clone(), Box::new(cloned_value));
        
        Ok(())
    }
    
    /// Check if a ResourceRegister is in the active state
    ///
    /// Returns true if the register exists and is in the active state.
    pub fn is_resource_register_active(&self, id: ContentId) -> Result<bool> {
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast to ResourceRegister and check its active state
        let value_ref = value.downcast_ref::<ResourceRegister>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        Ok(value_ref.is_active())
    }
    
    /// Check if a ResourceRegister is in the consumed state
    ///
    /// Returns true if the register exists and has been consumed.
    pub fn is_resource_register_consumed(&self, id: ContentId) -> Result<bool> {
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast to ResourceRegister and check its consumed state
        let value_ref = value.downcast_ref::<ResourceRegister>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        Ok(value_ref.is_consumed())
    }
    
    /// Consume a ResourceRegister
    ///
    /// Marks a ResourceRegister as consumed and releases the resource,
    /// making it available to other requestors.
    pub fn consume_resource_register(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> Result<()> {
        // First update the register's state to Consumed
        self.update_resource_register(id.clone(), |register| {
            register.consume()?;
            Ok(())
        })?;
        
        // Then release the resource so others can acquire it
        self.release_resource(id, requestor)
    }
}

// Implement Default for ResourceManager using the new() method
impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Custom Clone implementation for ResourceManager to handle mutex cloning properly
impl Clone for ResourceManager {
    fn clone(&self) -> Self {
        ResourceManager {
            resources: Mutex::new(self.resources.lock().unwrap().clone()),
            wait_queue: self.wait_queue.clone(),
            values: Mutex::new(self.values.lock().unwrap().clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[tokio::test]
    async fn test_resource_manager_basic() -> Result<()> {
        let manager = ResourceManager::new();
        
        // Register a new test resource with a string value
        manager.register_resource(ContentId::new("test"), String::from("test resource"))?;
        
        // Acquire the resource with a specific requestor ID
        let guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor1").await?;
        
        // Verify that the resource value matches what we registered
        assert_eq!(*guard, "test resource");
        
        // Resource should not be available while we hold the guard
        assert!(!manager.is_resource_available(ContentId::new("test"))?);
        
        // The owner should be set to our requestor ID
        assert_eq!(manager.resource_owner(ContentId::new("test"))?, Some("requestor1".to_string()));
        
        // Release the resource by dropping the guard
        drop(guard);
        
        // Resource should now be available again
        assert!(manager.is_resource_available(ContentId::new("test"))?);
        
        // No owner should be set after release
        assert_eq!(manager.resource_owner(ContentId::new("test"))?, None);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_resource_manager_contention() -> Result<()> {
        let manager = Arc::new(ResourceManager::new());
        
        // Register a test resource
        manager.register_resource(ContentId::new("test"), String::from("test resource"))?;
        
        // Spawn a task that acquires the resource and holds it for 100ms
        let manager1 = manager.clone();
        let handle1 = tokio::spawn(async move {
            let guard = manager1.acquire_resource::<String>(ContentId::new("test"), "requestor1").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
            Result::<(), Error>::Ok(())
        });
        
        // Give the first task time to acquire the resource
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Try to acquire the same resource in this task - should wait
        let start = Instant::now();
        let guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor2").await?;
        let elapsed = start.elapsed();
        
        // Verify that we had to wait for approximately 90ms
        assert!(elapsed >= Duration::from_millis(90));
        
        // Verify we got the correct resource value
        assert_eq!(*guard, "test resource");
        
        // Wait for the first task to complete
        handle1.await.unwrap()?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_resource_manager_update_value() -> Result<()> {
        let manager = ResourceManager::new();
        
        // Register a test resource with a string value
        manager.register_resource(ContentId::new("test"), String::from("test resource"))?;
        
        // Acquire the resource and get a mutable guard
        let mut guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor1").await?;
        
        // Modify the resource value through the guard
        guard.push_str(" modified");
        
        // Release the resource, updates should be saved
        drop(guard);
        
        // Acquire the resource again with a different requestor
        let guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor2").await?;
        
        // Verify that the modified value was persisted
        assert_eq!(*guard, "test resource modified");
        
        Ok(())
    }
} 
