// Replay filtering for log entries
// Original file: src/log/replay/filter.rs

// Replay filtering module for Causality Unified Log System
//
// This module provides filtering capabilities for the replay engine to select
// which log entries to process.

use std::collections::HashSet;
use crate::log::entry::{LogEntry, EntryType, EntryData};
use causality_types::{ContentId, DomainId};
use chrono::{DateTime, Utc};

/// Filter for selecting which log entries to process during replay
#[derive(Debug, Clone)]
pub struct ReplayFilter {
    /// Start time for the replay (inclusive)
    pub start_time: Option<DateTime<Utc>>,
    /// End time for the replay (inclusive)
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by trace ID
    pub trace_id: Option<String>,
    /// Filter by entry type
    pub entry_types: Option<HashSet<EntryType>>,
    /// Filter by resource ID
    pub resources: Option<HashSet<ContentId>>,
    /// Filter by domain ID
    pub domains: Option<HashSet<DomainId>>,
}

impl ReplayFilter {
    /// Create a new replay filter
    pub fn new() -> Self {
        ReplayFilter {
            start_time: None,
            end_time: None,
            trace_id: None,
            entry_types: None,
            resources: None,
            domains: None,
        }
    }
    
    /// Set the start time for filtering
    pub fn with_start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }
    
    /// Set the end time for filtering
    pub fn with_end_time(mut self, time: DateTime<Utc>) -> Self {
        self.end_time = Some(time);
        self
    }
    
    /// Set the trace ID for filtering
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Set the entry types for filtering
    pub fn with_entry_types(mut self, entry_types: HashSet<EntryType>) -> Self {
        self.entry_types = Some(entry_types);
        self
    }
    
    /// Add an entry type for filtering
    pub fn add_entry_type(mut self, entry_type: EntryType) -> Self {
        let mut types = self.entry_types.unwrap_or_else(HashSet::new);
        types.insert(entry_type);
        self.entry_types = Some(types);
        self
    }
    
    /// Set the resources for filtering
    pub fn with_resources(mut self, resources: HashSet<ContentId>) -> Self {
        self.resources = Some(resources);
        self
    }
    
    /// Add a resource for filtering
    pub fn add_resource(mut self, resource: ContentId) -> Self {
        let mut resources = self.resources.unwrap_or_else(HashSet::new);
        resources.insert(resource);
        self.resources = Some(resources);
        self
    }
    
    /// Set the domains for filtering
    pub fn with_domains(mut self, domains: HashSet<DomainId>) -> Self {
        self.domains = Some(domains);
        self
    }
    
    /// Add a domain for filtering
    pub fn add_domain(mut self, domain: DomainId) -> Self {
        let mut domains = self.domains.unwrap_or_else(HashSet::new);
        domains.insert(domain);
        self.domains = Some(domains);
        self
    }
    
    /// Check if an entry matches this filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check time bounds
        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }
        
        // Check trace ID
        if let Some(trace_id) = &self.trace_id {
            if let Some(entry_trace_id) = &entry.trace_id {
                if trace_id != entry_trace_id {
                    return false;
                }
            } else {
                // Entry has no trace ID, but filter requires one
                return false;
            }
        }
        
        // Check entry type
        if let Some(types) = &self.entry_types {
            if !types.contains(&entry.entry_type) {
                return false;
            }
        }
        
        // Check resources
        if let Some(resource_ids) = &self.resources {
            // Get resources from the entry based on type
            let entry_resources = match &entry.data {
                EntryData::Effect(effect) => &effect.resources,
                EntryData::Fact(fact) => &fact.resources,
                EntryData::Event(event) => {
                    // Event might not have resources
                    if let Some(resources) = &event.resources {
                        resources
                    } else {
                        // No resources to match
                        return false;
                    }
                }
            };
            
            // Check if any resource matches
            let mut found = false;
            for resource in entry_resources {
                if resource_ids.contains(resource) {
                    found = true;
                    break;
                }
            }
            
            if !found {
                return false;
            }
        }
        
        // Check domains
        if let Some(domain_ids) = &self.domains {
            // Get domains from the entry based on type
            let entry_domains = match &entry.data {
                EntryData::Effect(effect) => {
                    if let Some(domains) = &effect.domains {
                        domains
                    } else {
                        // No domains to match
                        return false;
                    }
                },
                EntryData::Fact(fact) => {
                    if let Some(domains) = &fact.domains {
                        domains
                    } else {
                        // No domains to match
                        return false;
                    }
                },
                EntryData::Event(event) => {
                    if let Some(domains) = &event.domains {
                        domains
                    } else {
                        // No domains to match
                        return false;
                    }
                }
            };
            
            // Check if any domain matches
            let mut found = false;
            for domain in entry_domains {
                if domain_ids.contains(domain) {
                    found = true;
                    break;
                }
            }
            
            if !found {
                return false;
            }
        }
        
        // All checks passed
        true
    }
    
    /// Create an empty filter that matches everything
    pub fn allow_all() -> Self {
        Self::new()
    }
    
    /// Check if this filter would reject everything
    pub fn is_rejecting_all(&self) -> bool {
        // If we have empty collections for resources or domains, but they're required
        if let Some(resources) = &self.resources {
            if resources.is_empty() {
                return true;
            }
        }
        
        if let Some(domains) = &self.domains {
            if domains.is_empty() {
                return true;
            }
        }
        
        if let Some(types) = &self.entry_types {
            if types.is_empty() {
                return true;
            }
        }
        
        // Check for impossible time range
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            if start > end {
                return true;
            }
        }
        
        false
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
        
        let now = Utc::now();
        let resource_id = ContentId::new_from_bytes(vec![1]);
        let domain_id = DomainId::new("test_domain");
        
        let filter = filter
            .with_start_time(now)
            .with_end_time(now)
            .with_trace_id("test_trace".to_string())
            .add_resource(resource_id)
            .add_domain(domain_id)
            .add_entry_type(EntryType::Event);
        
        assert_eq!(filter.start_time.unwrap(), now);
        assert_eq!(filter.end_time.unwrap(), now);
        assert_eq!(filter.trace_id.unwrap(), "test_trace");
        assert!(filter.resources.unwrap().contains(&resource_id));
        assert!(filter.domains.unwrap().contains(&domain_id));
        assert!(filter.entry_types.unwrap().contains(&EntryType::Event));
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
                details: serde_json::Value::Null,
                resources: Some(vec![ContentId::new_from_bytes(vec![1])]),
                domains: Some(vec![DomainId::new("test_domain")]),
            }),
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
        assert!(time_filter.matches(&entry));
        
        let time_filter = ReplayFilter::new()
            .with_start_time(future);
        assert!(!time_filter.matches(&entry));
        
        // Test trace filter
        let trace_filter = ReplayFilter::new()
            .with_trace_id("test_trace".to_string());
        assert!(trace_filter.matches(&entry));
        
        let trace_filter = ReplayFilter::new()
            .with_trace_id("other_trace".to_string());
        assert!(!trace_filter.matches(&entry));
        
        // Test type filter
        let type_filter = ReplayFilter::new()
            .add_entry_type(EntryType::Event);
        assert!(type_filter.matches(&entry));
        
        let type_filter = ReplayFilter::new()
            .add_entry_type(EntryType::Fact);
        assert!(!type_filter.matches(&entry));
        
        // Test resource filter
        let resource_filter = ReplayFilter::new()
            .add_resource(ContentId::new_from_bytes(vec![1]));
        assert!(resource_filter.matches(&entry));
        
        let resource_filter = ReplayFilter::new()
            .add_resource(ContentId::new_from_bytes(vec![2]));
        assert!(!resource_filter.matches(&entry));
        
        // Test domain filter
        let domain_filter = ReplayFilter::new()
            .add_domain(DomainId::new("test_domain"));
        assert!(domain_filter.matches(&entry));
        
        let domain_filter = ReplayFilter::new()
            .add_domain(DomainId::new("other_domain"));
        assert!(!domain_filter.matches(&entry));
    }
} 
