// Replay filtering for log entries
// Original file: src/log/replay/filter.rs

// Replay filtering module for Causality Unified Log System
//
// This module provides filtering capabilities for the replay engine to select
// which log entries to process.

use crate::log::types::{LogEntry, EntryType, EntryData, BorshJsonValue};
use causality_types::{ContentId, DomainId, Timestamp, TraceId};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::str::FromStr;

/// Time filter types
#[derive(Debug, Clone)]
pub enum TimeFilter {
    /// After a specific time
    After(DateTime<Utc>),
    /// Before a specific time
    Before(DateTime<Utc>),
    /// Between two times (inclusive)
    Between(DateTime<Utc>, DateTime<Utc>),
}

/// Filter for replay entries
#[derive(Debug, Clone)]
pub struct ReplayFilter {
    /// Start time filter
    pub start_time: Option<DateTime<Utc>>,
    /// End time filter
    pub end_time: Option<DateTime<Utc>>,
    /// Trace ID filter
    pub trace_id: Option<String>,
    /// Entry types to include
    pub entry_types: Vec<EntryType>,
    /// Resource IDs to include
    pub resources: Vec<ContentId>,
    /// Domain IDs to include
    pub domains: Vec<DomainId>,
    /// Time filter type
    pub time_filter: Option<TimeFilter>,
    /// Trace filter type
    pub trace_filter: Option<String>,
}

impl ReplayFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        ReplayFilter {
            start_time: None,
            end_time: None,
            trace_id: None,
            entry_types: Vec::new(),
            resources: Vec::new(),
            domains: Vec::new(),
            time_filter: None,
            trace_filter: None,
        }
    }
    
    /// Set the start time filter
    pub fn with_start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = Some(start_time);
        if let Some(end_time) = self.end_time {
            self.time_filter = Some(TimeFilter::Between(start_time, end_time));
        } else {
            self.time_filter = Some(TimeFilter::After(start_time));
        }
        self
    }
    
    /// Set the end time filter
    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = Some(end_time);
        if let Some(start_time) = self.start_time {
            self.time_filter = Some(TimeFilter::Between(start_time, end_time));
        } else {
            self.time_filter = Some(TimeFilter::Before(end_time));
        }
        self
    }
    
    /// Set the trace ID filter
    pub fn with_trace_id(mut self, trace_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self.trace_filter = Some(trace_id.to_string());
        self
    }
    
    /// Add an entry type to the filter
    pub fn with_entry_type(mut self, entry_type: EntryType) -> Self {
        self.entry_types.push(entry_type);
        self
    }
    
    /// Add a resource ID to the filter
    pub fn with_resource(mut self, resource_id: ContentId) -> Self {
        self.resources.push(resource_id);
        self
    }
    
    /// Add a domain ID to the filter
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domains.push(domain_id);
        self
    }
    
    /// Check if a log entry should be included based on the filter criteria.
    pub fn should_include(&self, entry: &LogEntry) -> bool {
        // Check basic time and trace ID first
        if !self.should_include_time_range(entry) || !self.should_include_trace_id(entry) {
            return false;
        }
        
        // Check entry type
        if !self.entry_types.is_empty() && !self.entry_types.contains(&entry.entry_type) {
            // TODO: This check might fail if EntryType cannot derive Eq/Hash
            // return false;
        }
        
        // Check resources
        if !self.resources.is_empty() {
            let entry_resources = get_entry_resources(entry);
            
            if let Some(resources) = entry_resources {
                if !resources.iter().any(|r| self.resources.contains(r)) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check domains
        if !self.domains.is_empty() {
            let entry_domains = get_entry_domains(entry);
            
            if let Some(domains) = entry_domains {
                if !domains.iter().any(|d| self.domains.contains(d)) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }

    /// Check if an entry should be included based on a time range
    fn should_include_time_range(&self, entry: &LogEntry) -> bool {
        if let Some(start_time) = self.start_time {
            let entry_time = entry.timestamp.to_datetime();
            if entry_time < start_time {
                return false;
            }
        }
        
        if let Some(end_time) = self.end_time {
            let entry_time = entry.timestamp.to_datetime();
            if entry_time > end_time {
                return false;
            }
        }
        
        true
    }
    
    /// Check if an entry should be included based on a trace ID
    fn should_include_trace_id(&self, entry: &LogEntry) -> bool {
        if let Some(trace_id_filter_str) = &self.trace_id {
            if !trace_id_filter_str.is_empty() {
                // Create a TraceId from the filter string
                let filter_trace_id = TraceId::from_str(trace_id_filter_str);
                // Check if entry has a TraceId and if it matches the filter
            if let Some(entry_trace_id) = &entry.trace_id {
                    return entry_trace_id.0 == filter_trace_id.0;
                } else {
                    return false; // No trace ID in entry
                }
            }
            // Empty filter string passes
        }
        // No trace filter set, include the entry
        true
    }
}

impl Default for ReplayFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::log::types::{FactEntry, EffectEntry, ResourceAccessEntry, SystemEventEntry, OperationEntry, BorshJsonValue};
    use crate::log::{EventEntry, EventSeverity};
    use serde_json::json;
    
    fn test_entry(id: &str, timestamp: u64, entry_type: EntryType, trace_id_str: Option<&str>) -> LogEntry {
        // Create trace_id using the constructor
        let trace_id = trace_id_str.map(TraceId::from_str);
        
        LogEntry {
            id: id.to_string(),
            timestamp: Timestamp::from_millis(timestamp),
            entry_type,
            data: EntryData::SystemEvent(SystemEventEntry {
                 event_type: "test_event".to_string(),
                 data: BorshJsonValue(serde_json::json!({ "test": true })), // Use data field
                 resources: Vec::new(), // Provide default empty Vec
                 domains: Vec::new(),   // Provide default empty Vec
            }),
            trace_id, // Use the parsed Option<TraceId>
            parent_id: None,
            metadata: HashMap::new(),
        }
    }
    
    fn test_op_entry(id: &str, timestamp: u64, domains: Option<Vec<DomainId>>, resources: Option<Vec<ContentId>>, trace_id_str: Option<&str>) -> LogEntry {
         // Create trace_id using the constructor
        let trace_id = trace_id_str.map(TraceId::from_str);

        LogEntry {
            id: id.to_string(),
            timestamp: Timestamp::from_millis(timestamp),
            entry_type: EntryType::Operation,
            data: EntryData::Operation(OperationEntry {
                 operation_id: id.to_string(), // Use id for operation_id
                 operation_type: "test_op".to_string(),
                 status: "Started".to_string(), // Use String for status
                 details: BorshJsonValue(json!({})), // Default details
                 resources: resources.unwrap_or_default(), // Handle Option
                 domains: domains.unwrap_or_default(),   // Handle Option
            }),
             trace_id, // Use the parsed Option<TraceId>
             parent_id: None,
             metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_filter_creation() {
        let filter = ReplayFilter::new();
        assert!(filter.start_time.is_none());
        assert!(filter.end_time.is_none());
        assert!(filter.trace_id.is_none());
        assert!(filter.resources.is_empty());
        assert!(filter.domains.is_empty());
        assert!(filter.entry_types.is_empty());
        
        let now = Utc::now();
        let now_ts = Timestamp::now();
        let resource_id = ContentId::from_str("test-resource-id-1").unwrap(); // Keep unwrap in test setup
        let domain_id = DomainId::new("test_domain");
        let trace_id_str = "test_trace";
        // let trace_id = TraceId::from_str(trace_id_str).unwrap(); // No need for this parsed variable
        
        let filter = ReplayFilter::new()
            .with_start_time(now)
            .with_end_time(now)
            .with_trace_id(trace_id_str)
            .with_resource(resource_id.clone())
            .with_domain(domain_id.clone())
            .with_entry_type(EntryType::Custom("Event".to_string()));
        
        assert_eq!(filter.start_time.unwrap(), now);
        assert_eq!(filter.end_time.unwrap(), now);
        assert_eq!(filter.trace_id.unwrap(), trace_id_str);
        assert!(filter.resources.contains(&resource_id));
        assert!(filter.domains.contains(&domain_id));
        assert!(filter.entry_types.contains(&EntryType::Custom("Event".to_string())));
    }
    
    #[test]
    fn test_trace_id_match() {
        let trace_id_str = "trace-123";
        let filter = ReplayFilter::new().with_trace_id(trace_id_str);
        
        let entry_match = test_entry("e1", 100, EntryType::Fact, Some(trace_id_str));
        let entry_mismatch = test_entry("e2", 101, EntryType::Effect, Some("trace-456"));
        let entry_no_trace = test_entry("e3", 102, EntryType::SystemEvent, None);
        
        assert!(filter.should_include(&entry_match));
        assert!(!filter.should_include(&entry_mismatch));
        assert!(!filter.should_include(&entry_no_trace)); // No trace ID means no match
    }
    
    #[test]
    fn test_filter_without_trace_id() {
        let filter = ReplayFilter::new();
        let entry_match = test_entry("e1", 100, EntryType::Fact, None);
        let entry_mismatch = test_entry("e2", 101, EntryType::Effect, None);
        let entry_no_trace = test_entry("e3", 102, EntryType::SystemEvent, None);
        
        assert!(filter.should_include(&entry_match));
        assert!(!filter.should_include(&entry_mismatch));
        assert!(!filter.should_include(&entry_no_trace)); // No trace ID means no match
    }
}

/// Check if a log entry matches the replay filter criteria.
pub fn entry_matches_filter(entry: &LogEntry, filter: &ReplayFilter) -> bool {
    // Check timestamp range
    if let Some(start_dt) = filter.start_time {
        let start_ts = Timestamp::from_datetime(&start_dt);
        if entry.timestamp < start_ts {
            return false;
        }
    }
    if let Some(end_dt) = filter.end_time {
        let end_ts = Timestamp::from_datetime(&end_dt);
        if entry.timestamp > end_ts {
            return false;
        }
    }

    // Check entry types
    if !filter.entry_types.is_empty() {
        if !filter.entry_types.contains(&entry.entry_type) {
            return false;
        }
    }

    // Check resource IDs
    if !filter.resources.is_empty() {
        let entry_resources = get_entry_resources(entry);
        if entry_resources.map_or(true, |res_set| !res_set.iter().any(|r| filter.resources.contains(r))) {
            return false;
        }
    }

    // Check domain IDs
    if !filter.domains.is_empty() {
        let entry_domains = get_entry_domains(entry);
        if entry_domains.map_or(true, |dom_set| !dom_set.iter().any(|d| filter.domains.contains(d))) {
            return false;
        }
    }

    // Check trace ID
    if let Some(trace_id_filter_str) = &filter.trace_id {
        if let Some(entry_trace_id) = &entry.trace_id {
            // Simple string comparison - TraceId is just a wrapper around String
            if trace_id_filter_str != &entry_trace_id.0 {
                return false;
            }
        } else {
            // Entry has no trace ID but filter requires one
            return false;
        }
    }

    // All checks passed
    true
}

/// Helper to extract resource IDs associated with a log entry.
fn get_entry_resources(entry: &LogEntry) -> Option<HashSet<ContentId>> {
    match &entry.data {
        EntryData::Fact(fact) => Some(HashSet::from_iter(fact.resources.iter().cloned())), // Return Some(HashSet)
        EntryData::Effect(effect) => Some(HashSet::from_iter(effect.resources.iter().cloned())), // Return Some(HashSet)
        EntryData::ResourceAccess(ra) => {
            // Parse the ID and return Some(HashSet) containing it if successful
            ContentId::from_str(&ra.resource_id).ok().map(|cid| HashSet::from([cid])) // Already returns Option<HashSet>
        },
        EntryData::Operation(op) => Some(HashSet::from_iter(op.resources.iter().cloned())), // Return Some(HashSet)
        EntryData::SystemEvent(_) => None, // No resources
        EntryData::Event(event_entry) => {
             // Convert Option<Vec<ContentId>> to Option<HashSet<ContentId>>
            event_entry.resources.as_ref().map(|vec| vec.iter().cloned().collect::<HashSet<ContentId>>())
        },
        EntryData::Custom(_, _) => None, // No resources
    }
}

/// Helper to extract domain IDs associated with a log entry.
fn get_entry_domains(entry: &LogEntry) -> Option<HashSet<DomainId>> {
     match &entry.data {
        EntryData::Fact(fact) => Some(HashSet::from([fact.domain_id.clone()])), // Use domain_id directly
        EntryData::Effect(effect) => Some(HashSet::from([effect.domain_id.clone()])), // Use domain_id directly
        EntryData::ResourceAccess(_) => None, // ResourceAccess doesn't have domains
        EntryData::Operation(op) => None, // TODO: Revisit - op.domains.clone(),
        EntryData::SystemEvent(_) => None,
        EntryData::Event(event_entry) => event_entry.domains.clone().map(|v| v.into_iter().collect::<HashSet<_>>()),
        EntryData::Custom(_, _) => None, // Fixed pattern match
    }
} 

