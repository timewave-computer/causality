// Replay callback system
// Original file: src/log/replay/callback.rs

// Replay callback implementation for Causality Unified Log System
//
// This module provides callback interfaces for log replay.

use chrono::{DateTime, Utc};
use crate::log::types::{LogEntry, FactEntry, EffectEntry, ResourceAccessEntry, SystemEventEntry, OperationEntry, BorshJsonValue};
use crate::log::replay::{ReplayResult, ReplayStatus};

use causality_error::{EngineError, Result as EngineResult};
use crate::log::event_entry::EventEntry;
use causality_types::{ContentId, DomainId, Timestamp};
use crate::log::replay::ReplayState;
use serde_json::json;
use std::collections::HashMap;

/// Callback interface for log replay
///
/// Implement this trait to handle callbacks during log replay.
pub trait ReplayCallback: Send + Sync {
    /// Called before replay begins
    fn on_replay_start(&self, _start_time: DateTime<Utc>) {}
    
    /// Called when replay ends
    fn on_replay_end(&self, _end_time: DateTime<Utc>, _status: &ReplayStatus) {}
    
    /// Called for each entry during replay
    fn on_entry(&self, _entry: &LogEntry, _index: usize, _total: usize) -> bool {
        true // Return true to continue, false to abort
    }
    
    /// Called when an effect is processed
    fn on_effect(&self, _effect: &EffectEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when a fact is processed
    fn on_fact(&self, _fact: &FactEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when an event is processed
    fn on_event(&self, _event: &EventEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when a resource access is processed
    fn on_resource_access(&self, _ra: &ResourceAccessEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when a system event is processed
    fn on_system_event(&self, _se: &SystemEventEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when an operation is processed
    fn on_operation(&self, _op: &OperationEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when a custom entry is processed
    fn on_custom_entry(&self, _custom_type: &str, _data: &BorshJsonValue, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> { Ok(()) }
    
    /// Called when replay is complete
    fn on_complete(&self, _result: &ReplayResult) {}
    
    /// Called when replay fails
    fn on_error(&self, _error: &EngineError) {}
}

/// A no-op implementation of ReplayCallback
///
/// This implementation does nothing for each callback method.
pub struct NoopReplayCallback;

impl ReplayCallback for NoopReplayCallback {}

/// A callback that collects statistics during replay
#[derive(Default)]
pub struct StatsCallback {
    /// The number of entries processed
    pub entries_processed: std::sync::atomic::AtomicUsize,
    /// The number of effects processed
    pub effects_processed: std::sync::atomic::AtomicUsize,
    /// The number of facts processed
    pub facts_processed: std::sync::atomic::AtomicUsize,
    /// The number of events processed
    pub events_processed: std::sync::atomic::AtomicUsize,
    /// Whether replay was successful
    pub success: std::sync::atomic::AtomicBool,
}

impl StatsCallback {
    /// Create a new stats callback
    pub fn new() -> Self {
        Self {
            entries_processed: std::sync::atomic::AtomicUsize::new(0),
            effects_processed: std::sync::atomic::AtomicUsize::new(0),
            facts_processed: std::sync::atomic::AtomicUsize::new(0),
            events_processed: std::sync::atomic::AtomicUsize::new(0),
            success: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    /// Get the number of entries processed
    pub fn entries_processed(&self) -> usize {
        self.entries_processed.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Get the number of effects processed
    pub fn effects_processed(&self) -> usize {
        self.effects_processed.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Get the number of facts processed
    pub fn facts_processed(&self) -> usize {
        self.facts_processed.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Get the number of events processed
    pub fn events_processed(&self) -> usize {
        self.events_processed.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Check if replay was successful
    pub fn is_success(&self) -> bool {
        self.success.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl ReplayCallback for StatsCallback {
    fn on_entry(&self, _entry: &LogEntry, _index: usize, _total: usize) -> bool {
        self.entries_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        true
    }
    
    fn on_effect(&self, _effect: &EffectEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> {
        self.effects_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    fn on_fact(&self, _fact: &FactEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> {
        self.facts_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    fn on_event(&self, _event: &EventEntry, _metadata: &HashMap<String, String>, _timestamp: Timestamp) -> EngineResult<()> {
        self.events_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    fn on_complete(&self, _result: &ReplayResult) {
        self.success.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// A callback that executes closures for each callback method
pub struct ClosureCallback {
    /// Called on replay start
    pub on_start: Box<dyn Fn(DateTime<Utc>) + Send + Sync>,
    /// Called for each entry
    pub on_entry: Box<dyn Fn(&LogEntry, usize, usize) -> bool + Send + Sync>,
    /// Called for each effect
    pub on_effect: Box<dyn Fn(&EffectEntry, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called for each fact
    pub on_fact: Box<dyn Fn(&FactEntry, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called for each event
    pub on_event: Box<dyn Fn(&EventEntry, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called for each resource access
    pub on_resource_access: Box<dyn Fn(&ResourceAccessEntry, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called for each system event
    pub on_system_event: Box<dyn Fn(&SystemEventEntry, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called for each operation
    pub on_operation: Box<dyn Fn(&OperationEntry, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called for each custom entry
    pub on_custom_entry: Box<dyn Fn(&str, &BorshJsonValue, &HashMap<String, String>, Timestamp) -> EngineResult<()> + Send + Sync>,
    /// Called on error
    pub on_error: Box<dyn Fn(&EngineError) + Send + Sync>,
}

impl Default for ClosureCallback {
    fn default() -> Self {
        Self {
            on_start: Box::new(|_| {}),
            on_entry: Box::new(|_, _, _| true),
            on_effect: Box::new(|_, _, _| Ok(())),
            on_fact: Box::new(|_, _, _| Ok(())),
            on_event: Box::new(|_, _, _| Ok(())),
            on_resource_access: Box::new(|_, _, _| Ok(())),
            on_system_event: Box::new(|_, _, _| Ok(())),
            on_operation: Box::new(|_, _, _| Ok(())),
            on_custom_entry: Box::new(|_, _, _, _| Ok(())),
            on_error: Box::new(|_| {}),
        }
    }
}

impl ReplayCallback for ClosureCallback {
    fn on_replay_start(&self, start_time: DateTime<Utc>) {
        (self.on_start)(start_time);
    }
    
    fn on_entry(&self, entry: &LogEntry, index: usize, total: usize) -> bool {
        (self.on_entry)(entry, index, total)
    }
    
    fn on_effect(&self, effect: &EffectEntry, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_effect)(effect, metadata, timestamp)
    }
    
    fn on_fact(&self, fact: &FactEntry, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_fact)(fact, metadata, timestamp)
    }
    
    fn on_event(&self, event: &EventEntry, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_event)(event, metadata, timestamp)
    }
    
    fn on_resource_access(&self, ra: &ResourceAccessEntry, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_resource_access)(ra, metadata, timestamp)
    }

    fn on_system_event(&self, se: &SystemEventEntry, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_system_event)(se, metadata, timestamp)
    }

    fn on_operation(&self, op: &OperationEntry, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_operation)(op, metadata, timestamp)
    }

    fn on_custom_entry(&self, custom_type: &str, data: &BorshJsonValue, metadata: &HashMap<String, String>, timestamp: Timestamp) -> EngineResult<()> {
        (self.on_custom_entry)(custom_type, data, metadata, timestamp)
    }
    
    fn on_error(&self, error: &EngineError) {
        (self.on_error)(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;
    use crate::log::{EntryType, EntryData};
    use crate::log::event_entry::{EventEntry, EventSeverity};
    use causality_types::{ContentId, DomainId, Timestamp};
    use crate::log::replay::ReplayState;
    use serde_json::json;

    #[test]
    fn test_noop_callback() {
        let callback = NoopReplayCallback;
        
        // These should not panic
        callback.on_replay_start(Utc::now());
        assert!(callback.on_entry(&create_test_entry(), 0, 1));
        callback.on_effect(&create_test_effect(), &create_test_metadata(), Timestamp::now());
        callback.on_fact(&create_test_fact(), &create_test_metadata(), Timestamp::now());
        callback.on_event(&create_test_event(), &create_test_metadata(), Timestamp::now());
        callback.on_complete(&create_test_result());
        callback.on_error(&EngineError::Other("test".to_string()));
    }
    
    #[test]
    fn test_stats_callback() {
        let callback = StatsCallback::new();
        
        assert_eq!(callback.entries_processed(), 0);
        assert_eq!(callback.effects_processed(), 0);
        assert_eq!(callback.facts_processed(), 0);
        assert_eq!(callback.events_processed(), 0);
        assert!(!callback.is_success());
        
        callback.on_entry(&create_test_entry(), 0, 1);
        callback.on_effect(&create_test_effect(), &create_test_metadata(), Timestamp::now());
        callback.on_fact(&create_test_fact(), &create_test_metadata(), Timestamp::now());
        callback.on_event(&create_test_event(), &create_test_metadata(), Timestamp::now());
        callback.on_complete(&create_test_result());
        
        assert_eq!(callback.entries_processed(), 1);
        assert_eq!(callback.effects_processed(), 1);
        assert_eq!(callback.facts_processed(), 1);
        assert_eq!(callback.events_processed(), 1);
        assert!(callback.is_success());
    }
    
    #[test]
    fn test_closure_callback() {
        let starts = std::sync::atomic::AtomicUsize::new(0);
        let entries = std::sync::atomic::AtomicUsize::new(0);
        let effects = std::sync::atomic::AtomicUsize::new(0);
        let facts = std::sync::atomic::AtomicUsize::new(0);
        let events = std::sync::atomic::AtomicUsize::new(0);
        let errors = std::sync::atomic::AtomicUsize::new(0);
        
        let callback = ClosureCallback {
            on_start: Box::new(|_| { starts.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }),
            on_entry: Box::new(|_, _, _| { entries.fetch_add(1, std::sync::atomic::Ordering::Relaxed); true }),
            on_effect: Box::new(|_, _, _| { effects.fetch_add(1, std::sync::atomic::Ordering::Relaxed); Ok(()) }),
            on_fact: Box::new(|_, _, _| { facts.fetch_add(1, std::sync::atomic::Ordering::Relaxed); Ok(()) }),
            on_event: Box::new(|_, _, _| { events.fetch_add(1, std::sync::atomic::Ordering::Relaxed); Ok(()) }),
            on_resource_access: Box::new(|_, _, _| Ok(())),
            on_system_event: Box::new(|_, _, _| Ok(())),
            on_operation: Box::new(|_, _, _| Ok(())),
            on_custom_entry: Box::new(|_, _, _, _| Ok(())),
            on_error: Box::new(|_| { errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }),
        };
        
        callback.on_replay_start(Utc::now());
        callback.on_entry(&create_test_entry(), 0, 1);
        callback.on_effect(&create_test_effect(), &create_test_metadata(), Timestamp::now());
        callback.on_fact(&create_test_fact(), &create_test_metadata(), Timestamp::now());
        callback.on_event(&create_test_event(), &create_test_metadata(), Timestamp::now());
        callback.on_error(&EngineError::Other("test".to_string()));
        
        assert_eq!(starts.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(entries.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(effects.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(facts.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(events.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(errors.load(std::sync::atomic::Ordering::Relaxed), 1);
    }
    
    // Helper functions updated for current struct definitions
    fn create_test_entry() -> LogEntry {
        LogEntry {
            id: "test-entry-id".to_string(),
            timestamp: Timestamp::now(),
            entry_type: EntryType::Fact, // Example type
            data: EntryData::Fact(create_test_fact()), // Example data
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
        }
    }
    
    fn create_test_effect() -> EffectEntry {
        EffectEntry {
            domain_id: DomainId::new("test_domain"),
            effect_id: "test-effect-id".to_string(),
            status: "Success".to_string(),
            outcome: Some(BorshJsonValue(json!({ "result": true }))),
            error: None,
        }
    }
    
    fn create_test_fact() -> FactEntry {
        FactEntry {
            domain_id: DomainId::new("test_domain"),
            fact_id: "test-fact-id".to_string(),
            details: BorshJsonValue(json!({ "info": "some data"})),
        }
    }
    
    fn create_test_event() -> EventEntry {
        EventEntry {
            event_name: "test.event".to_string(),
            severity: EventSeverity::Info,
            component: "test-component".to_string(),
            details: BorshJsonValue(json!({ "payload": 123 })),
            resources: None,
            domains: None,
        }
    }
    
    fn create_test_result() -> ReplayResult {
        ReplayResult {
            status: ReplayStatus::Complete,
            processed_entries: 10,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            error: None,
            state: Some(ReplayState::new()), // Use new for empty state
        }
    }
     
    fn create_test_metadata() -> HashMap<String, String> {
        HashMap::new()
    }
} 