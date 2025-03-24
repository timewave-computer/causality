// Content-addressed storage module
//
// This module defines interfaces and implementations for storing
// and retrieving content-addressed objects.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

use crate::crypto::{
    ContentAddressed, ContentId, HashOutput, HashError
};

/// Error type for content-addressed storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    /// Object not found in storage
    #[error("Object not found: {0}")]
    NotFound(String),
    
    /// Duplicate object in storage
    #[error("Duplicate object: {0}")]
    Duplicate(String),
    
    /// Hash mismatch during verification
    #[error("Hash mismatch during verification: {0}")]
    HashMismatch(String),
    
    /// Storage I/O error
    #[error("Storage I/O error: {0}")]
    IoError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Hash computation error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
}

/// Interface for content-addressed storage
pub trait ContentAddressedStorage: Send + Sync {
    /// Store an object in the content-addressed storage
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentId, StorageError>;
    
    /// Check if an object exists in storage
    fn contains(&self, id: &ContentId) -> bool;
    
    /// Retrieve binary data for an object
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError>;
    
    /// Retrieve an object from storage by its content ID
    fn get<T: ContentAddressed>(&self, id: &ContentId) -> Result<T, StorageError> {
        let bytes = self.get_bytes(id)?;
        T::from_bytes(&bytes).map_err(|e| StorageError::from(e))
    }
    
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

/// In-memory implementation of content-addressed storage
pub struct InMemoryStorage {
    objects: RwLock<HashMap<ContentId, Vec<u8>>>,
}

impl InMemoryStorage {
    /// Create a new empty in-memory storage
    pub fn new() -> Self {
        Self {
            objects: RwLock::new(HashMap::new()),
        }
    }
}

impl ContentAddressedStorage for InMemoryStorage {
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentId, StorageError> {
        let id = object.content_id();
        let data = object.to_bytes();
        
        // Verify the object's content hash
        if !object.verify() {
            return Err(StorageError::HashMismatch(
                format!("Object verification failed: {}", id)
            ));
        }
        
        let mut objects = self.objects.write().unwrap();
        
        // Check for duplicates
        if objects.contains_key(&id) {
            return Err(StorageError::Duplicate(
                format!("Object already exists: {}", id)
            ));
        }
        
        objects.insert(id.clone(), data);
        Ok(id)
    }
    
    fn contains(&self, id: &ContentId) -> bool {
        let objects = self.objects.read().unwrap();
        objects.contains_key(id)
    }
    
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
        let objects = self.objects.read().unwrap();
        
        objects.get(id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(
                format!("Object not found: {}", id)
            ))
    }
    
    fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
        let mut objects = self.objects.write().unwrap();
        
        if objects.remove(id).is_none() {
            return Err(StorageError::NotFound(
                format!("Object not found: {}", id)
            ));
        }
        
        Ok(())
    }
    
    fn clear(&self) {
        let mut objects = self.objects.write().unwrap();
        objects.clear();
    }
    
    fn len(&self) -> usize {
        let objects = self.objects.read().unwrap();
        objects.len()
    }
}

/// Factory for creating content-addressed storage instances
pub struct StorageFactory {
    storage_type: StorageType,
}

/// Type of storage to create
#[derive(Debug, Clone, Copy)]
pub enum StorageType {
    /// In-memory storage
    InMemory,
}

impl StorageFactory {
    /// Create a new storage factory
    pub fn new(storage_type: StorageType) -> Self {
        Self { storage_type }
    }
    
    /// Create a storage instance
    pub fn create_storage(&self) -> Arc<dyn ContentAddressedStorage> {
        match self.storage_type {
            StorageType::InMemory => Arc::new(InMemoryStorage::new()),
        }
    }
}

impl Default for StorageFactory {
    fn default() -> Self {
        Self::new(StorageType::InMemory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::{BorshSerialize, BorshDeserialize};
    
    // Test object that implements ContentAddressed
    #[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
    struct TestObject {
        id: String,
        data: Vec<u8>,
    }
    
    impl TestObject {
        fn new(id: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
            Self {
                id: id.into(),
                data: data.into(),
            }
        }
    }
    
    impl ContentAddressed for TestObject {
        fn content_hash(&self) -> HashOutput {
            // Get a hash factory
            let hasher = crate::crypto::HashFactory::default()
                .create_hasher()
                .unwrap();
            
            // Serialize and hash
            let data = self.try_to_vec().unwrap();
            hasher.hash(&data)
        }
        
        fn verify(&self) -> bool {
            true  // Simplified for tests
        }
        
        fn to_bytes(&self) -> Vec<u8> {
            self.try_to_vec().unwrap()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            BorshDeserialize::try_from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[test]
    fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();
        
        // Create a test object
        let obj = TestObject::new("test-1", b"test data");
        
        // Store the object
        let id = storage.store(&obj).unwrap();
        
        // Check that the object exists
        assert!(storage.contains(&id));
        
        // Retrieve the object
        let retrieved: TestObject = storage.get(&id).unwrap();
        assert_eq!(obj, retrieved);
        
        // Remove the object
        storage.remove(&id).unwrap();
        
        // Check that the object no longer exists
        assert!(!storage.contains(&id));
        
        // Try to retrieve a non-existent object
        let result: Result<TestObject, _> = storage.get(&id);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_storage_factory() {
        let factory = StorageFactory::default();
        let storage = factory.create_storage();
        
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }
} 