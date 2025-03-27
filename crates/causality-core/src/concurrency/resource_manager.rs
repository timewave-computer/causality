// Resource management utilities for concurrency
// Original file: src/concurrency/primitives/resource_manager.rs

// Resource manager for safe resource access
//
// This module provides a resource manager for tracking and controlling 
// access to resources in a deterministic way.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use causality_types::{Error, Result};
use causality_crypto::ContentId;
use :ResourceRegister:causality_core::resource::Resource::{ResourceRegister, RegisterState};

use super::{SharedWaitQueue, WaitQueue};
use super::ResourceGuard;
use super::resource_guard::ResourceRegisterGuard;

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
    // Resources and their current owners
    resources: Mutex<HashMap<ContentId, ResourceOwnership>>,
    // Wait queue for resource contention
    wait_queue: SharedWaitQueue,
    // Resource values
    values: Mutex<HashMap<ContentId, Box<dyn Any + Send + Sync>>>,
}

/// Resource ownership information
#[derive(Debug, Clone)]
struct ResourceOwnership {
    /// The ID of the resource
    id: ContentId,
    /// The current owner, if any
    owner: Option<String>,
    /// Whether the resource is locked for exclusive access
    locked: bool,
    /// Set of processes or tasks waiting for this resource
    waiters: HashSet<String>,
}

use std::any::Any;

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        ResourceManager {
            resources: Mutex::new(HashMap::new()),
            wait_queue: super::wait_queue::shared(),
            values: Mutex::new(HashMap::new()),
        }
    }
    
    /// Register a new resource with the manager
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
        
        resources.insert(id.clone(), ResourceOwnership {
            id: id.clone(),
            owner: None,
            locked: false,
            waiters: HashSet::new(),
        });
        
        // Store the initial value
        let mut values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        values.insert(id, Box::new(initial_value));
        
        Ok(())
    }
    
    /// Register a ResourceRegister with the manager
    pub fn register_resource_register(
        &self,
        register: ResourceRegister,
    ) -> Result<()> {
        let id = register.id.clone();
        
        // Use register_resource to handle the implementation details
        self.register_resource(id, register)
    }
    
    /// Attempt to acquire a resource
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
                
                // Try to acquire the resource
                match this.manager.try_acquire_resource::<T>(this.id.clone(), &this.requestor) {
                    Ok(Some(guard)) => {
                        // Successfully acquired the resource
                        Poll::Ready(Ok(guard))
                    },
                    Ok(None) => {
                        // Resource is locked, wait for it
                        // Add to wait queue if this is the first attempt
                        if this.attempts == 0 {
                            let owned_resources = match this.manager.get_resources_owned_by(&this.requestor) {
                                Ok(owned) => owned,
                                Err(e) => return Poll::Ready(Err(e)),
                            };
                            
                            if let Err(e) = this.manager.wait_queue.add_requestor(
                                this.id.clone(),
                                this.requestor.clone(),
                                owned_resources,
                            ) {
                                return Poll::Ready(Err(e));
                            }
                        }
                        
                        // Check if we're the next in line and the resource is available
                        match this.manager.wait_queue.is_next_requestor(this.id.clone(), &this.requestor) {
                            Ok(true) => {
                                // We're next, try to acquire again
                                match this.manager.try_acquire_resource::<T>(this.id.clone(), &this.requestor) {
                                    Ok(Some(guard)) => {
                                        // Successfully acquired the resource
                                        Poll::Ready(Ok(guard))
                                    },
                                    Ok(None) => {
                                        // Resource is still locked, wait more
                                        this.attempts += 1;
                                        Poll::Pending
                                    },
                                    Err(e) => Poll::Ready(Err(e)),
                                }
                            },
                            Ok(false) => {
                                // Not our turn yet, wait
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
        
        AcquireResourceFuture {
            manager: self,
            id,
            requestor: requestor.to_string(),
            attempts: 0,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Acquire a ResourceRegister
    pub fn acquire_resource_register(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> impl Future<Output = Result<ResourceRegisterGuard>> + '_ {
        // Use acquire_resource with ResourceRegister type
        self.acquire_resource::<ResourceRegister>(id, requestor)
    }
    
    /// Try to acquire a resource without waiting
    pub fn try_acquire_resource<T: Any + Send + Sync>(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> Result<Option<ResourceGuard<T>>> {
        let mut resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get_mut(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        if ownership.locked {
            return Ok(None);
        }
        
        // Lock the resource and set owner
        ownership.locked = true;
        ownership.owner = Some(requestor.to_string());
        
        // Get the resource value
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast to the requested type
        let value_ref = value.downcast_ref::<T>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        // Clone the value to return it with the guard
        let cloned_value = Clone::clone(value_ref);
        
        Ok(Some(ResourceGuard::new(
            id,
            Arc::new(self.clone()),
            requestor.to_string(),
            cloned_value,
        )))
    }
    
    /// Try to acquire a ResourceRegister without waiting
    pub fn try_acquire_resource_register(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> Result<Option<ResourceRegisterGuard>> {
        // Use try_acquire_resource with ResourceRegister type
        self.try_acquire_resource::<ResourceRegister>(id, requestor)
    }
    
    /// Get resources owned by a specific requestor
    fn get_resources_owned_by(&self, requestor: &str) -> Result<HashSet<ContentId>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let mut owned = HashSet::new();
        
        for (id, ownership) in resources.iter() {
            if ownership.owner.as_ref() == Some(&requestor.to_string()) {
                owned.insert(id.clone());
            }
        }
        
        Ok(owned)
    }
    
    /// Check if a resource is available
    pub fn is_resource_available(&self, id: ContentId) -> Result<bool> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        Ok(!ownership.locked)
    }
    
    /// Get the current owner of a resource
    pub fn resource_owner(&self, id: ContentId) -> Result<Option<String>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        Ok(ownership.owner.clone())
    }
    
    /// Get the waiters for a resource
    pub fn resource_waiters(&self, id: ContentId) -> Result<HashSet<String>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        Ok(ownership.waiters.clone())
    }
    
    /// Get a list of all resources managed by this resource manager
    pub fn list_resources(&self) -> Result<Vec<ContentId>> {
        let resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
            
        Ok(resources.keys().cloned().collect())
    }
    
    /// Release a resource
    ///
    /// This is typically called by the ResourceGuard's drop implementation.
    pub(crate) fn release_resource(&self, id: ContentId, owner: &str) -> Result<()> {
        let mut resources = self.resources.lock().map_err(|_| 
            Error::InternalError("Failed to lock resources".to_string()))?;
        
        let ownership = resources.get_mut(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
        
        // Verify the owner
        if ownership.owner.as_ref() != Some(&owner.to_string()) {
            return Err(Error::OperationFailed(
                format!("Resource not owned by {}: {:?}", owner, id)));
        }
        
        // Unlock the resource
        ownership.locked = false;
        ownership.owner = None;
        
        // Remove from wait queue
        let _ = self.wait_queue.remove_requestor(id.clone(), owner);
        
        // Notify the next waiter (if any) that the resource is available
        if let Some(next_waiter) = self.wait_queue.get_next_requestor(id.clone())? {
            // Remove the waiter from the resource's waiters
            ownership.waiters.remove(&next_waiter);
        }
        
        Ok(())
    }
    
    /// Update a resource value
    ///
    /// This is used internally by the resource guards to update the value
    /// when they release the resource.
    pub(crate) fn update_resource_value<T: Any + Send + Sync>(
        &self,
        id: ContentId,
        value: T,
    ) -> Result<()> {
        let mut values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        if !values.contains_key(&id) {
            return Err(Error::ResourceNotFound(id));
        }
        
        values.insert(id, Box::new(value));
        
        Ok(())
    }
    
    /// Update a ResourceRegister
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
            
        // Downcast to ResourceRegister
        let value_ref = value.downcast_ref::<ResourceRegister>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        // Clone the value to apply the update
        let mut cloned_value = value_ref.clone();
        
        // Apply the update
        update_fn(&mut cloned_value)?;
        
        // Store the updated value
        drop(values);
        
        let mut values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        values.insert(id.clone(), Box::new(cloned_value));
        
        Ok(())
    }
    
    /// Check if a ResourceRegister is active
    pub fn is_resource_register_active(&self, id: ContentId) -> Result<bool> {
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast to ResourceRegister
        let value_ref = value.downcast_ref::<ResourceRegister>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        Ok(value_ref.is_active())
    }
    
    /// Check if a ResourceRegister is consumed
    pub fn is_resource_register_consumed(&self, id: ContentId) -> Result<bool> {
        let values = self.values.lock().map_err(|_| 
            Error::InternalError("Failed to lock resource values".to_string()))?;
            
        let value = values.get(&id).ok_or_else(|| 
            Error::ResourceNotFound(id.clone()))?;
            
        // Downcast to ResourceRegister
        let value_ref = value.downcast_ref::<ResourceRegister>().ok_or_else(|| 
            Error::OperationFailed(format!("Resource type mismatch: {:?}", id)))?;
            
        Ok(value_ref.is_consumed())
    }
    
    /// Consume a ResourceRegister
    pub fn consume_resource_register(
        &self,
        id: ContentId,
        requestor: &str,
    ) -> Result<()> {
        // Update the register state to Consumed
        self.update_resource_register(id.clone(), |register| {
            register.consume()?;
            Ok(())
        })?;
        
        // Release the resource
        self.release_resource(id, requestor)
    }
}

// Implement Default for ResourceManager
impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone for ResourceManager
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
        
        // Register a resource
        manager.register_resource(ContentId::new("test"), String::from("test resource"))?;
        
        // Acquire the resource
        let guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor1").await?;
        
        // Verify the resource value
        assert_eq!(*guard, "test resource");
        
        // Check resource availability
        assert!(!manager.is_resource_available(ContentId::new("test"))?);
        
        // Check resource owner
        assert_eq!(manager.resource_owner(ContentId::new("test"))?, Some("requestor1".to_string()));
        
        // Release the resource
        drop(guard);
        
        // Check resource availability after release
        assert!(manager.is_resource_available(ContentId::new("test"))?);
        
        // Check resource owner after release
        assert_eq!(manager.resource_owner(ContentId::new("test"))?, None);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_resource_manager_contention() -> Result<()> {
        let manager = Arc::new(ResourceManager::new());
        
        // Register a resource
        manager.register_resource(ContentId::new("test"), String::from("test resource"))?;
        
        // Spawn a task to acquire the resource
        let manager1 = manager.clone();
        let handle1 = tokio::spawn(async move {
            let guard = manager1.acquire_resource::<String>(ContentId::new("test"), "requestor1").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
            Result::<(), Error>::Ok(())
        });
        
        // Give the first task time to acquire the resource
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Try to acquire the resource in this task
        let start = Instant::now();
        let guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor2").await?;
        let elapsed = start.elapsed();
        
        // Verify that we waited for the resource
        assert!(elapsed >= Duration::from_millis(90));
        
        // Verify the resource value
        assert_eq!(*guard, "test resource");
        
        // Wait for the first task to complete
        handle1.await.unwrap()?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_resource_manager_update_value() -> Result<()> {
        let manager = ResourceManager::new();
        
        // Register a resource
        manager.register_resource(ContentId::new("test"), String::from("test resource"))?;
        
        // Acquire the resource
        let mut guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor1").await?;
        
        // Modify the resource
        guard.push_str(" modified");
        
        // Release the resource
        drop(guard);
        
        // Acquire the resource again
        let guard = manager.acquire_resource::<String>(ContentId::new("test"), "requestor2").await?;
        
        // Verify the modified resource value
        assert_eq!(*guard, "test resource modified");
        
        Ok(())
    }
} 
