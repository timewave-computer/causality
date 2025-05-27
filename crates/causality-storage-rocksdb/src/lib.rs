//! Mock RocksDB Storage Implementation
//!
//! This module provides a mock implementation of a RocksDB-backed storage system
//! for Causality. Rather than using an actual RocksDB database, it uses the memory
//! database implementation from causality-db.
//!
//! The real RocksDB implementation will be integrated from the Almanac project later.

use async_trait::async_trait;
use causality_db::{Database, DbConfig, DbFactory, BatchOp};
use causality_error::{BoxError, StorageError};
use std::sync::Arc;
use std::path::Path;

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

/// Mock RocksDB client that internally uses memory storage
pub struct RocksDbClient {
    /// Path where the database would be stored (for logging/debugging only)
    path: String,
    /// In-memory database used for storage
    db: Arc<Box<dyn Database>>,
}

impl RocksDbClient {
    /// Create a new RocksDB client with the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, BoxError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        tracing::info!("Mock RocksDB client opening database at: {}", path_str);
        
        // Use memory database under the hood
        let db = DbFactory::create_memory_db()
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to create memory DB: {}", e));
                Box::new(storage_err) as BoxError
            })?;
        
        Ok(Self {
            path: path_str,
            db: Arc::new(db),
        })
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
    
    /// Create a transaction
    pub fn transaction(&self) -> RocksDbTransaction {
        RocksDbTransaction {
            db: Arc::clone(&self.db),
            batch: Vec::new(),
        }
    }
}

/// Mock RocksDB transaction
pub struct RocksDbTransaction {
    /// Reference to the database
    db: Arc<Box<dyn Database>>,
    /// Batch operations to be applied
    batch: Vec<BatchOp>,
}

impl RocksDbTransaction {
    /// Add a put operation to the transaction
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> &mut Self {
        self.batch.push(BatchOp::Put(key.to_vec(), value.to_vec()));
        self
    }
    
    /// Add a delete operation to the transaction
    pub fn delete(&mut self, key: &[u8]) -> &mut Self {
        self.batch.push(BatchOp::Delete(key.to_vec()));
        self
    }
    
    /// Commit the transaction
    pub fn commit(self) -> Result<(), BoxError> {
        self.db.write_batch(&self.batch)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to commit transaction: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rocksdb_client() {
        // Create a mock RocksDB client
        let client = RocksDbClient::open("/tmp/mock_rocksdb").unwrap();
        
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
    
    #[test]
    fn test_rocksdb_transaction() {
        // Create a mock RocksDB client
        let client = RocksDbClient::open("/tmp/mock_rocksdb").unwrap();
        
        // Create test resource IDs
        let resource_id1 = ResourceId::from_string("test:resource:1").unwrap();
        let resource_id2 = ResourceId::from_string("test:resource:2").unwrap();
        
        // Create a transaction
        let mut tx = client.transaction();
        
        // Add operations to the transaction
        tx.put(&resource_id1.to_string().into_bytes(), b"data1")
          .put(&resource_id2.to_string().into_bytes(), b"data2");
          
        // Commit the transaction
        tx.commit().unwrap();
        
        // Verify both resources were stored
        assert_eq!(
            client.get_resource(&resource_id1).unwrap(),
            Some(b"data1".to_vec())
        );
        assert_eq!(
            client.get_resource(&resource_id2).unwrap(),
            Some(b"data2".to_vec())
        );
    }
} 