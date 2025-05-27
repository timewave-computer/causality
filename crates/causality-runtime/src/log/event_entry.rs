// Event log entry implementation
// Original file: src/log/entry/event_entry.rs

// Event entry implementation for Causality Unified Log System
//
// This module provides the EventEntry struct for representing events in the log.

use std::fmt;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use borsh::{BorshSerialize, BorshDeserialize, io::Read, io::Write as BorshWrite};
use causality_types::{ContentId, DomainId};
use crate::log::types::BorshJsonValue; // Import BorshJsonValue

/// The severity level of an event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EventSeverity {
    /// Debug-level event (lowest importance)
    Debug,
    /// Informational event
    Info,
    /// Warning event
    Warning,
    /// Error event (high importance)
    Error,
    /// Critical event (highest importance)
    Critical,
}

impl fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventSeverity::Debug => write!(f, "Debug"),
            EventSeverity::Info => write!(f, "Info"),
            EventSeverity::Warning => write!(f, "Warning"),
            EventSeverity::Error => write!(f, "Error"),
            EventSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// An entry representing a system event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventEntry {
    /// The event name
    pub event_name: String,
    /// The event severity
    pub severity: EventSeverity,
    /// The component that generated this event
    pub component: String,
    /// The event details
    pub details: BorshJsonValue,
    /// Related resources, if any
    pub resources: Option<Vec<ContentId>>,
    /// Related domains, if any
    pub domains: Option<Vec<DomainId>>,
}

// Manual BorshSerialize implementation for EventEntry
impl BorshSerialize for EventEntry {
    fn serialize<W: BorshWrite>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.event_name, writer)?;
        BorshSerialize::serialize(&self.severity, writer)?;
        BorshSerialize::serialize(&self.component, writer)?;
        // Manually serialize the skipped field
        BorshSerialize::serialize(&self.details, writer)?;
        BorshSerialize::serialize(&self.resources, writer)?;
        BorshSerialize::serialize(&self.domains, writer)?;
        Ok(())
    }
}

// Manual BorshDeserialize implementation for EventEntry
impl BorshDeserialize for EventEntry {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let event_name = BorshDeserialize::deserialize_reader(reader)?;
        let severity = BorshDeserialize::deserialize_reader(reader)?;
        let component = BorshDeserialize::deserialize_reader(reader)?;
        // Manually deserialize the skipped field
        let details = BorshDeserialize::deserialize_reader(reader)?;
        let resources = BorshDeserialize::deserialize_reader(reader)?;
        let domains = BorshDeserialize::deserialize_reader(reader)?;
        Ok(Self {
            event_name,
            severity,
            component,
            details,
            resources,
            domains,
        })
    }
}

impl EventEntry {
    /// Create a new event entry
    pub fn new(
        event_name: String,
        severity: EventSeverity,
        component: String,
        details: BorshJsonValue,
        resources: Option<Vec<ContentId>>,
        domains: Option<Vec<DomainId>>,
    ) -> Self {
        Self {
            event_name,
            severity,
            component,
            details,
            resources,
            domains,
        }
    }
    
    /// Get the event name
    pub fn event_name(&self) -> &str {
        &self.event_name
    }
    
    /// Get the event severity
    pub fn severity(&self) -> &EventSeverity {
        &self.severity
    }
    
    /// Get the component that generated this event
    pub fn component(&self) -> &str {
        &self.component
    }
    
    /// Get the event details
    pub fn details(&self) -> &BorshJsonValue {
        &self.details
    }
    
    /// Get the related resources, if any
    pub fn resources(&self) -> Option<&[ContentId]> {
        self.resources.as_deref()
    }
    
    /// Get the related domains, if any
    pub fn domains(&self) -> Option<&[DomainId]> {
        self.domains.as_deref()
    }
    
    /// Create a debug event
    pub fn debug(
        component: impl Into<String>,
        event_name: impl Into<String>,
        details: BorshJsonValue,
    ) -> Self {
        Self::new(
            event_name.into(),
            EventSeverity::Debug,
            component.into(),
            details,
            None,
            None,
        )
    }
    
    /// Create an info event
    pub fn info(
        component: impl Into<String>,
        event_name: impl Into<String>,
        details: BorshJsonValue,
    ) -> Self {
        Self::new(
            event_name.into(),
            EventSeverity::Info,
            component.into(),
            details,
            None,
            None,
        )
    }
    
    /// Create a warning event
    pub fn warning(
        component: impl Into<String>,
        event_name: impl Into<String>,
        details: BorshJsonValue,
    ) -> Self {
        Self::new(
            event_name.into(),
            EventSeverity::Warning,
            component.into(),
            details,
            None,
            None,
        )
    }
    
    /// Create an error event
    pub fn error(
        component: impl Into<String>,
        event_name: impl Into<String>,
        details: BorshJsonValue,
    ) -> Self {
        Self::new(
            event_name.into(),
            EventSeverity::Error,
            component.into(),
            details,
            None,
            None,
        )
    }
    
    /// Create a critical event
    pub fn critical(
        component: impl Into<String>,
        event_name: impl Into<String>,
        details: BorshJsonValue,
    ) -> Self {
        Self::new(
            event_name.into(),
            EventSeverity::Critical,
            component.into(),
            details,
            None,
            None,
        )
    }
    
    /// Add resources to this event
    pub fn with_resources(mut self, resources: Vec<ContentId>) -> Self {
        self.resources = Some(resources);
        self
    }
    
    /// Add domains to this event
    pub fn with_domains(mut self, domains: Vec<DomainId>) -> Self {
        self.domains = Some(domains);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_entry_creation() {
        let event_name = "test_event".to_string();
        let component = "test".to_string();
        let details = BorshJsonValue(serde_json::json!({"message": "test message"}));
        let resources = Some(vec![ContentId::new("resource-id-1")]);  // Use string ID
        let domains = Some(vec![DomainId::new("domain-id-1")]);  // Use string ID
        
        let entry = EventEntry::new(
            event_name.clone(),
            EventSeverity::Info,
            component.clone(),
            details.clone(),
            resources.clone(),
            domains.clone(),
        );
        
        assert_eq!(entry.event_name, event_name);
        assert_eq!(entry.severity, EventSeverity::Info);
        assert_eq!(entry.component, component);
        assert_eq!(entry.details.0, details.0);
        assert_eq!(entry.resources, resources);
        assert_eq!(entry.domains, domains);
        
        assert_eq!(entry.event_name(), event_name);
        assert_eq!(entry.severity(), &EventSeverity::Info);
        assert_eq!(entry.component(), component);
        assert_eq!(entry.details().0, details.0);
        
        let debug = EventEntry::debug("test", "debug_event", BorshJsonValue(serde_json::json!({})))
            .with_resources(vec![ContentId::new("resource-id-1")]);  // Use string ID
        assert_eq!(debug.severity, EventSeverity::Debug);
        assert_eq!(debug.component, "test");
        assert_eq!(debug.event_name, "debug_event");
        assert_eq!(debug.resources.unwrap()[0], ContentId::new("resource-id-1"));  // Use string ID
        
        let info = EventEntry::info("test", "info_event", BorshJsonValue(serde_json::json!({})))
            .with_domains(vec![DomainId::new("domain-id-1")]);  // Use string ID
        assert_eq!(info.severity, EventSeverity::Info);
        assert_eq!(info.component, "test");
        assert_eq!(info.event_name, "info_event");
        assert_eq!(info.domains.unwrap()[0], DomainId::new("domain-id-1"));  // Use string ID
        
        let warning = EventEntry::warning("test", "warning_event", BorshJsonValue(serde_json::json!({})));
        assert_eq!(warning.severity, EventSeverity::Warning);
        
        let error = EventEntry::error("test", "error_event", BorshJsonValue(serde_json::json!({})));
        assert_eq!(error.severity, EventSeverity::Error);
        
        let critical = EventEntry::critical("test", "critical_event", BorshJsonValue(serde_json::json!({})));
        assert_eq!(critical.severity, EventSeverity::Critical);
    }
} 
