// Event logging system
// Original file: src/log/event.rs

// Event logging implementation for Causality Unified Log System
//
// This module provides a flexible event logging system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use causality_error::EngineError;
use causality_types::{Timestamp, ContentId, TraceId, DomainId};
use causality_types::content_addressing::{content_id_from_string, content_hash_from_bytes};

use crate::log::types::{LogEntry, EntryType, EntryData, SystemEventEntry, BorshJsonValue};
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
    
    /// Log a standard event
    pub fn log_event(
        &self,
        entry_type: &str,
        severity: EventSeverity,
        event_name: impl Into<String>,
        payload: Option<serde_json::Value>,
        trace_id: Option<TraceId>,
        resources: Option<Vec<ContentId>>,
    ) -> std::result::Result<(), Box<dyn CausalityError>> {
        let event_name_str = event_name.into();
        let event = EventEntry {
            event_name: event_name_str.clone(),
            severity: severity.clone(),
            component: "engine".to_string(),
            details: BorshJsonValue(payload.unwrap_or_else(|| serde_json::json!({}))),
            resources,
            domains: None,
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), entry_type.to_string());
        metadata.insert("severity".to_string(), format!("{:?}", severity));
        
        let event_str = format!("{}-{}-{}-{}",
            entry_type,
            format!("{:?}", severity),
            event_name_str,
            Timestamp::now().to_string()
        );
        
        let hash = content_hash_from_bytes(event_str.as_bytes());
        let id = ContentId::from(hash).to_string();
        
        let log_entry = LogEntry {
            id,
            timestamp: Timestamp::now(),
            entry_type: EntryType::Custom(entry_type.to_string()),
            data: EntryData::Event(event),
            trace_id,
            parent_id: None,
            metadata,
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
        _metadata: Option<EventMetadata>,
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
        _metadata: Option<EventMetadata>,
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
        _metadata: Option<EventMetadata>,
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
        _metadata: Option<EventMetadata>,
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
        
        let _metadata_with_severity = match metadata {
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
                if let Ok(metadata) = serde_json::from_value::<EventMetadata>(event.details.0.clone()) {
                    metadata.timestamp >= start_time
                } else {
                    true
                }
            });
        }
        
        if let Some(end_time) = self.end_time {
            results.retain(|event| {
                if let Ok(metadata) = serde_json::from_value::<EventMetadata>(event.details.0.clone()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::memory_storage::MemoryLogStorage;
    use crate::log::types::{EntryData, EntryType, BorshJsonValue};
    use std::sync::Mutex;
    use serde_json::json;
    use crate::log::event_entry::EventSeverity;

    fn create_test_logger() -> EventLogger<MemoryLogStorage> {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::default()));
        EventLogger::new(storage, DomainId::new("test_domain"))
    }

    #[test]
    fn test_log_basic_event() {
        let logger = create_test_logger();
        let trace_id = TraceId::new();
        let result = logger.log_event(
            "test.event",
            EventSeverity::Info,
            "test_component",
            Some(json!({ "key": "value" })),
            Some(trace_id.clone()),
            None,
        );
        assert!(result.is_ok());

        let storage_locked = logger.storage.lock().unwrap();
        let count = storage_locked.entry_count().expect("Failed to get entry count");
        let entries = storage_locked.read(0, count).expect("Failed to read entries");
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        let trace_id_str = trace_id.to_string();
        assert_eq!(entry.trace_id.as_ref().map(|t| t.to_string()), Some(trace_id_str));

        if let EntryData::Event(event_data) = &entry.data {
            assert_eq!(event_data.event_name, "test.event");
            assert_eq!(event_data.severity, EventSeverity::Info);
            assert_eq!(event_data.component, "test_component");
            assert_eq!(event_data.details.0, json!({ "key": "value" }));
        } else {
            panic!("Incorrect entry data type: expected Event, got {:?}", entry.data);
        }
    }

    #[test]
    fn test_log_with_metadata() {
        let logger = create_test_logger();
        let trace_id = TraceId::new();
        let metadata = EventMetadata::new("test_source")
            .with_severity(EventSeverity::Warning)
            .with_tag("important");

        let result = logger.log_lifecycle_event(
            trace_id.clone(), 
            "created", 
            "user", 
            "user123", 
            Some(metadata)
        );
        assert!(result.is_ok());
        
        let storage_locked = logger.storage.lock().unwrap();
        let count = storage_locked.entry_count().expect("Failed to get entry count");
        let entries = storage_locked.read(0, count).expect("Failed to read entries");
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.metadata.get("event_type"), Some(&"lifecycle.event".to_string()));
        assert_eq!(entry.metadata.get("severity"), Some(&"Info".to_string()));
    }

    #[test]
    fn test_specialized_event_types() {
        let logger = create_test_logger();
        let trace_id = TraceId::new();

        let res1 = logger.log_system_event(trace_id.clone(), "auth", "login_failed", "Invalid password", None);
        assert!(res1.is_ok());
        let res2 = logger.log_user_activity(trace_id.clone(), "user123", "file_upload", "Uploaded report.pdf", None);
        assert!(res2.is_ok());
        let res3 = logger.log_resource_access(trace_id.clone(), "database", "orders_table", "read", "service_account", None);
        assert!(res3.is_ok());
        let res4 = logger.log_error(trace_id.clone(), "DatabaseError", "Connection timeout", Some(504), None, None);
        assert!(res4.is_ok());
        let res5 = logger.log_performance_event(trace_id.clone(), "query_execution", 150, None);
        assert!(res5.is_ok());
        let res6 = logger.log_debug(trace_id.clone(), "cache_module", "Cache miss for key X", None);
        assert!(res6.is_ok());

        let storage_locked = logger.storage.lock().unwrap();
        let count = storage_locked.entry_count().expect("Failed to get entry count");
        assert_eq!(count, 6);
    }

    #[tokio::test]
    async fn test_event_query() {
        let logger = create_test_logger();
        let trace_id = TraceId::new();
        logger.log_event("event.one", EventSeverity::Info, "comp_a", None, Some(trace_id.clone()), None).unwrap();
        logger.log_event("event.two", EventSeverity::Warning, "comp_b", None, Some(trace_id.clone()), None).unwrap();
        logger.log_event("event.three", EventSeverity::Info, "comp_a", None, Some(trace_id.clone()), None).unwrap();

        let all_entries = logger.storage.lock().unwrap().get_all_entries().await.unwrap();
        
        let info_events: Vec<_> = all_entries.iter().filter(|e| 
            matches!(&e.data, EntryData::Event(ev) if ev.severity == EventSeverity::Info)
        ).collect();
        let comp_a_events: Vec<_> = all_entries.iter().filter(|e| 
            matches!(&e.data, EntryData::Event(ev) if ev.component == "comp_a")
        ).collect();

        assert_eq!(info_events.len(), 2);
        assert_eq!(comp_a_events.len(), 2);
    }
} 