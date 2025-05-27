// RocksDB Mock Implementation
//
// This module provides a minimal mock of the RocksDB implementation that simply
// forwards to the in-memory implementation. The actual RocksDB implementation will
// be provided by the Almanac project and integrated later.

use std::sync::Arc;
use crate::types::{Database, DbError, DbConfig, BatchOp, DbIterator};
use crate::memory::MemoryDb;

/// Mock RocksDB implementation that forwards to MemoryDb
#[derive(Debug)]
pub struct RocksDb {
    /// The underlying memory database
    memory_db: MemoryDb,
    /// Path to where the database would be stored (for logging/debugging only)
    path: String,
}

impl Database for RocksDb {
    fn open(config: DbConfig) -> Result<Self, DbError> {
        tracing::info!("Opening mock RocksDB at path: {}", config.path);
        let memory_db = MemoryDb::open(config.clone())?;
        
        Ok(Self {
            memory_db,
            path: config.path,
        })
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        self.memory_db.get(key)
    }
    
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DbError> {
        self.memory_db.put(key, value)
    }
    
    fn delete(&self, key: &[u8]) -> Result<(), DbError> {
        self.memory_db.delete(key)
    }
    
    fn contains(&self, key: &[u8]) -> Result<bool, DbError> {
        self.memory_db.contains(key)
    }
    
    fn write_batch(&self, batch: &[BatchOp]) -> Result<(), DbError> {
        self.memory_db.write_batch(batch)
    }
    
    fn iterator(&self) -> Result<Box<dyn DbIterator>, DbError> {
        self.memory_db.iterator()
    }
    
    fn prefix_iterator(&self, prefix: &[u8]) -> Result<Box<dyn DbIterator>, DbError> {
        self.memory_db.prefix_iterator(prefix)
    }
    
    fn flush(&self) -> Result<(), DbError> {
        self.memory_db.flush()
    }
    
    fn close(&self) -> Result<(), DbError> {
        self.memory_db.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_rocksdb() {
        // Create a fake path
        let db = RocksDb::open(DbConfig::new("/tmp/mock_rocksdb")).unwrap();
        
        // Test basic operations to ensure they're forwarded to memory DB
        db.put(b"key1", b"value1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        
        // The operations should work just like the memory DB
        db.delete(b"key1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), None);
    }
} 