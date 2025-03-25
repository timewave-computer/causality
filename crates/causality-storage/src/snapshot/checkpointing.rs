// Snapshot checkpointing functionality
// Original file: src/snapshot/checkpointing.rs

// Automatic checkpointing implementation
//
// This module provides functionality for automatically creating snapshots
// at specific points in execution, such as effect boundaries.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::effect::EffectType;
use causality_types::Result;
use causality_engine::ExecutionContext;
use causality_storage::{SnapshotError, SnapshotId, SnapshotManager};

/// Configuration for automatic checkpointing
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Whether automatic checkpointing is enabled
    pub enabled: bool,
    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,
    /// Minimum time between checkpoints (in milliseconds)
    pub min_interval_ms: u64,
    /// Effect types that trigger checkpoints
    pub effect_triggers: HashSet<EffectType>,
    /// Checkpoint after a certain number of operations
    pub operations_interval: usize,
    /// Whether to checkpoint at effect boundaries
    pub checkpoint_effect_boundaries: bool,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        let mut effect_triggers = HashSet::new();
        // Add default effect types that trigger checkpoints
        // effect_triggers.insert(EffectType::FileWrite);
        // effect_triggers.insert(EffectType::NetworkRequest);
        // effect_triggers.insert(EffectType::DatabaseWrite);
        
        CheckpointConfig {
            enabled: true,
            max_snapshots: 100,
            min_interval_ms: 1000, // 1 second
            effect_triggers,
            operations_interval: 1000,
            checkpoint_effect_boundaries: true,
        }
    }
}

/// Automatic checkpointing manager
pub struct CheckpointManager {
    /// Snapshot manager to use for creating snapshots
    snapshot_manager: Arc<dyn SnapshotManager>,
    /// Configuration for automatic checkpointing
    config: Mutex<CheckpointConfig>,
    /// Last checkpoint time
    last_checkpoint: Mutex<Option<Instant>>,
    /// Operation counter
    operation_count: Mutex<usize>,
    /// List of snapshot IDs managed by this checkpoint manager
    managed_snapshots: Mutex<Vec<SnapshotId>>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(snapshot_manager: Arc<dyn SnapshotManager>) -> Self {
        CheckpointManager {
            snapshot_manager,
            config: Mutex::new(CheckpointConfig::default()),
            last_checkpoint: Mutex::new(None),
            operation_count: Mutex::new(0),
            managed_snapshots: Mutex::new(Vec::new()),
        }
    }
    
    /// Create a new checkpoint manager with a specific configuration
    pub fn with_config(
        snapshot_manager: Arc<dyn SnapshotManager>,
        config: CheckpointConfig,
    ) -> Self {
        CheckpointManager {
            snapshot_manager,
            config: Mutex::new(config),
            last_checkpoint: Mutex::new(None),
            operation_count: Mutex::new(0),
            managed_snapshots: Mutex::new(Vec::new()),
        }
    }
    
    /// Update the configuration
    pub fn update_config(&self, config: CheckpointConfig) {
        let mut current_config = self.config.lock().unwrap();
        *current_config = config;
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> CheckpointConfig {
        let config = self.config.lock().unwrap();
        config.clone()
    }
    
    /// Check if a checkpoint should be created for an effect
    pub fn should_checkpoint_effect(&self, effect_type: &EffectType) -> bool {
        let config = self.config.lock().unwrap();
        
        if !config.enabled {
            return false;
        }
        
        if !config.checkpoint_effect_boundaries {
            return false;
        }
        
        if !config.effect_triggers.contains(effect_type) {
            return false;
        }
        
        // Check if enough time has passed since the last checkpoint
        let now = Instant::now();
        let mut last_checkpoint = self.last_checkpoint.lock().unwrap();
        
        if let Some(last) = *last_checkpoint {
            if now.duration_since(last) < Duration::from_millis(config.min_interval_ms) {
                return false;
            }
        }
        
        // If we've reached here, we should create a checkpoint
        *last_checkpoint = Some(now);
        true
    }
    
    /// Check if a checkpoint should be created based on operation count
    pub fn should_checkpoint_operations(&self) -> bool {
        let config = self.config.lock().unwrap();
        
        if !config.enabled {
            return false;
        }
        
        if config.operations_interval == 0 {
            return false;
        }
        
        // Increment operation count
        let mut count = self.operation_count.lock().unwrap();
        *count += 1;
        
        // Check if we've reached the interval
        if *count >= config.operations_interval {
            // Reset counter
            *count = 0;
            
            // Check if enough time has passed
            let now = Instant::now();
            let mut last_checkpoint = self.last_checkpoint.lock().unwrap();
            
            if let Some(last) = *last_checkpoint {
                if now.duration_since(last) < Duration::from_millis(config.min_interval_ms) {
                    return false;
                }
            }
            
            // Update last checkpoint time
            *last_checkpoint = Some(now);
            return true;
        }
        
        false
    }
    
    /// Create a checkpoint for a context
    pub fn create_checkpoint(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<SnapshotId, SnapshotError> {
        let config = self.config.lock().unwrap();
        
        if !config.enabled {
            return Err(SnapshotError::Other("Checkpointing is disabled".to_string()));
        }
        
        // Create snapshot
        let snapshot_id = self.snapshot_manager.create_snapshot(context)?;
        
        // Add to managed snapshots
        {
            let mut managed = self.managed_snapshots.lock().unwrap();
            managed.push(snapshot_id.clone());
            
            // If we have too many snapshots, remove the oldest ones
            if managed.len() > config.max_snapshots {
                let to_remove = managed.len() - config.max_snapshots;
                let removed: Vec<SnapshotId> = managed.drain(0..to_remove).collect();
                
                // Delete the removed snapshots
                for id in removed {
                    if let Err(e) = self.snapshot_manager.delete_snapshot(&id) {
                        eprintln!("Failed to delete old snapshot {}: {}", id, e);
                    }
                }
            }
        }
        
        Ok(snapshot_id)
    }
    
    /// Check and create a checkpoint if needed for an effect
    pub fn checkpoint_if_needed(
        &self,
        context: &ExecutionContext,
        effect_type: &EffectType,
    ) -> std::result::Result<Option<SnapshotId>, SnapshotError> {
        if self.should_checkpoint_effect(effect_type) {
            let snapshot_id = self.create_checkpoint(context)?;
            Ok(Some(snapshot_id))
        } else {
            Ok(None)
        }
    }
    
    /// Check and create a checkpoint based on operation count
    pub fn checkpoint_operations(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<Option<SnapshotId>, SnapshotError> {
        if self.should_checkpoint_operations() {
            let snapshot_id = self.create_checkpoint(context)?;
            Ok(Some(snapshot_id))
        } else {
            Ok(None)
        }
    }
    
    /// Register an effect execution
    pub fn register_effect(
        &self,
        context: &ExecutionContext,
        effect_type: &EffectType,
    ) -> std::result::Result<Option<SnapshotId>, SnapshotError> {
        // Increment operation count regardless
        {
            let mut count = self.operation_count.lock().unwrap();
            *count += 1;
        }
        
        // Check if we should create a checkpoint
        self.checkpoint_if_needed(context, effect_type)
    }
    
    /// Get the list of managed snapshot IDs
    pub fn get_managed_snapshots(&self) -> Vec<SnapshotId> {
        let managed = self.managed_snapshots.lock().unwrap();
        managed.clone()
    }
    
    /// Clear all managed snapshots
    pub fn clear_managed_snapshots(&self) -> std::result::Result<(), SnapshotError> {
        let mut managed = self.managed_snapshots.lock().unwrap();
        
        // Delete all snapshots
        for id in managed.iter() {
            if let Err(e) = self.snapshot_manager.delete_snapshot(id) {
                eprintln!("Failed to delete snapshot {}: {}", id, e);
            }
        }
        
        // Clear the list
        managed.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_storage::FileSystemSnapshotManager;
    use tempfile::TempDir;
    
    // More tests will be added in the future
    // For now, we just test basic creation of the manager
    
    #[test]
    fn test_create_manager() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let snapshot_manager = Arc::new(FileSystemSnapshotManager::new(temp_dir.path())?);
        let checkpoint_manager = CheckpointManager::new(snapshot_manager);
        
        assert!(checkpoint_manager.get_config().enabled);
        
        Ok(())
    }
    
    #[test]
    fn test_custom_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let snapshot_manager = Arc::new(FileSystemSnapshotManager::new(temp_dir.path())?);
        
        let config = CheckpointConfig {
            enabled: false,
            max_snapshots: 50,
            min_interval_ms: 500,
            effect_triggers: HashSet::new(),
            operations_interval: 500,
            checkpoint_effect_boundaries: false,
        };
        
        let checkpoint_manager = CheckpointManager::with_config(snapshot_manager, config.clone());
        
        assert_eq!(checkpoint_manager.get_config().enabled, config.enabled);
        assert_eq!(checkpoint_manager.get_config().max_snapshots, config.max_snapshots);
        assert_eq!(checkpoint_manager.get_config().min_interval_ms, config.min_interval_ms);
        
        Ok(())
    }
} 