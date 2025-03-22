// Replay filter implementation for Causality Unified Log System
//
// This module provides filtering capabilities for log replay.

use std::collections::HashSet;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::types::{ResourceId, DomainId};
use crate::log::entry::{LogEntry, EntryType};

/// A filter for log replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFilter {
    /// The start time for replay
    pub start_time: Option<DateTime<Utc>>,
    /// The end time for replay
    pub end_time: Option<DateTime<Utc>>,
    /// The trace ID to filter by
    pub trace_id: Option<String>,
    /// The resources to include
    pub resources: Option<HashSet<ResourceId>>,
    /// The domains to include
    pub domains: Option<HashSet<DomainId>>,
    /// The entry types to include
    pub entry_types: Option<HashSet<EntryType>>,
    /// The maximum number of entries to process
    pub max_entries: Option<usize>,
}

impl ReplayFilter {
    /// Create a new replay filter with default values
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            trace_id: None,
            resources: None,
            domains: None,
            entry_types: None,
            max_entries: None,
        }
    }
    
    /// Set the start time for replay
    pub fn with_start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }
    
    /// Set the end time for replay
    pub fn with_end_time(mut self, time: DateTime<Utc>) -> Self {
        self.end_time = Some(time);
        self
    }
    
    /// Set the trace ID to filter by
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }
    
    /// Add a resource to filter by
    pub fn with_resource(mut self, resource_id: ResourceId) -> Self {
        let resources = self.resources.get_or_insert_with(HashSet::new);
        resources.insert(resource_id);
        self
    }
    
    /// Add a domain to filter by
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        let domains = self.domains.get_or_insert_with(HashSet::new);
        domains.insert(domain_id);
        self
    }
    
    /// Add an entry type to filter by
    pub fn with_entry_type(mut self, entry_type: EntryType) -> Self {
        let entry_types = self.entry_types.get_or_insert_with(HashSet::new);
        entry_types.insert(entry_type);
        self
    }
    
    /// Set the maximum number of entries to process
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = Some(max);
        self
    }
    
    /// Check if an entry matches this filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check time range
        if let Some(start_time) = self.start_time {
            if entry.timestamp < start_time {
                return false;
            }
        }
        
        if let Some(end_time) = self.end_time {
            if entry.timestamp > end_time {
                return false;
            }
        }
        
        // Check trace ID
        if let Some(trace_id) = &self.trace_id {
            if entry.trace_id.as_deref() != Some(trace_id) {
                return false;
            }
        }
        
        // Check entry type
        if let Some(entry_types) = &self.entry_types {
            if !entry_types.contains(&entry.entry_type) {
                return false;
            }
        }
        
        // Check resources
        if let Some(resources) = &self.resources {
            let entry_resources = match &entry.data {
                crate::log::entry::EntryData::Effect(effect) => &effect.resources,
                crate::log::entry::EntryData::Fact(fact) => &fact.resources,
                crate::log::entry::EntryData::Event(event) => {
                    if let Some(res) = &event.resources {
                        res
                    } else {
                        return false;
                    }
                }
            };
            
            if !entry_resources.iter().any(|r| resources.contains(r)) {
                return false;
            }
        }
        
        // Check domains
        if let Some(domains) = &self.domains {
            match &entry.data {
                crate::log::entry::EntryData::Effect(effect) => {
                    if !effect.domains.iter().any(|d| domains.contains(d)) {
                        return false;
                    }
                }
                crate::log::entry::EntryData::Fact(fact) => {
                    if !domains.contains(&fact.domain) {
                        return false;
                    }
                }
                crate::log::entry::EntryData::Event(event) => {
                    if let Some(dom) = &event.domains {
                        if !dom.iter().any(|d| domains.contains(d)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
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
    use crate::types::ResourceId;
    use crate::log::entry::{EntryData, EventEntry, EventSeverity};
    
    #[test]
    fn test_filter_creation() {
        let filter = ReplayFilter::new();
        assert!(filter.start_time.is_none());
        assert!(filter.end_time.is_none());
        assert!(filter.trace_id.is_none());
        assert!(filter.resources.is_none());
        assert!(filter.domains.is_none());
        assert!(filter.entry_types.is_none());
        assert!(filter.max_entries.is_none());
        
        let now = Utc::now();
        let resource_id = ResourceId::new(1);
        let domain_id = DomainId::new(1);
        
        let filter = filter
            .with_start_time(now)
            .with_end_time(now)
            .with_trace_id("test_trace")
            .with_resource(resource_id)
            .with_domain(domain_id)
            .with_entry_type(EntryType::Event)
            .with_max_entries(100);
        
        assert_eq!(filter.start_time.unwrap(), now);
        assert_eq!(filter.end_time.unwrap(), now);
        assert_eq!(filter.trace_id.unwrap(), "test_trace");
        assert!(filter.resources.unwrap().contains(&resource_id));
        assert!(filter.domains.unwrap().contains(&domain_id));
        assert!(filter.entry_types.unwrap().contains(&EntryType::Event));
        assert_eq!(filter.max_entries.unwrap(), 100);
    }
    
    #[test]
    fn test_filter_matching() {
        // Create test entry
        let entry = LogEntry {
            id: "entry_1".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: "test_event".to_string(),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({}),
                resources: Some(vec![ResourceId::new(1)]),
                domains: Some(vec![DomainId::new(1)]),
            }),
            trace_id: Some("test_trace".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
        };
        
        // Test time filter
        let past = Utc::now() - chrono::Duration::days(1);
        let future = Utc::now() + chrono::Duration::days(1);
        
        let time_filter = ReplayFilter::new()
            .with_start_time(past)
            .with_end_time(future);
        assert!(time_filter.matches(&entry));
        
        let time_filter = ReplayFilter::new()
            .with_start_time(future);
        assert!(!time_filter.matches(&entry));
        
        // Test trace filter
        let trace_filter = ReplayFilter::new()
            .with_trace_id("test_trace");
        assert!(trace_filter.matches(&entry));
        
        let trace_filter = ReplayFilter::new()
            .with_trace_id("other_trace");
        assert!(!trace_filter.matches(&entry));
        
        // Test type filter
        let type_filter = ReplayFilter::new()
            .with_entry_type(EntryType::Event);
        assert!(type_filter.matches(&entry));
        
        let type_filter = ReplayFilter::new()
            .with_entry_type(EntryType::Fact);
        assert!(!type_filter.matches(&entry));
        
        // Test resource filter
        let resource_filter = ReplayFilter::new()
            .with_resource(ResourceId::new(1));
        assert!(resource_filter.matches(&entry));
        
        let resource_filter = ReplayFilter::new()
            .with_resource(ResourceId::new(2));
        assert!(!resource_filter.matches(&entry));
        
        // Test domain filter
        let domain_filter = ReplayFilter::new()
            .with_domain(DomainId::new(1));
        assert!(domain_filter.matches(&entry));
        
        let domain_filter = ReplayFilter::new()
            .with_domain(DomainId::new(2));
        assert!(!domain_filter.matches(&entry));
    }
} 