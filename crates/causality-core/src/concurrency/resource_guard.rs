// Resource guard for safe resource access
// Original file: src/concurrency/primitives/resource_guard.rs

// Resource guard for safe resource access
//
// This module provides a RAII-style guard for resources, ensuring that
// resources are properly released when the guard goes out of scope.

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use causality_types::{Error, Result};
use causality_crypto::ContentId;
use :ResourceRegister:causality_core::resource::Resource::ResourceRegister;

// Forward declarations for ResourceManager
use super::ResourceManager;

/// A guard for a locked resource
///
/// When this guard is dropped, the resource is automatically released.
/// This provides RAII-style resource management, ensuring that resources
/// are always properly released, even in the case of panics.
///
/// Works with the unified ResourceRegister model.
#[derive(Debug)]
pub struct ResourceGuard<T> {
    /// The ID of the resource
    id: ContentId,
    /// The resource manager
    manager: Arc<ResourceManager>,
    /// The owner of the resource
    owner: String,
    /// The resource value
    resource: T,
}

impl<T> ResourceGuard<T> {
    /// Create a new resource guard
    ///
    /// This is typically called by the ResourceManager when acquiring a resource.
    pub(crate) fn new(
        id: ContentId,
        manager: Arc<ResourceManager>,
        owner: String,
        resource: T,
    ) -> Self {
        ResourceGuard {
            id,
            manager,
            owner,
            resource,
        }
    }
    
    /// Get the ID of the resource
    pub fn id(&self) -> ContentId {
        self.id.clone()
    }
    
    /// Get the owner of the resource
    pub fn owner(&self) -> &str {
        &self.owner
    }
    
    /// Get a reference to the underlying resource
    pub fn get(&self) -> &T {
        &self.resource
    }
    
    /// Get a mutable reference to the underlying resource
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.resource
    }
    
    /// Manually release the resource
    ///
    /// This is not typically needed as the resource is automatically
    /// released when the guard is dropped.
    ///
    /// Returns the underlying resource.
    pub fn release(self) -> Result<T> {
        // We need to consume self to ensure it's not used again,
        // but we want to return the resource.
        // So we deconstruct the guard, release the resource, and return the value.
        let ResourceGuard { id, manager, owner, resource } = self;
        
        manager.release_resource(id, &owner)?;
        
        Ok(resource)
    }
    
    /// Convert the guard into the underlying resource without releasing the lock
    ///
    /// This is unsafe because it bypasses the normal resource release mechanism.
    /// Only use this when you need to transfer ownership of the resource to another
    /// component that will manage the resource's lifecycle.
    pub unsafe fn into_inner(self) -> T {
        // Forgetting self prevents the Drop implementation from running,
        // which means the resource won't be released when this guard is dropped.
        let resource = self.resource;
        std::mem::forget(self);
        resource
    }
    
    /// Map the guard to a different type using a closure
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ResourceGuard<U> {
        let ResourceGuard { id, manager, owner, resource } = self;
        let new_resource = f(resource);
        
        ResourceGuard {
            id,
            manager,
            owner,
            resource: new_resource,
        }
    }
}

// Implement Deref and DerefMut for convenient access to the resource
impl<T> Deref for ResourceGuard<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl<T> DerefMut for ResourceGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}

// When the guard is dropped, release the resource
impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        let _ = self.manager.release_resource(self.id.clone(), &self.owner);
    }
}

/// A weak reference to a resource guard
///
/// This allows tracking a resource guard without preventing its release.
/// Useful for monitoring resource ownership without holding onto the resource.
#[derive(Debug, Clone)]
pub struct WeakResourceRef {
    /// The ID of the resource
    id: ContentId,
    /// The owner of the resource
    owner: String,
}

impl WeakResourceRef {
    /// Create a new weak resource reference
    pub fn new<T>(guard: &ResourceGuard<T>) -> Self {
        WeakResourceRef {
            id: guard.id(),
            owner: guard.owner().to_string(),
        }
    }
    
    /// Get the ID of the resource
    pub fn id(&self) -> ContentId {
        self.id.clone()
    }
    
    /// Get the owner of the resource
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

/// A specialized ResourceGuard for ResourceRegister
/// 
/// This provides convenient access to ResourceRegister operations
/// while maintaining the safety guarantees of ResourceGuard.
pub type ResourceRegisterGuard = ResourceGuard<ResourceRegister>;

impl ResourceRegisterGuard {
    /// Create a new ResourceRegisterGuard from a ResourceRegister
    pub fn from_resource_register(
        id: ContentId,
        manager: Arc<ResourceManager>,
        owner: String,
        register: ResourceRegister,
    ) -> Self {
        ResourceGuard::new(id, manager, owner, register)
    }
    
    /// Backward compatibility function
    #[deprecated(since = "0.8.0", note = "Use from_resource_register instead")]
    pub fn from_register(
        id: ContentId,
        manager: Arc<ResourceManager>,
        owner: String,
        register: ResourceRegister,
    ) -> Self {
        Self::from_resource_register(id, manager, owner, register)
    }
    
    /// Lock the underlying ResourceRegister
    pub fn lock(&mut self) -> Result<()> {
        self.resource.lock()?;
        Ok(())
    }
    
    /// Unlock the underlying ResourceRegister
    pub fn unlock(&mut self) -> Result<()> {
        self.resource.unlock()?;
        Ok(())
    }
    
    /// Consume the underlying ResourceRegister
    pub fn consume(mut self) -> Result<()> {
        self.resource.consume()?;
        self.release().map(|_| ())
    }
    
    /// Freeze the underlying ResourceRegister
    pub fn freeze(&mut self) -> Result<()> {
        self.resource.freeze()?;
        Ok(())
    }
    
    /// Unfreeze the underlying ResourceRegister
    pub fn unfreeze(&mut self) -> Result<()> {
        self.resource.unfreeze()?;
        Ok(())
    }
    
    /// Archive the underlying ResourceRegister
    pub fn archive(&mut self) -> Result<()> {
        self.resource.archive()?;
        Ok(())
    }
    
    /// Check if the ResourceRegister is active
    pub fn is_active(&self) -> bool {
        self.resource.is_active()
    }
    
    /// Check if the ResourceRegister is consumed
    pub fn is_consumed(&self) -> bool {
        self.resource.is_consumed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::resource_manager::ResourceManager;
    use std::sync::Arc;
    
    #[test]
    fn test_resource_guard_deref() {
        // Create a simple resource
        let resource = String::from("test resource");
        
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a resource guard
        let guard = ResourceGuard::new(
            ContentId::new("test"),
            manager,
            "owner".to_string(),
            resource,
        );
        
        // Test deref
        assert_eq!(*guard, "test resource");
        
        // Use a string method via deref
        assert_eq!(guard.len(), 13);
    }
    
    #[test]
    fn test_resource_guard_deref_mut() {
        // Create a simple resource
        let resource = String::from("test resource");
        
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a resource guard
        let mut guard = ResourceGuard::new(
            ContentId::new("test"),
            manager,
            "owner".to_string(),
            resource,
        );
        
        // Test deref_mut by modifying the resource
        guard.push_str(" modified");
        
        // Verify the modification
        assert_eq!(*guard, "test resource modified");
    }
    
    #[test]
    fn test_resource_guard_map() {
        // Create a simple resource
        let resource = String::from("42");
        
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a resource guard
        let guard = ResourceGuard::new(
            ContentId::new("test"),
            manager.clone(),
            "owner".to_string(),
            resource,
        );
        
        // Map the guard to a different type
        let int_guard = guard.map(|s| s.parse::<i32>().unwrap());
        
        // Test the mapped guard
        assert_eq!(*int_guard, 42);
        assert_eq!(int_guard.id(), ContentId::new("test"));
        assert_eq!(int_guard.owner(), "owner");
    }
    
    #[test]
    fn test_weak_resource_ref() {
        // Create a simple resource
        let resource = String::from("test resource");
        
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a resource guard
        let guard = ResourceGuard::new(
            ContentId::new("test"),
            manager,
            "owner".to_string(),
            resource,
        );
        
        // Create a weak reference
        let weak_ref = WeakResourceRef::new(&guard);
        
        // Test the weak reference
        assert_eq!(weak_ref.id(), ContentId::new("test"));
        assert_eq!(weak_ref.owner(), "owner");
    }
} 
