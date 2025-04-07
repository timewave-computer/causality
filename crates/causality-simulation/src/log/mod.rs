// Purpose: Defines the logging interface for simulation and adapters for different storage backends.

use std::path::PathBuf;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use chrono;

use causality_core::resource::types::ResourceId;
use causality_core::effect::Effect;
use causality_types::ContentId;

use crate::scenario::Scenario;

/// Log event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogEventType {
    Info,
    Warning,
    Error,
    ScenarioStart,
    ScenarioEnd,
    AgentAction,
    ResourceCreated,
    ResourceUpdated,
    ResourceDeleted,
    Custom(String),
}

/// A log event in the simulation
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// Unique ID for this event
    pub id: String,
    /// The type of event
    pub event_type: LogEventType,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Name of the scenario this event belongs to
    pub scenario_name: String,
    /// The message/content of the event
    pub message: String,
    /// Additional data associated with the event
    pub metadata: HashMap<String, String>,
}

impl LogEvent {
    /// Create a new log event
    pub fn new(event_type: LogEventType, scenario_name: &str, message: &str) -> Self {
        Self {
            id: ContentId::new(format!("event-{}", chrono::Utc::now().timestamp_nanos())).to_string(),
            event_type,
            timestamp: chrono::Utc::now(),
            scenario_name: scenario_name.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the event
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// The LogStorage trait defines operations for accessing and manipulating simulation logs
pub trait LogStorage: Send + Sync {
    /// Add a log event to the storage
    fn add_event(&self, event: LogEvent) -> Result<()>;
    
    /// Get all events of a specific type
    fn get_events_by_type(&self, event_type: LogEventType) -> Result<Vec<LogEvent>>;
    
    /// Get all events for a specific scenario
    fn get_events_for_scenario(&self, scenario_name: &str) -> Result<Vec<LogEvent>>;
    
    /// Search for events containing the specified text
    fn search_events(&self, query: &str) -> Result<Vec<LogEvent>>;
    
    /// Clear all logs
    fn clear(&self) -> Result<()>;
}

/// In-memory implementation of LogStorage
pub struct InMemoryLog {
    events: std::sync::Mutex<Vec<LogEvent>>,
}

impl InMemoryLog {
    /// Create a new in-memory log storage
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl LogStorage for InMemoryLog {
    fn add_event(&self, event: LogEvent) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event);
        Ok(())
    }
    
    fn get_events_by_type(&self, event_type: LogEventType) -> Result<Vec<LogEvent>> {
        let events = self.events.lock().unwrap();
        Ok(events.iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect())
    }
    
    fn get_events_for_scenario(&self, scenario_name: &str) -> Result<Vec<LogEvent>> {
        let events = self.events.lock().unwrap();
        Ok(events.iter()
            .filter(|e| e.scenario_name == scenario_name)
            .cloned()
            .collect())
    }
    
    fn search_events(&self, query: &str) -> Result<Vec<LogEvent>> {
        let events = self.events.lock().unwrap();
        let query = query.to_lowercase();
        Ok(events.iter()
            .filter(|e| e.message.to_lowercase().contains(&query) ||
                e.metadata.values().any(|v| v.to_lowercase().contains(&query)))
            .cloned()
            .collect())
    }
    
    fn clear(&self) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.clear();
        Ok(())
    }
}

#[cfg(feature = "engine")]
pub mod adapter {
    use super::*;
    
    pub mod engine_adapter {
        use super::*;
        
        /// Adapter for using the engine's log storage
        pub struct EngineLogAdapter {
            // Implementation details
        }
        
        impl EngineLogAdapter {
            /// Create a new engine log adapter
            pub fn new() -> Self {
                Self {}
            }
        }
        
        impl LogStorage for EngineLogAdapter {
            fn add_event(&self, _event: LogEvent) -> Result<()> {
                // Implement based on engine requirements
                Ok(())
            }
            
            fn get_events_by_type(&self, _event_type: LogEventType) -> Result<Vec<LogEvent>> {
                // Implement based on engine requirements
                Ok(Vec::new())
            }
            
            fn get_events_for_scenario(&self, _scenario_name: &str) -> Result<Vec<LogEvent>> {
                // Implement based on engine requirements
                Ok(Vec::new())
            }
            
            fn search_events(&self, _query: &str) -> Result<Vec<LogEvent>> {
                // Implement based on engine requirements
                Ok(Vec::new())
            }
            
            fn clear(&self) -> Result<()> {
                // Implement based on engine requirements
                Ok(())
            }
        }
    }
} 