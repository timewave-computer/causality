// Database factory implementation
//
// This module provides a factory for creating database instances.
// It abstracts over the specific database implementation (memory, RocksDB, etc.)

use crate::types::{Database, DbError, DbConfig};
use crate::memory::MemoryDb;

#[cfg(feature = "rocks")]
use crate::rocks::RocksDb;

/// Factory for creating database instances
pub struct DbFactory;

impl DbFactory {
    /// Create an in-memory database
    pub fn create_memory_db() -> Result<Box<dyn Database>, DbError> {
        let db = MemoryDb::open(DbConfig::new("memory"))?;
        Ok(Box::new(db))
    }
    
    /// Create a RocksDB database (when feature is enabled)
    /// 
    /// Note: When the 'rocks' feature is disabled, this returns a memory database
    /// as a mock implementation.
    pub fn create_rocksdb(path: &str) -> Result<Box<dyn Database>, DbError> {
        #[cfg(feature = "rocks")]
        {
            let db = RocksDb::open(DbConfig::new(path))?;
            Ok(Box::new(db))
        }
        
        #[cfg(not(feature = "rocks"))]
        {
            tracing::warn!("RocksDB implementation not available, using memory database instead for path: {}", path);
            Self::create_memory_db()
        }
    }
    
    /// Create a default database based on available implementations
    pub fn create_default_db(path: &str) -> Result<Box<dyn Database>, DbError> {
        #[cfg(feature = "rocks")]
        {
            // Prefer RocksDB when available
            Self::create_rocksdb(path)
        }
        
        #[cfg(not(feature = "rocks"))]
        {
            // Fall back to memory when RocksDB is not available
            tracing::info!("Using in-memory database for path: {}", path);
            Self::create_memory_db()
        }
    }
} 