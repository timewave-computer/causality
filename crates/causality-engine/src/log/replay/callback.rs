// Replay callback system
// Original file: src/log/replay/callback.rs

// Replay callback implementation for Causality Unified Log System
//
// This module provides callback interfaces for log replay.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use causality_types::Error;
use causality_engine::{LogEntry, EffectEntry, FactEntry};
use causality_engine::ReplayResult;
use causality_engine::{EventEntry, EventSeverity};
use causality_core::effect::runtime::EffectRuntime;

/// Callback interface for log replay
///
/// Implement this trait to handle callbacks during log replay.
pub trait ReplayCallback: Send + Sync {
    /// Called before replay begins
    fn on_replay_start(&self, _start_time: DateTime<Utc>) {}
    
    /// Called for each entry during replay
    fn on_entry(&self, _entry: &LogEntry, _index: usize, _total: usize) -> bool {
        true // Return true to continue, false to abort
    }
    
    /// Called when an effect is processed
    fn on_effect(&self, _effect: &EffectEntry, _entry: &LogEntry) {}
    
    /// Called when a fact is processed
    fn on_fact(&self, _fact: &FactEntry, _entry: &LogEntry) {}
    
    /// Called when replay is complete
    fn on_complete(&self, _result: &ReplayResult) {}
    
    /// Called when replay fails
    fn on_error(&self, _error: &Error) {}
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
    
    fn on_effect(&self, _effect: &EffectEntry, _entry: &LogEntry) {
        self.effects_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn on_fact(&self, _fact: &FactEntry, _entry: &LogEntry) {
        self.facts_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn on_complete(&self, _result: &ReplayResult) {
        self.success.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// A callback that executes closures for each callback method
pub struct ClosureCallback<S, E, F, C, ER> {
    /// Called on replay start
    pub on_start: S,
    /// Called for each entry
    pub on_entry: E,
    /// Called for each effect
    pub on_effect: F,
    /// Called for each fact
    pub on_fact: C,
    /// Called on error
    pub on_error: ER,
}

impl<S, E, F, C, ER> ClosureCallback<S, E, F, C, ER>
where
    S: Fn(DateTime<Utc>) + Send + Sync,
    E: Fn(&LogEntry, usize, usize) -> bool + Send + Sync,
    F: Fn(&EffectEntry, &LogEntry) + Send + Sync,
    C: Fn(&FactEntry, &LogEntry) + Send + Sync,
    ER: Fn(&Error) + Send + Sync,
{
    /// Create a new closure callback with the given closures
    pub fn new(
        on_start: S,
        on_entry: E,
        on_effect: F,
        on_fact: C,
        on_error: ER,
    ) -> Self {
        Self {
            on_start,
            on_entry,
            on_effect,
            on_fact,
            on_error,
        }
    }
}

impl<S, E, F, C, ER> ReplayCallback for ClosureCallback<S, E, F, C, ER>
where
    S: Fn(DateTime<Utc>) + Send + Sync,
    E: Fn(&LogEntry, usize, usize) -> bool + Send + Sync,
    F: Fn(&EffectEntry, &LogEntry) + Send + Sync,
    C: Fn(&FactEntry, &LogEntry) + Send + Sync,
    ER: Fn(&Error) + Send + Sync,
{
    fn on_replay_start(&self, start_time: DateTime<Utc>) {
        (self.on_start)(start_time);
    }
    
    fn on_entry(&self, entry: &LogEntry, index: usize, total: usize) -> bool {
        (self.on_entry)(entry, index, total)
    }
    
    fn on_effect(&self, effect: &EffectEntry, entry: &LogEntry) {
        (self.on_effect)(effect, entry);
    }
    
    fn on_fact(&self, fact: &FactEntry, entry: &LogEntry) {
        (self.on_fact)(fact, entry);
    }
    
    fn on_error(&self, error: &Error) {
        (self.on_error)(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;
    use causality_engine::{EntryType, EntryData, EventEntry, EventSeverity};

    #[test]
    fn test_noop_callback() {
        let callback = NoopReplayCallback;
        
        // These should not panic
        callback.on_replay_start(Utc::now());
        assert!(callback.on_entry(&create_test_entry(), 0, 1));
        callback.on_effect(&create_test_effect(), &create_test_entry());
        callback.on_fact(&create_test_fact(), &create_test_entry());
        callback.on_complete(&create_test_result());
        callback.on_error(&Error::Other("test".to_string()));
    }
    
    #[test]
    fn test_stats_callback() {
        let callback = StatsCallback::new();
        
        assert_eq!(callback.entries_processed(), 0);
        assert_eq!(callback.effects_processed(), 0);
        assert_eq!(callback.facts_processed(), 0);
        assert!(!callback.is_success());
        
        callback.on_entry(&create_test_entry(), 0, 1);
        callback.on_effect(&create_test_effect(), &create_test_entry());
        callback.on_fact(&create_test_fact(), &create_test_entry());
        callback.on_complete(&create_test_result());
        
        assert_eq!(callback.entries_processed(), 1);
        assert_eq!(callback.effects_processed(), 1);
        assert_eq!(callback.facts_processed(), 1);
        assert!(callback.is_success());
    }
    
    #[test]
    fn test_closure_callback() {
        let mut entries = 0;
        let mut effects = 0;
        let mut facts = 0;
        let mut starts = 0;
        let mut errors = 0;
        
        let callback = ClosureCallback::new(
            |_| { starts += 1; },
            |_, _, _| { entries += 1; true },
            |_, _| { effects += 1; },
            |_, _| { facts += 1; },
            |_| { errors += 1; },
        );
        
        callback.on_replay_start(Utc::now());
        callback.on_entry(&create_test_entry(), 0, 1);
        callback.on_effect(&create_test_effect(), &create_test_entry());
        callback.on_fact(&create_test_fact(), &create_test_entry());
        callback.on_error(&Error::Other("test".to_string()));
        
        assert_eq!(starts, 1);
        assert_eq!(entries, 1);
        assert_eq!(effects, 1);
        assert_eq!(facts, 1);
        assert_eq!(errors, 1);
    }
    
    // Test helpers
    fn create_test_entry() -> LogEntry {
        LogEntry {
            id: "entry_1".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: "test_event".to_string(),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({}),
                resources: None,
                domains: None,
            }),
            trace_id: Some("test_trace".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }
    
    fn create_test_effect() -> EffectEntry {
        EffectEntry {
            effect_type: causality_core::effect::runtime::EffectRuntime::EffectType::Transfer,
            resources: Vec::new(),
            domains: Vec::new(),
            code_hash: None,
            parameters: HashMap::new(),
            result: None,
            success: true,
            error: None,
        }
    }
    
    fn create_test_fact() -> FactEntry {
        FactEntry {
            domain: causality_types::DomainId::new(1),
            block_height: causality_types::BlockHeight::new(100),
            block_hash: None,
            observed_at: causality_types::Timestamp::now(),
            fact_type: "test".to_string(),
            resources: Vec::new(),
            data: serde_json::json!({}),
            verified: true,
        }
    }
    
    fn create_test_result() -> ReplayResult {
        ReplayResult {
            status: causality_engine::ReplayStatus::Complete,
            processed_entries: 1,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            error: None,
            state: None,
        }
    }
} 