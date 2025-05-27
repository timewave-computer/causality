// Causality Storage Layer
//
// This module provides a high-level storage abstraction for the Causality system.
// It builds on top of causality-db and adds component-based storage functionality.

// Re-export the database types from causality-db
pub use causality_db::{Database, DbConfig, DbError, BatchOp, DbIterator, DbFactory};

use causality_error::{BoxError, StorageError};

// Re-export component modules
mod component;
mod schema;
mod snapshot;
mod provider;

pub use component::{Component, ComponentStorage};
pub use schema::{Schema, SchemaManager};
pub use snapshot::{Snapshot, SnapshotManager};
pub use provider::{StorageProvider, StorageProviderFactory};

/// Storage error conversion from causality_db error to causality_error's StorageError
impl From<DbError> for StorageError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => StorageError::KeyNotFound("Key not found".to_string()),
            DbError::OpenError(msg) => StorageError::DatabaseError(format!("Open Error: {}", msg)),
            DbError::ReadError(msg) => StorageError::DatabaseError(format!("Read Error: {}", msg)),
            DbError::WriteError(msg) => StorageError::DatabaseError(format!("Write Error: {}", msg)),
            DbError::DeleteError(msg) => StorageError::DatabaseError(format!("Delete Error: {}", msg)),
            DbError::GenericError(msg) => StorageError::DatabaseError(msg),
            DbError::Other(msg) => StorageError::DatabaseError(format!("Other Error: {}", msg)),
        }
    }
}

/// Storage error conversion from causality_db error to BoxError
impl From<DbError> for BoxError {
    fn from(err: DbError) -> Self {
        Box::new(StorageError::from(err))
    }
}

// Define convenience factory functions
pub mod factory {
    use super::*;

    /// Create an in-memory database (for testing)
    #[cfg(feature = "memory")]
    pub fn create_memory_db() -> Result<Box<dyn Database>, BoxError> {
        DbFactory::create_memory_db()
            .map_err(|e| e.into())
    }
    
    /// Create a RocksDB database (when feature is enabled)
    #[cfg(feature = "rocks")]
    pub fn create_rocksdb(path: &str) -> Result<Box<dyn Database>, BoxError> {
        DbFactory::create_rocksdb(path)
            .map_err(|e| e.into())
    }
    
    /// Create a default database based on available implementations
    pub fn create_default_db(path: &str) -> Result<Box<dyn Database>, BoxError> {
        DbFactory::create_default_db(path)
            .map_err(|e| e.into())
    }
} 