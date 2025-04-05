// Replay filtering for log entries
// Original file: src/log/replay/filter.rs

// Replay filtering module for Causality Unified Log System
//
// This module provides filtering capabilities for the replay engine to select
// which log entries to process.

use crate::log::types::{LogEntry, EntryType, EntryData};
use causality_types::{ContentId, DomainId};
use chrono::{DateTime, Utc};

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
    
    /// Check if an entry should be included based on this filter
    pub fn should_include(&self, entry: &LogEntry) -> bool {
        // Check time range
        if !self.should_include_time_range(entry) {
            return false;
        }
        
        // Check trace ID
        if !self.should_include_trace_id(entry) {
            return false;
        }
        
        // Check entry type
        if !self.entry_types.is_empty() && !self.entry_types.contains(&entry.entry_type) {
            return false;
        }
        
        // Check resources
        if !self.resources.is_empty() {
            let entry_resources = match &entry.data {
                EntryData::Fact(fact) => fact.resources.as_ref(),
                EntryData::Effect(effect) => effect.resources.as_ref(),
                EntryData::Event(event) => event.resources.as_ref(),
                EntryData::Operation(op) => op.resources.as_ref(),
                EntryData::SystemEvent(_) => None,
                EntryData::Custom(_) => None,
            };
            
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
            let entry_domains = match &entry.data {
                EntryData::Fact(fact) => fact.domains.as_ref(),
                EntryData::Effect(effect) => effect.domains.as_ref(),
                EntryData::Event(event) => event.domains.as_ref(),
                EntryData::Operation(op) => op.domains.as_ref(),
                EntryData::SystemEvent(_) => None,
                EntryData::Custom(_) => None,
            };
            
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
        if let Some(trace_id) = &self.trace_id {
            if let Some(entry_trace_id) = &entry.trace_id {
                let entry_trace_id_str = entry_trace_id.as_str();
                if trace_id != entry_trace_id_str {
                    return false;
                }
            } else {
                return false;
            }
        }
        
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
    use causality_types::ContentId;
    use crate::log::types::{EntryData, SystemEventEntry, OperationEntry};
    use crate::log::{EventEntry, EventSeverity};
    
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
        let resource_id = ContentId::new_from_bytes(vec![1]);
        let domain_id = DomainId::new("test_domain");
        
        let filter = filter
            .with_start_time(now)
            .with_end_time(now)
            .with_trace_id("test_trace")
            .with_resource(resource_id)
            .with_domain(domain_id)
            .with_entry_type(EntryType::Event);
        
        assert_eq!(filter.start_time.unwrap(), now);
        assert_eq!(filter.end_time.unwrap(), now);
        assert_eq!(filter.trace_id.unwrap(), "test_trace");
        assert!(filter.resources.contains(&resource_id));
        assert!(filter.domains.contains(&domain_id));
        assert!(filter.entry_types.contains(&EntryType::Event));
    }
    
    #[test]
    fn test_filter_matching() {
        // Create test entries
        let event_entry = LogEntry {
            id: "entry_1".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: "test_event".to_string(),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::Value::Null,
                resources: Some(vec![ContentId::new_from_bytes(vec![1])]),
                domains: Some(vec![DomainId::new("test_domain")]),
            }),
            trace_id: Some("test_trace".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        let system_event_entry = LogEntry {
            id: "entry_2".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::SystemEvent,
            data: EntryData::SystemEvent(SystemEventEntry {
                event_type: "test_event".to_string(),
                source: "test".to_string(),
                data: serde_json::Value::Null,
            }),
            trace_id: Some("test_trace".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        let operation_entry = LogEntry {
            id: "entry_3".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Operation,
            data: EntryData::Operation(OperationEntry {
                operation_id: "test_op".to_string(),
                operation_type: "test".to_string(),
                resources: Some(vec![ContentId::new_from_bytes(vec![1])]),
                domains: Some(vec![DomainId::new("test_domain")]),
                status: "test".to_string(),
                input: serde_json::Value::Null,
                output: None,
                error: None,
            }),
            trace_id: Some("test_trace".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        let custom_entry = LogEntry {
            id: "entry_4".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Custom("test".to_string()),
            data: EntryData::Custom(serde_json::Value::Null),
            trace_id: Some("test_trace".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        // Test time filter
        let past = Utc::now() - chrono::Duration::days(1);
        let future = Utc::now() + chrono::Duration::days(1);
        
        let time_filter = ReplayFilter::new()
            .with_start_time(past)
            .with_end_time(future);
        assert!(time_filter.should_include(&event_entry));
        assert!(time_filter.should_include(&system_event_entry));
        assert!(time_filter.should_include(&operation_entry));
        assert!(time_filter.should_include(&custom_entry));
        
        let time_filter = ReplayFilter::new()
            .with_start_time(future);
        assert!(!time_filter.should_include(&event_entry));
        assert!(!time_filter.should_include(&system_event_entry));
        assert!(!time_filter.should_include(&operation_entry));
        assert!(!time_filter.should_include(&custom_entry));
        
        // Test trace filter
        let trace_filter = ReplayFilter::new()
            .with_trace_id("test_trace");
        assert!(trace_filter.should_include(&event_entry));
        assert!(trace_filter.should_include(&system_event_entry));
        assert!(trace_filter.should_include(&operation_entry));
        assert!(trace_filter.should_include(&custom_entry));
        
        let trace_filter = ReplayFilter::new()
            .with_trace_id("other_trace");
        assert!(!trace_filter.should_include(&event_entry));
        assert!(!trace_filter.should_include(&system_event_entry));
        assert!(!trace_filter.should_include(&operation_entry));
        assert!(!trace_filter.should_include(&custom_entry));
        
        // Test type filter
        let type_filter = ReplayFilter::new()
            .with_entry_type(EntryType::Event);
        assert!(type_filter.should_include(&event_entry));
        assert!(!type_filter.should_include(&system_event_entry));
        assert!(!type_filter.should_include(&operation_entry));
        assert!(!type_filter.should_include(&custom_entry));
        
        let type_filter = ReplayFilter::new()
            .with_entry_type(EntryType::SystemEvent);
        assert!(!type_filter.should_include(&event_entry));
        assert!(type_filter.should_include(&system_event_entry));
        assert!(!type_filter.should_include(&operation_entry));
        assert!(!type_filter.should_include(&custom_entry));
        
        let type_filter = ReplayFilter::new()
            .with_entry_type(EntryType::Operation);
        assert!(!type_filter.should_include(&event_entry));
        assert!(!type_filter.should_include(&system_event_entry));
        assert!(type_filter.should_include(&operation_entry));
        assert!(!type_filter.should_include(&custom_entry));
        
        let type_filter = ReplayFilter::new()
            .with_entry_type(EntryType::Custom("test".to_string()));
        assert!(!type_filter.should_include(&event_entry));
        assert!(!type_filter.should_include(&system_event_entry));
        assert!(!type_filter.should_include(&operation_entry));
        assert!(type_filter.should_include(&custom_entry));
        
        // Test resource filter
        let resource_filter = ReplayFilter::new()
            .with_resource(ContentId::new_from_bytes(vec![1]));
        assert!(resource_filter.should_include(&event_entry));
        assert!(!resource_filter.should_include(&system_event_entry));
        assert!(resource_filter.should_include(&operation_entry));
        assert!(!resource_filter.should_include(&custom_entry));
        
        let resource_filter = ReplayFilter::new()
            .with_resource(ContentId::new_from_bytes(vec![2]));
        assert!(!resource_filter.should_include(&event_entry));
        assert!(!resource_filter.should_include(&system_event_entry));
        assert!(!resource_filter.should_include(&operation_entry));
        assert!(!resource_filter.should_include(&custom_entry));
        
        // Test domain filter
        let domain_filter = ReplayFilter::new()
            .with_domain(DomainId::new("test_domain"));
        assert!(domain_filter.should_include(&event_entry));
        assert!(!domain_filter.should_include(&system_event_entry));
        assert!(domain_filter.should_include(&operation_entry));
        assert!(!domain_filter.should_include(&custom_entry));
        
        let domain_filter = ReplayFilter::new()
            .with_domain(DomainId::new("other_domain"));
        assert!(!domain_filter.should_include(&event_entry));
        assert!(!domain_filter.should_include(&system_event_entry));
        assert!(!domain_filter.should_include(&operation_entry));
        assert!(!domain_filter.should_include(&custom_entry));
    }
} 

