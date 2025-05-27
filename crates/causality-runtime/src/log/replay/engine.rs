// Replay engine implementation
// Original file: src/log/replay/engine.rs

// Replay engine implementation for Causality Unified Log System
//
// This module provides the core engine for replaying log entries.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, trace, warn};
use std::str::FromStr;

use causality_error::{EngineResult, EngineError, Result as CausalityResult, CausalityError};
use crate::log::types::{LogEntry, EntryType, EntryData, FactEntry, EffectEntry};
use crate::log::storage::LogStorage;
use super::types::{ReplayStatus, ReplayResult, ReplayOptions};
use super::filter::ReplayFilter;
use super::callback::{ReplayCallback, NoopReplayCallback};
use super::state::{ReplayState, DomainState, ResourceState};
use causality_core::time::map::TimeMap;
use crate::log::segment_manager::LogSegmentManager;
use causality_types::{Timestamp, BlockHash, BlockHeight, DomainId, ContentId};
use async_trait::async_trait;
use causality_types::{Result as CausalityTypesResult, TraceId};

/// Integration between log entries and time map
pub trait LogTimeMapIntegration {
    /// Verify that a log entry is consistent with a time map
    fn verify_time_map(_entry: &LogEntry, _time_map: &TimeMap) -> EngineResult<bool>;
    
    /// Query entries in a time range using a time map
    fn query_time_range(
        _time_map: &TimeMap,
        storage: &dyn LogStorage,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> EngineResult<Vec<LogEntry>> {
        // Default implementation just reads all entries and filters by time
        let entries = storage.read(0, usize::MAX) // Read all entries for simplicity, consider batching
            .map_err(|e| EngineError::LogError(format!("Storage read error: {}", e)))?; // Map error here
            
        Ok(entries
            .into_iter()
            .filter(|entry| {
                let entry_time = entry.timestamp.to_datetime();
                entry_time >= start_time.to_datetime() && entry_time <= end_time.to_datetime()
            })
            .collect())
    }
}

/// Default implementation of log time map integration
pub struct DefaultLogTimeMapIntegration;

impl LogTimeMapIntegration for DefaultLogTimeMapIntegration {
    fn verify_time_map(_entry: &LogEntry, _time_map: &TimeMap) -> EngineResult<bool> {
        // For now, we'll just return true since we haven't implemented the actual verification logic
        Ok(true)
    }
}

/// The core engine for replaying log entries
pub struct ReplayEngine {
    /// The log storage to replay from
    storage: Arc<dyn LogStorage>,
    /// The replay options
    options: ReplayOptions,
    /// The replay callback
    callback: Arc<dyn ReplayCallback>,
    /// The current replay result
    result: Mutex<ReplayResult>,
    /// The time map used for verifying temporal consistency (if any)
    time_map: Option<TimeMap>,
    /// The segment manager for efficient log access (if any)
    segment_manager: Option<Arc<LogSegmentManager>>,
}

impl ReplayEngine {
    /// Create a new replay engine with the given storage, options, and callback
    pub fn new(
        storage: Arc<dyn LogStorage>,
        options: ReplayOptions,
        callback: Arc<dyn ReplayCallback>,
    ) -> Self {
        Self {
            storage,
            options,
            callback,
            result: Mutex::new(ReplayResult {
                status: ReplayStatus::Pending,
                processed_entries: 0,
                start_time: Utc::now(),
                end_time: None,
                error: None,
                state: None,
            }),
            time_map: None,
            segment_manager: None,
        }
    }
    
    /// Create a new replay engine with default options and a no-op callback
    pub fn with_storage(storage: Arc<dyn LogStorage>) -> Self {
        Self::new(
            storage,
            ReplayOptions::default(),
            Arc::new(NoopReplayCallback),
        )
    }
    
    /// Create a new replay engine with a time map for temporal verification
    pub fn with_time_map(
        storage: Arc<dyn LogStorage>,
        options: ReplayOptions,
        callback: Arc<dyn ReplayCallback>,
        time_map: TimeMap,
    ) -> Self {
        let mut engine = Self::new(storage, options, callback);
        engine.time_map = Some(time_map);
        engine
    }
    
    /// Create a new replay engine with a segment manager for efficient log access
    pub fn with_segment_manager(
        storage: Arc<dyn LogStorage>,
        options: ReplayOptions,
        callback: Arc<dyn ReplayCallback>,
        segment_manager: Arc<LogSegmentManager>,
    ) -> Self {
        let mut engine = Self::new(storage, options, callback);
        engine.segment_manager = Some(segment_manager);
        engine
    }
    
    /// Create a new replay engine with both time map and segment manager
    pub fn with_time_map_and_segment_manager(
        storage: Arc<dyn LogStorage>,
        options: ReplayOptions,
        callback: Arc<dyn ReplayCallback>,
        time_map: TimeMap,
        segment_manager: Arc<LogSegmentManager>,
    ) -> Self {
        let mut engine = Self::new(storage, options, callback);
        engine.time_map = Some(time_map);
        engine.segment_manager = Some(segment_manager);
        engine
    }
    
    /// Run the replay process
    pub fn run(&self) -> EngineResult<ReplayResult> {
        // Initialize the replay
        let start_time = Utc::now();
        self.callback.on_replay_start(start_time);
        
        // Update the result status
        {
            let mut result = self.result.lock().map_err(|_| {
                EngineError::LogError("Failed to acquire lock on replay result".to_string())
            })?;
            result.status = ReplayStatus::InProgress;
            result.start_time = start_time;
        }
        
        // Create a replay state
        let mut state = ReplayState::new();
        
        // Read entries in batches (consider making batch size configurable)
        let batch_size = 1000;
        let mut offset = 0;
        let entry_count = self.storage.entry_count()
            .map_err(|e| EngineError::LogError(format!("Storage entry count error: {}", e)))?; // Map error
        
        loop {
            let entries = self.storage.read(offset, batch_size)
                .map_err(|e| EngineError::LogError(format!("Storage read error during replay: {}", e)))?; // Map error
            
            if entries.is_empty() {
                break; // No more entries
            }
            
            // Process each entry
            for entry in &entries {
                if !self.should_include_entry(entry) {
                    continue;
                }
                
                // Call the entry callback
                if !self.callback.on_entry(entry, offset, entry_count) {
                    // Callback returned false, abort replay
                    {
                        let mut result = self.result.lock().map_err(|_| {
                            EngineError::LogError("Failed to acquire lock on replay result".to_string())
                        })?;
                        result.status = ReplayStatus::Complete;
                        result.processed_entries = offset;
                        result.end_time = Some(Utc::now());
                        result.state = Some(state.clone());
                    }
                    
                    return Ok(self.result()?)
                }
                
                // Verify time map consistency if a time map is provided
                if let Some(time_map) = &self.time_map {
                    if entry.entry_type == EntryType::Effect {
                        // Verify the entry's time map hash matches our time map
                        if !<DefaultLogTimeMapIntegration as LogTimeMapIntegration>::verify_time_map(entry, time_map)? {
                            // Time map verification failed
                            let error_string = format!(
                                "Time map verification failed for entry {}",
                                entry.id
                            );
                            
                            self.callback.on_error(&EngineError::LogError(error_string.clone()));
                            
                            {
                                let mut result = self.result.lock().map_err(|_| {
                                    EngineError::LogError("Failed to acquire lock on replay result".to_string())
                                })?;
                                result.status = ReplayStatus::Failed;
                                result.processed_entries = offset;
                                result.end_time = Some(Utc::now());
                                result.error = Some(error_string.clone());
                                result.state = Some(state.clone());
                            }
                            
                            return Err(EngineError::LogError(error_string));
                        }
                    }
                }
                
                // Process the entry
                if let Err(e) = self.process_entry(entry, &mut state) {
                    // Error processing entry
                    self.callback.on_error(&e);
                    
                    let error_string = format!("Error processing entry {}: {}", entry.id, e);
                    
                    {
                        let mut result = self.result.lock().map_err(|_| {
                            EngineError::LogError("Failed to acquire lock on replay result".to_string())
                        })?;
                        result.status = ReplayStatus::Failed;
                        result.processed_entries = offset;
                        result.end_time = Some(Utc::now());
                        result.error = Some(error_string.clone());
                        result.state = Some(state.clone());
                    }
                    
                    return Err(EngineError::LogError(error_string));
                }
                
                offset += entries.len();
            }
        }
        
        // Replay completed successfully
        self.complete_replay(state, None)
    }
    
    /// Run the replay with a specific filter
    pub fn run_with_filter(&self, filter: &ReplayFilter) -> EngineResult<ReplayResult> {
        // Convert the filter to ReplayOptions
        let options = ReplayOptions {
            start_time: filter.start_time,
            end_time: filter.end_time,
            trace_id: filter.trace_id.clone(),
            resources: if filter.resources.is_empty() {
                None
            } else {
                Some(filter.resources.iter().cloned().collect::<HashSet<_>>())
            },
            domains: if filter.domains.is_empty() {
                None
            } else {
                Some(filter.domains.iter().cloned().collect::<HashSet<_>>())
            },
            entry_types: if filter.entry_types.is_empty() {
                None
            } else {
                Some(filter.entry_types.iter().cloned().collect::<HashSet<_>>())
            },
            max_entries: self.options.max_entries,
        };
        
        // Create a new engine with the converted options
        let engine = ReplayEngine::new(
            self.storage.clone(),
            options,
            self.callback.clone(),
        );
        
        // Run the engine
        engine.run()
    }
    
    /// Run the replay with a time map time range filter
    pub fn run_with_time_range(
        &self,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> EngineResult<ReplayResult> {
        // Check if we have a time map
        if let Some(time_map) = &self.time_map {
            // Get entries within the time range
            let entries = <DefaultLogTimeMapIntegration as LogTimeMapIntegration>::query_time_range(
                time_map,
                &*self.storage,
                start_time,
                end_time
            )?;
            
            // Create a replay state
            let mut state = ReplayState::new();
            
            // Track processed entries
            let mut processed_entries = 0;
            
            // Process each entry
            for entry in &entries {
                // Call the entry callback
                if !self.callback.on_entry(entry, processed_entries, entries.len()) {
                    // Callback returned false, abort replay
                    return self.complete_replay(state, None);
                }
                
                // Process the entry
                if let Err(e) = self.process_entry(entry, &mut state) {
                    // Error processing entry
                    self.callback.on_error(&e);
                    
                    let error_string = format!("Error processing entry {}: {}", entry.id, e);
                    return self.complete_replay(state, Some(error_string));
                }
                
                processed_entries += 1;
            }
            
            // Replay completed successfully
            self.complete_replay(state, None)
        } else {
            Err(EngineError::LogError("Time map required for time range replay".to_string()))
        }
    }
    
    /// Run replay using the segment manager for time-based access
    fn run_with_segment_manager(
        &self,
        segment_manager: &Arc<LogSegmentManager>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        max_entries: usize,
        mut state: ReplayState,
    ) -> EngineResult<ReplayResult> {
        // Convert DateTime to Timestamp
        let start_timestamp = Timestamp::from_datetime(&start_time);
        let end_timestamp = Timestamp::from_datetime(&end_time);
        
        // Get entries in the time range from relevant segments
        let entries = segment_manager.get_entries_in_range(start_timestamp, end_timestamp)?;
        
        // Track processed entries
        let mut processed_entries = 0;
        let total_entries = entries.len();
        
        // Process each entry that matches our filters
        for entry in entries {
            // Stop if we've processed enough entries
            if processed_entries >= max_entries {
                break;
            }
            
            // Check if we should include this entry
            if !self.should_include_entry(&entry) {
                continue;
            }
            
            // Call the entry callback
            if !self.callback.on_entry(&entry, processed_entries, total_entries) {
                // Callback returned false, abort replay
                return self.complete_replay(state, None);
            }
            
            // Verify time map consistency if needed
            if let Some(time_map) = &self.time_map {
                if entry.entry_type == EntryType::Effect {
                    if !<DefaultLogTimeMapIntegration as LogTimeMapIntegration>::verify_time_map(&entry, time_map)? {
                        let error_string = format!(
                            "Time map verification failed for entry {}",
                            entry.id
                        );
                        
                        self.callback.on_error(&EngineError::LogError(error_string.clone()));
                        return self.complete_replay(
                            state, 
                            Some(error_string)
                        );
                    }
                }
            }
            
            // Process the entry
            if let Err(e) = self.process_entry(&entry, &mut state) {
                // Error processing entry
                self.callback.on_error(&e);
                
                let error_string = format!("Error processing entry {}: {}", entry.id, e);
                return self.complete_replay(
                    state, 
                    Some(error_string)
                );
            }
            
            processed_entries += 1;
        }
        
        // Replay completed successfully
        self.complete_replay(state, None)
    }
    
    /// Check if an entry should be included in the replay
    fn should_include_entry(&self, entry: &LogEntry) -> bool {
        // Check entry type filter
        if let Some(entry_types) = &self.options.entry_types {
            if !entry_types.contains(&entry.entry_type) {
                return false;
            }
        }
        
        // Check time range
        if let Some(start_time) = self.options.start_time {
            let entry_time = entry.timestamp.to_datetime();
            if entry_time < start_time {
                return false;
            }
        }
        
        if let Some(end_time) = self.options.end_time {
            let entry_time = entry.timestamp.to_datetime();
            if entry_time > end_time {
                return false;
            }
        }
        
        // Check trace ID
        if let Some(trace_id) = &self.options.trace_id {
            if entry.trace_id.as_ref().map(|t| t.as_str()) != Some(trace_id) {
                return false;
            }
        }
        
        // Check resources
        if let Some(resources) = &self.options.resources {
            match &entry.data {
                EntryData::Effect(effect) => {
                    // TODO: Revisit resource filtering. EffectEntry no longer has resources.
                    /*
                    if let Some(effect_resources) = &effect.resources {
                        if !resources.iter().any(|r| effect_resources.contains(r)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    */
                    // Temporary: Allow entry if resource filter exists, needs refinement.
                    // return false;
                }
                EntryData::Fact(fact) => {
                    // TODO: Revisit resource filtering. FactEntry no longer has resources.
                    /*
                    if let Some(fact_resources) = &fact.resources {
                        if !resources.iter().any(|r| fact_resources.contains(r)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    */
                    // Temporary: Allow entry if resource filter exists, needs refinement.
                     // return false;
                }
                EntryData::Event(event) => {
                    if let Some(event_resources) = &event.resources {
                        if !resources.iter().any(|r| event_resources.contains(r)) {
                            return false;
                        }
                    } else {
                        // If event has no resources but filter requires them
                        return false;
                    }
                }
                EntryData::ResourceAccess(ra) => {
                    // Attempt to parse resource_id
                    if let Ok(content_id) = ContentId::from_str(&ra.resource_id) { 
                        if !resources.contains(&content_id) {
                            return false;
                        }
                    } else {
                        // If parsing fails, treat as non-match
                        return false;
                    }
                }
                _ => { /* Other types might not have resources, allow for now unless filter is strict */ }
            }
        }
        
        // Check domains
        if let Some(domains) = &self.options.domains {
            match &entry.data {
                EntryData::Effect(effect) => {
                     // EffectEntry now has domain_id directly
                    if !domains.contains(&effect.domain_id) {
                        return false;
                    }
                }
                EntryData::Fact(fact) => {
                     // FactEntry now has domain_id directly
                    if !domains.contains(&fact.domain_id) {
                        return false;
                    }
                }
                // TODO: Check other EntryData variants that might have domains (e.g., Event, Operation)
                EntryData::Event(event) => {
                    if let Some(event_domains) = &event.domains {
                        if !domains.iter().any(|d| event_domains.contains(d)) {
                            return false;
                        }
                    } else {
                        // If event has no domains but filter requires them
                        return false;
                    }
                }
                 EntryData::Operation(op) => {
                    // OperationEntry has domain_id via metadata or needs structure change
                    // Assuming metadata for now, or return false if filter is strict
                    if let Some(op_domain_str) = entry.metadata.get("domain_id") {
                        if let Ok(op_domain_id) = DomainId::from_str(op_domain_str) {
                            if !domains.contains(&op_domain_id) {
                                return false;
                            }
                        } else {
                           return false; // Cannot parse domain ID from metadata
                        }
                    } else {
                        return false; // No domain ID in metadata
                    }
                }
                _ => { /* Other types might not have domains */ }
            }
        }
        
        true
    }

    /// Process a single log entry
    fn process_entry(&self, entry: &LogEntry, state: &mut ReplayState) -> EngineResult<()> {
        // Filters are applied in `should_include_entry`, no need for options.filter check here

        // Process entry based on type
        match &entry.data {
            EntryData::Fact(fact) => {
                let timestamp = entry.timestamp;
                // Use callback directly
                self.callback.on_fact(fact, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_fact failed: {}", e)))?;
            },
            EntryData::Effect(effect) => {
                let timestamp = entry.timestamp;
                // Use callback directly
                self.callback.on_effect(effect, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_effect failed: {}", e)))?;
            },
            EntryData::ResourceAccess(ra) => {
                 let timestamp = entry.timestamp;
                 // Use callback directly
                 self.callback.on_resource_access(ra, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_resource_access failed: {}", e)))?;
            },
             EntryData::SystemEvent(se) => {
                 let timestamp = entry.timestamp;
                 // Use callback directly
                 self.callback.on_system_event(se, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_system_event failed: {}", e)))?;
            },
             EntryData::Operation(op) => {
                 let timestamp = entry.timestamp;
                 // Use callback directly
                 self.callback.on_operation(op, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_operation failed: {}", e)))?;
            },
             EntryData::Event(event) => { // Handle Event explicitly
                 let timestamp = entry.timestamp;
                 // Use callback directly (assuming on_event exists in ReplayCallback)
                 self.callback.on_event(event, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_event failed: {}", e)))?;
             },
             EntryData::Custom(custom_type, data) => { // Match both fields
                 let timestamp = entry.timestamp;
                 // Use callback directly
                 self.callback.on_custom_entry(custom_type, data, &entry.metadata, timestamp)
                    .map_err(|e| EngineError::CallbackError(format!("on_custom_entry failed: {}", e)))?;
             }
        }

        Ok(())
    }

    /// Finalize the replay result
    fn complete_replay(&self, state: ReplayState, error: Option<String>) -> EngineResult<ReplayResult> {
        let end_time = Utc::now();
        let status = if error.is_some() { 
            ReplayStatus::Failed 
        } else { 
            ReplayStatus::Complete 
        };
        
        // Call the callback
        self.callback.on_replay_end(end_time, &status);
        
        // Lock and update the result
        let mut result = self.result.lock().map_err(|_| {
            EngineError::LogError("Failed to acquire lock on replay result".to_string())
        })?;
        
        result.status = status;
        result.end_time = Some(end_time);
        result.state = Some(state);
        
        if let Some(e) = error {
            result.error = Some(e.clone());
            return Err(EngineError::LogError(e));
        }
        
        // Clone the result before releasing the lock
        let result_clone = result.clone();
        
        Ok(result_clone)
    }
    
    /// Get the current replay result
    pub fn result(&self) -> EngineResult<ReplayResult> {
        self.result.lock()
            .map_err(|_| EngineError::LogError("Failed to acquire lock on replay result".to_string()))
            .map(|result| result.clone())
    }

    /// Query a time range of entries (default implementation)
    fn query_time_range(&self, storage: &dyn LogStorage, start_time: u64, end_time: u64) -> EngineResult<Vec<LogEntry>> {
        // Call the underlying storage's read_time_range method
        storage.read_time_range(start_time, end_time)
            .map_err(|e| EngineError::LogError(format!("Storage read error: {}", e))) // Map the CausalityResult error to EngineError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::memory_storage::MemoryLogStorage;
    use crate::log::types::{create_fact_observation, create_domain_effect, BorshJsonValue};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::runtime::Runtime;
    use serde_json::json;
    
    // Mock LogStorage for testing
    #[derive(Clone, Debug)]
    struct MockLogStorage {
        entries: Arc<Mutex<Vec<LogEntry>>>,
    }

    impl MockLogStorage {
        fn new() -> Self {
            MockLogStorage { entries: Arc::new(Mutex::new(Vec::new())) }
        }
    }

    #[async_trait]
    impl LogStorage for MockLogStorage {
        // --- Async Methods ---
        async fn append_entry(&self, entry: LogEntry) -> CausalityResult<()> {
            let mut entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            entries.push(entry);
            Ok(())
        }
        
        async fn get_all_entries(&self) -> CausalityResult<Vec<LogEntry>> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.clone())
        }
        
        async fn get_entries(&self, start: usize, end: usize) -> CausalityResult<Vec<LogEntry>> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            let count = entries.len();
            let real_end = std::cmp::min(end, count);
            if start >= real_end {
                return Ok(Vec::new());
            }
            Ok(entries[start..real_end].to_vec())
        }
        
        async fn get_entry_count(&self) -> CausalityResult<usize> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.len())
        }
        
        async fn clear(&self) -> CausalityResult<()> {
            let mut entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            entries.clear();
            Ok(())
        }
        
        async fn async_flush(&self) -> CausalityResult<()> {
            // No-op for mock
            Ok(())
        }

        // --- Sync Methods ---
        // Sync methods return CausalityResult<_, Box<dyn CausalityError + Send + Sync + 'static>>
        fn append(&self, entry: LogEntry) -> CausalityResult<()> { // Return CausalityResult
            let mut entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            entries.push(entry);
            Ok(())
        }
        
        fn entry_count(&self) -> CausalityResult<usize> { // Return CausalityResult
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.len())
        }

        fn read(&self, offset: usize, limit: usize) -> CausalityResult<Vec<LogEntry>> { // Return CausalityResult
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            let count = entries.len();
            let end = std::cmp::min(offset + limit, count);
            if offset >= end {
                return Ok(Vec::new());
            }
            Ok(entries[offset..end].to_vec())
        }
        
        fn append_batch(&self, batch: Vec<LogEntry>) -> CausalityResult<()> { // Return CausalityResult
             let mut entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
             entries.extend(batch);
             Ok(())
        }
        
        fn get_entry_by_id(&self, id: &str) -> CausalityResult<Option<LogEntry>> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.iter().find(|e| e.id == id).cloned())
        }
        
        fn get_entries_by_trace(&self, trace_id: &str) -> CausalityResult<Vec<LogEntry>> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.iter()
                .filter(|e| e.trace_id.as_ref().map_or(false, |tid| tid.to_string() == trace_id))
                .cloned()
                .collect())
        }

        fn find_entries_by_type(&self, entry_type: EntryType) -> CausalityResult<Vec<LogEntry>> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.iter().filter(|e| e.entry_type == entry_type).cloned().collect())
        }
        
        fn read_time_range(&self, start_time: u64, end_time: u64) -> CausalityResult<Vec<LogEntry>> {
            let entries = self.entries.lock().map_err(|e| Box::new(EngineError::SyncError(format!("Mutex poison: {}", e))) as Box<dyn CausalityError>)?; // Map error
            Ok(entries.iter()
                .filter(|e| e.timestamp.to_millis() >= start_time && e.timestamp.to_millis() < end_time)
                .cloned()
                .collect())
        }
        
        // These might not be needed for mock, but stub them if required by trait
        fn rotate(&self) -> CausalityResult<()> { Ok(()) } 
        fn compact(&self) -> CausalityResult<()> { Ok(()) }
        fn close(&self) -> CausalityResult<()> { Ok(()) }
    }

    // ... rest of the tests ...
} 