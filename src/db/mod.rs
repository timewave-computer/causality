// Database interface module
//
// This module provides a clean trait interface for database operations, which
// will be implemented for RocksDB and can be extended for other databases
// in the future. This abstraction allows the storage layer to be swapped out
// without affecting the rest of the system.

use std::fmt::{Debug, Display};
use std::error::Error;
use std::sync::Arc;

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

/// Memory database implementation (for testing)
pub mod in_memory;

/// RocksDB implementation (when feature is enabled)
#[cfg(feature = "rocksdb")]
pub mod rocksdb;

/// Database factory for creating database instances
pub struct DbFactory;

impl DbFactory {
    /// Create an in-memory database (for testing)
    pub fn create_memory_db() -> Result<Box<dyn Database>, DbError> {
        Ok(Box::new(in_memory::MemoryDb::open(DbConfig::new("in_memory"))?))
    }
    
    /// Create a RocksDB database (when feature is enabled)
    #[cfg(feature = "rocksdb")]
    pub fn create_rocksdb(path: &str) -> Result<Box<dyn Database>, DbError> {
        Ok(Box::new(rocksdb::RocksDb::open(DbConfig::new(path))?))
    }
} 