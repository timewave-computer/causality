// Content-addressed storage interfaces and implementations
//
// This module defines traits and implementations for content-addressed storage systems.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::crypto_primitives::{ContentId, ContentAddressed};

// Export error types
pub mod error;
pub use error::{StorageError, StorageResult};

/// Standard content-addressed storage interface
pub trait ContentAddressedStorage: Send + Sync + Debug {
    /// Store binary data and return content ID
    fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError>;
    
    /// Check if an object exists in storage
    fn contains(&self, id: &ContentId) -> bool;
    
    /// Retrieve binary data for an object
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError>;
    
    /// Remove an object from storage
    fn remove(&self, id: &ContentId) -> Result<(), StorageError>;
    
    /// Clear all objects from storage
    fn clear(&self);
    
    /// Get the number of objects in storage
    fn len(&self) -> usize;
    
    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension methods for ContentAddressedStorage
pub trait ContentAddressedStorageExt: ContentAddressedStorage {
    /// Store an object in the content-addressed storage
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentId, StorageError> {
        // Serialize the object
        let bytes = object.to_bytes()
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        // Store the bytes
        self.store_bytes(&bytes)
    }
    
    /// Retrieve an object from storage by its content ID
    fn get<T: ContentAddressed>(&self, id: &ContentId) -> Result<T, StorageError> {
        let bytes = self.get_bytes(id)?;
        T::from_bytes(&bytes)
            .map_err(|e| StorageError::DeserializationError(e.to_string()))
    }
}

// Automatically implement the extension trait for all implementors of ContentAddressedStorage
impl<T: ContentAddressedStorage> ContentAddressedStorageExt for T {}

/// In-memory implementation of content-addressed storage
#[derive(Debug)]
pub struct InMemoryStorage {
    /// Internal storage mapping content IDs to binary data
    data: RwLock<HashMap<ContentId, Vec<u8>>>,
}

impl InMemoryStorage {
    /// Create a new empty in-memory storage instance
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl ContentAddressedStorage for InMemoryStorage {
    fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError> {
        // Generate a content ID from the bytes
        let content_id = crate::content_addressing::content_id_from_bytes(bytes);
        
        // Check if already exists
        let mut data = self.data.write().map_err(|_| 
            StorageError::IoError("Failed to acquire write lock".to_string())
        )?;
        
        if data.contains_key(&content_id) {
            return Err(StorageError::Duplicate(format!("Object already exists: {}", content_id)));
        }
        
        // Store the data
        data.insert(content_id.clone(), bytes.to_vec());
        
        Ok(content_id)
    }
    
    fn contains(&self, id: &ContentId) -> bool {
        match self.data.read() {
            Ok(data) => data.contains_key(id),
            Err(_) => false, // If we can't acquire the lock, assume not found
        }
    }
    
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
        let data = self.data.read().map_err(|_| 
            StorageError::IoError("Failed to acquire read lock".to_string())
        )?;
        
        data.get(id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(format!("Object not found: {}", id)))
    }
    
    fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
        let mut data = self.data.write().map_err(|_| 
            StorageError::IoError("Failed to acquire write lock".to_string())
        )?;
        
        if data.remove(id).is_none() {
            return Err(StorageError::NotFound(format!("Object not found: {}", id)));
        }
        
        Ok(())
    }
    
    fn clear(&self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
    }
    
    fn len(&self) -> usize {
        match self.data.read() {
            Ok(data) => data.len(),
            Err(_) => 0, // If we can't acquire the lock, assume empty
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating different storage implementations
pub struct StorageFactory;

impl StorageFactory {
    /// Create a new in-memory storage instance
    pub fn create_memory_storage() -> Arc<InMemoryStorage> {
        Arc::new(InMemoryStorage::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_primitives::{HashOutput, HashAlgorithm};
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestResource {
        name: String,
        value: i32,
    }
    
    impl ContentAddressed for TestResource {
        fn content_hash(&self) -> Result<HashOutput, HashError> {
            // Create a simple content hash for testing
            let data = format!("{}:{}", self.name, self.value);
            let hash = blake3::hash(data.as_bytes());
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(hash.as_bytes());
            Ok(HashOutput::new(hash_bytes, HashAlgorithm::Blake3))
        }
        
        fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
            serde_json::to_vec(self)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            serde_json::from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[test]
    fn test_inmemory_storage_basic_operations() {
        let storage = InMemoryStorage::new();
        
        // Create test resource
        let resource = TestResource {
            name: "test".to_string(),
            value: 42,
        };
        
        // Test storing
        let id = storage.store(&resource).unwrap();
        assert!(storage.contains(&id));
        assert_eq!(storage.len(), 1);
        
        // Test retrieval
        let retrieved: TestResource = storage.get(&id).unwrap();
        assert_eq!(retrieved, resource);
        
        // Test removal
        storage.remove(&id).unwrap();
        assert!(!storage.contains(&id));
        assert_eq!(storage.len(), 0);
        
        // Test error handling
        let result = storage.get::<TestResource>(&id);
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }
    
    #[test]
    fn test_inmemory_storage_multiple_objects() {
        let storage = InMemoryStorage::new();
        
        // Store multiple objects
        let resource1 = TestResource { name: "first".to_string(), value: 1 };
        let resource2 = TestResource { name: "second".to_string(), value: 2 };
        
        let id1 = storage.store(&resource1).unwrap();
        let id2 = storage.store(&resource2).unwrap();
        
        assert_ne!(id1, id2);
        assert_eq!(storage.len(), 2);
        
        // Retrieve multiple objects
        let r1: TestResource = storage.get(&id1).unwrap();
        let r2: TestResource = storage.get(&id2).unwrap();
        
        assert_eq!(r1, resource1);
        assert_eq!(r2, resource2);
        
        // Clear storage
        storage.clear();
        assert_eq!(storage.len(), 0);
        assert!(!storage.contains(&id1));
        assert!(!storage.contains(&id2));
    }
} 