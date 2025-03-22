// Time map integration for log entries
// This module provides integration between the log system and time map
// Only available when the domain feature is enabled

// Define a minimal public API for when domain feature is not enabled
#[cfg(not(feature = "domain"))]
use std::sync::{Arc, Mutex};

#[cfg(not(feature = "domain"))]
use crate::log::{LogStorage, EntryType, LogEntry};

/// Integration between log storage and time map
/// This is a placeholder implementation when domain feature is not enabled
#[cfg(not(feature = "domain"))]
pub struct LogTimeMapIntegration {}

#[cfg(not(feature = "domain"))]
impl LogTimeMapIntegration {
    /// Create a new log-time map integration
    pub fn new(_storage: Arc<Mutex<dyn LogStorage + Send>>) -> Self {
        LogTimeMapIntegration {}
    }
    
    /// Index log entries up to the current point
    pub fn index_up_to_current(&mut self) -> crate::error::Result<()> {
        // No-op in minimal build
        Ok(())
    }
    
    /// Create a time indexed entry from a log entry
    pub fn time_indexed_entry_from_log_entry(&self, _entry: &LogEntry) -> crate::error::Result<()> {
        // No-op in minimal build
        Ok(())
    }
    
    /// Add time map information to a log entry (stub for minimal build)
    pub fn attach_time_map(_entry: &mut LogEntry, _time_map: &TimeMap) -> crate::error::Result<()> {
        // No-op in minimal build
        Ok(())
    }
    
    /// Calculate a hash of the time map (stub for minimal build)
    pub fn calculate_time_map_hash(_time_map: &TimeMap) -> crate::error::Result<String> {
        // No-op in minimal build
        Ok("".to_string())
    }
    
    /// Query log entries by time map time range (stub for minimal build)
    pub fn query_time_range(
        _time_map: &TimeMap,
        _storage: &dyn LogStorage,
        _start_time: Timestamp,
        _end_time: Timestamp
    ) -> crate::error::Result<Vec<LogEntry>> {
        // No-op in minimal build
        Ok(Vec::new())
    }
    
    /// Get the time map hash from a log entry (stub for minimal build)
    pub fn get_time_map_hash(_entry: &LogEntry) -> Option<&str> {
        // No-op in minimal build
        None
    }
    
    /// Verify that a log entry's time map hash matches a given time map (stub for minimal build)
    pub fn verify_time_map(_entry: &LogEntry, _time_map: &TimeMap) -> crate::error::Result<bool> {
        // No-op in minimal build
        Ok(false)
    }
}

#[cfg(not(feature = "domain"))]
pub struct TimeIndexedEntry {
    /// The timestamp of the entry
    pub timestamp: u64,
    /// The index in the log storage
    pub log_index: u64,
    /// The type of entry
    pub entry_type: EntryType,
    /// The trace ID
    pub trace_id: Vec<String>,
    /// The resource ID, if any
    pub resource_id: Option<u64>,
}

#[cfg(not(feature = "domain"))]
pub struct TimeMap {}

#[cfg(not(feature = "domain"))]
pub struct Timestamp {
    value: u64,
}

#[cfg(not(feature = "domain"))]
impl Timestamp {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

// Full implementation when domain feature is enabled
#[cfg(feature = "domain")]
mod domain_implementation {
    use std::collections::HashMap;
    use std::ops::Range;
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    
    use crate::log::{EntryType, LogEntry, LogStorage};
    // Import TimeMap directly from domain if it's at the top level
    use crate::domain::map::map::{TimeMap, TimeMapEntry};
    use crate::types::{DomainId, BlockHeight, Timestamp, Hash as BlockHash};
    use crate::effect::EffectType;
    use crate::error::{Error, Result};

    /// Represents a log entry in a time-indexed structure
    #[derive(Debug, Clone)]
    pub struct TimeIndexedEntry {
        /// The timestamp of the entry
        pub timestamp: u64,
        /// The index in the log storage
        pub log_index: u64,
        /// The type of entry
        pub entry_type: EntryType,
        /// The trace ID
        pub trace_id: Vec<String>,
        /// The domain ID
        pub domain: Vec<u8>,
        /// The resource ID, if any
        pub resource_id: Option<u64>,
    }

    /// Integration between log storage and time map
    pub struct LogTimeMapIntegration {
        /// The log storage
        storage: Arc<Mutex<dyn LogStorage + Send>>,
        /// The time map
        time_map: TimeMap,
        /// The last indexed entry
        indexed_up_to: u64,
    }

    impl LogTimeMapIntegration {
        /// Create a new log-time map integration
        pub fn new(storage: Arc<Mutex<dyn LogStorage + Send>>) -> Self {
            LogTimeMapIntegration {
                storage,
                time_map: TimeMap::new(),
                indexed_up_to: 0,
            }
        }
        
        /// Index log entries up to the current point
        pub fn index_up_to_current(&mut self) -> Result<()> {
            // Implementation would go here
            Ok(())
        }
        
        /// Create a time indexed entry from a log entry
        pub fn time_indexed_entry_from_log_entry(&self, _entry: &LogEntry) -> Result<()> {
            // Implementation would go here
            Ok(())
        }
        
        /// Build the complete index
        pub fn build_index(&mut self) -> Result<()> {
            // Implementation would go here
            Ok(())
        }
        
        /// Query entries in a time range
        pub fn query_time_range_indexed(&self, start_time: u64, end_time: u64) -> Result<Vec<TimeIndexedEntry>> {
            // Implementation would go here
            Ok(Vec::new())
        }
        
        /// Query entries by trace ID
        pub fn query_by_trace(&self, trace_id: &[u8], start_time: Option<u64>, end_time: Option<u64>) -> Result<Vec<TimeIndexedEntry>> {
            // Implementation would go here
            Ok(Vec::new())
        }
        
        /// Query entries by resource ID
        pub fn query_by_resource(&self, resource_id: u64, start_time: Option<u64>, end_time: Option<u64>) -> Result<Vec<TimeIndexedEntry>> {
            // Implementation would go here
            Ok(Vec::new())
        }
        
        /// Query entries by type
        pub fn query_by_type(&self, entry_type: EntryType, start_time: Option<u64>, end_time: Option<u64>) -> Result<Vec<TimeIndexedEntry>> {
            // Implementation would go here
            Ok(Vec::new())
        }
        
        /// Resolve full entries from indexed entries
        pub fn resolve_entries(&self, indexed_entries: &[TimeIndexedEntry]) -> Result<Vec<LogEntry>> {
            // Implementation would go here
            Ok(Vec::new())
        }
        
        /// Add time map information to a log entry
        pub fn attach_time_map(entry: &mut LogEntry, time_map: &TimeMap) -> Result<()> {
            // Generate a hash of the time map
            let time_map_hash = Self::calculate_time_map_hash(time_map)?;
            
            // Add the time map hash to the log entry's metadata
            entry.metadata.insert("time_map_hash".to_string(), time_map_hash);
            
            // Add a list of observed domains to the metadata
            let domains = time_map.entries.keys()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(",");
            
            entry.metadata.insert("time_map_domains".to_string(), domains);
            
            // For effects, add more detailed time map information
            if entry.entry_type == EntryType::Effect {
                // Add the time map version
                entry.metadata.insert("time_map_version".to_string(), time_map.version.to_string());
                
                // Add the time map created timestamp
                entry.metadata.insert("time_map_created_at".to_string(), 
                                    time_map.created_at.to_rfc3339());
            }
            
            Ok(())
        }
        
        /// Calculate a hash of the time map
        pub fn calculate_time_map_hash(time_map: &TimeMap) -> Result<String> {
            // Create a sorted representation for consistent hashing
            let mut domains: Vec<(&DomainId, &TimeMapEntry)> = time_map.entries.iter().collect();
            domains.sort_by_key(|(id, _)| *id);
            
            // Create a deterministic string representation
            let mut hash_input = String::new();
            
            // Add the time map version and created_at
            hash_input.push_str(&format!("v:{},t:{},", 
                                        time_map.version,
                                        time_map.created_at.timestamp()));
            
            // Add domain entries in deterministic order
            for (domain_id, entry) in domains {
                hash_input.push_str(&format!("{}:{}:{}:{};", 
                                        domain_id,
                                        entry.height,
                                        entry.hash,
                                        entry.timestamp));
            }
            
            // Calculate the hash
            let hash = format!("{:x}", blake3::hash(hash_input.as_bytes()));
            Ok(hash)
        }
        
        /// Query log entries by time map time range
        pub fn query_time_range(
            time_map: &TimeMap,
            storage: &dyn LogStorage,
            start_time: Timestamp,
            end_time: Timestamp
        ) -> Result<Vec<LogEntry>> {
            let mut result = Vec::new();
            
            // Read all entries from storage
            let entries = storage.read(0, storage.entry_count()?)?;
            
            // Filter entries based on time map information
            for entry in entries {
                // Check if the entry has time map information
                if let Some(time_map_hash) = entry.metadata.get("time_map_hash") {
                    // For effects, we can use their time map information
                    if entry.entry_type == EntryType::Effect {
                        // Get the domains observed in this entry
                        if let Some(domains_str) = entry.metadata.get("time_map_domains") {
                            let domains: Vec<DomainId> = domains_str.split(',')
                                .filter(|s| !s.is_empty())
                                .map(|s| DomainId::new(s.parse::<u64>().unwrap_or(0)))
                                .collect();
                            
                            // Check if any observed domain has a timestamp in our range
                            let in_range = domains.iter().any(|domain_id| {
                                if let Some(observed_time) = time_map.get_timestamp(domain_id) {
                                    observed_time >= start_time && observed_time <= end_time
                                } else {
                                    false
                                }
                            });
                            
                            if in_range {
                                result.push(entry);
                            }
                        }
                    } else {
                        // For facts and events, use their timestamp
                        if entry.timestamp >= start_time && entry.timestamp <= end_time {
                            result.push(entry);
                        }
                    }
                } else {
                    // Entries without time map information use only their timestamp
                    if entry.timestamp >= start_time && entry.timestamp <= end_time {
                        result.push(entry);
                    }
                }
            }
            
            Ok(result)
        }
        
        /// Get the time map hash from a log entry
        pub fn get_time_map_hash(entry: &LogEntry) -> Option<&str> {
            entry.metadata.get("time_map_hash").map(|s| s.as_str())
        }
        
        /// Verify that a log entry's time map hash matches a given time map
        pub fn verify_time_map(entry: &LogEntry, time_map: &TimeMap) -> Result<bool> {
            if let Some(entry_hash) = Self::get_time_map_hash(entry) {
                let calculated_hash = Self::calculate_time_map_hash(time_map)?;
                Ok(entry_hash == calculated_hash)
            } else {
                // Entry doesn't have a time map hash
                Ok(false)
            }
        }
    }
}

#[cfg(feature = "domain")]
pub use domain_implementation::*;

// Simplified tests that work with both feature configs
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    #[cfg(feature = "domain")]
    use crate::log::MemoryLogStorage;
    
    #[cfg(feature = "domain")]
    #[test]
    fn test_create_integration() {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let mut integration = LogTimeMapIntegration::new(storage);
        
        // Simple test to verify creation works
        assert!(integration.index_up_to_current().is_ok());
    }
    
    #[cfg(not(feature = "domain"))]
    #[test]
    fn test_minimal_integration() {
        // A basic test for the minimal implementation
        use crate::log::MemoryLogStorage;
        
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let integration = LogTimeMapIntegration::new(storage);
        
        // Verify no-op methods work
        assert!(integration.index_up_to_current().is_ok());
    }
    
    #[cfg(feature = "domain")]
    mod domain_tests {
        use super::*;
        use crate::log::entry::{EventEntry, EventSeverity};
        use crate::log::storage::MemoryLogStorage;
        use chrono::Utc;
        use crate::domain::map::map::{TimeMap, TimeMapEntry};
        use crate::types::{DomainId, BlockHeight, BlockHash, Timestamp};
        use std::collections::HashMap;
        
        #[test]
        fn test_time_map_hash_calculation() {
            // Create a test time map
            let mut time_map = TimeMap::new();
            
            // Add some domain entries
            let domain1 = DomainId::new(1);
            let entry1 = TimeMapEntry::new(
                domain1.clone(), 
                BlockHeight::new(100),
                BlockHash::new("abc123".to_string()),
                Timestamp::new(1000),
            );
            
            let domain2 = DomainId::new(2);
            let entry2 = TimeMapEntry::new(
                domain2.clone(), 
                BlockHeight::new(200),
                BlockHash::new("def456".to_string()),
                Timestamp::new(2000),
            );
            
            time_map.update_domain(entry1);
            time_map.update_domain(entry2);
            
            // Calculate the hash
            let hash = LogTimeMapIntegration::calculate_time_map_hash(&time_map).unwrap();
            
            // Hash should be non-empty
            assert!(!hash.is_empty());
            
            // Create a second identical time map
            let mut time_map2 = TimeMap::new();
            time_map2.update_domain(TimeMapEntry::new(
                domain1.clone(), 
                BlockHeight::new(100),
                BlockHash::new("abc123".to_string()),
                Timestamp::new(1000),
            ));
            time_map2.update_domain(TimeMapEntry::new(
                domain2.clone(), 
                BlockHeight::new(200),
                BlockHash::new("def456".to_string()),
                Timestamp::new(2000),
            ));
            
            // Ensure time_map2 has identical version and created_at
            time_map2.version = time_map.version;
            time_map2.created_at = time_map.created_at;
            
            // Calculate the hash for the second time map
            let hash2 = LogTimeMapIntegration::calculate_time_map_hash(&time_map2).unwrap();
            
            // Hashes should be identical for identical time maps
            assert_eq!(hash, hash2);
            
            // Modify the time map
            time_map.update_domain(TimeMapEntry::new(
                domain1.clone(), 
                BlockHeight::new(101), // Changed height
                BlockHash::new("abc123".to_string()),
                Timestamp::new(1000),
            ));
            
            // Calculate the hash for the modified time map
            let hash3 = LogTimeMapIntegration::calculate_time_map_hash(&time_map).unwrap();
            
            // Hashes should be different for different time maps
            assert_ne!(hash, hash3);
        }
        
        #[test]
        fn test_attach_time_map() {
            // Create a test time map
            let mut time_map = TimeMap::new();
            
            // Add some domain entries
            time_map.update_domain(TimeMapEntry::new(
                DomainId::new(1), 
                BlockHeight::new(100),
                BlockHash::new("abc123".to_string()),
                Timestamp::new(1000),
            ));
            
            // Create a log entry
            let mut entry = LogEntry::new_event(
                EventEntry {
                    event_name: "test_event".to_string(),
                    severity: EventSeverity::Info,
                    component: "test".to_string(),
                    details: serde_json::json!({}),
                    resources: None,
                    domains: None,
                }
            );
            
            // Attach time map information
            LogTimeMapIntegration::attach_time_map(&mut entry, &time_map).unwrap();
            
            // Check that the time map hash was added
            assert!(entry.metadata.contains_key("time_map_hash"));
            assert!(!entry.metadata.get("time_map_hash").unwrap().is_empty());
            
            // Check that the domains list was added
            assert!(entry.metadata.contains_key("time_map_domains"));
            assert_eq!(entry.metadata.get("time_map_domains").unwrap(), "1");
        }
        
        #[test]
        fn test_verify_time_map() {
            // Create a time map
            let mut time_map = TimeMap::new();
            
            // Add a domain entry
            time_map.update_domain(TimeMapEntry::new(
                DomainId::new(1), 
                BlockHeight::new(100),
                BlockHash::new("abc123".to_string()),
                Timestamp::new(1000),
            ));
            
            // Create a log entry and attach the time map
            let mut entry = LogEntry::new_event(
                EventEntry {
                    event_name: "test_event".to_string(),
                    severity: EventSeverity::Info,
                    component: "test".to_string(),
                    details: serde_json::json!({}),
                    resources: None,
                    domains: None,
                }
            );
            
            LogTimeMapIntegration::attach_time_map(&mut entry, &time_map).unwrap();
            
            // Verify the time map hash
            let result = LogTimeMapIntegration::verify_time_map(&entry, &time_map).unwrap();
            assert!(result);
            
            // Modify the time map
            time_map.update_domain(TimeMapEntry::new(
                DomainId::new(1), 
                BlockHeight::new(101), // Changed height
                BlockHash::new("abc123".to_string()),
                Timestamp::new(1000),
            ));
            
            // Verification should fail with the modified time map
            let result2 = LogTimeMapIntegration::verify_time_map(&entry, &time_map).unwrap();
            assert!(!result2);
        }
    }
} 