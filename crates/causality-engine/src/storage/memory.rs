// In-memory storage implementation for the Causality Engine
//
// This module provides an in-memory storage implementation for 
// the Causality Engine, useful for testing and development.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::Result;

/// A simple in-memory storage implementation for the engine
#[derive(Debug)]
pub struct InMemoryStorage {
    data: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryStorage {
    /// Create a new empty in-memory storage
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
    
    /// Store data with the given key
    pub fn store(&self, key: String, value: Vec<u8>) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| anyhow::anyhow!("Failed to lock data"))?;
        data.insert(key, value);
        Ok(())
    }
    
    /// Get data for the given key
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.lock().map_err(|_| anyhow::anyhow!("Failed to lock data"))?;
        Ok(data.get(key).cloned())
    }
    
    /// Check if the storage contains the given key
    pub fn contains(&self, key: &str) -> Result<bool> {
        let data = self.data.lock().map_err(|_| anyhow::anyhow!("Failed to lock data"))?;
        Ok(data.contains_key(key))
    }
    
    /// Remove data for the given key
    pub fn remove(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut data = self.data.lock().map_err(|_| anyhow::anyhow!("Failed to lock data"))?;
        Ok(data.remove(key))
    }
    
    /// Clear all data
    pub fn clear(&self) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| anyhow::anyhow!("Failed to lock data"))?;
        data.clear();
        Ok(())
    }
    
    /// Get all keys
    pub fn keys(&self) -> Result<Vec<String>> {
        let data = self.data.lock().map_err(|_| anyhow::anyhow!("Failed to lock data"))?;
        Ok(data.keys().cloned().collect())
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();
        
        // Test store and get
        storage.store("key1".to_string(), vec![1, 2, 3]).unwrap();
        let value = storage.get("key1").unwrap();
        assert_eq!(value, Some(vec![1, 2, 3]));
        
        // Test contains
        assert!(storage.contains("key1").unwrap());
        assert!(!storage.contains("key2").unwrap());
        
        // Test remove
        let removed = storage.remove("key1").unwrap();
        assert_eq!(removed, Some(vec![1, 2, 3]));
        assert!(!storage.contains("key1").unwrap());
        
        // Test clear
        storage.store("key1".to_string(), vec![1, 2, 3]).unwrap();
        storage.store("key2".to_string(), vec![4, 5, 6]).unwrap();
        storage.clear().unwrap();
        assert!(!storage.contains("key1").unwrap());
        assert!(!storage.contains("key2").unwrap());
        
        // Test keys
        storage.store("key1".to_string(), vec![1, 2, 3]).unwrap();
        storage.store("key2".to_string(), vec![4, 5, 6]).unwrap();
        let keys = storage.keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
} 