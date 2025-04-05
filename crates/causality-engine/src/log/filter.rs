// Log filtering functionality
// Allows selective retrieval of log entries based on various criteria

use std::collections::HashSet;
use causality_types::{ContentId, DomainId, Timestamp, TraceId};
use crate::log::types::{LogEntry, EntryType, EntryData};

/// Filter for log entries
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Filter by entry types
    pub entry_types: Option<HashSet<EntryType>>,
    /// Filter by resource IDs
    pub resource_ids: Option<HashSet<ContentId>>,
    /// Filter by domain IDs
    pub domain_ids: Option<HashSet<DomainId>>,
    /// Filter by trace IDs
    pub trace_ids: Option<HashSet<TraceId>>,
    /// Filter by start time
    pub start_time: Option<Timestamp>,
    /// Filter by end time
    pub end_time: Option<Timestamp>,
    /// Maximum number of entries to return
    pub limit: Option<usize>,
}

impl LogFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an entry type to the filter
    pub fn with_entry_type(mut self, entry_type: EntryType) -> Self {
        let entry_types = self.entry_types.get_or_insert_with(HashSet::new);
        entry_types.insert(entry_type);
        self
    }
    
    /// Add a resource ID to the filter
    pub fn with_resource_id(mut self, resource_id: ContentId) -> Self {
        let resource_ids = self.resource_ids.get_or_insert_with(HashSet::new);
        resource_ids.insert(resource_id);
        self
    }
    
    /// Add a domain ID to the filter
    pub fn with_domain_id(mut self, domain_id: DomainId) -> Self {
        let domain_ids = self.domain_ids.get_or_insert_with(HashSet::new);
        domain_ids.insert(domain_id);
        self
    }
    
    /// Add a trace ID to the filter
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        let trace_ids = self.trace_ids.get_or_insert_with(HashSet::new);
        trace_ids.insert(trace_id);
        self
    }
    
    /// Set the time range for the filter
    pub fn with_time_range(mut self, start: Timestamp, end: Timestamp) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }
    
    /// Set the maximum number of entries to return
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Check if an entry matches this filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check entry type
        if let Some(entry_types) = &self.entry_types {
            if !entry_types.contains(&entry.entry_type) {
                return false;
            }
        }
        
        // Check resource ID if available
        if let Some(resource_ids) = &self.resource_ids {
            let has_resource = match &entry.data {
                EntryData::Fact(fact) => fact.resources.as_ref().map(|resources| 
                    resources.iter().any(|id| resource_ids.contains(id))).unwrap_or(false),
                EntryData::Effect(effect) => effect.resources.as_ref().map(|resources| 
                    resources.iter().any(|id| resource_ids.contains(id))).unwrap_or(false),
                EntryData::Operation(op) => op.resources.as_ref().map(|resources| 
                    resources.iter().any(|id| resource_ids.contains(id))).unwrap_or(false),
                _ => false,
            };
            
            if !has_resource {
                return false;
            }
        }
        
        // Check domain ID if available
        if let Some(domain_ids) = &self.domain_ids {
            let has_domain = match &entry.data {
                EntryData::Fact(fact) => domain_ids.contains(&fact.domain_id),
                EntryData::Effect(effect) => effect.domains.as_ref().map(|domains| 
                    domains.iter().any(|id| domain_ids.contains(id))).unwrap_or(false),
                EntryData::Operation(op) => op.domains.as_ref().map(|domains| 
                    domains.iter().any(|id| domain_ids.contains(id))).unwrap_or(false),
                _ => false,
            };
            
            if !has_domain {
                return false;
            }
        }
        
        // Check trace ID
        if let Some(trace_ids) = &self.trace_ids {
            if let Some(trace_id) = &entry.trace_id {
                if !trace_ids.contains(trace_id) {
                    return false;
                }
            } else {
                // Entry has no trace ID but filter requires one
                return false;
            }
        }
        
        // Check time range
        if let Some(start) = &self.start_time {
            if &entry.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = &self.end_time {
            if &entry.timestamp > end {
                return false;
            }
        }
        
        // All filters passed
        true
    }
    
    /// Apply this filter to a collection of entries
    pub fn apply(&self, entries: &[LogEntry]) -> Vec<LogEntry> {
        let mut result: Vec<LogEntry> = entries.iter()
            .filter(|entry| self.matches(entry))
            .cloned()
            .collect();
        
        // Apply limit if specified
        if let Some(limit) = self.limit {
            if result.len() > limit {
                result.truncate(limit);
            }
        }
        
        result
    }
} 