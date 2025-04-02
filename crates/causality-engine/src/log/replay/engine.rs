// Replay engine implementation
// Original file: src/log/replay/engine.rs

// Replay engine implementation for Causality Unified Log System
//
// This module provides the core engine for replaying log entries.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use chrono::Utc;

use causality_error::{EngineResult, EngineError};
use crate::log::entry::{LogEntry, EntryType, EntryData};
use crate::log::storage::LogStorage;
use crate::log::replay_impl::{
    ReplayStatus, ReplayResult, ReplayOptions
};
use crate::log::replay::filter::ReplayFilter;
use crate::log::replay::callback::{ReplayCallback, NoopReplayCallback};
use crate::log::replay::state::{ReplayState, DomainState, ResourceState};
use crate::log::time_map::{TimeMap, LogTimeMapIntegration};
use crate::log::segment_manager::LogSegmentManager;
use causality_types::Timestamp;

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
        
        // Get the total number of entries
        let total_entries = self.storage.entry_count()?;
        
        // Create a replay state
        let mut state = ReplayState::new();
        
        // Determine max entries to process
        let max_entries = self.options.max_entries.unwrap_or(total_entries);
        
        // If we have a segment manager and time bounds, use it for more efficient replay
        if let Some(segment_manager) = &self.segment_manager {
            if let (Some(start), Some(end)) = (self.options.start_time, self.options.end_time) {
                return self.run_with_segment_manager(segment_manager, start, end, max_entries, state);
            }
        }
        
        // Read entries in batches of 100
        let batch_size = 100;
        let mut processed_entries = 0;
        let mut current_offset = 0;
        
        while processed_entries < max_entries {
            // Read a batch of entries
            let entries = self.storage.read(current_offset, batch_size)?;
            
            // Stop if there are no more entries
            if entries.is_empty() {
                break;
            }
            
            // Process each entry
            for entry in &entries {
                if !self.should_include_entry(entry) {
                    continue;
                }
                
                // Call the entry callback
                if !self.callback.on_entry(entry, processed_entries, total_entries) {
                    // Callback returned false, abort replay
                    {
                        let mut result = self.result.lock().map_err(|_| {
                            EngineError::LogError("Failed to acquire lock on replay result".to_string())
                        })?;
                        result.status = ReplayStatus::Complete;
                        result.processed_entries = processed_entries;
                        result.end_time = Some(Utc::now());
                        result.state = Some(state.clone());
                    }
                    
                    return Ok(self.result()?)
                }
                
                // Verify time map consistency if a time map is provided
                if let Some(time_map) = &self.time_map {
                    if entry.entry_type == EntryType::Effect {
                        // Verify the entry's time map hash matches our time map
                        if !LogTimeMapIntegration::verify_time_map(entry, time_map)? {
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
                                result.processed_entries = processed_entries;
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
                        result.processed_entries = processed_entries;
                        result.end_time = Some(Utc::now());
                        result.error = Some(error_string.clone());
                        result.state = Some(state.clone());
                    }
                    
                    return Err(EngineError::LogError(error_string));
                }
                
                processed_entries += 1;
                
                if processed_entries >= max_entries {
                    break;
                }
            }
            
            current_offset += entries.len();
        }
        
        // Replay completed successfully
        self.complete_replay(state, None)
    }
    
    /// Run the replay with a specific filter
    pub fn run_with_filter(&self, filter: &ReplayFilter) -> EngineResult<ReplayResult> {
        // Convert the filter to options
        let options = ReplayOptions {
            start_time: filter.start_time,
            end_time: filter.end_time,
            trace_id: filter.trace_id.clone(),
            resources: filter.resources.clone(),
            domains: filter.domains.clone(),
            entry_types: filter.entry_types.clone(),
            max_entries: self.options.max_entries,
        };
        
        // Create a new engine with the updated options
        let mut engine = ReplayEngine::new(
            self.storage.clone(),
            options,
            self.callback.clone(),
        );
        
        // Copy the time map if present
        if let Some(time_map) = &self.time_map {
            engine.time_map = Some(time_map.clone());
        }
        
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
            let entries = LogTimeMapIntegration::query_time_range(
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
        start_time: Timestamp,
        end_time: Timestamp,
        max_entries: usize,
        mut state: ReplayState,
    ) -> EngineResult<ReplayResult> {
        // Get entries in the time range from relevant segments
        let entries = segment_manager.get_entries_in_range(start_time, end_time)?;
        
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
                    if !LogTimeMapIntegration::verify_time_map(&entry, time_map)? {
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
            if entry.timestamp < start_time {
                return false;
            }
        }
        
        if let Some(end_time) = self.options.end_time {
            if entry.timestamp > end_time {
                return false;
            }
        }
        
        // Check trace ID
        if let Some(trace_id) = &self.options.trace_id {
            if entry.trace_id.as_ref() != Some(trace_id) {
                return false;
            }
        }
        
        // Check resources
        if let Some(resources) = &self.options.resources {
            match &entry.data {
                EntryData::Effect(effect) => {
                    if let Some(effect_resources) = &effect.resources {
                        if !resources.iter().any(|r| effect_resources.contains(r)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                EntryData::Fact(fact) => {
                    if let Some(fact_resources) = &fact.resources {
                        if !resources.iter().any(|r| fact_resources.contains(r)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        
        // Check domains
        if let Some(domains) = &self.options.domains {
            match &entry.data {
                EntryData::Effect(effect) => {
                    if let Some(effect_domains) = &effect.domains {
                        if !domains.iter().any(|d| effect_domains.contains(d)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                EntryData::Fact(fact) => {
                    if let Some(fact_domains) = &fact.domains {
                        if !domains.iter().any(|d| fact_domains.contains(d)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        
        true
    }

    /// Process a single log entry
    fn process_entry(&self, entry: &LogEntry, state: &mut ReplayState) -> EngineResult<()> {
        match entry.entry_type {
            EntryType::Fact => {
                if let EntryData::Fact(fact) = &entry.data {
                    // Update domain state if fact contains domain information
                    if let Some(domains) = &fact.domains {
                        for domain_id in domains {
                            let entry_id = entry.id.clone();
                            
                            // Create or update domain state
                            if let Some(domain_state) = state.domains.get_mut(domain_id) {
                                // Only update if this fact represents a later block
                                if fact.height > domain_state.height {
                                    domain_state.update(
                                        fact.height.clone(),
                                        Some(fact.hash.clone()),
                                        fact.timestamp.clone(),
                                        entry_id
                                    );
                                }
                            } else {
                                // Create new domain state
                                let mut domain_state = DomainState::new(
                                    domain_id.clone(), 
                                    entry_id
                                );
                                domain_state.update(
                                    fact.height.clone(),
                                    Some(fact.hash.clone()),
                                    fact.timestamp.clone(),
                                    entry_id
                                );
                                state.domains.insert(domain_id.clone(), domain_state);
                            }
                        }
                    }
                    
                    // Add fact to state
                    state.facts.push(fact.clone());
                    
                    // Call fact callback
                    self.callback.on_fact(fact, entry);
                    
                    Ok(())
                } else {
                    Err(EngineError::LogError("Invalid fact entry data".to_string()))
                }
            },
            EntryType::Effect => {
                if let EntryData::Effect(effect) = &entry.data {
                    // Process resources affected by this effect
                    if let Some(resources) = &effect.resources {
                        for resource_id in resources {
                            let entry_id = entry.id.clone();
                            
                            // Create or update resource state
                            if !state.resources.contains_key(resource_id) {
                                // Create new resource state
                                let resource_state = ResourceState::new(
                                    resource_id.clone(), 
                                    entry_id
                                );
                                state.resources.insert(resource_id.clone(), resource_state);
                            } else {
                                // Update existing resource state
                                let resource_state = state.resources.get_mut(resource_id).unwrap();
                                resource_state.update_modification(entry_id);
                            }
                        }
                    }
                    
                    // Add effect to state
                    state.effects.push(effect.clone());
                    
                    // Call effect callback
                    self.callback.on_effect(effect, entry);
                    
                    Ok(())
                } else {
                    Err(EngineError::LogError("Invalid effect entry data".to_string()))
                }
            },
            EntryType::Event => {
                if let EntryData::Event(event) = &entry.data {
                    // Call event callback
                    // The ReplayCallback trait doesn't have an on_event method
                    // but we can notify about the entry
                    // self.callback.on_event(event);
                    
                    Ok(())
                } else {
                    Err(EngineError::LogError("Invalid event entry data".to_string()))
                }
            }
        }
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
            result.error = Some(e);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use chrono::Utc;
    use crate::log::memory_storage::MemoryLogStorage;
    use crate::log::entry::{EntryType, EntryData, EventEntry, EventSeverity};
    
    #[test]
    fn test_replay_empty_log() -> EngineResult<()> {
        let storage = Arc::new(MemoryLogStorage::new());
        let engine = ReplayEngine::with_storage(storage);
        
        let result = engine.run()?;
        
        assert_eq!(result.status, ReplayStatus::Complete);
        assert_eq!(result.processed_entries, 0);
        assert!(result.error.is_none());
        
        Ok(())
    }
    
    // Add more test cases...
} 