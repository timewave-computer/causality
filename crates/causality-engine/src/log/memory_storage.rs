// In-memory log storage implementation
//
// This module provides an in-memory implementation of the LogStorage trait
// for testing and development.

use std::sync::RwLock;
use std::collections::HashMap;

use causality_error::{EngineError, CausalityError};
use async_trait::async_trait;

use crate::log::LogStorage;
use crate::log::types::LogEntry;
use crate::log::EntryType;

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
    pub fn find_by_id(&self, id: &str) -> causality_error::Result<Option<LogEntry>> {
        let index = self.entry_index.read().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire read lock on entry_index: {}", e))) as Box<dyn CausalityError>
        })?;
        
        Ok(index.get(id).and_then(|idx| {
            let entries = self.entries.read().ok()?;
            entries.get(*idx).cloned()
        }))
    }
}

#[async_trait]
impl LogStorage for MemoryLogStorage {
    fn entry_count(&self) -> causality_error::Result<usize> {
        let entries = self.entries.read().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire read lock on entries: {}", e))) as Box<dyn CausalityError>
        })?;
        
        Ok(entries.len())
    }
    
    fn read(&self, offset: usize, limit: usize) -> causality_error::Result<Vec<LogEntry>> {
        let entries = self.entries.read().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire read lock on entries: {}", e))) as Box<dyn CausalityError>
        })?;
        
        let end = (offset + limit).min(entries.len());
        
        Ok(entries[offset..end].to_vec())
    }
    
    fn append(&self, entry: LogEntry) -> causality_error::Result<()> {
        let mut entries = self.entries.write().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire write lock on entries: {}", e))) as Box<dyn CausalityError>
        })?;
        
        let mut index = self.entry_index.write().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire write lock on entry_index: {}", e))) as Box<dyn CausalityError>
        })?;
        
        // Store the entry index
        let idx = entries.len();
        index.insert(entry.id.clone(), idx);
        
        // Append the entry
        entries.push(entry);
        
        Ok(())
    }
    
    // Implement the required methods from the LogStorage trait
    fn append_batch(&self, entries: Vec<LogEntry>) -> causality_error::Result<()> {
        for entry in entries {
            // Handle errors manually instead of using ?
            match self.append(entry) {
                Ok(_) => {},
                Err(e) => return Err(e)
            }
        }
        Ok(())
    }
    
    fn read_time_range(&self, start_time: u64, end_time: u64) -> causality_error::Result<Vec<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        let entries = match self.read(0, entry_count) {
            Ok(entries) => entries,
            Err(e) => return Err(e)
        };
        
        Ok(entries.into_iter()
            .filter(|e| {
                let ts = e.timestamp.as_millis();
                ts >= start_time && ts <= end_time
            })
            .collect())
    }
    
    // Additional async methods
    async fn get_entry_count(&self) -> causality_error::Result<usize> {
        self.entry_count()
    }
    
    async fn get_all_entries(&self) -> causality_error::Result<Vec<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        self.read(0, entry_count)
    }
    
    async fn get_entries(&self, start: usize, end: usize) -> causality_error::Result<Vec<LogEntry>> {
        self.read(start, end - start)
    }
    
    async fn append_entry(&self, entry: LogEntry) -> causality_error::Result<()> {
        self.append(entry)
    }
    
    async fn append_entries_batch(&self, entries: Vec<LogEntry>) -> causality_error::Result<()> {
        self.append_batch(entries)
    }
    
    fn find_entries_by_type(&self, entry_type: EntryType) -> causality_error::Result<Vec<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        let entries = match self.read(0, entry_count) {
            Ok(entries) => entries,
            Err(e) => return Err(e)
        };
        
        Ok(entries.into_iter()
            .filter(|e| e.entry_type == entry_type)
            .collect())
    }
    
    async fn find_entries_by_type_async(&self, entry_type: EntryType) -> causality_error::Result<Vec<LogEntry>> {
        self.find_entries_by_type(entry_type)
    }
    
    fn find_entries_in_time_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> causality_error::Result<Vec<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        let entries = match self.read(0, entry_count) {
            Ok(entries) => entries,
            Err(e) => return Err(e)
        };
        
        Ok(entries.into_iter()
            .filter(|e| {
                let ts = crate::log::time_utils::timestamp_to_datetime(e.timestamp.clone());
                ts >= start && ts <= end
            })
            .collect())
    }
    
    async fn find_entries_in_time_range_async(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> causality_error::Result<Vec<LogEntry>> {
        self.find_entries_in_time_range(start, end)
    }
    
    async fn async_flush(&self) -> causality_error::Result<()> {
        // No-op for memory storage
        Ok(())
    }
    
    async fn clear(&self) -> causality_error::Result<()> {
        let mut entries = self.entries.write().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire write lock on entries: {}", e))) as Box<dyn CausalityError>
        })?;
        
        let mut index = self.entry_index.write().map_err(|e| {
            Box::new(EngineError::LogError(format!("Failed to acquire write lock on entry_index: {}", e))) as Box<dyn CausalityError>
        })?;
        
        entries.clear();
        index.clear();
        
        Ok(())
    }
    
    fn close(&self) -> causality_error::Result<()> {
        // No special closing needed for in-memory
        Ok(())
    }
    
    fn find_entries_by_trace_id(&self, trace_id: &causality_types::TraceId) -> causality_error::Result<Vec<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        let entries = match self.read(0, entry_count) {
            Ok(entries) => entries,
            Err(e) => return Err(e)
        };
        
        Ok(entries.into_iter()
            .filter(|e| {
                e.trace_id.as_ref().map_or(false, |t| t == trace_id)
            })
            .collect())
    }
    
    fn get_entries_by_trace(&self, trace_id: &str) -> causality_error::Result<Vec<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        let entries = match self.read(0, entry_count) {
            Ok(entries) => entries,
            Err(e) => return Err(e)
        };
        
        Ok(entries.into_iter()
            .filter(|e| {
                e.trace_id.as_ref().map_or(false, |t| t.as_str() == trace_id)
            })
            .collect())
    }
    
    fn get_entry_by_id(&self, id: &str) -> causality_error::Result<Option<LogEntry>> {
        self.find_by_id(id)
    }
    
    fn get_entry_by_hash(&self, hash: &str) -> causality_error::Result<Option<LogEntry>> {
        // Manual error handling
        let entry_count = match self.entry_count() {
            Ok(count) => count,
            Err(e) => return Err(e)
        };
        
        let entries = match self.read(0, entry_count) {
            Ok(entries) => entries,
            Err(e) => return Err(e)
        };
        
        Ok(entries.into_iter()
            .find(|entry| entry.entry_hash.as_ref().map_or(false, |h| h == hash)))
    }
    
    fn rotate(&self) -> causality_error::Result<()> {
        // No-op for memory storage
        Ok(())
    }
    
    fn compact(&self) -> causality_error::Result<()> {
        // No-op for memory storage
        Ok(())
    }
} 