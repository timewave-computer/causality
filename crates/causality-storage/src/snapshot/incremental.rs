// Incremental snapshot system
// Original file: src/snapshot/incremental.rs

// Incremental snapshot implementation
//
// This module provides functionality for creating and managing
// incremental snapshots, which store only the differences between snapshots.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json;

use causality_types::Result;
use causality_engine::{ExecutionContext, ContextId};
use causality_storage::{ExecutionSnapshot, SnapshotError, SnapshotId, SnapshotManager};

/// Represents the differences between two snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// The ID of this diff
    pub diff_id: String,
    /// The parent snapshot ID
    pub parent_id: SnapshotId,
    /// The child snapshot ID
    pub child_id: SnapshotId,
    /// Variables that were added
    pub added_variables: HashMap<String, serde_json::Value>,
    /// Variables that were modified
    pub modified_variables: HashMap<String, serde_json::Value>,
    /// Variables that were removed
    pub removed_variables: HashSet<String>,
    /// Changes to the call stack
    pub call_stack_changes: CallStackChanges,
    /// Changes to resource usage
    pub resource_usage_diff: ResourceUsageDiff,
}

/// Represents changes to the call stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackChanges {
    /// Frames that were added
    pub added_frames: Vec<usize>,
    /// Frames that were modified
    pub modified_frames: HashMap<usize, ModifiedFrame>,
    /// Frames that were removed
    pub removed_frames: Vec<usize>,
}

/// Represents a modification to a call frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedFrame {
    /// The index of the modified frame
    pub frame_index: usize,
    /// The updated arguments (if changed)
    pub updated_arguments: Option<Vec<serde_json::Value>>,
}

/// Represents changes to resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageDiff {
    /// Change in memory usage (bytes)
    pub memory_bytes_delta: i64,
    /// Change in CPU usage (milliseconds)
    pub cpu_millis_delta: i64,
    /// Change in I/O operations
    pub io_operations_delta: i64,
    /// Change in effect count
    pub effect_count_delta: i64,
}

/// A manager for incremental snapshots
pub struct IncrementalSnapshotManager {
    /// The base directory for storing diffs
    base_dir: PathBuf,
    /// The underlying snapshot manager
    snapshot_manager: Arc<dyn SnapshotManager>,
    /// Cache of recent diffs
    diff_cache: Mutex<HashMap<String, SnapshotDiff>>,
}

impl IncrementalSnapshotManager {
    /// Create a new incremental snapshot manager
    pub fn new<P: AsRef<Path>>(
        base_dir: P,
        snapshot_manager: Arc<dyn SnapshotManager>,
    ) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // Ensure the base directory exists
        std::fs::create_dir_all(&base_dir)?;
        
        Ok(IncrementalSnapshotManager {
            base_dir,
            snapshot_manager,
            diff_cache: Mutex::new(HashMap::new()),
        })
    }
    
    /// Create a snapshot diff between two snapshots
    pub fn create_diff(
        &self,
        parent_snapshot: &ExecutionSnapshot,
        child_snapshot: &ExecutionSnapshot,
    ) -> std::result::Result<SnapshotDiff, SnapshotError> {
        // Compare variables
        let mut added_variables = HashMap::new();
        let mut modified_variables = HashMap::new();
        let mut removed_variables = HashSet::new();
        
        // Find added and modified variables
        for (key, child_value) in &child_snapshot.variables {
            match parent_snapshot.variables.get(key) {
                Some(parent_value) => {
                    // If values differ, it's a modification
                    if parent_value != child_value {
                        modified_variables.insert(key.clone(), child_value.clone());
                    }
                }
                None => {
                    // If not in parent, it's an addition
                    added_variables.insert(key.clone(), child_value.clone());
                }
            }
        }
        
        // Find removed variables
        for key in parent_snapshot.variables.keys() {
            if !child_snapshot.variables.contains_key(key) {
                removed_variables.insert(key.clone());
            }
        }
        
        // Compare call stacks
        let call_stack_changes = self.compute_call_stack_changes(
            &parent_snapshot.call_stack,
            &child_snapshot.call_stack,
        );
        
        // Compute resource usage diff
        let resource_usage_diff = ResourceUsageDiff {
            memory_bytes_delta: (child_snapshot.resource_usage.memory_bytes as i64)
                - (parent_snapshot.resource_usage.memory_bytes as i64),
            cpu_millis_delta: (child_snapshot.resource_usage.cpu_millis as i64)
                - (parent_snapshot.resource_usage.cpu_millis as i64),
            io_operations_delta: (child_snapshot.resource_usage.io_operations as i64)
                - (parent_snapshot.resource_usage.io_operations as i64),
            effect_count_delta: (child_snapshot.resource_usage.effect_count as i64)
                - (parent_snapshot.resource_usage.effect_count as i64),
        };
        
        // Create the diff
        let diff_id = format!("diff-{}-{}", parent_snapshot.snapshot_id, child_snapshot.snapshot_id);
        
        let diff = SnapshotDiff {
            diff_id,
            parent_id: parent_snapshot.snapshot_id.clone(),
            child_id: child_snapshot.snapshot_id.clone(),
            added_variables,
            modified_variables,
            removed_variables,
            call_stack_changes,
            resource_usage_diff,
        };
        
        // Cache the diff
        {
            let mut cache = self.diff_cache.lock().unwrap();
            cache.insert(diff.diff_id.clone(), diff.clone());
        }
        
        // Save the diff to disk
        self.save_diff(&diff)?;
        
        Ok(diff)
    }
    
    /// Compute changes between two call stacks
    fn compute_call_stack_changes(
        &self,
        parent_stack: &[causality_engine::CallFrame],
        child_stack: &[causality_engine::CallFrame],
    ) -> CallStackChanges {
        let mut added_frames = Vec::new();
        let mut modified_frames = HashMap::new();
        let mut removed_frames = Vec::new();
        
        // Find added and modified frames
        for (i, child_frame) in child_stack.iter().enumerate() {
            if i >= parent_stack.len() {
                // If beyond parent length, it's an addition
                added_frames.push(i);
            } else {
                let parent_frame = &parent_stack[i];
                
                // Check if the frame was modified
                if parent_frame.code_hash != child_frame.code_hash
                    || parent_frame.arguments != child_frame.arguments
                {
                    modified_frames.insert(
                        i,
                        ModifiedFrame {
                            frame_index: i,
                            updated_arguments: Some(
                                // This is a simplification - in a real implementation,
                                // we'd need to properly convert the arguments
                                serde_json::to_value(&child_frame.arguments)
                                    .unwrap_or(serde_json::Value::Null)
                                    .as_array()
                                    .unwrap_or(&Vec::new())
                                    .clone(),
                            ),
                        },
                    );
                }
            }
        }
        
        // Find removed frames
        for i in child_stack.len()..parent_stack.len() {
            removed_frames.push(i);
        }
        
        CallStackChanges {
            added_frames,
            modified_frames,
            removed_frames,
        }
    }
    
    /// Apply a diff to a snapshot
    pub fn apply_diff(
        &self,
        parent_snapshot: &ExecutionSnapshot,
        diff: &SnapshotDiff,
    ) -> std::result::Result<ExecutionSnapshot, SnapshotError> {
        // Verify this diff applies to the given parent
        if parent_snapshot.snapshot_id != diff.parent_id {
            return Err(SnapshotError::InvalidSnapshot(format!(
                "Diff {} is for parent {}, not {}",
                diff.diff_id, diff.parent_id, parent_snapshot.snapshot_id
            )));
        }
        
        // Start with a clone of the parent
        let mut new_snapshot = parent_snapshot.clone();
        new_snapshot.snapshot_id = diff.child_id.clone();
        
        // Apply variable changes
        // Add new variables
        for (key, value) in &diff.added_variables {
            new_snapshot.variables.insert(key.clone(), value.clone());
        }
        
        // Modify existing variables
        for (key, value) in &diff.modified_variables {
            new_snapshot.variables.insert(key.clone(), value.clone());
        }
        
        // Remove variables
        for key in &diff.removed_variables {
            new_snapshot.variables.remove(key);
        }
        
        // Apply call stack changes
        // This is a simplified implementation
        // In a real implementation, we would need to properly handle the call stack changes
        
        // Apply resource usage changes
        new_snapshot.resource_usage.memory_bytes = (new_snapshot.resource_usage.memory_bytes as i64
            + diff.resource_usage_diff.memory_bytes_delta)
            .max(0) as usize;
        
        new_snapshot.resource_usage.cpu_millis = (new_snapshot.resource_usage.cpu_millis as i64
            + diff.resource_usage_diff.cpu_millis_delta)
            .max(0) as usize;
        
        new_snapshot.resource_usage.io_operations = (new_snapshot.resource_usage.io_operations as i64
            + diff.resource_usage_diff.io_operations_delta)
            .max(0) as usize;
        
        new_snapshot.resource_usage.effect_count = (new_snapshot.resource_usage.effect_count as i64
            + diff.resource_usage_diff.effect_count_delta)
            .max(0) as usize;
        
        Ok(new_snapshot)
    }
    
    /// Create a chain of incremental snapshots
    pub fn create_incremental_chain(
        &self,
        base_snapshot: &ExecutionSnapshot,
        new_snapshots: &[ExecutionSnapshot],
    ) -> std::result::Result<Vec<SnapshotDiff>, SnapshotError> {
        if new_snapshots.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut diffs = Vec::new();
        let mut last_snapshot = base_snapshot;
        
        for snapshot in new_snapshots {
            let diff = self.create_diff(last_snapshot, snapshot)?;
            diffs.push(diff);
            last_snapshot = snapshot;
        }
        
        Ok(diffs)
    }
    
    /// Save a diff to disk
    fn save_diff(&self, diff: &SnapshotDiff) -> std::result::Result<(), SnapshotError> {
        let diff_path = self.base_dir.join(format!("{}.json", diff.diff_id));
        
        let serialized = serde_json::to_string_pretty(diff).map_err(|e| {
            SnapshotError::SerializationError(format!("Failed to serialize diff: {}", e))
        })?;
        
        std::fs::write(&diff_path, serialized).map_err(|e| {
            SnapshotError::StorageError(format!("Failed to write diff: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Load a diff from disk
    fn load_diff(&self, diff_id: &str) -> std::result::Result<SnapshotDiff, SnapshotError> {
        // Check cache first
        {
            let cache = self.diff_cache.lock().unwrap();
            if let Some(diff) = cache.get(diff_id) {
                return Ok(diff.clone());
            }
        }
        
        // Load from disk
        let diff_path = self.base_dir.join(format!("{}.json", diff_id));
        
        if !diff_path.exists() {
            return Err(SnapshotError::NotFound(format!(
                "Diff not found: {}",
                diff_id
            )));
        }
        
        let content = std::fs::read_to_string(&diff_path).map_err(|e| {
            SnapshotError::StorageError(format!("Failed to read diff file: {}", e))
        })?;
        
        let diff: SnapshotDiff = serde_json::from_str(&content).map_err(|e| {
            SnapshotError::SerializationError(format!("Failed to deserialize diff: {}", e))
        })?;
        
        // Add to cache
        {
            let mut cache = self.diff_cache.lock().unwrap();
            cache.insert(diff_id.to_string(), diff.clone());
        }
        
        Ok(diff)
    }
    
    /// Recreate a snapshot by applying a chain of diffs
    pub fn recreate_snapshot(
        &self,
        base_snapshot_id: &SnapshotId,
        target_snapshot_id: &SnapshotId,
    ) -> std::result::Result<ExecutionSnapshot, SnapshotError> {
        // First, check if the target snapshot already exists
        match self.snapshot_manager.get_snapshot(target_snapshot_id) {
            Ok(snapshot) => return Ok(snapshot),
            Err(SnapshotError::NotFound(_)) => {
                // If not found, we'll try to reconstruct it
            }
            Err(e) => return Err(e),
        }
        
        // Load the base snapshot
        let base_snapshot = self.snapshot_manager.get_snapshot(base_snapshot_id)?;
        
        // Find the path of diffs from base to target
        // This is a simplified implementation
        // In a real implementation, we would need to find the optimal path
        
        // For now, we'll assume a direct chain from base to target
        let diff_id = format!("diff-{}-{}", base_snapshot_id, target_snapshot_id);
        
        // Try to load the diff
        match self.load_diff(&diff_id) {
            Ok(diff) => {
                // Apply the diff to recreate the target snapshot
                self.apply_diff(&base_snapshot, &diff)
            }
            Err(SnapshotError::NotFound(_)) => {
                // If no direct diff, we need to find a path
                // This would require a more complex algorithm
                Err(SnapshotError::Other(format!(
                    "No direct diff found from {} to {}",
                    base_snapshot_id, target_snapshot_id
                )))
            }
            Err(e) => Err(e),
        }
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
        let snapshot_manager = Arc::new(FileSystemSnapshotManager::new(temp_dir.path().join("snapshots"))?);
        let incremental_manager = IncrementalSnapshotManager::new(
            temp_dir.path().join("diffs"),
            snapshot_manager,
        )?;
        
        assert!(temp_dir.path().join("diffs").exists());
        
        Ok(())
    }
} 