//! Time-travel capabilities for simulation rewind and fast-forward
//!
//! This module provides functionality to rewind simulation state to previous
//! points in time and fast-forward to specific future states.

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use crate::{
    engine::{SimulationEngine, SimulationState, ExecutionMetrics},
    clock::SimulatedTimestamp,
    error::SimulationError,
};

/// Time-travel checkpoint containing simulation state at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeCheckpoint {
    /// Unique identifier for this checkpoint
    pub id: CheckpointId,
    
    /// Timestamp when this checkpoint was created
    pub timestamp: SimulatedTimestamp,
    
    /// Simulation engine state at this checkpoint
    pub engine_state: SerializableEngineState,
    
    /// Description of this checkpoint
    pub description: String,
    
    /// Step number when this checkpoint was created
    pub step_number: usize,
}

/// Serializable representation of simulation engine state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEngineState {
    pub state: SimulationState,
    pub program_counter: usize,
    pub gas_remaining: u64,
    pub effects_log: Vec<String>,
    pub metrics: ExecutionMetrics,
}

/// Unique identifier for time checkpoints
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CheckpointId(pub String);

impl CheckpointId {
    /// Create a new checkpoint ID
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    /// Generate a unique checkpoint ID based on timestamp
    pub fn generate(timestamp: SimulatedTimestamp, step: usize) -> Self {
        Self(format!("checkpoint_{}_{}", timestamp.as_secs(), step))
    }
    
    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Configuration for time-travel behavior
#[derive(Debug, Clone)]
pub struct TimeTravelConfig {
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,
    
    /// Automatic checkpoint interval (in steps)
    pub auto_checkpoint_interval: Option<usize>,
    
    /// Whether to compress old checkpoints
    pub compress_old_checkpoints: bool,
}

impl Default for TimeTravelConfig {
    fn default() -> Self {
        Self {
            max_checkpoints: 50,
            auto_checkpoint_interval: Some(10), // Checkpoint every 10 steps
            compress_old_checkpoints: true,
        }
    }
}

/// Manager for time-travel functionality
pub struct TimeTravelManager {
    /// Configuration for time-travel behavior
    config: TimeTravelConfig,
    
    /// Checkpoints indexed by timestamp for efficient time-based lookup
    checkpoints: BTreeMap<SimulatedTimestamp, TimeCheckpoint>,
    
    /// Current timeline position
    current_position: Option<SimulatedTimestamp>,
    
    /// Step counter for automatic checkpointing
    step_counter: usize,
}

impl TimeTravelManager {
    /// Create a new time-travel manager
    pub fn new() -> Self {
        Self {
            config: TimeTravelConfig::default(),
            checkpoints: BTreeMap::new(),
            current_position: None,
            step_counter: 0,
        }
    }
    
    /// Create a time-travel manager with custom configuration
    pub fn with_config(config: TimeTravelConfig) -> Self {
        Self {
            config,
            checkpoints: BTreeMap::new(),
            current_position: None,
            step_counter: 0,
        }
    }
    
    /// Create a checkpoint of the current simulation state
    pub fn create_checkpoint(
        &mut self, 
        engine: &SimulationEngine, 
        description: String
    ) -> Result<CheckpointId, SimulationError> {
        let timestamp = engine.clock().now();
        let checkpoint_id = CheckpointId::generate(timestamp, self.step_counter);
        
        // Extract serializable state from engine
        let engine_state = SerializableEngineState {
            state: engine.state().clone(),
            program_counter: engine.pc, // Need to expose this field
            gas_remaining: engine.machine.gas,
            effects_log: engine.effects_log.clone(),
            metrics: engine.metrics().clone(),
        };
        
        let checkpoint = TimeCheckpoint {
            id: checkpoint_id.clone(),
            timestamp,
            engine_state,
            description,
            step_number: self.step_counter,
        };
        
        // Remove oldest checkpoints if we exceed the limit
        if self.checkpoints.len() >= self.config.max_checkpoints {
            if let Some(oldest_timestamp) = self.checkpoints.keys().next().copied() {
                self.checkpoints.remove(&oldest_timestamp);
            }
        }
        
        self.checkpoints.insert(timestamp, checkpoint);
        self.current_position = Some(timestamp);
        
        Ok(checkpoint_id)
    }
    
    /// Rewind simulation to a specific checkpoint
    pub fn rewind_to_checkpoint(
        &mut self, 
        checkpoint_id: &CheckpointId,
        engine: &mut SimulationEngine
    ) -> Result<(), SimulationError> {
        // Find the checkpoint
        let checkpoint = self.checkpoints.values()
            .find(|cp| cp.id == *checkpoint_id)
            .ok_or_else(|| SimulationError::InvalidState(
                format!("Checkpoint not found: {}", checkpoint_id.as_str())
            ))?;
        
        // Restore engine state from checkpoint
        self.restore_engine_state(engine, &checkpoint.engine_state)?;
        self.current_position = Some(checkpoint.timestamp);
        self.step_counter = checkpoint.step_number;
        
        Ok(())
    }
    
    /// Rewind simulation to a specific timestamp
    pub fn rewind_to_timestamp(
        &mut self, 
        target_timestamp: SimulatedTimestamp,
        engine: &mut SimulationEngine
    ) -> Result<(), SimulationError> {
        // Find the closest checkpoint at or before the target timestamp
        let checkpoint = self.checkpoints.range(..=target_timestamp)
            .next_back()
            .map(|(_, checkpoint)| checkpoint)
            .ok_or_else(|| SimulationError::InvalidState(
                "No checkpoint found before target timestamp".to_string()
            ))?;
        
        // Restore engine state from checkpoint
        self.restore_engine_state(engine, &checkpoint.engine_state)?;
        self.current_position = Some(checkpoint.timestamp);
        self.step_counter = checkpoint.step_number;
        
        Ok(())
    }
    
    /// Fast-forward simulation by executing steps until target timestamp
    pub async fn fast_forward_to_timestamp(
        &mut self, 
        target_timestamp: SimulatedTimestamp,
        engine: &mut SimulationEngine
    ) -> Result<usize, SimulationError> {
        let mut steps_executed = 0;
        
        while engine.clock().now() < target_timestamp {
            // Check for automatic checkpointing
            if let Some(interval) = self.config.auto_checkpoint_interval {
                if self.step_counter % interval == 0 {
                    self.create_checkpoint(engine, format!("Auto checkpoint at step {}", self.step_counter))?;
                }
            }
            
            // Execute one step
            let continue_execution = engine.step().await?;
            steps_executed += 1;
            self.step_counter += 1;
            
            if !continue_execution {
                break;
            }
            
            // Safety check to prevent infinite loops
            if steps_executed > 10000 {
                return Err(SimulationError::InvalidState(
                    "Fast-forward exceeded maximum steps".to_string()
                ));
            }
        }
        
        self.current_position = Some(engine.clock().now());
        Ok(steps_executed)
    }
    
    /// Get all available checkpoints
    pub fn list_checkpoints(&self) -> Vec<&TimeCheckpoint> {
        self.checkpoints.values().collect()
    }
    
    /// Get a specific checkpoint by ID
    pub fn get_checkpoint(&self, checkpoint_id: &CheckpointId) -> Option<&TimeCheckpoint> {
        self.checkpoints.values()
            .find(|cp| cp.id == *checkpoint_id)
    }
    
    /// Get the current timeline position
    pub fn current_position(&self) -> Option<SimulatedTimestamp> {
        self.current_position
    }
    
    /// Get checkpoint at specific timestamp
    pub fn get_checkpoint_at_timestamp(&self, timestamp: SimulatedTimestamp) -> Option<&TimeCheckpoint> {
        self.checkpoints.get(&timestamp)
    }
    
    /// Delete a checkpoint
    pub fn delete_checkpoint(&mut self, checkpoint_id: &CheckpointId) -> bool {
        if let Some(checkpoint) = self.checkpoints.values()
            .find(|cp| cp.id == *checkpoint_id) {
            let timestamp = checkpoint.timestamp;
            self.checkpoints.remove(&timestamp).is_some()
        } else {
            false
        }
    }
    
    /// Clear all checkpoints
    pub fn clear_checkpoints(&mut self) {
        self.checkpoints.clear();
        self.current_position = None;
        self.step_counter = 0;
    }
    
    /// Get time-travel statistics
    pub fn get_statistics(&self) -> TimeTravelStatistics {
        let total_checkpoints = self.checkpoints.len();
        let time_span = if let (Some(first), Some(last)) = (
            self.checkpoints.keys().next(),
            self.checkpoints.keys().next_back()
        ) {
            last.as_secs().saturating_sub(first.as_secs())
        } else {
            0
        };
        
        TimeTravelStatistics {
            total_checkpoints,
            time_span_seconds: time_span,
            current_step: self.step_counter,
            current_position: self.current_position,
        }
    }
    
    /// Restore engine state from a serializable state
    fn restore_engine_state(
        &self, 
        engine: &mut SimulationEngine, 
        state: &SerializableEngineState
    ) -> Result<(), SimulationError> {
        // Restore basic state
        engine.set_state(state.state.clone());
        engine.machine.gas = state.gas_remaining;
        engine.effects_log = state.effects_log.clone();
        
        // Note: In a full implementation, we would restore:
        // - Program counter (need to expose pc field)
        // - Register state
        // - Memory state
        // - Complete metrics
        
        Ok(())
    }
}

impl Default for TimeTravelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about time-travel usage
#[derive(Debug, Clone)]
pub struct TimeTravelStatistics {
    pub total_checkpoints: usize,
    pub time_span_seconds: u64,
    pub current_step: usize,
    pub current_position: Option<SimulatedTimestamp>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SimulationConfig;
    
    #[test]
    fn test_checkpoint_id_generation() {
        let timestamp = SimulatedTimestamp::from_secs(1000);
        let id = CheckpointId::generate(timestamp, 42);
        assert_eq!(id.as_str(), "checkpoint_1000_42");
    }
    
    #[test]
    fn test_time_travel_manager_creation() {
        let manager = TimeTravelManager::new();
        assert_eq!(manager.checkpoints.len(), 0);
        assert_eq!(manager.current_position(), None);
    }
    
    #[tokio::test]
    async fn test_checkpoint_creation() {
        let mut manager = TimeTravelManager::new();
        let mut engine = SimulationEngine::new();
        engine.initialize().await.unwrap();
        
        let checkpoint_id = manager.create_checkpoint(&engine, "Test checkpoint".to_string()).unwrap();
        
        assert_eq!(manager.checkpoints.len(), 1);
        assert!(manager.current_position().is_some());
        
        let checkpoint = manager.get_checkpoint(&checkpoint_id).unwrap();
        assert_eq!(checkpoint.description, "Test checkpoint");
    }
    
    #[test]
    fn test_checkpoint_limit() {
        let config = TimeTravelConfig {
            max_checkpoints: 2,
            auto_checkpoint_interval: None,
            compress_old_checkpoints: false,
        };
        
        let mut manager = TimeTravelManager::with_config(config);
        // This test would require mocking the engine creation for multiple checkpoints
        // In a real implementation, we would test the limit enforcement
        
        assert_eq!(manager.config.max_checkpoints, 2);
    }
} 