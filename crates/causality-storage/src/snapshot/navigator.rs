// Snapshot navigation utilities
// Original file: src/snapshot/navigator.rs

// Time-travel navigation for Causality Content-Addressed Code System
//
// This module provides functionality for navigating execution history
// bidirectionally, enabling powerful time-travel debugging.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, Duration};

use causality_types::{Error, Result};
use causality_engine::{ExecutionContext, ExecutionEvent, Value, ContextId};
use causality_engine::ExecutionTracer;
use causality_engine::ExecutionReplayer;
use causality_storage::{SnapshotManager, SnapshotId, ExecutionSnapshot};

/// Error type for time-travel debugging operations
#[derive(Debug, Clone)]
pub enum DebugError {
    /// Error during snapshot operations
    SnapshotError(String),
    /// Error during replay
    ReplayError(String),
    /// Invalid position
    InvalidPosition(String),
    /// State inspection error
    InspectionError(String),
    /// General error
    GeneralError(String),
}

impl From<Error> for DebugError {
    fn from(err: Error) -> Self {
        DebugError::GeneralError(format!("{:?}", err))
    }
}

/// Interface for time-travel debugging
pub trait TimeTravel: Send + Sync {
    /// Step forward by one event
    fn step_forward(
        &self,
        context: &mut ExecutionContext,
    ) -> std::result::Result<ExecutionEvent, DebugError>;
    
    /// Step backward by one event
    fn step_backward(
        &self,
        context: &mut ExecutionContext,
    ) -> std::result::Result<ExecutionEvent, DebugError>;
    
    /// Jump to a specific point in the execution trace
    fn jump_to_position(
        &self,
        context: &mut ExecutionContext,
        position: usize,
    ) -> std::result::Result<ExecutionEvent, DebugError>;
    
    /// Jump to a specific effect application
    fn jump_to_effect(
        &self,
        context: &mut ExecutionContext,
        effect_type: &str,
        occurrence: usize,
    ) -> std::result::Result<ExecutionEvent, DebugError>;
    
    /// Inspect the state at the current position
    fn inspect_state(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<HashMap<String, Value>, DebugError>;
    
    /// Compare state between two execution points
    fn compare_states(
        &self,
        context: &ExecutionContext,
        position1: usize,
        position2: usize,
    ) -> std::result::Result<HashMap<String, (Option<Value>, Option<Value>)>, DebugError>;
}

/// Implementation of the TimeTravel trait
pub struct TimeTravelNavigator {
    /// The snapshot manager to use for state persistence
    snapshot_manager: Arc<dyn SnapshotManager>,
    /// The execution tracer to use for trace retrieval
    tracer: Arc<ExecutionTracer>,
    /// The execution replayer to use for trace replay
    replayer: Arc<ExecutionReplayer>,
    /// Current positions for each context
    positions: RwLock<HashMap<ContextId, usize>>,
    /// Cache of context snapshots
    snapshot_cache: RwLock<HashMap<(ContextId, usize), SnapshotId>>,
    /// Snapshot interval (how many events between snapshots)
    snapshot_interval: usize,
}

impl TimeTravelNavigator {
    /// Create a new time-travel navigator
    pub fn new(
        snapshot_manager: Arc<dyn SnapshotManager>,
        tracer: Arc<ExecutionTracer>,
        replayer: Arc<ExecutionReplayer>,
    ) -> Self {
        TimeTravelNavigator {
            snapshot_manager,
            tracer,
            replayer,
            positions: RwLock::new(HashMap::new()),
            snapshot_cache: RwLock::new(HashMap::new()),
            snapshot_interval: 100,  // Take a snapshot every 100 events by default
        }
    }
    
    /// Set the snapshot interval
    pub fn with_snapshot_interval(mut self, interval: usize) -> Self {
        self.snapshot_interval = interval;
        self
    }
    
    /// Get the current position for a context
    fn get_position(&self, context: &ExecutionContext) -> std::result::Result<usize, DebugError> {
        let positions = self.positions.read().map_err(|_| 
            DebugError::GeneralError("Failed to lock positions".to_string()))?;
        
        let context_id = context.id().clone();
        Ok(*positions.get(&context_id).unwrap_or(&0))
    }
    
    /// Set the current position for a context
    fn set_position(&self, context: &ExecutionContext, position: usize) -> std::result::Result<(), DebugError> {
        let mut positions = self.positions.write().map_err(|_| 
            DebugError::GeneralError("Failed to lock positions".to_string()))?;
        
        let context_id = context.id().clone();
        positions.insert(context_id, position);
        
        Ok(())
    }
    
    /// Create a snapshot of the context at the current position
    fn create_snapshot(&self, context: &ExecutionContext, position: usize) -> std::result::Result<SnapshotId, DebugError> {
        // Create the snapshot
        let snapshot_id = self.snapshot_manager.create_snapshot(context)
            .map_err(|e| DebugError::SnapshotError(format!("Failed to create snapshot: {:?}", e)))?;
        
        // Cache the snapshot ID
        let mut cache = self.snapshot_cache.write().map_err(|_| 
            DebugError::GeneralError("Failed to lock snapshot cache".to_string()))?;
        
        let key = (context.id().clone(), position);
        cache.insert(key, snapshot_id.clone());
        
        Ok(snapshot_id)
    }
    
    /// Find the closest snapshot before a given position
    fn find_nearest_snapshot(
        &self,
        context: &ExecutionContext,
        target_position: usize,
    ) -> std::result::Result<Option<(SnapshotId, usize)>, DebugError> {
        // Get all snapshots for this context
        let context_id = context.id().clone();
        let snapshots = self.snapshot_manager.list_snapshots(&context_id)
            .map_err(|e| DebugError::SnapshotError(format!("Failed to list snapshots: {:?}", e)))?;
        
        // Find the closest snapshot before the target position
        let mut best_snapshot = None;
        let mut best_position = 0;
        
        for snapshot in snapshots {
            let position = snapshot.execution_position;
            if position <= target_position && position > best_position {
                best_snapshot = Some(snapshot.snapshot_id.clone());
                best_position = position;
            }
        }
        
        if let Some(snapshot_id) = best_snapshot {
            Ok(Some((snapshot_id, best_position)))
        } else {
            Ok(None)
        }
    }
    
    /// Move the context to a specific position
    fn move_to_position(
        &self,
        context: &mut ExecutionContext,
        target_position: usize,
    ) -> std::result::Result<ExecutionEvent, DebugError> {
        // Get current position
        let current_position = self.get_position(context)?;
        
        // If we're already at the target position, just return the current event
        if current_position == target_position {
            let events = context.execution_trace()
                .map_err(|e| DebugError::ReplayError(format!("Failed to get execution trace: {:?}", e)))?;
            
            if target_position < events.len() {
                return Ok(events[target_position].clone());
            } else {
                return Err(DebugError::InvalidPosition(format!(
                    "Target position {} is beyond trace length {}", target_position, events.len()
                )));
            }
        }
        
        // If we need to move backward, we need to restore from a snapshot
        if target_position < current_position {
            // Find nearest snapshot
            let nearest = self.find_nearest_snapshot(context, target_position)?;
            
            if let Some((snapshot_id, snapshot_position)) = nearest {
                // Restore from snapshot
                *context = self.snapshot_manager.restore_snapshot(&snapshot_id)
                    .map_err(|e| DebugError::SnapshotError(format!("Failed to restore snapshot: {:?}", e)))?;
                
                // Replay from snapshot position to target position
                if snapshot_position < target_position {
                    for _ in snapshot_position..target_position {
                        self.replayer.step_forward(context)
                            .map_err(|e| DebugError::ReplayError(format!("Failed to step forward: {:?}", e)))?;
                    }
                }
            } else {
                // No snapshot available, reset to beginning and replay
                // This would be inefficient in practice, but it's a fallback
                *context = self.replayer.reset_to_beginning(context.id().clone())
                    .map_err(|e| DebugError::ReplayError(format!("Failed to reset to beginning: {:?}", e)))?;
                
                for _ in 0..target_position {
                    self.replayer.step_forward(context)
                        .map_err(|e| DebugError::ReplayError(format!("Failed to step forward: {:?}", e)))?;
                }
            }
        } else { // Moving forward is simpler, just replay
            for _ in current_position..target_position {
                self.replayer.step_forward(context)
                    .map_err(|e| DebugError::ReplayError(format!("Failed to step forward: {:?}", e)))?;
            }
        }
        
        // Update position
        self.set_position(context, target_position)?;
        
        // Take a snapshot if we're at an interval
        if target_position % self.snapshot_interval == 0 {
            let _ = self.create_snapshot(context, target_position);
        }
        
        // Get the event at the current position
        let events = context.execution_trace()
            .map_err(|e| DebugError::ReplayError(format!("Failed to get execution trace: {:?}", e)))?;
        
        if target_position < events.len() {
            Ok(events[target_position].clone())
        } else {
            Err(DebugError::InvalidPosition(format!(
                "Target position {} is beyond trace length {}", target_position, events.len()
            )))
        }
    }
}

impl TimeTravel for TimeTravelNavigator {
    fn step_forward(
        &self,
        context: &mut ExecutionContext,
    ) -> std::result::Result<ExecutionEvent, DebugError> {
        let current_position = self.get_position(context)?;
        self.jump_to_position(context, current_position + 1)
    }
    
    fn step_backward(
        &self,
        context: &mut ExecutionContext,
    ) -> std::result::Result<ExecutionEvent, DebugError> {
        let current_position = self.get_position(context)?;
        if current_position == 0 {
            return Err(DebugError::InvalidPosition("Already at the beginning of the trace".to_string()));
        }
        
        self.jump_to_position(context, current_position - 1)
    }
    
    fn jump_to_position(
        &self,
        context: &mut ExecutionContext,
        position: usize,
    ) -> std::result::Result<ExecutionEvent, DebugError> {
        self.move_to_position(context, position)
    }
    
    fn jump_to_effect(
        &self,
        context: &mut ExecutionContext,
        effect_type: &str,
        occurrence: usize,
    ) -> std::result::Result<ExecutionEvent, DebugError> {
        // Get the execution trace
        let events = context.execution_trace()
            .map_err(|e| DebugError::ReplayError(format!("Failed to get execution trace: {:?}", e)))?;
        
        // Find the target effect
        let mut found_count = 0;
        for (i, event) in events.iter().enumerate() {
            if let ExecutionEvent::EffectApplied { effect_type: event_type, .. } = event {
                if event_type.as_str() == effect_type {
                    found_count += 1;
                    if found_count > occurrence {
                        return self.jump_to_position(context, i);
                    }
                }
            }
        }
        
        Err(DebugError::InvalidPosition(format!(
            "Effect {} occurrence {} not found in trace", effect_type, occurrence
        )))
    }
    
    fn inspect_state(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<HashMap<String, Value>, DebugError> {
        // Get the variable bindings for the current state
        let bindings = context.get_variable_bindings()
            .map_err(|e| DebugError::InspectionError(format!("Failed to get variable bindings: {:?}", e)))?;
        
        Ok(bindings)
    }
    
    fn compare_states(
        &self,
        context: &ExecutionContext,
        position1: usize,
        position2: usize,
    ) -> std::result::Result<HashMap<String, (Option<Value>, Option<Value>)>, DebugError> {
        // Create a mutable clone of the context to navigate
        let mut context_clone = context.clone();
        
        // Jump to position1 and get the state
        self.jump_to_position(&mut context_clone, position1)?;
        let state1 = self.inspect_state(&context_clone)?;
        
        // Jump to position2 and get the state
        self.jump_to_position(&mut context_clone, position2)?;
        let state2 = self.inspect_state(&context_clone)?;
        
        // Combine all keys from both states
        let mut all_keys = state1.keys().collect::<Vec<_>>();
        for key in state2.keys() {
            if !all_keys.contains(&key) {
                all_keys.push(key);
            }
        }
        
        // Build the diff map
        let mut diff = HashMap::new();
        for key in all_keys {
            let value1 = state1.get(key).cloned();
            let value2 = state2.get(key).cloned();
            
            // Only include in diff if the values are different
            if value1 != value2 {
                diff.insert(key.clone(), (value1, value2));
            }
        }
        
        Ok(diff)
    }
}

// Add tests for the TimeTravel implementation
#[cfg(test)]
mod tests {
    use super::*;
    use causality_storage::manager::MockSnapshotManager;
    use causality_engine::MockExecutionTracer;
    use causality_engine::MockExecutionReplayer;
    
    // This would be a real test in the actual implementation
    // For now, we'll just have a placeholder
    #[test]
    fn test_step_forward() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_step_backward() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_jump_to_position() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_jump_to_effect() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_inspect_state() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_compare_states() {
        // Test implementation would go here
    }
} 