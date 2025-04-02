// In-memory log storage implementation
//
// This module provides an in-memory implementation of the LogStorage trait
// for testing and development.

use std::sync::RwLock;
use std::collections::HashMap;

use causality_error::{BoxError, Result, Error};

use crate::log::LogStorage;
use crate::log::types::LogEntry;

/// In-memory implementation of the LogStorage trait
pub struct MemoryLogStorage {
    /// The entries stored in memory
    entries: RwLock<Vec<LogEntry>>,
    /// Index of entries by ID
    entry_index: RwLock<HashMap<String, usize>>,
}

impl MemoryLogStorage {
    /// Create a new memory log storage
    pub fn new() -> Self {
        MemoryLogStorage {
            entries: RwLock::new(Vec::new()),
            entry_index: RwLock::new(HashMap::new()),
        }
    }
    
    /// Find an entry by ID
    pub fn find_by_id(&self, id: &str) -> Result<Option<LogEntry>> {
        let index = self.entry_index.read().map_err(|e| 
            Box::new(Error::Unavailable(format!("Failed to acquire read lock on entry_index: {}", e)))
        )?;
        
        Ok(index.get(id).and_then(|idx| {
            let entries = self.entries.read().ok()?;
            entries.get(*idx).cloned()
        }))
    }
}

impl LogStorage for MemoryLogStorage {
    fn entry_count(&self) -> Result<usize> {
        let entries = self.entries.read().map_err(|e| 
            Box::new(Error::Unavailable(format!("Failed to acquire read lock on entries: {}", e)))
        )?;
        
        Ok(entries.len())
    }
    
    fn read(&self, offset: usize, limit: usize) -> Result<Vec<LogEntry>> {
        let entries = self.entries.read().map_err(|e| 
            Box::new(Error::Unavailable(format!("Failed to acquire read lock on entries: {}", e)))
        )?;
        
        let end = (offset + limit).min(entries.len());
        
        Ok(entries[offset..end].to_vec())
    }
    
    fn append(&self, entry: LogEntry) -> Result<()> {
        let mut entries = self.entries.write().map_err(|e| 
            Box::new(Error::Unavailable(format!("Failed to acquire write lock on entries: {}", e)))
        )?;
        
        let mut index = self.entry_index.write().map_err(|e| 
            Box::new(Error::Unavailable(format!("Failed to acquire write lock on entry_index: {}", e)))
        )?;
        
        // Store the entry index
        let idx = entries.len();
        index.insert(entry.id.clone(), idx);
        
        // Append the entry
        entries.push(entry);
        
        Ok(())
    }
} 