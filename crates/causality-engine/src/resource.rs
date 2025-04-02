// Resource module for Causality Engine
//
// This module provides interfaces and implementations for resource allocation
// and management in the Causality system.

use std::fmt::Debug;
use async_trait::async_trait;
use causality_error::Result;
use causality_types::crypto_primitives::ContentHash;
use causality_core::resource::types::ResourceId;

/// A resource allocator for managing system resources
#[async_trait]
pub trait ResourceAllocator: Debug + Send + Sync {
    /// Allocate a resource and get its ID
    async fn allocate(&self, resource_type: &str, data: &[u8]) -> Result<ResourceId>;
    
    /// Get a resource by its ID
    async fn get_resource(&self, id: &ResourceId) -> Result<Option<Vec<u8>>>;
    
    /// Check if a resource exists
    async fn has_resource(&self, id: &ResourceId) -> Result<bool>;
    
    /// Release a resource
    async fn release(&self, id: &ResourceId) -> Result<bool>;
    
    /// Get the type of a resource
    async fn get_resource_type(&self, id: &ResourceId) -> Result<Option<String>>;
}

/// A simple in-memory implementation of a resource allocator for testing
#[cfg(test)]
pub mod memory {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use causality_core::resource::types::ResourceId;
    
    /// Resource data stored in memory
    struct ResourceData {
        resource_type: String,
        data: Vec<u8>,
    }
    
    /// An in-memory resource allocator
    #[derive(Debug)]
    pub struct InMemoryResourceAllocator {
        resources: Arc<RwLock<HashMap<ResourceId, ResourceData>>>,
    }
    
    impl InMemoryResourceAllocator {
        /// Create a new in-memory resource allocator
        pub fn new() -> Self {
            InMemoryResourceAllocator {
                resources: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }
    
    #[async_trait]
    impl ResourceAllocator for InMemoryResourceAllocator {
        async fn allocate(&self, resource_type: &str, data: &[u8]) -> Result<ResourceId> {
            use causality_crypto::{HashFactory, ContentAddressed};
            
            // Create a unique ID based on type and data
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher()?;
            
            // Combine type and data for hashing
            let mut combined = resource_type.as_bytes().to_vec();
            combined.extend_from_slice(data);
            
            let hash = hasher.hash(&combined);
            let content_hash = ContentHash::from_bytes(hash.as_bytes())
                .map_err(|e| causality_error::Error::CryptoError(e.to_string()))?;
            
            // Create a resource ID
            let resource_id = ResourceId::from_content_hash(content_hash);
            
            // Store the resource
            let resource_data = ResourceData {
                resource_type: resource_type.to_string(),
                data: data.to_vec(),
            };
            
            let mut resources = self.resources.write().map_err(|_| causality_error::Error::LockError)?;
            resources.insert(resource_id.clone(), resource_data);
            
            Ok(resource_id)
        }
        
        async fn get_resource(&self, id: &ResourceId) -> Result<Option<Vec<u8>>> {
            let resources = self.resources.read().map_err(|_| causality_error::Error::LockError)?;
            
            if let Some(resource_data) = resources.get(id) {
                Ok(Some(resource_data.data.clone()))
            } else {
                Ok(None)
            }
        }
        
        async fn has_resource(&self, id: &ResourceId) -> Result<bool> {
            let resources = self.resources.read().map_err(|_| causality_error::Error::LockError)?;
            Ok(resources.contains_key(id))
        }
        
        async fn release(&self, id: &ResourceId) -> Result<bool> {
            let mut resources = self.resources.write().map_err(|_| causality_error::Error::LockError)?;
            Ok(resources.remove(id).is_some())
        }
        
        async fn get_resource_type(&self, id: &ResourceId) -> Result<Option<String>> {
            let resources = self.resources.read().map_err(|_| causality_error::Error::LockError)?;
            
            if let Some(resource_data) = resources.get(id) {
                Ok(Some(resource_data.resource_type.clone()))
            } else {
                Ok(None)
            }
        }
    }
} 