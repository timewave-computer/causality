// Event entry implementation for Causality Unified Log System
//
// This module provides the EventEntry struct for representing events in the log.

use std::fmt;
use serde::{Serialize, Deserialize};

use crate::types::{ResourceId, DomainId};

/// The severity level of an event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub resources: Option<Vec<ResourceId>>,
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
        resources: Option<Vec<ResourceId>>,
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
    pub fn resources(&self) -> Option<&[ResourceId]> {
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
    pub fn with_resources(mut self, resources: Vec<ResourceId>) -> Self {
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
    
    #[test]
    fn test_event_entry_creation() {
        let event_name = "test_event".to_string();
        let severity = EventSeverity::Info;
        let component = "test".to_string();
        let details = serde_json::json!({"message": "test message"});
        let resources = Some(vec![ResourceId::new(1)]);
        let domains = Some(vec![DomainId::new(1)]);
        
        let entry = EventEntry::new(
            event_name.clone(),
            severity,
            component.clone(),
            details.clone(),
            resources.clone(),
            domains.clone(),
        );
        
        assert_eq!(entry.event_name, event_name);
        assert_eq!(entry.severity, severity);
        assert_eq!(entry.component, component);
        assert_eq!(entry.details, details);
        assert_eq!(entry.resources, resources);
        assert_eq!(entry.domains, domains);
        
        // Test convenience methods
        let debug = EventEntry::debug("test", "debug_event", serde_json::json!({}))
            .with_resources(vec![ResourceId::new(1)]);
        assert_eq!(debug.severity, EventSeverity::Debug);
        assert_eq!(debug.component, "test");
        assert_eq!(debug.event_name, "debug_event");
        assert_eq!(debug.resources.unwrap()[0], ResourceId::new(1));
        
        let info = EventEntry::info("test", "info_event", serde_json::json!({}))
            .with_domains(vec![DomainId::new(1)]);
        assert_eq!(info.severity, EventSeverity::Info);
        assert_eq!(info.component, "test");
        assert_eq!(info.event_name, "info_event");
        assert_eq!(info.domains.unwrap()[0], DomainId::new(1));
        
        let warning = EventEntry::warning("test", "warning_event", serde_json::json!({}));
        assert_eq!(warning.severity, EventSeverity::Warning);
        
        let error = EventEntry::error("test", "error_event", serde_json::json!({}));
        assert_eq!(error.severity, EventSeverity::Error);
        
        let critical = EventEntry::critical("test", "critical_event", serde_json::json!({}));
        assert_eq!(critical.severity, EventSeverity::Critical);
    }
} 