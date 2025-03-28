// Log module
//
// This module provides functionality for storing and retrieving log entries.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Error type for log storage operations
#[derive(Error, Debug)]
pub enum LogStorageError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid entry: {0}")]
    InvalidEntry(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Type alias for LogStorage results
pub type Result<T> = std::result::Result<T, LogStorageError>;

/// A log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log identifier
    pub log_id: String,
    /// Sequence number in the log
    pub sequence: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Entry data
    pub data: serde_json::Value,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Storage interface for logs
#[async_trait]
pub trait LogStorage: Send + Sync {
    /// Append an entry to the log
    fn append(&self, entry: LogEntry) -> Result<()>;
    
    /// Read entries from the log
    fn read(&self, start: usize, count: usize) -> Result<Vec<LogEntry>>;
    
    /// Get the current length of the log
    fn len(&self) -> Result<usize>;
    
    /// Check if the log is empty
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}

/// A memory-based implementation of LogStorage
pub struct MemoryLogStorage {
    entries: Arc<std::sync::RwLock<Vec<LogEntry>>>,
}

impl MemoryLogStorage {
    /// Create a new memory log storage
    pub fn new() -> Self {
        MemoryLogStorage {
            entries: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }
}

impl LogStorage for MemoryLogStorage {
    fn append(&self, mut entry: LogEntry) -> Result<()> {
        let mut entries = self.entries.write().map_err(|e| 
            LogStorageError::Internal(format!("Failed to lock entries: {}", e)))?;
            
        // Set sequence
        entry.sequence = entries.len() as u64;
        
        // Append
        entries.push(entry);
        
        Ok(())
    }
    
    fn read(&self, start: usize, count: usize) -> Result<Vec<LogEntry>> {
        let entries = self.entries.read().map_err(|e| 
            LogStorageError::Internal(format!("Failed to lock entries: {}", e)))?;
            
        if start >= entries.len() {
            return Ok(Vec::new());
        }
        
        let end = std::cmp::min(start + count, entries.len());
        
        Ok(entries[start..end].to_vec())
    }
    
    fn len(&self) -> Result<usize> {
        let entries = self.entries.read().map_err(|e| 
            LogStorageError::Internal(format!("Failed to lock entries: {}", e)))?;
            
        Ok(entries.len())
    }
}

impl Default for MemoryLogStorage {
    fn default() -> Self {
        Self::new()
    }
} 