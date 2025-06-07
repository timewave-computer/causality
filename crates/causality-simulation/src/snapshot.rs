//! Snapshot management for simulation state capture and rollback

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{
    clock::SimulatedTimestamp, 
    error::{SnapshotError, SimulationResult}
};

/// Snapshot identifier for simulation checkpoints
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(String);

impl SnapshotId {
    /// Create a new snapshot ID
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    /// Generate a unique snapshot ID based on timestamp
    pub fn generate(timestamp: SimulatedTimestamp) -> Self {
        Self(format!("snapshot_{}", timestamp.as_secs()))
    }
    
    /// Get the inner string value (for testing)
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Execution metrics captured in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub effects_executed: usize,
    pub resources_allocated: usize,
    pub resources_consumed: usize,
    pub total_execution_time: std::time::Duration,
    pub average_effect_time: std::time::Duration,
    pub memory_usage_bytes: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            effects_executed: 0,
            resources_allocated: 0,
            resources_consumed: 0,
            total_execution_time: std::time::Duration::ZERO,
            average_effect_time: std::time::Duration::ZERO,
            memory_usage_bytes: 0,
        }
    }
}

/// Simulation snapshot containing complete state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    pub id: SnapshotId,
    pub timestamp: SimulatedTimestamp,
    pub description: String,
    pub resource_state: Vec<u8>, // Serialized state placeholder
    pub effects_log: Vec<EffectExecution>,
    pub metrics: PerformanceMetrics,
}

/// Record of an effect execution for debugging and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectExecution {
    pub effect_id: String,
    pub effect_expr: String, // Serialized EffectExpr for debugging
    pub start_time: SimulatedTimestamp,
    pub end_time: Option<SimulatedTimestamp>,
    pub result: ExecutionResult,
    pub resources_consumed: Vec<String>,
    pub resources_produced: Vec<String>,
}

/// Result of an effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success,
    Failed { error: String },
    Timeout,
    Cancelled,
}

/// Manages simulation snapshots for debugging and testing
#[derive(Debug)]
pub struct SnapshotManager {
    snapshots: HashMap<SnapshotId, SimulationSnapshot>,
    max_snapshots: usize,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: HashMap::new(),
            max_snapshots,
        }
    }
    
    /// Create a snapshot of the current simulation state
    pub fn create_snapshot(
        &mut self,
        id: SnapshotId,
        timestamp: SimulatedTimestamp,
        description: String,
        _resource_heap: &causality_core::machine::ResourceHeap, // TODO: Add proper serialization
        effects_log: Vec<EffectExecution>,
        metrics: PerformanceMetrics,
    ) -> SimulationResult<()> {
        // TODO: Replace with proper ResourceHeap serialization when serde support is added
        let resource_state = vec![]; // Placeholder serialized state
        
        let snapshot = SimulationSnapshot {
            id: id.clone(),
            timestamp,
            description,
            resource_state,
            effects_log,
            metrics,
        };
        
        // Remove oldest snapshots if we exceed the limit
        if self.snapshots.len() >= self.max_snapshots && !self.snapshots.contains_key(&id) {
            if let Some(oldest_id) = self.find_oldest_snapshot() {
                self.snapshots.remove(&oldest_id);
            }
        }
        
        self.snapshots.insert(id, snapshot);
        Ok(())
    }
    
    /// Restore simulation state from a snapshot
    pub fn restore_snapshot(&self, id: &SnapshotId) -> Result<(causality_core::machine::ResourceHeap, Vec<EffectExecution>, PerformanceMetrics), SnapshotError> {
        let snapshot = self.snapshots.get(id)
            .ok_or_else(|| SnapshotError::NotFound { id: id.as_str().to_string() })?;
        
        // TODO: Replace with proper ResourceHeap deserialization when serde support is added
        let resource_heap = causality_core::machine::ResourceHeap::new(); // Placeholder new heap
        
        Ok((resource_heap, snapshot.effects_log.clone(), snapshot.metrics.clone()))
    }
    
    /// Get information about a snapshot without restoring it
    pub fn get_snapshot_info(&self, id: &SnapshotId) -> Option<&SimulationSnapshot> {
        self.snapshots.get(id)
    }
    
    /// List all available snapshots
    pub fn list_snapshots(&self) -> Vec<&SnapshotId> {
        self.snapshots.keys().collect()
    }
    
    /// Delete a snapshot
    pub fn delete_snapshot(&mut self, id: &SnapshotId) -> bool {
        self.snapshots.remove(id).is_some()
    }
    
    /// Clear all snapshots
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }
    
    /// Find the oldest snapshot by timestamp
    fn find_oldest_snapshot(&self) -> Option<SnapshotId> {
        self.snapshots
            .values()
            .min_by_key(|snapshot| snapshot.timestamp)
            .map(|snapshot| snapshot.id.clone())
    }
    
    /// Get a snapshot by its ID
    pub fn get_snapshot(&self, id: &SnapshotId) -> Option<&SimulationSnapshot> {
        self.snapshots.get(id)
    }
    
    /// Create a checkpoint with arbitrary data
    pub fn create_checkpoint<T: Clone>(
        &mut self, 
        checkpoint_id: &str, 
        checkpoint_name: &str, 
        data: T
    ) -> Result<(), crate::error::SimulationError> 
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + 'static,
    {
        let serialized = serde_json::to_string(&data)
            .map_err(|e| crate::error::SimulationError::SnapshotError(format!("Serialization failed: {}", e)))?;
            
        let snapshot = SimulationSnapshot {
            id: SnapshotId::new(checkpoint_id.to_string()),
            timestamp: SimulatedTimestamp::new(0), // Use default timestamp
            description: checkpoint_name.to_string(),
            resource_state: serialized.into_bytes(), // Store serialized data as resource state
            effects_log: Vec::new(), // Empty for checkpoints
            metrics: PerformanceMetrics::default(),
        };
        
        self.snapshots.insert(SnapshotId::new(checkpoint_id.to_string()), snapshot);
        Ok(())
    }
    
    /// Get checkpoint data
    pub fn get_checkpoint<T>(&self, checkpoint_id: &str) -> Result<T, crate::error::SimulationError>
    where
        T: serde::de::DeserializeOwned,
    {
        let snapshot = self.snapshots
            .get(&SnapshotId::new(checkpoint_id.to_string()))
            .ok_or_else(|| crate::error::SimulationError::SnapshotError("Checkpoint not found".to_string()))?;
            
        let data_str = String::from_utf8(snapshot.resource_state.clone())
            .map_err(|e| crate::error::SimulationError::SnapshotError(format!("UTF-8 conversion failed: {}", e)))?;
            
        serde_json::from_str(&data_str)
            .map_err(|e| crate::error::SimulationError::SnapshotError(format!("Deserialization failed: {}", e)))
    }
    
    /// Helper method to calculate checksum
    fn calculate_checksum(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new(10) // Default to keeping 10 snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_id_generation() {
        let timestamp = SimulatedTimestamp::from_secs(1234567890);
        let id = SnapshotId::generate(timestamp);
        assert_eq!(id.as_str(), "snapshot_1234567890");
    }
    
    #[test]
    fn test_snapshot_manager_basic() {
        let mut manager = SnapshotManager::new(2);
        let id1 = SnapshotId::new("test1".to_string());
        
        // Initially empty
        assert_eq!(manager.list_snapshots().len(), 0);
        
        // Create snapshots
        let resource_heap = causality_core::machine::ResourceHeap::new();
        let timestamp = SimulatedTimestamp::from_secs(1000);
        let metrics = PerformanceMetrics::default();
        
        manager.create_snapshot(
            id1.clone(),
            timestamp,
            "Test snapshot 1".to_string(),
            &resource_heap,
            vec![],
            metrics.clone(),
        ).unwrap();
        
        assert_eq!(manager.list_snapshots().len(), 1);
        assert!(manager.get_snapshot_info(&id1).is_some());
    }
    
    #[test]
    fn test_snapshot_id_creation() {
        let id1 = SnapshotId::new("test1".to_string());
        let id2 = SnapshotId::new("test2".to_string());
        
        assert_ne!(id1, id2);
        assert_eq!(id1.as_str(), "test1");
        assert_eq!(id2.as_str(), "test2");
    }
} 