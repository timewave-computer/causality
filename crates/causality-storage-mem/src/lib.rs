//! In-memory Storage Implementation
//!
//! This module provides an implementation of storage interfaces using in-memory
//! structures. This is primarily useful for testing and development environments.

use causality_db::{Database, DbFactory};
use causality_error::{BoxError, StorageError};
use std::sync::Arc;

/// Mock ResourceId type for the storage implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl ResourceId {
    /// Create a new ResourceId from a string
    pub fn from_string(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Err("ResourceId cannot be empty".to_string());
        }
        Ok(Self(s.to_string()))
    }
    
    /// Convert the ResourceId to a string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// In-memory client for resource storage
pub struct MemoryClient {
    /// The underlying database
    db: Arc<Box<dyn Database>>,
}

impl MemoryClient {
    /// Create a new in-memory client
    pub fn new() -> Self {
        let db = DbFactory::create_memory_db()
            .expect("In-memory database creation should never fail");
            
        Self {
            db: Arc::new(db),
        }
    }
    
    /// Get a resource by ID
    pub fn get_resource(&self, id: &ResourceId) -> Result<Option<Vec<u8>>, BoxError> {
        let key = id.to_string().into_bytes();
        self.db.get(&key)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to get resource: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
    
    /// Store a resource
    pub fn store_resource(&self, id: &ResourceId, data: &[u8]) -> Result<(), BoxError> {
        let key = id.to_string().into_bytes();
        self.db.put(&key, data)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to store resource: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
    
    /// Delete a resource
    pub fn delete_resource(&self, id: &ResourceId) -> Result<(), BoxError> {
        let key = id.to_string().into_bytes();
        self.db.delete(&key)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to delete resource: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
}

impl Default for MemoryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_client() {
        // Create a memory client
        let client = MemoryClient::new();
        
        // Create a test resource ID
        let resource_id = ResourceId::from_string("test:resource:1").unwrap();
        let data = b"test data".to_vec();
        
        // Store and retrieve data
        client.store_resource(&resource_id, &data).unwrap();
        let retrieved = client.get_resource(&resource_id).unwrap();
        
        assert_eq!(retrieved, Some(data));
        
        // Delete data
        client.delete_resource(&resource_id).unwrap();
        let retrieved = client.get_resource(&resource_id).unwrap();
        
        assert_eq!(retrieved, None);
    }
} 