// Execution tracing utilities
// Original file: src/execution/trace.rs

// Execution trace module for Causality Content-Addressed Code System
//
// This module provides functionality for recording and analyzing execution traces,
// allowing for detailed understanding of execution flow and time-travel debugging.

use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};

use causality_core::effect::runtime::EffectRuntime;
use causality_core::ContentHash;
use causality_types::{Error, Result};
use crate::execution::{ContextId, ExecutionEvent};
use crate::Value;

/// Metadata for an execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// ID of the execution context
    pub context_id: ContextId,
    /// Start time of the execution
    pub start_time: u64,
    /// End time of the execution
    pub end_time: Option<u64>,
    /// Number of events in the trace
    pub event_count: usize,
    /// Whether the execution completed successfully
    pub completed: bool,
    /// Custom labels for this trace
    pub labels: HashMap<String, String>,
}

impl TraceMetadata {
    /// Create new trace metadata
    pub fn new(context_id: ContextId) -> Self {
        TraceMetadata {
            context_id,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            end_time: None,
            event_count: 0,
            completed: false,
            labels: HashMap::new(),
        }
    }
    
    /// Mark the trace as completed
    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
    }
    
    /// Add a label to the trace
    pub fn add_label(&mut self, key: &str, value: &str) {
        self.labels.insert(key.to_string(), value.to_string());
    }
    
    /// Get the duration of the execution in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.end_time.map(|end| end - self.start_time)
    }
}

/// A complete execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Metadata for this trace
    pub metadata: TraceMetadata,
    /// All events in the trace
    pub events: Vec<ExecutionEvent>,
}

impl ExecutionTrace {
    /// Create a new execution trace
    pub fn new(context_id: ContextId) -> Self {
        ExecutionTrace {
            metadata: TraceMetadata::new(context_id),
            events: Vec::new(),
        }
    }
    
    /// Add an event to the trace
    pub fn add_event(&mut self, event: ExecutionEvent) {
        self.events.push(event);
        self.metadata.event_count = self.events.len();
    }
    
    /// Mark the trace as completed
    pub fn mark_completed(&mut self) {
        self.metadata.mark_completed();
    }
    
    /// Get the duration of the execution in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.metadata.duration_ms()
    }
    
    /// Get all function call events
    pub fn function_calls(&self) -> Vec<&ExecutionEvent> {
        self.events
            .iter()
            .filter(|event| matches!(event, ExecutionEvent::FunctionCall { .. }))
            .collect()
    }
    
    /// Get all function return events
    pub fn function_returns(&self) -> Vec<&ExecutionEvent> {
        self.events
            .iter()
            .filter(|event| matches!(event, ExecutionEvent::FunctionReturn { .. }))
            .collect()
    }
    
    /// Get all effect application events
    pub fn effect_applications(&self) -> Vec<&ExecutionEvent> {
        self.events
            .iter()
            .filter(|event| matches!(event, ExecutionEvent::EffectApplied { .. }))
            .collect()
    }
    
    /// Get all error events
    pub fn errors(&self) -> Vec<&ExecutionEvent> {
        self.events
            .iter()
            .filter(|event| matches!(event, ExecutionEvent::Error(_)))
            .collect()
    }
    
    /// Get events related to a specific code hash
    pub fn events_for_code(&self, hash: &ContentHash) -> Vec<&ExecutionEvent> {
        self.events
            .iter()
            .filter(|event| match event {
                ExecutionEvent::FunctionCall { hash: h, .. } => h == hash,
                ExecutionEvent::FunctionReturn { hash: h, .. } => h == hash,
                _ => false,
            })
            .collect()
    }
    
    /// Serialize the trace to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| Error::SerializationError(e.to_string()))
    }
    
    /// Deserialize the trace from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::DeserializationError(e.to_string()))
    }
}

/// A component for recording and managing execution traces
pub struct ExecutionTracer {
    /// The directory for storing traces
    trace_dir: PathBuf,
    /// Currently active traces
    active_traces: Arc<RwLock<HashMap<ContextId, ExecutionTrace>>>,
}

impl ExecutionTracer {
    /// Create a new execution tracer
    pub fn new<P: AsRef<Path>>(trace_dir: P) -> Result<Self> {
        let trace_dir = trace_dir.as_ref().to_path_buf();
        fs::create_dir_all(&trace_dir)?;
        
        Ok(ExecutionTracer {
            trace_dir,
            active_traces: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Start a new trace
    pub fn start_trace(&self, context_id: ContextId) -> Result<()> {
        let trace = ExecutionTrace::new(context_id.clone());
        
        let mut active_traces = self.active_traces.write().map_err(|_| Error::LockError)?;
        active_traces.insert(context_id, trace);
        
        Ok(())
    }
    
    /// Record an event
    pub fn record_event(&self, context_id: &ContextId, event: ExecutionEvent) -> Result<()> {
        let mut active_traces = self.active_traces.write().map_err(|_| Error::LockError)?;
        
        if let Some(trace) = active_traces.get_mut(context_id) {
            trace.add_event(event);
            Ok(())
        } else {
            Err(Error::TraceNotFound(context_id.to_string()))
        }
    }
    
    /// Complete a trace and save it
    pub fn complete_trace(&self, context_id: &ContextId) -> Result<PathBuf> {
        let mut trace = {
            let mut active_traces = self.active_traces.write().map_err(|_| Error::LockError)?;
            
            if let Some(trace) = active_traces.remove(context_id) {
                trace
            } else {
                return Err(Error::TraceNotFound(context_id.to_string()));
            }
        };
        
        // Mark as completed
        trace.mark_completed();
        
        // Save to disk
        self.save_trace(&trace)
    }
    
    /// Save a trace to disk
    pub fn save_trace(&self, trace: &ExecutionTrace) -> Result<PathBuf> {
        let context_id = &trace.metadata.context_id;
        let timestamp = trace.metadata.start_time;
        
        // Create filename with context ID and timestamp
        let filename = format!("{}-{}.json", context_id, timestamp);
        let file_path = self.trace_dir.join(filename);
        
        // Serialize and write
        let json = trace.to_json()?;
        fs::write(&file_path, json)?;
        
        Ok(file_path)
    }
    
    /// Load a trace from disk
    pub fn load_trace<P: AsRef<Path>>(&self, path: P) -> Result<ExecutionTrace> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        ExecutionTrace::from_json(&contents)
    }
    
    /// List all available traces
    pub fn list_traces(&self) -> Result<Vec<TraceMetadata>> {
        let mut traces = Vec::new();
        
        for entry in fs::read_dir(&self.trace_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    if let Ok(trace) = self.load_trace(&path) {
                        traces.push(trace.metadata);
                    }
                }
            }
        }
        
        // Sort by start time (most recent first)
        traces.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        
        Ok(traces)
    }
    
    /// Query traces by criteria
    pub fn query_traces(&self, query: &TraceQuery) -> Result<Vec<TraceMetadata>> {
        let all_traces = self.list_traces()?;
        
        let filtered = all_traces
            .into_iter()
            .filter(|metadata| query.matches(metadata))
            .collect();
        
        Ok(filtered)
    }
}

/// A query for filtering traces
#[derive(Debug, Clone)]
pub struct TraceQuery {
    /// Filter by context ID
    pub context_id: Option<ContextId>,
    /// Filter by completion status
    pub completed: Option<bool>,
    /// Filter by start time range (min, max)
    pub start_time_range: Option<(u64, u64)>,
    /// Filter by labels (all must match)
    pub labels: HashMap<String, String>,
}

impl TraceQuery {
    /// Create a new trace query
    pub fn new() -> Self {
        TraceQuery {
            context_id: None,
            completed: None,
            start_time_range: None,
            labels: HashMap::new(),
        }
    }
    
    /// Set the context ID filter
    pub fn with_context_id(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }
    
    /// Set the completion status filter
    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = Some(completed);
        self
    }
    
    /// Set the start time range filter
    pub fn with_start_time_range(mut self, min: u64, max: u64) -> Self {
        self.start_time_range = Some((min, max));
        self
    }
    
    /// Add a label filter
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if a trace metadata matches this query
    pub fn matches(&self, metadata: &TraceMetadata) -> bool {
        // Check context ID
        if let Some(context_id) = &self.context_id {
            if &metadata.context_id != context_id {
                return false;
            }
        }
        
        // Check completion status
        if let Some(completed) = self.completed {
            if metadata.completed != completed {
                return false;
            }
        }
        
        // Check start time range
        if let Some((min, max)) = self.start_time_range {
            if metadata.start_time < min || metadata.start_time > max {
                return false;
            }
        }
        
        // Check labels
        for (key, value) in &self.labels {
            if !metadata.labels.get(key).map_or(false, |v| v == value) {
                return false;
            }
        }
        
        true
    }
}

impl Default for TraceQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_trace_creation() {
        let context_id = ContextId::new();
        let mut trace = ExecutionTrace::new(context_id.clone());
        
        assert_eq!(trace.metadata.context_id, context_id);
        assert_eq!(trace.events.len(), 0);
        assert_eq!(trace.metadata.event_count, 0);
        assert!(!trace.metadata.completed);
    }
    
    #[test]
    fn test_tracer_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let tracer = ExecutionTracer::new(temp_dir.path())?;
        
        assert!(temp_dir.path().exists());
        Ok(())
    }
    
    #[test]
    fn test_trace_serialization() -> Result<()> {
        let context_id = ContextId::new();
        let mut trace = ExecutionTrace::new(context_id);
        
        // Add an event
        trace.add_event(ExecutionEvent::function_call(
            ContentHash::from_str("blake3:0123456789abcdef0123456789abcdef").unwrap(),
            Some("test_function".to_string()),
            vec![Value::String("arg1".to_string())],
        ));
        
        let json = trace.to_json()?;
        let deserialized = ExecutionTrace::from_json(&json)?;
        
        assert_eq!(deserialized.metadata.context_id, trace.metadata.context_id);
        assert_eq!(deserialized.events.len(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_trace_query() {
        let mut metadata = TraceMetadata::new(ContextId::new());
        metadata.add_label("service", "test-service");
        metadata.add_label("environment", "testing");
        
        let query1 = TraceQuery::new()
            .with_label("service", "test-service");
        
        assert!(query1.matches(&metadata));
        
        let query2 = TraceQuery::new()
            .with_label("service", "wrong-service");
        
        assert!(!query2.matches(&metadata));
        
        let query3 = TraceQuery::new()
            .with_completed(true);
        
        assert!(!query3.matches(&metadata));
        
        metadata.mark_completed();
        
        assert!(query3.matches(&metadata));
    }
} 