// RocksDB implementation
//
// This module provides a RocksDB implementation of the Database trait, which
// provides persistent storage using the RocksDB key-value store.

use std::path::Path;
use std::sync::Arc;

use causality_storage::{
    Database, DbError, DbConfig, BatchOp, DbIterator,
};

use rocksdb::{DB, Options, IteratorMode, Direction, ReadOptions, WriteBatch};

/// RocksDB implementation of the Database trait
#[derive(Debug)]
pub struct RocksDb {
    /// The internal RocksDB database instance
    db: Arc<DB>,
    /// The path to the database
    path: String,
}

impl Database for RocksDb {
    fn open(config: DbConfig) -> Result<Self, DbError> {
        let mut options = Options::default();
        options.create_if_missing(config.create_if_missing);
        
        let db = if config.read_only {
            DB::open_for_read_only(&options, &config.path, false)
                .map_err(|e| DbError::OpenError(format!("Failed to open RocksDB: {}", e)))?
        } else {
            DB::open(&options, &config.path)
                .map_err(|e| DbError::OpenError(format!("Failed to open RocksDB: {}", e)))?
        };
        
        Ok(Self {
            db: Arc::new(db),
            path: config.path,
        })
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        self.db.get(key)
            .map_err(|e| DbError::ReadError(format!("Failed to read from RocksDB: {}", e)))
    }
    
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DbError> {
        self.db.put(key, value)
            .map_err(|e| DbError::WriteError(format!("Failed to write to RocksDB: {}", e)))
    }
    
    fn delete(&self, key: &[u8]) -> Result<(), DbError> {
        self.db.delete(key)
            .map_err(|e| DbError::DeleteError(format!("Failed to delete from RocksDB: {}", e)))
    }
    
    fn contains(&self, key: &[u8]) -> Result<bool, DbError> {
        self.db.get(key)
            .map(|v| v.is_some())
            .map_err(|e| DbError::ReadError(format!("Failed to read from RocksDB: {}", e)))
    }
    
    fn write_batch(&self, batch: &[BatchOp]) -> Result<(), DbError> {
        let mut wb = WriteBatch::default();
        
        for op in batch {
            match op {
                BatchOp::Put(key, value) => {
                    wb.put(key, value);
                },
                BatchOp::Delete(key) => {
                    wb.delete(key);
                },
            }
        }
        
        self.db.write(wb)
            .map_err(|e| DbError::WriteError(format!("Failed to write batch to RocksDB: {}", e)))
    }
    
    fn iterator(&self) -> Result<Box<dyn DbIterator>, DbError> {
        let iterator = self.db.iterator(IteratorMode::Start);
        Ok(Box::new(RocksDbIterator { iterator }))
    }
    
    fn prefix_iterator(&self, prefix: &[u8]) -> Result<Box<dyn DbIterator>, DbError> {
        let mut read_options = ReadOptions::default();
        read_options.set_iterate_range(rocksdb::PrefixRange(prefix.to_vec()));
        
        let iterator = self.db.iterator_opt(IteratorMode::Start, read_options);
        Ok(Box::new(RocksDbIterator { iterator }))
    }
    
    fn flush(&self) -> Result<(), DbError> {
        self.db.flush()
            .map_err(|e| DbError::WriteError(format!("Failed to flush RocksDB: {}", e)))
    }
    
    fn close(&self) -> Result<(), DbError> {
        // RocksDB will be closed when the Arc<DB> is dropped
        // We don't need to do anything special here
        Ok(())
    }
}

/// RocksDB iterator implementation
pub struct RocksDbIterator {
    /// The internal RocksDB iterator
    iterator: rocksdb::DBIterator<'static>,
}

impl DbIterator for RocksDbIterator {
    fn next(&mut self) -> Option<Result<(Vec<u8>, Vec<u8>), DbError>> {
        self.iterator.next().map(|result| {
            result.map_err(|e| {
                DbError::ReadError(format!("Failed to read next item from RocksDB: {}", e))
            }).map(|(k, v)| (k.to_vec(), v.to_vec()))
        })
    }
    
    fn seek(&mut self, key: &[u8]) -> Result<(), DbError> {
        self.iterator.set_mode(IteratorMode::From(key, Direction::Forward));
        Ok(())
    }
    
    fn seek_to_first(&mut self) -> Result<(), DbError> {
        self.iterator.set_mode(IteratorMode::Start);
        Ok(())
    }
    
    fn seek_to_last(&mut self) -> Result<(), DbError> {
        self.iterator.set_mode(IteratorMode::End);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_rocksdb_basic() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        // Open the database
        let db = RocksDb::open(DbConfig::new(path)).unwrap();
        
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
        
        // Close the database
        db.close().unwrap();
        
        // Clean up
        temp_dir.close().unwrap();
    }
    
    #[test]
    fn test_rocksdb_batch() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        // Open the database
        let db = RocksDb::open(DbConfig::new(path)).unwrap();
        
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
        
        // Close the database
        db.close().unwrap();
        
        // Clean up
        temp_dir.close().unwrap();
    }
    
    #[test]
    fn test_rocksdb_iterator() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        // Open the database
        let db = RocksDb::open(DbConfig::new(path)).unwrap();
        
        // Add some data
        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        db.put(b"key3", b"value3").unwrap();
        
        // Get an iterator
        let mut iter = db.iterator().unwrap();
        
        // Collect all items
        let mut items = Vec::new();
        while let Some(item) = iter.next() {
            items.push(item.unwrap());
        }
        
        // Verify all items are found
        assert_eq!(items.len(), 3);
        assert!(items.contains(&(b"key1".to_vec(), b"value1".to_vec())));
        assert!(items.contains(&(b"key2".to_vec(), b"value2".to_vec())));
        assert!(items.contains(&(b"key3".to_vec(), b"value3".to_vec())));
        
        // Close the database
        db.close().unwrap();
        
        // Clean up
        temp_dir.close().unwrap();
    }
    
    #[test]
    fn test_rocksdb_prefix_iterator() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        // Open the database
        let db = RocksDb::open(DbConfig::new(path)).unwrap();
        
        // Add some data
        db.put(b"aaa1", b"value1").unwrap();
        db.put(b"aaa2", b"value2").unwrap();
        db.put(b"bbb1", b"value3").unwrap();
        db.put(b"bbb2", b"value4").unwrap();
        
        // Get a prefix iterator
        let mut iter = db.prefix_iterator(b"aaa").unwrap();
        
        // Collect all items
        let mut items = Vec::new();
        while let Some(item) = iter.next() {
            items.push(item.unwrap());
        }
        
        // Verify only the 'aaa' prefix items are found
        assert_eq!(items.len(), 2);
        assert!(items.contains(&(b"aaa1".to_vec(), b"value1".to_vec())));
        assert!(items.contains(&(b"aaa2".to_vec(), b"value2".to_vec())));
        assert!(!items.contains(&(b"bbb1".to_vec(), b"value3".to_vec())));
        assert!(!items.contains(&(b"bbb2".to_vec(), b"value4".to_vec())));
        
        // Close the database
        db.close().unwrap();
        
        // Clean up
        temp_dir.close().unwrap();
    }
} 