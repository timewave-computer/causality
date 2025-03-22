// Map of time module for Causality
//
// This module provides functionality for map of time synchronization, correlation,
// and causal consistency tracking across domains to ensure causal consistency 
// between different domains.

pub mod map;
pub mod sync;
pub mod types;

// Re-export key types and components
pub use map::{TimeMap, TimeMapEntry, TimeMapHistory, TimeMapNotifier, SharedTimeMap};
pub use types::{TimePoint, TimeRange};
pub use sync::{TimeSyncConfig, SyncStatus, SyncResult, TimeSource, SyncStrategy, VerificationStatus, TimeCommitment};

// Export additional components from sync
pub use sync::{TimeSyncManager, TimeVerificationService, ConsensusVerificationManager};

// Domain Map
//
// This module provides a mapping between domains and their time-related information,
// allowing for time synchronization across multiple domains.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::types::{DomainId, BlockHeight, BlockHash, Timestamp};
use crate::error::{Error, Result};
use crate::domain::types::TimeMapEntry;

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
        let mut entries = self.entries.write().map_err(|_| Error::SystemError("Failed to acquire write lock on domain map".to_string()))?;
        
        let domain_entries = entries.entry(domain_id.clone()).or_insert_with(HashMap::new);
        domain_entries.insert(entry.height.clone(), entry);
        
        Ok(())
    }
    
    /// Get a time map entry for a specific domain and block height
    pub fn get_entry(&self, domain_id: &DomainId, height: &BlockHeight) -> Result<TimeMapEntry> {
        let entries = self.entries.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain map".to_string()))?;
        
        let domain_entries = entries.get(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;
            
        domain_entries.get(height)
            .cloned()
            .ok_or_else(|| Error::TimeMapError(format!("No entry for domain {} at height {}", domain_id, height)))
    }
    
    /// Get all entries for a specific domain
    pub fn get_domain_entries(&self, domain_id: &DomainId) -> Result<Vec<TimeMapEntry>> {
        let entries = self.entries.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain map".to_string()))?;
        
        let domain_entries = entries.get(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;
            
        Ok(domain_entries.values().cloned().collect())
    }
    
    /// Get the closest entry for a given timestamp
    pub fn get_entry_by_time(&self, domain_id: &DomainId, timestamp: &Timestamp) -> Result<TimeMapEntry> {
        let entries = self.entries.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain map".to_string()))?;
        
        let domain_entries = entries.get(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;
            
        if domain_entries.is_empty() {
            return Err(Error::TimeMapError(format!("No entries for domain {}", domain_id)));
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
            .ok_or_else(|| Error::TimeMapError(format!("Failed to find closest entry for domain {} at time {}", domain_id, timestamp)))
    }
    
    /// Clear all entries for a specific domain
    pub fn clear_domain(&self, domain_id: &DomainId) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| Error::SystemError("Failed to acquire write lock on domain map".to_string()))?;
        
        entries.remove(domain_id);
        
        Ok(())
    }
    
    /// Clear all entries
    pub fn clear_all(&self) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| Error::SystemError("Failed to acquire write lock on domain map".to_string()))?;
        
        entries.clear();
        
        Ok(())
    }
    
    /// Get all domain IDs
    pub fn get_domain_ids(&self) -> Result<Vec<DomainId>> {
        let entries = self.entries.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain map".to_string()))?;
        
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