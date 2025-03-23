// Memory database implementation
//
// This module provides an in-memory database implementation for testing purposes.
// It implements the Database trait defined in the parent module, and stores all
// data in memory using a HashMap. This is useful for testing and for small data
// sets that don't need persistence.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{Database, DbError, DbConfig, BatchOp, DbIterator};

/// In-memory database implementation
pub struct MemoryDb {
    /// The in-memory key-value store
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    /// Whether the database is closed
    closed: Arc<RwLock<bool>>,
}

impl MemoryDb {
    /// Check if the database is closed
    fn check_closed(&self) -> Result<(), DbError> {
        let closed = self.closed.read().map_err(|e| {
            DbError::GenericError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        if *closed {
            return Err(DbError::GenericError("Database is closed".to_string()));
        }
        
        Ok(())
    }
}

impl Database for MemoryDb {
    fn open(_config: DbConfig) -> Result<Self, DbError> {
        Ok(Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            closed: Arc::new(RwLock::new(false)),
        })
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        self.check_closed()?;
        
        let data = self.data.read().map_err(|e| {
            DbError::ReadError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        Ok(data.get(key).cloned())
    }
    
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DbError> {
        self.check_closed()?;
        
        let mut data = self.data.write().map_err(|e| {
            DbError::WriteError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
    
    fn delete(&self, key: &[u8]) -> Result<(), DbError> {
        self.check_closed()?;
        
        let mut data = self.data.write().map_err(|e| {
            DbError::DeleteError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        data.remove(key);
        Ok(())
    }
    
    fn contains(&self, key: &[u8]) -> Result<bool, DbError> {
        self.check_closed()?;
        
        let data = self.data.read().map_err(|e| {
            DbError::ReadError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        Ok(data.contains_key(key))
    }
    
    fn write_batch(&self, batch: &[BatchOp]) -> Result<(), DbError> {
        self.check_closed()?;
        
        let mut data = self.data.write().map_err(|e| {
            DbError::WriteError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        for op in batch {
            match op {
                BatchOp::Put(key, value) => {
                    data.insert(key.clone(), value.clone());
                },
                BatchOp::Delete(key) => {
                    data.remove(key);
                },
            }
        }
        
        Ok(())
    }
    
    fn iterator(&self) -> Result<Box<dyn DbIterator>, DbError> {
        self.check_closed()?;
        
        let data = self.data.read().map_err(|e| {
            DbError::ReadError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let items: Vec<(Vec<u8>, Vec<u8>)> = data.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            
        Ok(Box::new(MemoryIterator::new(items)))
    }
    
    fn prefix_iterator(&self, prefix: &[u8]) -> Result<Box<dyn DbIterator>, DbError> {
        self.check_closed()?;
        
        let data = self.data.read().map_err(|e| {
            DbError::ReadError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let items: Vec<(Vec<u8>, Vec<u8>)> = data.iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            
        Ok(Box::new(MemoryIterator::new(items)))
    }
    
    fn flush(&self) -> Result<(), DbError> {
        // Nothing to do for in-memory database
        Ok(())
    }
    
    fn close(&self) -> Result<(), DbError> {
        let mut closed = self.closed.write().map_err(|e| {
            DbError::GenericError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        *closed = true;
        Ok(())
    }
}

/// In-memory database iterator
pub struct MemoryIterator {
    /// All key-value pairs
    items: Vec<(Vec<u8>, Vec<u8>)>,
    /// Current position in the iterator
    position: usize,
}

impl MemoryIterator {
    /// Create a new memory iterator with the given items
    fn new(items: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self {
            items,
            position: 0,
        }
    }
}

impl DbIterator for MemoryIterator {
    fn next(&mut self) -> Option<Result<(Vec<u8>, Vec<u8>), DbError>> {
        if self.position >= self.items.len() {
            return None;
        }
        
        let item = self.items[self.position].clone();
        self.position += 1;
        Some(Ok(item))
    }
    
    fn seek(&mut self, key: &[u8]) -> Result<(), DbError> {
        self.position = self.items.iter()
            .position(|(k, _)| k >= &key.to_vec())
            .unwrap_or(self.items.len());
        Ok(())
    }
    
    fn seek_to_first(&mut self) -> Result<(), DbError> {
        self.position = 0;
        Ok(())
    }
    
    fn seek_to_last(&mut self) -> Result<(), DbError> {
        self.position = self.items.len().saturating_sub(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_db_basic() {
        let db = MemoryDb::open(DbConfig::new("test")).unwrap();
        
        // Initially empty
        assert!(!db.contains(b"key1").unwrap());
        
        // Put and get
        db.put(b"key1", b"value1").unwrap();
        assert!(db.contains(b"key1").unwrap());
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        
        // Update
        db.put(b"key1", b"newvalue").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(b"newvalue".to_vec()));
        
        // Delete
        db.delete(b"key1").unwrap();
        assert!(!db.contains(b"key1").unwrap());
        assert_eq!(db.get(b"key1").unwrap(), None);
    }
    
    #[test]
    fn test_memory_db_batch() {
        let db = MemoryDb::open(DbConfig::new("test")).unwrap();
        
        // Create a batch of operations
        let batch = vec![
            BatchOp::Put(b"key1".to_vec(), b"value1".to_vec()),
            BatchOp::Put(b"key2".to_vec(), b"value2".to_vec()),
            BatchOp::Put(b"key3".to_vec(), b"value3".to_vec()),
            BatchOp::Delete(b"key2".to_vec()),
        ];
        
        // Execute the batch
        db.write_batch(&batch).unwrap();
        
        // Verify results
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(b"key2").unwrap(), None);
        assert_eq!(db.get(b"key3").unwrap(), Some(b"value3".to_vec()));
    }
    
    #[test]
    fn test_memory_db_iterator() {
        let db = MemoryDb::open(DbConfig::new("test")).unwrap();
        
        // Add some data
        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        db.put(b"key3", b"value3").unwrap();
        
        // Create an iterator
        let mut iter = db.iterator().unwrap();
        
        // Collect all items
        let mut items = Vec::new();
        while let Some(result) = iter.next() {
            items.push(result.unwrap());
        }
        
        // Should have 3 items (order not guaranteed)
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|(k, v)| k == b"key1" && v == b"value1"));
        assert!(items.iter().any(|(k, v)| k == b"key2" && v == b"value2"));
        assert!(items.iter().any(|(k, v)| k == b"key3" && v == b"value3"));
    }
    
    #[test]
    fn test_memory_db_prefix_iterator() {
        let db = MemoryDb::open(DbConfig::new("test")).unwrap();
        
        // Add some data with different prefixes
        db.put(b"prefix1_key1", b"value1").unwrap();
        db.put(b"prefix1_key2", b"value2").unwrap();
        db.put(b"prefix2_key1", b"value3").unwrap();
        
        // Create a prefix iterator
        let mut iter = db.prefix_iterator(b"prefix1_").unwrap();
        
        // Collect all items
        let mut items = Vec::new();
        while let Some(result) = iter.next() {
            items.push(result.unwrap());
        }
        
        // Should have 2 items with the prefix
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|(k, v)| k == b"prefix1_key1" && v == b"value1"));
        assert!(items.iter().any(|(k, v)| k == b"prefix1_key2" && v == b"value2"));
        assert!(!items.iter().any(|(k, _)| k == b"prefix2_key1"));
    }
    
    #[test]
    fn test_memory_db_close() {
        let db = MemoryDb::open(DbConfig::new("test")).unwrap();
        
        // Add some data
        db.put(b"key1", b"value1").unwrap();
        
        // Close the database
        db.close().unwrap();
        
        // Operations should fail
        assert!(db.put(b"key2", b"value2").is_err());
        assert!(db.get(b"key1").is_err());
        assert!(db.delete(b"key1").is_err());
        assert!(db.contains(b"key1").is_err());
        assert!(db.iterator().is_err());
    }
} 