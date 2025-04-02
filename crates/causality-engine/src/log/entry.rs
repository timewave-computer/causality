// Entry type definitions for the log system
// Original file: src/log/entry.rs

// Entry types for the Causality Log system

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use causality_types::{DomainId, TraceId, Timestamp, ContentId};
use causality_error::{Result, Error};

/// Types of log entries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    /// Fact entry
    Fact,
    /// Effect entry
    Effect,
    /// Event entry
    Event,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::Fact => write!(f, "fact"),
            EntryType::Effect => write!(f, "effect"),
            EntryType::Event => write!(f, "event"),
        }
    }
}

/// A fact entry in the causality log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactEntry {
    /// The trace ID
    pub trace_id: TraceId,
    /// The timestamp when this fact was observed
    pub timestamp: Timestamp,
    /// The fact ID
    pub fact_id: String,
    /// The domain ID
    pub domain_id: DomainId,
    /// The resource ID, if any
    pub resource_id: Option<ContentId>,
    /// The fact type
    pub fact_type: crate::log::fact::FactType,
    /// The metadata for this fact
    pub metadata: crate::log::FactMetadata,
}

/// An effect entry in the causality log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectEntry {
    /// The trace ID
    pub trace_id: TraceId,
    /// The timestamp when this effect was executed
    pub timestamp: Timestamp,
    /// The effect ID
    pub effect_id: String,
    /// The domain ID
    pub domain_id: DomainId,
    /// The resource ID, if any
    pub resource_id: Option<ContentId>,
    /// The effect type
    pub effect_type: String,
    /// The parameters for this effect
    pub parameters: HashMap<String, String>,
    /// Whether the effect was successful
    pub success: bool,
    /// The error message, if any
    pub error: Option<String>,
}

/// Severity level for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Debug-level event
    Debug,
    /// Info-level event
    Info,
    /// Warning-level event
    Warning,
    /// Error-level event
    Error,
    /// Critical-level event
    Critical,
}

/// An event entry in the causality log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    /// The trace ID
    pub trace_id: TraceId,
    /// The timestamp when this event occurred
    pub timestamp: Timestamp,
    /// The event ID
    pub event_id: String,
    /// The domain ID
    pub domain_id: DomainId,
    /// The event name
    pub event_name: String,
    /// The event severity
    pub severity: EventSeverity,
    /// The component that generated the event
    pub component: String,
    /// The event details
    pub details: serde_json::Value,
}

/// Generic log entry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryData {
    /// Fact data
    Fact(FactEntry),
    /// Effect data
    Effect(EffectEntry),
    /// Event data
    Event(EventEntry),
}

/// A log entry in the causality system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEntry {
    /// Fact observation
    Fact(FactEntry),
    /// Effect execution
    Effect(EffectEntry),
    /// System event
    Event(EventEntry),
}

impl LogEntry {
    /// Get the trace ID
    pub fn trace_id(&self) -> &TraceId {
        match self {
            LogEntry::Fact(entry) => &entry.trace_id,
            LogEntry::Effect(entry) => &entry.trace_id,
            LogEntry::Event(entry) => &entry.trace_id,
        }
    }
    
    /// Get the timestamp
    pub fn timestamp(&self) -> &Timestamp {
        match self {
            LogEntry::Fact(entry) => &entry.timestamp,
            LogEntry::Effect(entry) => &entry.timestamp,
            LogEntry::Event(entry) => &entry.timestamp,
        }
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        match self {
            LogEntry::Fact(entry) => &entry.domain_id,
            LogEntry::Effect(entry) => &entry.domain_id,
            LogEntry::Event(entry) => &entry.domain_id,
        }
    }
    
    /// Get the resource ID if any
    pub fn resource_id(&self) -> Option<&ContentId> {
        match self {
            LogEntry::Fact(entry) => entry.resource_id.as_ref(),
            LogEntry::Effect(entry) => entry.resource_id.as_ref(),
            LogEntry::Event(_) => None,
        }
    }
    
    /// Get the entry type
    pub fn entry_type(&self) -> EntryType {
        match self {
            LogEntry::Fact(_) => EntryType::Fact,
            LogEntry::Effect(_) => EntryType::Effect,
            LogEntry::Event(_) => EntryType::Event,
        }
    }
    
    /// Get the entry data
    pub fn entry_data(&self) -> EntryData {
        match self {
            LogEntry::Fact(fact) => EntryData::Fact(fact.clone()),
            LogEntry::Effect(effect) => EntryData::Effect(effect.clone()),
            LogEntry::Event(event) => EntryData::Event(event.clone()),
        }
    }
    
    /// Get the ID of this entry
    pub fn id(&self) -> String {
        match self {
            LogEntry::Fact(fact) => fact.fact_id.clone(),
            LogEntry::Effect(effect) => effect.effect_id.clone(),
            LogEntry::Event(event) => event.event_id.clone(),
        }
    }
} 
