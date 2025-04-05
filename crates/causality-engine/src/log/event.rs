// Event logging system
// Original file: src/log/event.rs

// Event logging implementation for Causality Unified Log System
//
// This module provides a flexible event logging system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use causality_error::EngineError;
use causality_types::{Timestamp, ContentId, TraceId, DomainId};

use crate::log::types::{LogEntry, EntryType, EntryData, SystemEventEntry};
use crate::log::LogStorage;
use crate::log::event_entry::{EventEntry, EventSeverity};
use crate::error_conversions::convert_boxed_error;
use causality_error::CausalityError;
use causality_error::Result as CausalityResult;

/// Manages event logging
pub struct EventLogger<S: LogStorage + Send + Sync + 'static> {
    /// The underlying storage
    storage: Arc<Mutex<S>>,
    /// The domain ID for this logger
    domain_id: DomainId,
    /// The tracing context
    context: Option<TraceId>,
}

/// Metadata for an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// The time when the event occurred
    pub timestamp: DateTime<Utc>,
    /// The source of the event
    pub source: String,
    /// The severity level
    pub severity: EventSeverity,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Related trace IDs
    pub related_traces: Vec<TraceId>,
    /// Custom metadata
    pub custom: Option<Vec<u8>>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        EventMetadata {
            timestamp: Utc::now(),
            source: "system".to_string(),
            severity: EventSeverity::Info,
            tags: vec![],
            related_traces: vec![],
            custom: None,
        }
    }
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(source: &str) -> Self {
        EventMetadata {
            timestamp: Utc::now(),
            source: source.to_string(),
            severity: EventSeverity::Info,
            tags: vec![],
            related_traces: vec![],
            custom: None,
        }
    }
    
    /// Set the severity level
    pub fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Add a single tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    /// Add related trace IDs
    pub fn with_related_traces(mut self, traces: Vec<TraceId>) -> Self {
        self.related_traces = traces;
        self
    }
    
    /// Add a related trace ID
    pub fn with_related_trace(mut self, trace_id: TraceId) -> Self {
        self.related_traces.push(trace_id);
        self
    }
    
    /// Add custom metadata
    pub fn with_custom<T: Serialize>(mut self, custom: &T) -> std::result::Result<Self, EngineError> {
        self.custom = Some(bincode::serialize(custom)
            .map_err(|e| EngineError::SerializationFailed(e.to_string()))?);
        Ok(self)
    }
    
    /// Convert the metadata to a HashMap
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("timestamp".to_string(), self.timestamp.to_rfc3339());
        map.insert("source".to_string(), self.source.clone());
        map.insert("severity".to_string(), self.severity.to_string());
        
        if !self.tags.is_empty() {
            map.insert("tags".to_string(), serde_json::to_string(&self.tags).unwrap_or_default());
        }
        
        if !self.related_traces.is_empty() {
            let traces: Vec<String> = self.related_traces.iter()
                .map(|t| t.to_string())
                .collect();
            map.insert("related_traces".to_string(), serde_json::to_string(&traces).unwrap_or_default());
        }
        
        if let Some(ref custom) = self.custom {
            map.insert("custom".to_string(), base64::encode(custom));
        }
        
        map
    }
}

impl<S: LogStorage + Send + Sync + 'static> EventLogger<S> {
    /// Create a new event logger
    pub fn new(
        storage: Arc<Mutex<S>>,
        domain_id: DomainId,
    ) -> Self {
        Self {
            storage,
            domain_id,
            context: None,
        }
    }
    
    /// Create a new event logger with a trace ID
    pub fn with_trace(
        storage: Arc<Mutex<S>>,
        domain_id: DomainId,
        trace_id: TraceId,
    ) -> Self {
        Self {
            storage,
            domain_id,
            context: Some(trace_id),
        }
    }
    
    /// Log an event
    pub fn log_event(
        &self,
        entry_type: &str,
        severity: EventSeverity,
        event_name: impl Into<String>,
        payload: Option<serde_json::Value>,
        trace_id: Option<TraceId>,
        resources: Option<Vec<ContentId>>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let event = EventEntry {
            event_name: event_name.into(),
            severity: severity.clone(),
            component: "engine".to_string(),
            details: payload.unwrap_or_else(|| serde_json::json!({})),
            resources,
            domains: None,
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), entry_type.to_string());
        metadata.insert("severity".to_string(), format!("{:?}", severity));
        
        let log_entry = LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Timestamp::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(event),
            trace_id,
            parent_id: None,
            metadata,
            entry_hash: None,
        };
        
        let storage = self.storage.lock()
            .map_err(|e| Box::new(EngineError::LogError(format!("Mutex poison error: {}", e))) as Box<dyn CausalityError>)?;
        storage.append(log_entry)?;
        
        Ok(())
    }
    
    /// Log a lifecycle event
    pub fn log_lifecycle_event(
        &self,
        trace_id: TraceId,
        lifecycle_stage: &str,
        entity_type: &str,
        entity_id: &str,
        metadata: Option<EventMetadata>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let lifecycle_data = LifecycleEventData {
            lifecycle_stage: lifecycle_stage.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
        };
        
        let json_data = match serde_json::to_value(lifecycle_data) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let resource_id = ContentId::new("resource-1");
        
        match self.log_event(
            "lifecycle.event",
            EventSeverity::Info,
            "system",
            Some(json_data),
            Some(trace_id),
            Some(vec![resource_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Log a system event
    pub fn log_system_event(
        &self,
        trace_id: TraceId,
        system_component: &str,
        event_name: &str,
        message: &str,
        metadata: Option<EventMetadata>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let system_data = SystemEventData {
            component: system_component.to_string(),
            event_name: event_name.to_string(),
            message: message.to_string(),
        };
        
        let json_data = match serde_json::to_value(system_data) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let resource_id = ContentId::new("resource-1");
        
        match self.log_event(
            "system.event",
            EventSeverity::Info,
            system_component,
            Some(json_data),
            Some(trace_id),
            Some(vec![resource_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Log a user activity event
    pub fn log_user_activity(
        &self,
        trace_id: TraceId,
        user_id: &str,
        activity_type: &str,
        details: &str,
        metadata: Option<EventMetadata>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let activity_data = UserActivityData {
            user_id: user_id.to_string(),
            activity_type: activity_type.to_string(),
            details: details.to_string(),
        };
        
        let json_data = match serde_json::to_value(activity_data) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let resource_id = ContentId::new("resource-1");
        
        match self.log_event(
            "user.activity",
            EventSeverity::Info,
            "system",
            Some(json_data),
            Some(trace_id),
            Some(vec![resource_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Log a resource access event
    pub fn log_resource_access(
        &self,
        trace_id: TraceId,
        resource_type: &str,
        resource_id: &str,
        access_type: &str,
        accessor_id: &str,
        metadata: Option<EventMetadata>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let access_data = ResourceAccessData {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            access_type: access_type.to_string(),
            accessor_id: accessor_id.to_string(),
        };
        
        let json_data = match serde_json::to_value(access_data) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let content_id = ContentId::new("resource-1");
        
        match self.log_event(
            "resource_access",
            EventSeverity::Info,
            "system",
            Some(json_data),
            Some(trace_id),
            Some(vec![content_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Log an error event
    pub fn log_error(
        &self,
        trace_id: TraceId,
        error_type: &str,
        message: &str,
        code: Option<i32>,
        stack_trace: Option<&str>,
        metadata: Option<EventMetadata>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let error_data = ErrorEventData {
            error_type: error_type.to_string(),
            message: message.to_string(),
            code,
            stack_trace: stack_trace.map(|s| s.to_string()),
        };
        
        let metadata_with_severity = match metadata {
            Some(meta) => {
                let meta_clone = EventMetadata {
                    timestamp: meta.timestamp,
                    source: meta.source.clone(),
                    severity: EventSeverity::Error,
                    tags: meta.tags.clone(),
                    related_traces: meta.related_traces.clone(),
                    custom: meta.custom.clone(),
                };
                Some(meta_clone)
            },
            None => {
                Some(EventMetadata::default().with_severity(EventSeverity::Error))
            }
        };
        
        let json_data = match serde_json::to_value(error_data) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let content_id = ContentId::new("resource-1");
        
        match self.log_event(
            "error",
            EventSeverity::Error,
            "system",
            Some(json_data),
            Some(trace_id),
            Some(vec![content_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Log an event for performance tracking
    pub fn log_performance_event(
        &self,
        trace_id: TraceId,
        operation: &str,
        duration_ms: u64,
        metadata: Option<HashMap<String, String>>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let perf_data = PerformanceEventData {
            operation: operation.to_string(),
            duration_ms,
            metadata: metadata.unwrap_or_default(),
        };
        
        let json_data = match serde_json::to_value(perf_data) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let content_id = ContentId::new("resource-1");
        
        match self.log_event(
            "performance",
            EventSeverity::Info,
            "system",
            Some(json_data),
            Some(trace_id),
            Some(vec![content_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Log a debug event
    pub fn log_debug(
        &self,
        trace_id: TraceId,
        component: &str,
        message: &str,
        debug_data: Option<HashMap<String, String>>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let debug_data_obj = DebugEventData {
            component: component.to_string(),
            message: message.to_string(),
            debug_info: debug_data,
        };
        
        let json_data = match serde_json::to_value(debug_data_obj) {
            Ok(val) => val,
            Err(e) => return Err(Box::new(EngineError::SerializationFailed(e.to_string())) as Box<dyn CausalityError>),
        };
        
        let content_id = ContentId::new("resource-1");
        
        match self.log_event(
            "debug",
            EventSeverity::Debug,
            component,
            Some(json_data),
            Some(trace_id),
            Some(vec![content_id]),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Get the total number of entries in the log
    pub async fn get_entry_count(&self) -> CausalityResult<usize> {
        let storage = self.storage.lock()
            .map_err(|e| EngineError::LogError(format!("Mutex poison error: {}", e)))?;
        
        storage.get_entry_count()
            .await
    }
}

/// Data for a lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LifecycleEventData {
    /// The lifecycle stage (e.g., "created", "updated", "deleted")
    pub lifecycle_stage: String,
    /// The type of entity
    pub entity_type: String,
    /// The ID of the entity
    pub entity_id: String,
}

/// Data for a system event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemEventData {
    /// The system component
    pub component: String,
    /// The name of the event
    pub event_name: String,
    /// The message
    pub message: String,
}

/// Data for a user activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserActivityData {
    /// The user ID
    pub user_id: String,
    /// The type of activity
    pub activity_type: String,
    /// Additional details
    pub details: String,
}

/// Data for a resource access event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceAccessData {
    /// The type of resource
    pub resource_type: String,
    /// The ID of the resource
    pub resource_id: String,
    /// The type of access (e.g., "read", "write", "delete")
    pub access_type: String,
    /// The ID of the accessor
    pub accessor_id: String,
}

/// Data for an error event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEventData {
    /// The type of error
    pub error_type: String,
    /// The error message
    pub message: String,
    /// The error code, if any
    pub code: Option<i32>,
    /// The stack trace, if any
    pub stack_trace: Option<String>,
}

/// Data for a performance event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceEventData {
    /// The operation being measured
    pub operation: String,
    /// The duration in milliseconds
    pub duration_ms: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Data for a debug event
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DebugEventData {
    /// The component being debugged
    pub component: String,
    /// The debug message
    pub message: String,
    /// Additional debug information
    pub debug_info: Option<HashMap<String, String>>,
}

/// A builder for event query operations
pub struct EventQuery<'a, S: LogStorage + Send + Sync + 'static> {
    /// The event logger - Store as dyn LogStorage if S is not needed directly
    logger: &'a EventLogger<S>,
    /// The event type to query
    event_type: Option<String>,
    /// The minimum severity level
    min_severity: Option<EventSeverity>,
    /// Tags to filter by
    tags: Option<Vec<String>>,
    /// The source to filter by
    source: Option<String>,
    /// Time range start
    start_time: Option<DateTime<Utc>>,
    /// Time range end
    end_time: Option<DateTime<Utc>>,
    /// The maximum number of events to return
    limit: Option<usize>,
}

impl<'a, S: LogStorage + Send + Sync + 'static> EventQuery<'a, S> {
    /// Create a new event query
    pub fn new(logger: &'a EventLogger<S>) -> Self {
        EventQuery {
            logger,
            event_type: None,
            min_severity: None,
            tags: None,
            source: None,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
    
    /// Set the event type to query
    pub fn of_type(mut self, event_type: &str) -> Self {
        self.event_type = Some(event_type.to_string());
        self
    }
    
    /// Set the minimum severity level
    pub fn min_severity(mut self, severity: EventSeverity) -> Self {
        self.min_severity = Some(severity);
        self
    }
    
    /// Set tags to filter by
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
    
    /// Add a tag to filter by
    pub fn with_tag(mut self, tag: &str) -> Self {
        if let Some(tags) = &mut self.tags {
            tags.push(tag.to_string());
        } else {
            self.tags = Some(vec![tag.to_string()]);
        }
        self
    }
    
    /// Set the source to filter by
    pub fn from_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }
    
    /// Set the time range
    pub fn in_time_range(
        mut self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        self
    }
    
    /// Set the maximum number of events to return
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Execute the query and return the events
    pub async fn execute(&self) -> std::result::Result<Vec<EventEntry>, EngineError> {
        let storage = self.logger.storage.lock()
            .map_err(|e| EngineError::LogError(format!("Mutex poison error: {}", e)))?;
            
        let count = storage.get_entry_count()
             .await
             .map_err(convert_boxed_error)?;
        let entries = storage.read(0, count)
             .map_err(convert_boxed_error)?;
            
        let mut results: Vec<EventEntry> = entries.into_iter()
            .filter_map(|entry| {
                if let EntryData::Event(event) = entry.data {
                    Some(event)
                } else {
                    None
                }
            })
            .collect();
            
        if let Some(event_type) = &self.event_type {
            results.retain(|event| &event.event_name == event_type);
        }
        
        if let Some(min_severity) = &self.min_severity {
            results.retain(|event| {
                &event.severity >= min_severity
            });
        }
        
        if let Some(source) = &self.source {
            results.retain(|event| &event.component == source);
        }
        
        if let Some(start_time) = self.start_time {
            results.retain(|event| {
                if let Ok(metadata) = serde_json::from_value::<EventMetadata>(event.details.clone()) {
                    metadata.timestamp >= start_time
                } else {
                    true
                }
            });
        }
        
        if let Some(end_time) = self.end_time {
            results.retain(|event| {
                if let Ok(metadata) = serde_json::from_value::<EventMetadata>(event.details.clone()) {
                    metadata.timestamp <= end_time
                } else {
                    true
                }
            });
        }
        
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
}

/// Create a new event entry
pub fn create_event_entry(
    event_name: &str,
    severity: EventSeverity,
    component: &str,
    details: Option<Value>,
) -> LogEntry {
    LogEntry {
        id: format!("event-{}", uuid::Uuid::new_v4()),
        timestamp: Timestamp::now(),
        entry_type: EntryType::Event,
        data: EntryData::SystemEvent(SystemEventEntry {
            event_type: event_name.to_string(),
            source: component.to_string(),
            data: details.unwrap_or(json!({})),
        }),
        trace_id: None,
        parent_id: None,
        metadata: HashMap::new(),
        entry_hash: None,
    }
}

/// Create a fact observation event
pub fn create_fact_observation(
    event_name: &str,
    fact_id: &str,
    domains: Option<Vec<String>>,
    _metadata: Option<EventMetadata>,
) -> LogEntry {
    let mut entry = LogEntry::new();
    // Implementation would go here
    entry
}

/// Create a resource access event
pub fn create_resource_access(
    event_name: &str,
    resource_id: &str,
    domains: Option<Vec<String>>,
    _metadata: Option<EventMetadata>,
) -> LogEntry {
    let mut entry = LogEntry::new();
    // Implementation would go here
    entry
}

/// Create a system event
pub fn create_system_event(
    event_name: &str,
    domains: Option<Vec<String>>,
    _metadata: Option<EventMetadata>,
) -> LogEntry {
    let mut entry = LogEntry::new();
    // Implementation would go here
    entry
}

/// Create a domain event
pub fn create_domain_event(
    event_name: &str,
    domains: Option<Vec<String>>,
    _metadata: Option<EventMetadata>,
) -> LogEntry {
    let mut entry = LogEntry::new();
    // Implementation would go here
    entry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::MemoryLogStorage;
    use crate::log::event_entry::EventSeverity;
    use chrono::Duration;
    use tokio;
    
    fn create_test_logger() -> EventLogger<MemoryLogStorage> {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let domain_id = DomainId::new("test-domain");
        EventLogger::new(storage, domain_id)
    }
    
    #[test]
    fn test_log_basic_event() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        let resource = ContentId::new("resource-1");
        
        let result = logger.log_event(
            "event",
            EventSeverity::Info,
            "test-component",
            Some(json!({"message": "Test event"})),
            Some(trace_id),
            Some(vec![resource]),
        );
        
        assert!(result.is_ok());
        let storage = logger.storage.lock().unwrap();
        let entries = storage.read(0, storage.entry_count().unwrap()).unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        if let EntryData::Event(event) = &entry.data {
            assert_eq!(event.event_name, "test-component");
            assert_eq!(event.severity, EventSeverity::Info);
            assert_eq!(event.component, "engine");
            assert_eq!(event.details, json!({"message": "Test event"}));
            assert_eq!(event.resources.as_ref().unwrap()[0], resource);
        } else {
            panic!("Expected EntryData::Event");
        }
    }
    
    #[test]
    fn test_log_with_metadata() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        let resource = ContentId::new("resource-1");
        
        let result = logger.log_event(
            "event",
            EventSeverity::Warning,
            "test-component",
            Some(json!({"message": "Test event with metadata"})),
            Some(trace_id),
            Some(vec![resource]),
        );
        
        assert!(result.is_ok());
        let storage = logger.storage.lock().unwrap();
        let entries = storage.read(0, storage.entry_count().unwrap()).unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        if let EntryData::Event(event) = &entry.data {
            assert_eq!(event.event_name, "test-component");
            assert_eq!(event.severity, EventSeverity::Warning);
            assert_eq!(event.component, "engine");
            assert_eq!(event.details, json!({"message": "Test event with metadata"}));
            assert_eq!(event.resources.as_ref().unwrap()[0], resource);
        } else {
            panic!("Expected EntryData::Event");
        }
        
        assert_eq!(entry.metadata.get("event_type"), Some(&"event".to_string()));
        assert_eq!(entry.metadata.get("severity"), Some(&"Warning".to_string()));
    }
    
    #[test]
    fn test_specialized_event_types() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        
        let resource_id = ContentId::new("resource-1");
        
        let result = logger.log_event(
            "system-event",
            EventSeverity::Info,
            "system",
            Some(json!({"message": "test message"})),
            Some(trace_id.clone()),
            Some(vec![resource_id.clone()]),
        );
        
        assert!(result.is_ok());
        
        let result = logger.log_event(
            "user-activity",
            EventSeverity::Info,
            "user",
            Some(json!({"user_id": "user1", "message": "user logged in"})),
            Some(trace_id.clone()),
            Some(vec![resource_id.clone()]),
        );
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_event_query() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        let resource_id = ContentId::new("resource-1");
        
        logger.log_event(
            "event1",
            EventSeverity::Info,
            "system",
            Some(json!({"data": "data1"})),
            Some(trace_id.clone()),
            Some(vec![resource_id.clone()]),
        ).unwrap();
        
        logger.log_event(
            "event2",
            EventSeverity::Warning,
            "system",
            Some(json!({"data": "data2"})),
            Some(trace_id.clone()),
            Some(vec![resource_id.clone()]),
        ).unwrap();
        
        logger.log_event(
            "event3",
            EventSeverity::Error,
            "system",
            Some(json!({"data": "data3"})),
            Some(trace_id.clone()),
            Some(vec![resource_id.clone()]),
        ).unwrap();
        
        let events = EventQuery::new(&logger)
            .of_type("event1")
            .execute()
            .await
            .unwrap();
        
        assert!(!events.is_empty());
        
        let events = EventQuery::new(&logger)
            .min_severity(EventSeverity::Warning)
            .execute()
            .await
            .unwrap();
        
        assert!(!events.is_empty());
    }
} 