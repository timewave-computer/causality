// Content-addressed storage utilities

// Content-addressed storage module
//
// This module defines interfaces and implementations for storing
// and retrieving content-addressed objects.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Use our local types instead of causality-types
use crate::hash::{ContentId, HashOutput, HashError};
use crate::traits::{ContentAddressed, ContentAddressedStorage, ContentAddressedStorageExt, StorageError};

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
    fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError> {
        // Create a content ID from bytes
        let hash_factory = crate::hash::HashFactory::default();
        let hasher = hash_factory.create_hasher()
            .map_err(|e| StorageError::HashError(e))?;
        let hash = hasher.hash(bytes);
        let content_id = ContentId::from(hash);
        
        // Store the bytes with the content ID as the key
        let mut objects = self.objects.write().unwrap();
        
        // Skip if already exists
        if objects.contains_key(&content_id) {
            return Ok(content_id);
        }
        
        // Store the object
        objects.insert(content_id.clone(), bytes.to_vec());
        
        Ok(content_id)
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

// Add implementation of the extension trait
impl ContentAddressedStorageExt for InMemoryStorage {}

/// Caching storage that wraps another storage
pub struct CachingStorage {
    /// The backing storage
    backing: Arc<dyn ContentAddressedStorage>,
    /// Cache for content data
    cache: RwLock<HashMap<ContentId, Vec<u8>>>,
    /// Maximum cache size
    capacity: usize,
}

impl CachingStorage {
    /// Create a new caching storage with the specified backing store and capacity
    pub fn new(backing: Arc<dyn ContentAddressedStorage>, capacity: usize) -> Self {
        Self {
            backing,
            cache: RwLock::new(HashMap::with_capacity(capacity)),
            capacity,
        }
    }
    
    /// Clear the cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}

impl ContentAddressedStorage for CachingStorage {
    fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError> {
        // Store in backing storage
        let content_id = self.backing.store_bytes(bytes)?;
        
        // Cache the bytes
        let mut cache = self.cache.write().unwrap();
        
        // If cache is at capacity, remove an item
        if cache.len() >= self.capacity && !cache.contains_key(&content_id) {
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);
            }
        }
        
        cache.insert(content_id.clone(), bytes.to_vec());
        
        Ok(content_id)
    }
    
    fn contains(&self, id: &ContentId) -> bool {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if cache.contains_key(id) {
                return true;
            }
        }
        
        // Fall back to backing storage
        self.backing.contains(id)
    }
    
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(data) = cache.get(id) {
                return Ok(data.clone());
            }
        }
        
        // Fall back to backing storage
        let data = self.backing.get_bytes(id)?;
        
        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            
            // If cache is at capacity, remove an item
            if cache.len() >= self.capacity && !cache.contains_key(id) {
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }
            
            cache.insert(id.clone(), data.clone());
        }
        
        Ok(data)
    }
    
    fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
        // Remove from backing storage
        self.backing.remove(id)?;
        
        // Remove from cache
        let mut cache = self.cache.write().unwrap();
        cache.remove(id);
        
        Ok(())
    }
    
    fn clear(&self) {
        // Clear backing storage
        self.backing.clear();
        
        // Clear cache
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
    
    fn len(&self) -> usize {
        // Use backing storage for accurate count
        self.backing.len()
    }
}

// Add implementation of the extension trait
impl ContentAddressedStorageExt for CachingStorage {}

/// Factory for creating content-addressed storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create an in-memory storage
    pub fn create_memory_storage() -> Arc<InMemoryStorage> {
        Arc::new(InMemoryStorage::new())
    }
    
    /// Create a caching memory storage with the specified cache size
    pub fn create_caching_memory_storage(cache_size: usize) -> Arc<CachingStorage> {
        let backing = Self::create_memory_storage();
        Arc::new(CachingStorage::new(backing, cache_size))
    }
}

/// Simple wrapper for content-addressed storage with a default implementation
pub struct ContentStore {
    storage: Arc<dyn ContentAddressedStorage>,
}

impl ContentStore {
    /// Create a new content store with in-memory storage
    pub fn new() -> Self {
        Self {
            storage: StorageFactory::create_memory_storage(),
        }
    }
    
    /// Create a new content store with the provided storage
    pub fn with_storage(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self { storage }
    }
    
    /// Get the underlying storage
    pub fn storage(&self) -> &Arc<dyn ContentAddressedStorage> {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();
        
        // Store some bytes
        let id = storage.store_bytes(b"test data").unwrap();
        
        // Check that the object exists
        assert!(storage.contains(&id));
        
        // Retrieve the object
        let retrieved = storage.get_bytes(&id).unwrap();
        assert_eq!(b"test data".to_vec(), retrieved);
        
        // Remove the object
        storage.remove(&id).unwrap();
        
        // Check that the object no longer exists
        assert!(!storage.contains(&id));
        
        // Try to retrieve a non-existent object
        let result = storage.get_bytes(&id);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_caching_storage() {
        let backing = Arc::new(InMemoryStorage::new());
        let storage = CachingStorage::new(backing, 10);
        
        // Store some bytes
        let id = storage.store_bytes(b"cached data").unwrap();
        
        // Check that the object exists
        assert!(storage.contains(&id));
        
        // Retrieve the object (should come from cache now)
        let retrieved = storage.get_bytes(&id).unwrap();
        assert_eq!(b"cached data".to_vec(), retrieved);
        
        // Clear cache and check retrieval from backing store
        storage.clear_cache();
        let retrieved = storage.get_bytes(&id).unwrap();
        assert_eq!(b"cached data".to_vec(), retrieved);
    }
} 