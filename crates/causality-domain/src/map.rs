// Domain mapping system
// Original file: src/domain/map.rs

// Map of time module for Causality
//
// This module provides functionality for map of time synchronization, correlation,
// and causal consistency tracking across domains to ensure causal consistency 
// between different domains.

// Import submodules
pub mod types;
pub mod sync;

// The time map implementation is in time_map.rs and map_impl.rs
mod time_map;
mod map_impl;

// Re-export key types and components
pub use map_impl::TimeMapImpl;

// Simple TimeMapEntry structure 
#[derive(Debug, Clone)]
pub struct TimeMapEntry {
    /// Domain ID
    pub domain_id: crate::selection::DomainId,
    /// Time point
    pub time_point: types::TimePoint,
    /// Confidence value (0.0 to 1.0)
    pub confidence: f64,
    /// Verification status
    pub verified: bool,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Time map for tracking time points across domains
pub trait TimeMap: Send + Sync {
    /// Add an entry to the time map
    fn add_entry(&self, entry: TimeMapEntry) -> crate::error::Result<()>;
    
    /// Get an entry by domain ID and time point
    fn get_entry(&self, domain_id: &crate::selection::DomainId, time_point: &types::TimePoint) -> crate::error::Result<Option<TimeMapEntry>>;
    
    /// Get entries for a specific domain
    fn get_entries_for_domain(&self, domain_id: &crate::selection::DomainId) -> crate::error::Result<Vec<TimeMapEntry>>;
    
    /// Remove an entry
    fn remove_entry(&self, domain_id: &crate::selection::DomainId, time_point: &types::TimePoint) -> crate::error::Result<bool>;
    
    /// Clear all entries for a domain
    fn clear_domain(&self, domain_id: &crate::selection::DomainId) -> crate::error::Result<()>;
    
    /// Clear all entries
    fn clear_all(&self) -> crate::error::Result<()>;
    
    /// Get all domain IDs
    fn get_domain_ids(&self) -> crate::error::Result<Vec<crate::selection::DomainId>>;
}

/// Time map that keeps a history of changes
pub trait TimeMapHistory: TimeMap {
    /// Get the history of changes for a specific entry
    fn get_entry_history(&self, domain_id: &crate::selection::DomainId, time_point: &types::TimePoint) -> crate::error::Result<Vec<TimeMapEntry>>;
    
    /// Clear history for a domain
    fn clear_history_for_domain(&self, domain_id: &crate::selection::DomainId) -> crate::error::Result<()>;
}

/// Notification callback for time map changes
pub type TimeMapCallback = Box<dyn Fn(&TimeMapEntry) + Send + Sync>;

/// Time map that can notify on changes
pub trait TimeMapNotifier: TimeMap {
    /// Register a callback for when entries are added
    fn on_entry_added(&self, callback: TimeMapCallback) -> crate::error::Result<()>;
    
    /// Register a callback for when entries are updated
    fn on_entry_updated(&self, callback: TimeMapCallback) -> crate::error::Result<()>;
    
    /// Register a callback for when entries are removed
    fn on_entry_removed(&self, callback: TimeMapCallback) -> crate::error::Result<()>;
}

/// Shared time map implementation
pub struct SharedTimeMap {
    /// Inner implementation
    inner: std::sync::Arc<map_impl::TimeMapImpl>,
}

// Domain Map
//
// This module provides a mapping between domains and their time-related information,
// allowing for time synchronization across multiple domains.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use causality_types::{DomainId, BlockHeight, BlockHash, Timestamp};
use crate::error::{Result, system_error, domain_not_found, time_map_error};

use crate::adapter::TimeMapEntry;

/// Domain map for tracking time synchronization across domains
pub struct DomainMap {
    /// Time map entries by domain ID and block height
    entries: RwLock<HashMap<DomainId, HashMap<BlockHeight, TimeMapEntry>>>,
}

impl DomainMap {
    /// Create a new domain map
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a time map entry
    pub fn add_entry(&self, domain_id: &DomainId, entry: TimeMapEntry) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on domain map"))?;
        
        let domain_entries = entries.entry(domain_id.clone()).or_insert_with(HashMap::new);
        domain_entries.insert(entry.height.clone(), entry);
        
        Ok(())
    }
    
    /// Get a time map entry for a specific domain and block height
    pub fn get_entry(&self, domain_id: &DomainId, height: &BlockHeight) -> Result<TimeMapEntry> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on domain map"))?;
        
        let domain_entries = entries.get(domain_id)
            .ok_or_else(|| domain_not_found(domain_id.clone()))?;
            
        domain_entries.get(height)
            .cloned()
            .ok_or_else(|| time_map_error(format!("No entry for domain {} at height {}", domain_id, height)))
    }
    
    /// Get all entries for a specific domain
    pub fn get_domain_entries(&self, domain_id: &DomainId) -> Result<Vec<TimeMapEntry>> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on domain map"))?;
        
        let domain_entries = entries.get(domain_id)
            .ok_or_else(|| domain_not_found(domain_id.clone()))?;
            
        Ok(domain_entries.values().cloned().collect())
    }
    
    /// Get the closest entry for a given timestamp
    pub fn get_entry_by_time(&self, domain_id: &DomainId, timestamp: &Timestamp) -> Result<TimeMapEntry> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on domain map"))?;
        
        let domain_entries = entries.get(domain_id)
            .ok_or_else(|| domain_not_found(domain_id.clone()))?;
            
        if domain_entries.is_empty() {
            return Err(time_map_error(format!("No entries for domain {}", domain_id)));
        }
        
        // Find the entry with the closest timestamp
        let mut closest_entry: Option<&TimeMapEntry> = None;
        let mut min_diff: u64 = u64::MAX;
        
        for entry in domain_entries.values() {
            let diff = if entry.timestamp.value() > timestamp.value() {
                entry.timestamp.value() - timestamp.value()
            } else {
                timestamp.value() - entry.timestamp.value()
            };
            
            if diff < min_diff {
                min_diff = diff;
                closest_entry = Some(entry);
            }
        }
        
        closest_entry
            .cloned()
            .ok_or_else(|| time_map_error(format!("Failed to find closest entry for domain {} at time {}", domain_id, timestamp)))
    }
    
    /// Clear all entries for a specific domain
    pub fn clear_domain(&self, domain_id: &DomainId) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on domain map"))?;
        
        entries.remove(domain_id);
        
        Ok(())
    }
    
    /// Clear all entries
    pub fn clear_all(&self) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on domain map"))?;
        
        entries.clear();
        
        Ok(())
    }
    
    /// Get all domain IDs
    pub fn get_domain_ids(&self) -> Result<Vec<DomainId>> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on domain map"))?;
        
        Ok(entries.keys().cloned().collect())
    }
}

impl Default for DomainMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_map() {
        let map = DomainMap::new();
        
        // Add entries
        let domain1 = DomainId::new("domain1");
        let entry1 = TimeMapEntry {
            height: BlockHeight::new(100),
            hash: BlockHash::new(vec![1, 2, 3, 4]),
            timestamp: Timestamp::new(1000),
        };
        
        let entry2 = TimeMapEntry {
            height: BlockHeight::new(200),
            hash: BlockHash::new(vec![5, 6, 7, 8]),
            timestamp: Timestamp::new(2000),
        };
        
        map.add_entry(&domain1, entry1.clone()).unwrap();
        map.add_entry(&domain1, entry2.clone()).unwrap();
        
        // Get entry by height
        let retrieved_entry = map.get_entry(&domain1, &BlockHeight::new(100)).unwrap();
        assert_eq!(retrieved_entry.height, BlockHeight::new(100));
        assert_eq!(retrieved_entry.timestamp, Timestamp::new(1000));
        
        // Get entry by time
        let time_entry = map.get_entry_by_time(&domain1, &Timestamp::new(1500)).unwrap();
        assert_eq!(time_entry.height, BlockHeight::new(100));
        
        // Get all entries for domain
        let domain_entries = map.get_domain_entries(&domain1).unwrap();
        assert_eq!(domain_entries.len(), 2);
        
        // Get all domain IDs
        let domain_ids = map.get_domain_ids().unwrap();
        assert_eq!(domain_ids.len(), 1);
        assert_eq!(domain_ids[0], domain1);
        
        // Clear domain
        map.clear_domain(&domain1).unwrap();
        
        // Verify domain is cleared
        let domain_ids_after = map.get_domain_ids().unwrap();
        assert_eq!(domain_ids_after.len(), 0);
    }
} 