// Memory storage implementation for Causality Unified Log System
//
// This module provides a memory-based implementation of the LogStorage trait.

use std::sync::{Arc, RwLock};
use std::collections::{HashMap, BTreeMap};

use crate::error::{Error, Result};
use crate::log::entry::LogEntry;
use crate::log::storage::{LogStorage, StorageConfig};
use crate::types::Timestamp;

/// Memory-based storage for log entries
///
/// This implementation stores log entries in memory.
pub struct MemoryLogStorage {
    /// The log entries
    entries: RwLock<Vec<LogEntry>>,
    /// The storage configuration
    config: Arc<RwLock<StorageConfig>>,
    /// Index of entries by ID for faster lookup
    entry_index: RwLock<HashMap<String, usize>>,
    /// Index of entries by trace ID for faster trace-based lookups
    trace_index: RwLock<HashMap<String, Vec<usize>>>,
    /// Index of entries by timestamp for faster time range queries
    time_index: RwLock<BTreeMap<Timestamp, Vec<usize>>>,
}

impl MemoryLogStorage {
    /// Create a new memory log storage with default configuration
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            config: Arc::new(RwLock::new(StorageConfig::default())),
            entry_index: RwLock::new(HashMap::new()),
            trace_index: RwLock::new(HashMap::new()),
            time_index: RwLock::new(BTreeMap::new()),
        }
    }
    
    /// Create a new memory log storage with a specific configuration
    pub fn new_with_config(config: StorageConfig) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            config: Arc::new(RwLock::new(config)),
            entry_index: RwLock::new(HashMap::new()),
            trace_index: RwLock::new(HashMap::new()),
            time_index: RwLock::new(BTreeMap::new()),
        }
    }
    
    /// Get a reference to the storage configuration
    pub fn config(&self) -> Arc<RwLock<StorageConfig>> {
        Arc::clone(&self.config)
    }
    
    /// Set the storage configuration
    pub fn set_config(&self, config: StorageConfig) -> Result<()> {
        let mut current_config = self.config.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on config".to_string())
        })?;
        *current_config = config;
        Ok(())
    }
    
    /// Update the indexes for a new entry
    fn update_indexes(&self, entry: &LogEntry, index: usize) -> Result<()> {
        // Update ID index
        let mut id_index = self.entry_index.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on entry_index".to_string())
        })?;
        id_index.insert(entry.id.clone(), index);
        
        // Update trace index if entry has a trace ID
        if let Some(trace_id) = &entry.trace_id {
            let mut trace_index = self.trace_index.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on trace_index".to_string())
            })?;
            
            trace_index
                .entry(trace_id.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }
        
        // Update time index
        let mut time_index = self.time_index.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on time_index".to_string())
        })?;
        
        time_index
            .entry(entry.timestamp)
            .or_insert_with(Vec::new)
            .push(index);
        
        Ok(())
    }
}

impl LogStorage for MemoryLogStorage {
    fn append(&self, mut entry: LogEntry) -> Result<()> {
        // Get config for hash verification
        let config = self.config.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on config".to_string())
        })?;
        
        // Ensure the entry has a valid hash if required
        if config.enforce_hash_verification {
            self.ensure_valid_hash(&mut entry)?;
        }
        
        // Verify the hash before storing
        self.verify_entry_hash(&entry, &config)?;
        
        // Store the entry
        let mut entries = self.entries.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on entries".to_string())
        })?;
        
        let entry_index = entries.len();
        entries.push(entry.clone());
        
        // Update indexes
        drop(entries); // Release the lock before updating indexes
        self.update_indexes(&entry, entry_index)?;
        
        Ok(())
    }
    
    fn append_batch(&self, mut entries: Vec<LogEntry>) -> Result<()> {
        // Get config for hash verification
        let config = self.config.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on config".to_string())
        })?;
        
        // Ensure entries have valid hashes if required
        if config.enforce_hash_verification {
            for entry in &mut entries {
                self.ensure_valid_hash(entry)?;
            }
        }
        
        // Verify hashes before storing
        for entry in &entries {
            self.verify_entry_hash(entry, &config)?;
        }
        
        // Store the entries
        let mut storage_entries = self.entries.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on entries".to_string())
        })?;
        
        let start_index = storage_entries.len();
        
        // Update indexes (need to do this outside the lock)
        let clone_entries = entries.clone();
        storage_entries.extend(entries);
        drop(storage_entries); // Release the lock
        
        for (i, entry) in clone_entries.iter().enumerate() {
            self.update_indexes(entry, start_index + i)?;
        }
        
        Ok(())
    }
    
    fn read(&self, start: usize, count: usize) -> Result<Vec<LogEntry>> {
        let entries = self.entries.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on entries".to_string())
        })?;
        
        if start >= entries.len() {
            return Ok(Vec::new());
        }
        
        let end = (start + count).min(entries.len());
        let result = entries[start..end].to_vec();
        
        Ok(result)
    }
    
    fn read_time_range(&self, start_time: u64, end_time: u64) -> Result<Vec<LogEntry>> {
        // Use time index for efficient lookup
        let time_index = self.time_index.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on time_index".to_string())
        })?;
        
        let entries = self.entries.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on entries".to_string())
        })?;
        
        let mut result = Vec::new();
        
        // Get all entries with timestamps in the range
        for (timestamp, indices) in time_index.range(start_time..=end_time) {
            for &idx in indices {
                if idx < entries.len() {
                    result.push(entries[idx].clone());
                }
            }
        }
        
        // Sort by timestamp
        result.sort_by_key(|e| e.timestamp);
        
        Ok(result)
    }
    
    fn entry_count(&self) -> Result<usize> {
        let entries = self.entries.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on entries".to_string())
        })?;
        Ok(entries.len())
    }
    
    fn get_entry_by_id(&self, id: &str) -> Result<Option<LogEntry>> {
        // Use ID index for efficient lookup
        let id_index = self.entry_index.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on entry_index".to_string())
        })?;
        
        let entries = self.entries.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on entries".to_string())
        })?;
        
        if let Some(&idx) = id_index.get(id) {
            if idx < entries.len() {
                return Ok(Some(entries[idx].clone()));
            }
        }
        
        Ok(None)
    }
    
    fn get_entries_by_trace(&self, trace_id: &str) -> Result<Vec<LogEntry>> {
        // Use trace index for efficient lookup
        let trace_index = self.trace_index.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on trace_index".to_string())
        })?;
        
        let entries = self.entries.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on entries".to_string())
        })?;
        
        if let Some(indices) = trace_index.get(trace_id) {
            let mut result = Vec::with_capacity(indices.len());
            for &idx in indices {
                if idx < entries.len() {
                    result.push(entries[idx].clone());
                }
            }
            return Ok(result);
        }
        
        Ok(Vec::new())
    }
    
    fn flush(&self) -> Result<()> {
        // No-op for memory storage
        Ok(())
    }
    
    fn close(&self) -> Result<()> {
        // No-op for memory storage
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use crate::log::entry::{EntryType, EntryData, EventEntry, EventSeverity};
    
    #[test]
    fn test_memory_storage_operations() -> Result<()> {
        let storage = MemoryLogStorage::new();
        
        // Initial state
        assert_eq!(storage.entry_count()?, 0);
        assert_eq!(storage.read(0, 10)?.len(), 0);
        
        // Create test entries
        let entries = (0..5).map(|i| {
            let event_entry = EventEntry {
                event_name: format!("test_event_{}", i),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"index": i}),
                resources: None,
                domains: None,
            };
            
            LogEntry {
                id: format!("entry_{}", i),
                timestamp: Utc::now().timestamp() as u64 + i as u64,
                entry_type: EntryType::Event,
                data: EntryData::Event(event_entry),
                trace_id: Some("test_trace".to_string()),
                parent_id: None,
                metadata: HashMap::new(),
                entry_hash: None,
            }
        }).collect::<Vec<_>>();
        
        // Add entries
        for entry in entries.clone() {
            storage.append(entry)?;
        }
        
        // Check count
        assert_eq!(storage.entry_count()?, 5);
        
        // Read entries
        let read_entries = storage.read(0, 10)?;
        assert_eq!(read_entries.len(), 5);
        
        // Read with pagination
        let first_page = storage.read(0, 2)?;
        assert_eq!(first_page.len(), 2);
        assert_eq!(first_page[0].id, "entry_0");
        
        let second_page = storage.read(2, 2)?;
        assert_eq!(second_page.len(), 2);
        assert_eq!(second_page[0].id, "entry_2");
        
        // Read beyond end
        let beyond_end = storage.read(5, 2)?;
        assert_eq!(beyond_end.len(), 0);
        
        // Test get_entry_by_id
        let entry = storage.get_entry_by_id("entry_2")?;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().id, "entry_2");
        
        // Test get_entries_by_trace
        let trace_entries = storage.get_entries_by_trace("test_trace")?;
        assert_eq!(trace_entries.len(), 5);
        
        // Test time range query
        let time_entries = storage.read_time_range(
            entries[1].timestamp,
            entries[3].timestamp
        )?;
        assert_eq!(time_entries.len(), 3);  // Should include entries 1, 2, and 3
        
        // Test batch append
        let more_entries = (5..10).map(|i| {
            let event_entry = EventEntry {
                event_name: format!("test_event_{}", i),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"index": i}),
                resources: None,
                domains: None,
            };
            
            LogEntry {
                id: format!("entry_{}", i),
                timestamp: Utc::now().timestamp() as u64 + i as u64,
                entry_type: EntryType::Event,
                data: EntryData::Event(event_entry),
                trace_id: Some("test_trace_2".to_string()),
                parent_id: None,
                metadata: HashMap::new(),
                entry_hash: None,
            }
        }).collect::<Vec<_>>();
        
        storage.append_batch(more_entries)?;
        assert_eq!(storage.entry_count()?, 10);
        
        // Test trace query for second batch
        let trace2_entries = storage.get_entries_by_trace("test_trace_2")?;
        assert_eq!(trace2_entries.len(), 5);
        
        Ok(())
    }
    
    #[test]
    fn test_hash_verification() -> Result<()> {
        // Create storage with hash verification
        let mut config = StorageConfig::default();
        config.enforce_hash_verification = true;
        let storage = MemoryLogStorage::new_with_config(config);
        
        // Create an entry with no hash
        let event_entry = EventEntry {
            event_name: "test_event".to_string(),
            severity: EventSeverity::Info,
            component: "test".to_string(),
            details: serde_json::json!({}),
            resources: None,
            domains: None,
        };
        
        let mut entry = LogEntry {
            id: "entry_1".to_string(),
            timestamp: Utc::now().timestamp() as u64,
            entry_type: EntryType::Event,
            data: EntryData::Event(event_entry),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        // Entry should be accepted with auto-hash generation
        storage.append(entry.clone())?;
        
        // Create a new entry with an invalid hash
        entry.id = "entry_2".to_string();
        entry.entry_hash = Some("invalid_hash".to_string());
        
        // This should generate a new valid hash
        storage.append(entry)?;
        
        // Disable hash verification
        let mut config = StorageConfig::default();
        config.enforce_hash_verification = false;
        storage.set_config(config)?;
        
        // Create another entry with no hash
        let entry3 = LogEntry {
            id: "entry_3".to_string(),
            timestamp: Utc::now().timestamp() as u64,
            entry_type: EntryType::Event,
            data: EntryData::Event(event_entry),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        // Entry should be accepted without a hash
        storage.append(entry3)?;
        
        assert_eq!(storage.entry_count()?, 3);
        
        Ok(())
    }
} 