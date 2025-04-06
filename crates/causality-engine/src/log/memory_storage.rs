// In-memory log storage implementation
//
// This module provides an in-memory storage implementation for the log system.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;
use anyhow::Result;
use causality_types::{DomainId, Timestamp};

use crate::log::{LogStorage, Log, LogEntry, EntryType, EntryData};
use causality_error::{EngineError, CausalityError};

/// In-memory storage for log entries
#[derive(Debug)]
pub struct MemoryLogStorage {
    entries: Mutex<Vec<LogEntry>>,
}

impl MemoryLogStorage {
    /// Create a new empty memory log storage
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }
}

impl LogStorage for MemoryLogStorage {
    fn entry_count(&self) -> causality_error::Result<usize> {
        match self.entries.lock() {
            Ok(entries) => Ok(entries.len()),
            Err(_) => Err(Box::new(EngineError::SyncError("Failed to lock entries".into()))),
        }
    }
    
    fn read(&self, offset: usize, limit: usize) -> causality_error::Result<Vec<LogEntry>> {
        match self.entries.lock() {
            Ok(entries) => {
                let start = std::cmp::min(offset, entries.len());
                let end = std::cmp::min(offset + limit, entries.len());
                Ok(entries[start..end].to_vec())
            },
            Err(_) => Err(Box::new(EngineError::SyncError("Failed to lock entries".into()))),
        }
    }
    
    fn append(&self, entry: LogEntry) -> causality_error::Result<()> {
        match self.entries.lock() {
            Ok(mut entries) => {
        entries.push(entry);
        Ok(())
            },
            Err(_) => Err(Box::new(EngineError::SyncError("Failed to lock entries".into()))),
        }
    }

    // Implement the get_entry_by_id method required by the trait
    fn get_entry_by_id(&self, id: &str) -> causality_error::Result<Option<LogEntry>> {
        match self.entries.lock() {
            Ok(entries) => {
                let found = entries.iter()
                    .find(|e| e.id == id)
                    .cloned();
                Ok(found)
            },
            Err(_) => Err(Box::new(EngineError::SyncError("Failed to lock entries".into()))),
        }
    }

    // Implement the get_entries_by_trace method required by the trait
    fn get_entries_by_trace(&self, trace_id: &str) -> causality_error::Result<Vec<LogEntry>> {
        match self.entries.lock() {
            Ok(entries) => {
                let filtered = entries.iter()
                    .filter(|e| e.trace_id.as_ref().map_or(false, |t| t.as_str() == trace_id))
                    .cloned()
                    .collect();
                Ok(filtered)
            },
            Err(_) => Err(Box::new(EngineError::SyncError("Failed to lock entries".into()))),
        }
    }
}

#[async_trait]
impl Log for MemoryLogStorage {
    async fn add_entry(&self, entry: LogEntry) -> Result<()> {
        // Use the LogStorage append method
        LogStorage::append(self, entry).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
    
    async fn query_entries(&self, domain: &DomainId, entry_type: EntryType, since: Option<u64>) -> Result<Vec<LogEntry>> {
        match self.entries.lock() {
            Ok(entries) => {
                let filtered = entries.iter()
                    .filter(|e| {
                        // Match entry type
                        if e.entry_type != entry_type {
                            return false;
                        }
                        
                        // Match domain
                        let domain_matches = match &e.data {
                            EntryData::Fact(fact) => &fact.domain == domain,
                            EntryData::Effect(effect) => effect.domains.contains(domain),
                            // Handle other entry types as needed
                            _ => false,
                        };
                        
                        // Match timestamp if provided
                        let time_matches = match since {
                            Some(ts) => e.timestamp.to_millis() >= ts,
                            None => true,
                        };
                        
                        domain_matches && time_matches
                    })
                    .cloned()
                    .collect();
                
                Ok(filtered)
            },
            Err(_) => Err(anyhow::anyhow!("Failed to lock entries")),
        }
    }
    
    async fn get_entry_by_id(&self, id: &str) -> Result<Option<LogEntry>> {
        // Use the LogStorage get_entry_by_id method
        LogStorage::get_entry_by_id(self, id).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
    
    async fn get_entries_by_trace(&self, trace_id: &str) -> Result<Vec<LogEntry>> {
        // Use the LogStorage get_entries_by_trace method
        LogStorage::get_entries_by_trace(self, trace_id).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
    
    async fn get_entries_in_time_range(&self, start_time: u64, end_time: u64) -> Result<Vec<LogEntry>> {
        match self.entries.lock() {
            Ok(entries) => {
                let filtered = entries.iter()
                    .filter(|e| {
                        let ts = e.timestamp.to_millis();
                        ts >= start_time && ts <= end_time
                    })
                    .cloned()
                    .collect();
                
                Ok(filtered)
            },
            Err(_) => Err(anyhow::anyhow!("Failed to lock entries")),
        }
    }
    
    async fn get_all_entries(&self) -> Result<Vec<LogEntry>> {
        match self.entries.lock() {
            Ok(entries) => {
                Ok(entries.clone())
            },
            Err(_) => Err(anyhow::anyhow!("Failed to lock entries")),
        }
    }
    
    async fn get_entry_count(&self) -> Result<usize> {
        match self.entries.lock() {
            Ok(entries) => {
                Ok(entries.len())
            },
            Err(_) => Err(anyhow::anyhow!("Failed to lock entries")),
        }
    }
    
    async fn clear(&self) -> Result<()> {
        match self.entries.lock() {
            Ok(mut entries) => {
                entries.clear();
                Ok(())
            },
            Err(_) => Err(anyhow::anyhow!("Failed to lock entries")),
        }
    }
}

impl Default for MemoryLogStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::types::{BorshJsonValue, FactEntry};
    use std::collections::HashMap;
    
    fn create_test_entry(id: &str) -> LogEntry {
        let fact_entry = FactEntry {
            domain: DomainId::from_str("test_domain").unwrap(),
            block_height: 0,
            block_hash: None,
            observed_at: 123456789,
            fact_type: "test_fact".to_string(),
            resources: Vec::new(),
            data: BorshJsonValue(serde_json::Value::Null),
            verified: false,
            domain_id: DomainId::from_str("test_domain").unwrap(),
            fact_id: "test_fact".to_string(),
        };
        
        LogEntry {
            id: id.to_string(),
            timestamp: (123456789u64).into(),
            entry_type: EntryType::Fact,
            data: EntryData::Fact(fact_entry),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_memory_storage() {
        // Create storage
        let storage = MemoryLogStorage::new();
        
        // Append entries
        let entry1 = create_test_entry("entry1");
        let entry2 = create_test_entry("entry2");
        storage.append(entry1.clone()).unwrap();
        storage.append(entry2.clone()).unwrap();
        
        // Test count
        assert_eq!(storage.entry_count().unwrap(), 2);
        
        // Test read
        let entries = storage.read(0, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "entry1");
        assert_eq!(entries[1].id, "entry2");
        
        // Test read with offset and limit
        let entries = storage.read(1, 1).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "entry2");
    }
    
    #[tokio::test]
    async fn test_log_interface() {
        let log = MemoryLogStorage::new();
        
        // Add entries
        let entry1 = create_test_entry("entry1");
        let entry2 = create_test_entry("entry2");
        Log::add_entry(&log, entry1.clone()).await.unwrap();
        Log::add_entry(&log, entry2.clone()).await.unwrap();
        
        // Test count
        assert_eq!(Log::get_entry_count(&log).await.unwrap(), 2);
        
        // Test get all
        let entries = Log::get_all_entries(&log).await.unwrap();
        assert_eq!(entries.len(), 2);
        
        // Test get by ID
        let found = Log::get_entry_by_id(&log, "entry1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "entry1");
        
        // Test query
        let domain = DomainId::from_str("test_domain").unwrap();
        let queried = Log::query_entries(&log, &domain, EntryType::Fact, None).await.unwrap();
        assert_eq!(queried.len(), 2);
        
        // Test clear
        Log::clear(&log).await.unwrap();
        assert_eq!(Log::get_entry_count(&log).await.unwrap(), 0);
    }
} 