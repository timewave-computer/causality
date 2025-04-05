// Database abstraction layer
// Original file: src/db/mod.rs

// Database interface module
//
// This module provides a clean trait interface for database operations, which
// will be implemented for RocksDB and can be extended for other databases
// in the future. This abstraction allows the storage layer to be swapped out
// without affecting the rest of the system.

use std::fmt::{Debug, Display};
use std::error::Error;

// Re-export the database interface types
pub use causality_db::*;

/// Database error types
#[derive(Debug)]
pub enum DbError {
    /// Error opening the database
    OpenError(String),
    /// Error reading from the database
    ReadError(String),
    /// Error writing to the database
    WriteError(String),
    /// Error deleting from the database
    DeleteError(String),
    /// Key not found
    NotFound,
    /// Generic database error
    GenericError(String),
    /// Other error
    Other(String),
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenError(msg) => write!(f, "Database open error: {}", msg),
            Self::ReadError(msg) => write!(f, "Database read error: {}", msg),
            Self::WriteError(msg) => write!(f, "Database write error: {}", msg),
            Self::DeleteError(msg) => write!(f, "Database delete error: {}", msg),
            Self::NotFound => write!(f, "Key not found"),
            Self::GenericError(msg) => write!(f, "Database error: {}", msg),
            Self::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl Error for DbError {}

/// Database configuration options
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Path to the database directory
    pub path: String,
    /// Whether to create the database if it doesn't exist
    pub create_if_missing: bool,
    /// Whether to create a read-only database
    pub read_only: bool,
}

impl DbConfig {
    /// Create a new database configuration
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            create_if_missing: true,
            read_only: false,
        }
    }
    
    /// Set whether to create the database if it doesn't exist
    pub fn create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
        self
    }
    
    /// Set whether to create a read-only database
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

/// Key-value batch operation type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOp {
    /// Put a key-value pair
    Put(Vec<u8>, Vec<u8>),
    /// Delete a key
    Delete(Vec<u8>),
}

/// Database key iterator
///
/// This trait defines the interface for iterating over keys in a database.
pub trait DbIterator {
    /// Move to the next key
    fn next(&mut self) -> Option<Result<(Vec<u8>, Vec<u8>), DbError>>;
    
    /// Seek to the specified key
    fn seek(&mut self, key: &[u8]) -> Result<(), DbError>;
    
    /// Seek to the first key
    fn seek_to_first(&mut self) -> Result<(), DbError>;
    
    /// Seek to the last key
    fn seek_to_last(&mut self) -> Result<(), DbError>;
}

/// Database interface
///
/// This trait defines the interface for a key-value database. It provides
/// methods for basic CRUD operations, as well as batch operations and iterators.
pub trait Database: Send + Sync {
    /// Open a database with the given configuration
    fn open(config: DbConfig) -> Result<Self, DbError> where Self: Sized;
    
    /// Get a value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError>;
    
    /// Put a key-value pair
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DbError>;
    
    /// Delete a key
    fn delete(&self, key: &[u8]) -> Result<(), DbError>;
    
    /// Check if a key exists
    fn contains(&self, key: &[u8]) -> Result<bool, DbError>;
    
    /// Execute a batch of operations atomically
    fn write_batch(&self, batch: &[BatchOp]) -> Result<(), DbError>;
    
    /// Create an iterator over the database
    fn iterator(&self) -> Result<Box<dyn DbIterator>, DbError>;
    
    /// Create an iterator with a prefix filter
    fn prefix_iterator(&self, prefix: &[u8]) -> Result<Box<dyn DbIterator>, DbError>;
    
    /// Flush any pending writes to disk
    fn flush(&self) -> Result<(), DbError>;
    
    /// Close the database
    fn close(&self) -> Result<(), DbError>;
}

/// Database factory for creating database instances
pub struct DbFactory;

impl DbFactory {
    /// Create an in-memory database (for testing)
    #[cfg(feature = "memory")]
    pub fn create_memory_db() -> Result<Box<dyn Database>, DbError> {
        causality_db::DbFactory::create_memory_db()
    }
    
    /// Create a RocksDB database (when feature is enabled)
    #[cfg(feature = "rocks")]
    pub fn create_rocksdb(path: &str) -> Result<Box<dyn Database>, DbError> {
        causality_db::DbFactory::create_rocksdb(path)
    }
    
    /// Create a default database based on available implementations
    pub fn create_default_db(path: &str) -> Result<Box<dyn Database>, DbError> {
        // Create the causality_db database
        let db = causality_db::DbFactory::create_default_db(path)?;
        
        // Create an adapter that wraps the causality_db database
        let adapter = DbAdapter { inner: db };
        
        Ok(Box::new(adapter))
    }
}

/// Database adapter to convert between causality_db::Database and storage::Database
struct DbAdapter {
    inner: Box<dyn causality_db::Database>,
}

impl Database for DbAdapter {
    fn open(config: DbConfig) -> Result<Self, DbError> where Self: Sized {
        // Convert the config
        let _db_config = causality_db::DbConfig::new(&config.path)
            .create_if_missing(config.create_if_missing)
            .read_only(config.read_only);
            
        // Open the database using the causality_db interface
        let inner = causality_db::DbFactory::create_default_db(&config.path)?;
        
        Ok(Self { inner })
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        self.inner.get(key).map_err(Into::into)
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DbError> {
        self.inner.put(key, value).map_err(Into::into)
    }

    fn delete(&self, key: &[u8]) -> Result<(), DbError> {
        self.inner.delete(key).map_err(Into::into)
    }

    fn contains(&self, key: &[u8]) -> Result<bool, DbError> {
        self.inner.contains(key).map_err(Into::into)
    }

    fn write_batch(&self, ops: &[BatchOp]) -> Result<(), DbError> {
        // Convert the BatchOp enum
        let converted_ops: Vec<causality_db::BatchOp> = ops.iter().map(|op| {
            match op {
                BatchOp::Put(k, v) => causality_db::BatchOp::Put(k.clone(), v.clone()),
                BatchOp::Delete(k) => causality_db::BatchOp::Delete(k.clone()),
            }
        }).collect();
        
        self.inner.write_batch(&converted_ops).map_err(Into::into)
    }

    fn flush(&self) -> Result<(), DbError> {
        self.inner.flush().map_err(Into::into)
    }

    fn close(&self) -> Result<(), DbError> {
        self.inner.close().map_err(Into::into)
    }

    fn iterator(&self) -> Result<Box<dyn DbIterator>, DbError> {
        let inner_iter = self.inner.iterator()?;
        Ok(Box::new(DbIteratorAdapter { inner: inner_iter }))
    }

    fn prefix_iterator(&self, prefix: &[u8]) -> Result<Box<dyn DbIterator>, DbError> {
        let inner_iter = self.inner.prefix_iterator(prefix)?;
        Ok(Box::new(DbIteratorAdapter { inner: inner_iter }))
    }
}

/// Iterator adapter to convert between causality_db::DbIterator and storage::DbIterator
struct DbIteratorAdapter {
    inner: Box<dyn causality_db::DbIterator>,
}

impl DbIterator for DbIteratorAdapter {
    fn next(&mut self) -> Option<Result<(Vec<u8>, Vec<u8>), DbError>> {
        self.inner.next().map(|result| result.map_err(Into::into))
    }

    fn seek(&mut self, key: &[u8]) -> Result<(), DbError> {
        self.inner.seek(key).map_err(Into::into)
    }

    fn seek_to_first(&mut self) -> Result<(), DbError> {
        self.inner.seek_to_first().map_err(Into::into)
    }

    fn seek_to_last(&mut self) -> Result<(), DbError> {
        self.inner.seek_to_last().map_err(Into::into)
    }
}

// Add a From implementation to convert causality_db::DbError to our DbError
impl From<causality_db::DbError> for DbError {
    fn from(err: causality_db::DbError) -> Self {
        match err {
            causality_db::DbError::NotFound => DbError::NotFound,
            causality_db::DbError::OpenError(msg) => DbError::GenericError(format!("Open Error: {}", msg)),
            causality_db::DbError::ReadError(msg) => DbError::GenericError(format!("Read Error: {}", msg)),
            causality_db::DbError::WriteError(msg) => DbError::GenericError(format!("Write Error: {}", msg)),
            causality_db::DbError::DeleteError(msg) => DbError::GenericError(format!("Delete Error: {}", msg)),
            causality_db::DbError::GenericError(msg) => DbError::GenericError(msg),
        }
    }
} 