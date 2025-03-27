// Causality Database Implementations
//
// This crate provides database implementations for the causality project.
// Two implementations are provided:
// 1. Memory: An in-memory database for testing and development
// 2. RocksDB: A persistent disk-based database for production use

use causality_storage::{
    Database, DbError, DbConfig, BatchOp, DbIterator,
};

#[cfg(feature = "rocks")]
use rocksdb::{DB, Options, IteratorMode, Direction, ReadOptions, WriteBatch};

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::path::Path;

/// Module containing the in-memory database implementation
#[cfg(feature = "memory")]
pub mod memory;

/// Module containing the RocksDB database implementation
#[cfg(feature = "rocks")]
pub mod rocks;

// Re-export the implementations
#[cfg(feature = "memory")]
pub use memory::MemoryDb;

#[cfg(feature = "rocks")]
pub use rocks::RocksDb;

/// Database factory for creating database instances
pub struct DbFactory;

impl DbFactory {
    /// Create an in-memory database (for testing)
    #[cfg(feature = "memory")]
    pub fn create_memory_db() -> Result<Box<dyn Database>, DbError> {
        Ok(Box::new(memory::MemoryDb::open(DbConfig::new("in_memory"))?))
    }
    
    /// Create a RocksDB database
    #[cfg(feature = "rocks")]
    pub fn create_rocksdb(path: &str) -> Result<Box<dyn Database>, DbError> {
        Ok(Box::new(rocks::RocksDb::open(DbConfig::new(path))?))
    }
    
    /// Create a default database (depends on enabled features)
    pub fn create_default_db(path: &str) -> Result<Box<dyn Database>, DbError> {
        #[cfg(feature = "rocks")]
        {
            Self::create_rocksdb(path)
        }
        
        #[cfg(all(feature = "memory", not(feature = "rocks")))]
        {
            Self::create_memory_db()
        }
        
        #[cfg(not(any(feature = "memory", feature = "rocks")))]
        {
            Err(DbError::OpenError("No database implementation available".to_string()))
        }
    }
} 