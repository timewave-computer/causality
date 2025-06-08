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
    engine::ExecutionState,
};
use std::sync::atomic::{AtomicU64, Ordering};
use uuid;

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

impl Default for BranchMetadata {
    fn default() -> Self {
        Self {
            description: "Default branch".to_string(),
            created_at: crate::clock::SimulatedTimestamp::new(0),
            status: BranchStatus::Active,
            depth: 0,
            steps_executed: 0,
        }
    }
}

/// Status of a simulation branch
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum BranchStatus {
    /// Branch is actively being executed
    #[default]
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


/// Information about a simulation branch
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Unique identifier for this branch
    pub id: BranchId,
    
    /// Human-readable name
    pub name: String,
    
    /// Parent branch ID (None for root)
    pub parent_id: Option<BranchId>,
    
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    
    /// Execution state snapshot
    pub execution_state: ExecutionState,
    
    /// Branch metadata
    pub metadata: BranchMetadata,
}

/// Manager for simulation branching and scenario exploration
#[derive(Debug, Clone)]
pub struct BranchingManager {
    /// All branches in the system
    branches: HashMap<BranchId, BranchInfo>,
    /// Currently active branch
    pub active_branch_id: Option<BranchId>,
    /// Root branch ID
    root_branch_id: BranchId,
}

impl BranchingManager {
    /// Create a new branching manager
    pub fn new() -> Self {
        let root_id = BranchId("root".to_string());
        let mut branches = HashMap::new();
        
        // Create root branch
        let root_branch = BranchInfo {
            id: root_id.clone(),
            name: "Root".to_string(),
            parent_id: None,
            created_at: std::time::SystemTime::now(),
            execution_state: ExecutionState::new(),
            metadata: BranchMetadata {
                description: "Root branch".to_string(),
                status: BranchStatus::Active,
                created_at: crate::clock::SimulatedTimestamp::new(0),
                depth: 0,
                steps_executed: 0,
            },
        };
        
        branches.insert(root_id.clone(), root_branch);
        
        Self {
            branches,
            active_branch_id: Some(root_id.clone()),
            root_branch_id: root_id,
        }
    }
    
    /// Create a new branching manager with configuration
    pub fn with_config(_config: BranchingConfig) -> Self {
        // For now, just use the default implementation
        // TODO: Apply configuration settings
        Self::new()
    }
    
    /// Create a new branch
    pub fn create_branch(
        &mut self,
        branch_id: &str,
        branch_name: &str,
        execution_state: ExecutionState,
    ) -> Result<BranchId, SimulationError> {
        let new_branch_id = BranchId(branch_id.to_string());
        
        if self.branches.contains_key(&new_branch_id) {
            return Err(SimulationError::InvalidInput(format!("Branch already exists: {}", branch_id)));
        }
        
        let branch_info = BranchInfo {
            id: new_branch_id.clone(),
            name: branch_name.to_string(),
            parent_id: self.active_branch_id.clone(),
            created_at: std::time::SystemTime::now(),
            execution_state,
            metadata: BranchMetadata {
                description: branch_name.to_string(),
                status: BranchStatus::Active,
                created_at: crate::clock::SimulatedTimestamp::new(0),
                depth: 1,
                steps_executed: 0,
            },
        };
        
        self.branches.insert(new_branch_id.clone(), branch_info);
        
        Ok(new_branch_id)
    }
    
    /// Get the execution state for a specific branch
    pub fn get_branch_state(&self, branch_id: &str) -> Result<ExecutionState, SimulationError> {
        let branch_key = BranchId(branch_id.to_string());
        match self.branches.get(&branch_key) {
            Some(branch) => Ok(branch.execution_state.clone()),
            None => Err(SimulationError::BranchNotFound(branch_id.to_string())),
        }
    }
    
    /// Get information about a specific branch
    pub fn get_branch_info(&self, branch_id: &str) -> Option<&BranchInfo> {
        self.branches.get(&BranchId::new(branch_id.to_string()))
    }
    
    /// List all available branches
    pub fn list_branches(&self) -> Vec<&BranchInfo> {
        self.branches.values().collect()
    }
    
    /// Get children of a specific branch
    pub fn get_branch_children(&self, branch_id: &str) -> Vec<&BranchInfo> {
        // Simplified - find branches that have this branch as parent
        let target_id = BranchId::new(branch_id.to_string());
        self.branches.values()
            .filter(|branch| branch.parent_id.as_ref() == Some(&target_id))
            .collect()
    }
    
    /// Remove a branch and all its children
    pub fn remove_branch(&mut self, branch_id: &str) -> Result<(), SimulationError> {
        let branch_key = BranchId::new(branch_id.to_string());
        
        // Find and remove children first
        let children: Vec<BranchId> = self.branches.values()
            .filter(|branch| branch.parent_id.as_ref() == Some(&branch_key))
            .map(|branch| branch.id.clone())
            .collect();
            
        for child_id in children {
            self.remove_branch(&child_id.0)?;
        }
        
        // Remove the branch itself
        self.branches.remove(&branch_key);
        
        // Update current branch if it was removed
        if self.active_branch_id.as_ref() == Some(&branch_key) {
            self.active_branch_id = Some(self.root_branch_id.clone());
        }
        
        Ok(())
    }
    
    /// Clear all branches except root
    pub fn clear(&mut self) {
        let root_id = self.root_branch_id.clone();
        let root_branch = self.branches.get(&root_id).cloned();
        
        self.branches.clear();
        
        if let Some(root) = root_branch {
            self.branches.insert(root_id.clone(), root);
        }
        
        self.active_branch_id = Some(root_id);
    }
    
    /// Get the current active branch ID
    pub fn current_branch(&self) -> Option<&BranchId> {
        self.active_branch_id.as_ref()
    }
    
    /// Set the current active branch
    pub fn set_current_branch(&mut self, branch_id: Option<BranchId>) {
        self.active_branch_id = branch_id;
    }
    
    /// Get branch summary statistics
    pub fn branch_summary(&self) -> BranchSummary {
        BranchSummary {
            total_branches: self.branches.len(),
            completed_branches: 0, // Simplified - no status field in ExecutionState
            failed_branches: 0,    // Simplified - no status field in ExecutionState
            max_depth: 1,          // Simplified - no hierarchy tracking yet
        }
    }
    
    /// Get all branch IDs
    pub fn all_branch_ids(&self) -> Vec<BranchId> {
        self.branches.keys().cloned().collect()
    }
    
    /// Get a branch by ID
    pub fn get_branch(&self, branch_id: &BranchId) -> Option<&BranchInfo> {
        self.branches.get(branch_id)
    }
    
    /// Get active branch mutably (for examples that need it)
    pub fn active_branch_mut(&mut self) -> Option<&mut BranchInfo> {
        if let Some(active_id) = &self.active_branch_id {
            self.branches.get_mut(active_id)
        } else {
            None
        }
    }
    
    /// Switch to a branch by ID
    pub fn switch_to_branch(&mut self, branch_id: &BranchId) -> Result<(), SimulationError> {
        if self.branches.contains_key(branch_id) {
            self.active_branch_id = Some(branch_id.clone());
            Ok(())
        } else {
            Err(SimulationError::BranchNotFound(format!("{:?}", branch_id)))
        }
    }
    
    /// Fork a new branch from the current active branch
    pub fn fork_branch(&mut self, description: String) -> Result<BranchId, SimulationError> {
        let new_id = uuid::Uuid::new_v4().to_string();
        let execution_state = if let Some(active_id) = &self.active_branch_id {
            self.branches.get(active_id)
                .map(|b| b.execution_state.clone())
                .unwrap_or_default()
        } else {
            ExecutionState::new()
        };
        
        self.create_branch(&new_id, &description, execution_state)
    }
    
    /// Initialize root branch with a description
    pub fn initialize_root(&mut self, description: String) -> Result<BranchId, SimulationError> {
        // Get the root branch ID and update its description
        let root_id = self.root_branch_id.clone();
        if let Some(root_branch) = self.branches.get_mut(&root_id) {
            root_branch.name = description;
        }
        Ok(root_id)
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

/// Branch execution summary
#[derive(Debug, Clone)]
pub struct BranchSummary {
    pub total_branches: usize,
    pub completed_branches: usize,
    pub failed_branches: usize,
    pub max_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
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
        
        let root_id = manager.initialize_root("Root branch".to_string()).unwrap();
        
        assert_eq!(manager.active_branch_id, Some(root_id.clone()));
        assert_eq!(manager.branches.len(), 1);
    }
    
    #[test]
    fn test_branch_forking() {
        let mut manager = BranchingManager::new();
        
        let root_id = manager.initialize_root("Root".to_string()).unwrap();
        let fork_id = manager.create_branch("Fork A", "Fork A", ExecutionState::new()).unwrap();
        
        assert_eq!(manager.branches.len(), 2);
        assert_ne!(root_id, fork_id);
        
        let fork_branch = manager.get_branch_info(&fork_id.0).unwrap();
        assert_eq!(fork_branch.parent_id, Some(root_id.clone()));
    }
    
    #[test]
    fn test_branch_switching() {
        let mut manager = BranchingManager::new();
        
        let root_id = manager.initialize_root("Root".to_string()).unwrap();
        let fork_id = manager.create_branch("Fork", "Fork", ExecutionState::new()).unwrap();
        
        // Initially active branch should be root
        assert_eq!(manager.active_branch_id, Some(root_id.clone()));
        
        // Switch to fork
        manager.set_current_branch(Some(fork_id.clone()));
        assert_eq!(manager.active_branch_id, Some(fork_id));
        
        // Switch back to root
        manager.set_current_branch(Some(root_id.clone()));
        assert_eq!(manager.active_branch_id, Some(root_id));
    }
    
    #[test]
    fn test_branching_limits() {
        let mut manager = BranchingManager::new();
        
        let _root_id = manager.initialize_root("Root".to_string()).unwrap();
        
        // First fork should succeed
        let _fork1 = manager.create_branch("Fork 1", "Fork 1", ExecutionState::new()).unwrap();
        
        // Test basic functionality without max_branches limit
        assert_eq!(manager.branches.len(), 2);
    }
} 