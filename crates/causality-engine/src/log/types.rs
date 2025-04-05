// Log entry types
// This file defines the core types for log entries

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use causality_types::{TraceId, Timestamp, DomainId, ContentId};
use crate::log::event_entry::EventEntry;

/// Type of log entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EntryType {
    /// A fact entry
    Fact,
    /// An effect entry
    Effect,
    /// A system event entry
    SystemEvent,
    /// An operation entry
    Operation,
    /// An event entry
    Event,
    /// A custom entry type
    Custom(String),
}

/// Log entry data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryData {
    /// Fact data
    Fact(FactEntry),
    /// Effect data
    Effect(EffectEntry),
    /// System event data
    SystemEvent(SystemEventEntry),
    /// Operation data
    Operation(OperationEntry),
    /// Event data
    Event(EventEntry),
    /// Custom entry data (stored as JSON)
    Custom(serde_json::Value),
}

/// A log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique ID for this entry
    pub id: String,
    /// Timestamp of the entry
    pub timestamp: Timestamp,
    /// Type of entry
    pub entry_type: EntryType,
    /// Entry data
    pub data: EntryData,
    /// Trace ID for correlated entries
    pub trace_id: Option<TraceId>,
    /// Parent entry ID
    pub parent_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Hash of the entry
    pub entry_hash: Option<String>,
}

/// Fact entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactEntry {
    /// Fact ID
    pub fact_id: String,
    /// Fact type
    pub fact_type: String,
    /// Domain ID
    pub domain_id: DomainId,
    /// Height in the causal graph
    pub height: u64,
    /// Hash of the fact
    pub hash: String,
    /// Timestamp of the fact
    pub timestamp: Timestamp,
    /// Resources associated with this fact
    pub resources: Option<Vec<ContentId>>,
    /// Domains associated with this fact
    pub domains: Option<Vec<DomainId>>,
    /// Fact data
    pub data: serde_json::Value,
}

/// Effect entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectEntry {
    /// Effect type
    pub effect_type: String,
    /// Resources affected by this effect
    pub resources: Option<Vec<ContentId>>,
    /// Domains involved in this effect
    pub domains: Option<Vec<DomainId>>,
    /// Code hash (if any)
    pub code_hash: Option<String>,
    /// Parameters for the effect
    pub parameters: serde_json::Value,
    /// Effect result data
    pub result: Option<serde_json::Value>,
    /// Whether the effect was successful
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
}

/// System event entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEventEntry {
    /// Event type
    pub event_type: String,
    /// Source of the event
    pub source: String,
    /// Event data
    pub data: serde_json::Value,
}

/// Operation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEntry {
    /// Operation ID
    pub operation_id: String,
    /// Operation type
    pub operation_type: String,
    /// Resources involved in this operation
    pub resources: Option<Vec<ContentId>>,
    /// Domains involved in this operation
    pub domains: Option<Vec<DomainId>>,
    /// Operation status
    pub status: String,
    /// Input parameters
    pub input: serde_json::Value,
    /// Output result
    pub output: Option<serde_json::Value>,
    /// Error message (if any)
    pub error: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        id: String,
        entry_type: EntryType,
        data: EntryData,
    ) -> Self {
        LogEntry {
            id,
            timestamp: Timestamp::now(),
            entry_type,
            data,
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        }
    }
    
    /// Set the trace ID
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Set the parent ID
    pub fn with_parent_id(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
} 