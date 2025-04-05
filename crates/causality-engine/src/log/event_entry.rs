// Event log entry implementation
// Original file: src/log/entry/event_entry.rs

// Event entry implementation for Causality Unified Log System
//
// This module provides the EventEntry struct for representing events in the log.

use std::fmt;
use serde::{Serialize, Deserialize};

use causality_types::{ContentId, DomainId};

/// The severity level of an event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    /// The event name
    pub event_name: String,
    /// The event severity
    pub severity: EventSeverity,
    /// The component that generated this event
    pub component: String,
    /// The event details
    pub details: serde_json::Value,
    /// Related resources, if any
    pub resources: Option<Vec<ContentId>>,
    /// Related domains, if any
    pub domains: Option<Vec<DomainId>>,
}

impl EventEntry {
    /// Create a new event entry
    pub fn new(
        event_name: String,
        severity: EventSeverity,
        component: String,
        details: serde_json::Value,
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
    pub fn details(&self) -> &serde_json::Value {
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
        details: serde_json::Value,
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
        details: serde_json::Value,
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
        details: serde_json::Value,
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
        details: serde_json::Value,
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
        details: serde_json::Value,
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
        let details = serde_json::json!({"message": "test message"});
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
        assert_eq!(entry.details, details);
        assert_eq!(entry.resources, resources);
        assert_eq!(entry.domains, domains);
        
        // Test getters
        assert_eq!(entry.event_name(), event_name);
        assert_eq!(entry.severity(), &EventSeverity::Info);
        assert_eq!(entry.component(), component);
        assert_eq!(entry.details(), &details);
        
        // Test convenience methods
        let debug = EventEntry::debug("test", "debug_event", serde_json::json!({}))
            .with_resources(vec![ContentId::new("resource-id-1")]);  // Use string ID
        assert_eq!(debug.severity, EventSeverity::Debug);
        assert_eq!(debug.component, "test");
        assert_eq!(debug.event_name, "debug_event");
        assert_eq!(debug.resources.unwrap()[0], ContentId::new("resource-id-1"));  // Use string ID
        
        let info = EventEntry::info("test", "info_event", serde_json::json!({}))
            .with_domains(vec![DomainId::new("domain-id-1")]);  // Use string ID
        assert_eq!(info.severity, EventSeverity::Info);
        assert_eq!(info.component, "test");
        assert_eq!(info.event_name, "info_event");
        assert_eq!(info.domains.unwrap()[0], DomainId::new("domain-id-1"));  // Use string ID
        
        let warning = EventEntry::warning("test", "warning_event", serde_json::json!({}));
        assert_eq!(warning.severity, EventSeverity::Warning);
        
        let error = EventEntry::error("test", "error_event", serde_json::json!({}));
        assert_eq!(error.severity, EventSeverity::Error);
        
        let critical = EventEntry::critical("test", "critical_event", serde_json::json!({}));
        assert_eq!(critical.severity, EventSeverity::Critical);
    }
} 
