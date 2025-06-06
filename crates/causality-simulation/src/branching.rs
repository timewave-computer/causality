//! Simulation branching for exploring multiple execution paths
//!
//! This module provides functionality to fork simulation states and explore
//! different execution paths in parallel, enabling "what-if" analysis.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::{
    engine::SimulationEngine,
    clock::SimulatedTimestamp,
    error::SimulationError,
};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for ensuring unique branch IDs
static BRANCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique identifier for a simulation branch
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub String);

impl BranchId {
    /// Create a new branch ID with a specific string
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    /// Generate a unique branch ID
    pub fn generate() -> Self {
        let counter = BRANCH_COUNTER.fetch_add(1, Ordering::SeqCst);
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self(format!("branch_{}_{}", timestamp, counter))
    }
}

/// Configuration for branching behavior
#[derive(Debug, Clone)]
pub struct BranchingConfig {
    /// Maximum number of concurrent branches
    pub max_branches: usize,
    
    /// Maximum depth of branching
    pub max_depth: usize,
    
    /// Whether to automatically prune unsuccessful branches
    pub auto_prune: bool,
}

impl Default for BranchingConfig {
    fn default() -> Self {
        Self {
            max_branches: 10,
            max_depth: 5,
            auto_prune: true,
        }
    }
}

/// Represents a single branch in the execution tree
pub struct SimulationBranch {
    /// Unique identifier for this branch
    pub id: BranchId,
    
    /// Parent branch ID (None for root branch)
    pub parent_id: Option<BranchId>,
    
    /// Simulation engine state for this branch
    pub engine: SimulationEngine,
    
    /// Branch metadata
    pub metadata: BranchMetadata,
    
    /// Child branches spawned from this branch
    pub children: Vec<BranchId>,
    
    /// Branch creation timestamp
    pub created_at: SimulatedTimestamp,
}

/// Metadata about a simulation branch
#[derive(Debug, Clone)]
pub struct BranchMetadata {
    /// Human-readable description of this branch
    pub description: String,
    
    /// Timestamp when this branch was created
    pub created_at: SimulatedTimestamp,
    
    /// Current status of the branch
    pub status: BranchStatus,
    
    /// Depth in the branching tree
    pub depth: usize,
    
    /// Number of steps executed in this branch
    pub steps_executed: usize,
}

/// Status of a simulation branch
#[derive(Debug, Clone, PartialEq)]
pub enum BranchStatus {
    /// Branch is actively being executed
    Active,
    
    /// Branch execution completed successfully
    Completed,
    
    /// Branch execution failed
    Failed(String),
    
    /// Branch was pruned/discarded
    Pruned,
    
    /// Branch is paused/suspended
    Paused,
}

/// Manages multiple simulation branches for scenario exploration
pub struct BranchingManager {
    /// Configuration for branching behavior
    config: BranchingConfig,
    
    /// All branches indexed by their ID
    branches: HashMap<BranchId, SimulationBranch>,
    
    /// ID of the currently active branch
    active_branch_id: Option<BranchId>,
    
    /// Clock for timestamps
    clock: SimulatedTimestamp,
}

impl BranchingManager {
    /// Create a new branching manager
    pub fn new() -> Self {
        Self {
            config: BranchingConfig::default(),
            branches: HashMap::new(),
            active_branch_id: None,
            clock: SimulatedTimestamp::new(0),
        }
    }
    
    /// Create a branching manager with custom configuration
    pub fn with_config(config: BranchingConfig) -> Self {
        Self {
            config,
            branches: HashMap::new(),
            active_branch_id: None,
            clock: SimulatedTimestamp::new(0),
        }
    }
    
    /// Initialize with a root simulation engine
    pub fn initialize_root(&mut self, engine: SimulationEngine, description: String) -> Result<BranchId, SimulationError> {
        let branch_id = BranchId::generate();
        
        let metadata = BranchMetadata {
            description,
            created_at: self.clock,
            status: BranchStatus::Active,
            depth: 0,
            steps_executed: 0,
        };
        
        let branch = SimulationBranch {
            id: branch_id.clone(),
            parent_id: None,
            engine,
            metadata,
            children: Vec::new(),
            created_at: self.clock,
        };
        
        self.branches.insert(branch_id.clone(), branch);
        self.active_branch_id = Some(branch_id.clone());
        
        Ok(branch_id)
    }
    
    /// Fork the current active branch, creating a new branch
    pub fn fork_branch(&mut self, description: String) -> Result<BranchId, SimulationError> {
        let parent_id = self.active_branch_id.clone()
            .ok_or_else(|| SimulationError::InvalidState("No active branch to fork".to_string()))?;
        
        // Check branching limits
        if self.branches.len() >= self.config.max_branches {
            return Err(SimulationError::InvalidState("Maximum branches reached".to_string()));
        }
        
        let parent_branch = self.branches.get(&parent_id)
            .ok_or_else(|| SimulationError::InvalidState("Parent branch not found".to_string()))?;
        
        // Check depth limit
        if parent_branch.metadata.depth >= self.config.max_depth {
            return Err(SimulationError::InvalidState("Maximum branching depth reached".to_string()));
        }
        
        // Create new branch ID
        let new_branch_id = BranchId::generate();
        
        // Clone the parent engine state
        let new_engine = parent_branch.engine.clone();
        
        // Create metadata for the new branch
        let metadata = BranchMetadata {
            description,
            created_at: new_engine.clock().now(),
            status: BranchStatus::Active,
            depth: parent_branch.metadata.depth + 1,
            steps_executed: 0,
        };
        
        // Create the new branch
        let new_branch = SimulationBranch {
            id: new_branch_id.clone(),
            parent_id: Some(parent_id.clone()),
            engine: new_engine,
            metadata,
            children: Vec::new(),
            created_at: self.clock,
        };
        
        // Update parent branch to include this child
        if let Some(parent) = self.branches.get_mut(&parent_id) {
            parent.children.push(new_branch_id.clone());
        }
        
        // Add the new branch
        self.branches.insert(new_branch_id.clone(), new_branch);
        
        Ok(new_branch_id)
    }
    
    /// Switch to a different branch
    pub fn switch_to_branch(&mut self, branch_id: &BranchId) -> Result<(), SimulationError> {
        if !self.branches.contains_key(branch_id) {
            return Err(SimulationError::InvalidState("Branch not found".to_string()));
        }
        
        self.active_branch_id = Some(branch_id.clone());
        Ok(())
    }
    
    /// Get the currently active branch
    pub fn active_branch(&self) -> Option<&SimulationBranch> {
        self.active_branch_id.as_ref()
            .and_then(|id| self.branches.get(id))
    }
    
    /// Get a mutable reference to the currently active branch
    pub fn active_branch_mut(&mut self) -> Option<&mut SimulationBranch> {
        let active_id = self.active_branch_id.clone()?;
        self.branches.get_mut(&active_id)
    }
    
    /// Get a specific branch by ID
    pub fn get_branch(&self, branch_id: &BranchId) -> Option<&SimulationBranch> {
        self.branches.get(branch_id)
    }
    
    /// Get all branch IDs
    pub fn all_branch_ids(&self) -> Vec<BranchId> {
        self.branches.keys().cloned().collect()
    }
    
    /// Get branches by status
    pub fn branches_by_status(&self, status: BranchStatus) -> Vec<BranchId> {
        self.branches.iter()
            .filter(|(_, branch)| branch.metadata.status == status)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Prune completed or failed branches
    pub fn prune_inactive_branches(&mut self) -> usize {
        if !self.config.auto_prune {
            return 0;
        }
        
        let to_prune: Vec<BranchId> = self.branches.iter()
            .filter(|(id, branch)| {
                // Don't prune root or active branch
                if Some(*id) == self.active_branch_id.as_ref() {
                    return false;
                }
                
                // Prune completed or failed branches
                matches!(branch.metadata.status, BranchStatus::Completed | BranchStatus::Failed(_))
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        let pruned_count = to_prune.len();
        
        for branch_id in to_prune {
            self.branches.remove(&branch_id);
        }
        
        pruned_count
    }
    
    /// Get branch execution summary
    pub fn branch_summary(&self) -> BranchingSummary {
        let total_branches = self.branches.len();
        let active_branches = self.branches_by_status(BranchStatus::Active).len();
        let completed_branches = self.branches_by_status(BranchStatus::Completed).len();
        let failed_branches = self.branches.iter()
            .filter(|(_, branch)| matches!(branch.metadata.status, BranchStatus::Failed(_)))
            .count();
        
        BranchingSummary {
            total_branches,
            active_branches,
            completed_branches,
            failed_branches,
            max_depth: self.branches.values()
                .map(|b| b.metadata.depth)
                .max()
                .unwrap_or(0),
        }
    }
}

impl Default for BranchingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of branching execution
#[derive(Debug, Clone)]
pub struct BranchingSummary {
    pub total_branches: usize,
    pub active_branches: usize,
    pub completed_branches: usize,
    pub failed_branches: usize,
    pub max_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SimulationConfig;
    
    #[test]
    fn test_branch_id_generation() {
        let id1 = BranchId::generate();
        let id2 = BranchId::generate();
        
        assert_ne!(id1, id2);
        assert!(id1.0.starts_with("branch_"));
    }
    
    #[test]
    fn test_branching_manager_initialization() {
        let mut manager = BranchingManager::new();
        let engine = SimulationEngine::new();
        
        let root_id = manager.initialize_root(engine, "Root branch".to_string()).unwrap();
        
        assert_eq!(manager.active_branch_id, Some(root_id.clone()));
        assert_eq!(manager.all_branch_ids().len(), 1);
    }
    
    #[test]
    fn test_branch_forking() {
        let mut manager = BranchingManager::new();
        let engine = SimulationEngine::new();
        
        let root_id = manager.initialize_root(engine, "Root".to_string()).unwrap();
        let fork_id = manager.fork_branch("Fork A".to_string()).unwrap();
        
        assert_eq!(manager.all_branch_ids().len(), 2);
        assert_ne!(root_id, fork_id);
        
        let fork_branch = manager.get_branch(&fork_id).unwrap();
        assert_eq!(fork_branch.parent_id, Some(root_id));
        assert_eq!(fork_branch.metadata.depth, 1);
    }
    
    #[test]
    fn test_branch_switching() {
        let mut manager = BranchingManager::new();
        let engine = SimulationEngine::new();
        
        let root_id = manager.initialize_root(engine, "Root".to_string()).unwrap();
        let fork_id = manager.fork_branch("Fork".to_string()).unwrap();
        
        // Initially active branch should be root
        assert_eq!(manager.active_branch_id, Some(root_id.clone()));
        
        // Switch to fork
        manager.switch_to_branch(&fork_id).unwrap();
        assert_eq!(manager.active_branch_id, Some(fork_id));
        
        // Switch back to root
        manager.switch_to_branch(&root_id).unwrap();
        assert_eq!(manager.active_branch_id, Some(root_id));
    }
    
    #[test]
    fn test_branching_limits() {
        let config = BranchingConfig {
            max_branches: 2,
            max_depth: 1,
            auto_prune: false,
        };
        
        let mut manager = BranchingManager::with_config(config);
        let engine = SimulationEngine::new();
        
        let root_id = manager.initialize_root(engine, "Root".to_string()).unwrap();
        
        // First fork should succeed
        let fork1 = manager.fork_branch("Fork 1".to_string()).unwrap();
        
        // Second fork should fail due to max_branches limit
        let result = manager.fork_branch("Fork 2".to_string());
        assert!(result.is_err());
    }
} 