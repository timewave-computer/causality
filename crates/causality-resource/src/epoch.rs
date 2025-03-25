// Epoch-based resource management
// Original file: src/resource/epoch.rs

// Epoch management for register lifecycle
//
// This module implements epoch-based management for registers as described in ADR-006.
// It provides functionality for:
// - Managing epoch boundaries
// - Tracking registers within epochs
// - Configuring archival policies
// - Supporting register summarization and garbage collection

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use causality_types::{Error, Result};
use crate::resource::register::{Register, ContentId, BlockHeight};
use causality_types::{Address, Domain};

/// Epoch identifier type
pub type EpochId = u64;

/// Summarization group identifier
pub type SummaryGroup = String;

/// Archive identifier
pub type ArchiveId = String;

/// Strategy for summarizing registers during garbage collection
#[derive(Debug, Clone)]
pub enum SummaryStrategy {
    /// Create one summary per resource per epoch
    SummarizeByResource,
    
    /// Create one summary per account/owner per epoch
    SummarizeByAccount,
    
    /// Create one summary per register type per epoch
    SummarizeByType,
    
    /// Custom grouping function
    CustomSummary(Arc<dyn Fn(&Register) -> SummaryGroup + Send + Sync>),
}

/// Location for storing archived registers
#[derive(Debug, Clone)]
pub enum ArchiveLocation {
    /// Local storage with path
    LocalStorage(String),
    
    /// Remote storage with URL and authentication
    RemoteStorage {
        url: String,
        api_key: String,
    },
    
    /// Distributed storage across peers
    DistributedStorage {
        peers: Vec<String>,
    },
}

/// Archival policy configuration
#[derive(Debug, Clone)]
pub struct ArchivalPolicy {
    /// How many epochs to keep fully accessible
    pub keep_epochs: i32,
    
    /// When to start garbage collection (age in epochs)
    pub prune_after: i32,
    
    /// Strategy for creating register summaries
    pub summary_strategy: SummaryStrategy,
    
    /// Where to store archived register data
    pub archive_location: ArchiveLocation,
}

impl Default for ArchivalPolicy {
    fn default() -> Self {
        Self {
            keep_epochs: 2,          // Keep last 2 epochs fully accessible
            prune_after: 3,          // Start GC after 3 epochs
            summary_strategy: SummaryStrategy::SummarizeByResource,
            archive_location: ArchiveLocation::LocalStorage("./archives".to_string()),
        }
    }
}

/// Manages epochs and register lifecycle
pub struct EpochManager {
    /// Current epoch identifier
    current_epoch: RwLock<EpochId>,
    
    /// Mapping of epochs to block heights (boundaries)
    epoch_boundaries: RwLock<HashMap<EpochId, BlockHeight>>,
    
    /// Registers grouped by epoch
    registers_per_epoch: RwLock<HashMap<EpochId, HashSet<ContentId>>>,
    
    /// Archival policy configuration
    archival_policy: RwLock<ArchivalPolicy>,
}

impl EpochManager {
    /// Create a new epoch manager
    pub fn new() -> Self {
        let current_epoch = 1; // Start at epoch 1
        let mut epoch_boundaries = HashMap::new();
        epoch_boundaries.insert(current_epoch, 0); // Epoch 1 starts at block 0
        
        Self {
            current_epoch: RwLock::new(current_epoch),
            epoch_boundaries: RwLock::new(epoch_boundaries),
            registers_per_epoch: RwLock::new(HashMap::new()),
            archival_policy: RwLock::new(ArchivalPolicy::default()),
        }
    }
    
    /// Create a new epoch manager with specified starting epoch and policy
    pub fn with_config(
        starting_epoch: EpochId,
        starting_block: BlockHeight,
        policy: ArchivalPolicy,
    ) -> Self {
        let mut epoch_boundaries = HashMap::new();
        epoch_boundaries.insert(starting_epoch, starting_block);
        
        Self {
            current_epoch: RwLock::new(starting_epoch),
            epoch_boundaries: RwLock::new(epoch_boundaries),
            registers_per_epoch: RwLock::new(HashMap::new()),
            archival_policy: RwLock::new(policy),
        }
    }
    
    /// Get the current epoch
    pub fn current_epoch(&self) -> Result<EpochId> {
        let epoch = self.current_epoch.read().map_err(|_| 
            Error::LockError("Failed to acquire epoch lock".to_string())
        )?;
        Ok(*epoch)
    }
    
    /// Get the epoch boundary (block height) for a specific epoch
    pub fn get_epoch_boundary(&self, epoch: EpochId) -> Result<Option<BlockHeight>> {
        let boundaries = self.epoch_boundaries.read().map_err(|_| 
            Error::LockError("Failed to acquire epoch boundaries lock".to_string())
        )?;
        
        Ok(boundaries.get(&epoch).cloned())
    }
    
    /// Advance to the next epoch at a specific block height
    pub fn advance_epoch(&self, block_height: BlockHeight) -> Result<EpochId> {
        let mut current = self.current_epoch.write().map_err(|_| 
            Error::LockError("Failed to acquire epoch lock for writing".to_string())
        )?;
        
        let next_epoch = *current + 1;
        *current = next_epoch;
        
        // Set the boundary for this new epoch
        let mut boundaries = self.epoch_boundaries.write().map_err(|_| 
            Error::LockError("Failed to acquire epoch boundaries lock for writing".to_string())
        )?;
        
        boundaries.insert(next_epoch, block_height);
        
        // Create a new empty set for this epoch's registers
        let mut registers = self.registers_per_epoch.write().map_err(|_| 
            Error::LockError("Failed to acquire registers lock for writing".to_string())
        )?;
        
        registers.entry(next_epoch).or_insert_with(HashSet::new);
        
        Ok(next_epoch)
    }
    
    /// Define a custom epoch boundary
    pub fn set_epoch_boundary(&self, epoch: EpochId, block_height: BlockHeight) -> Result<()> {
        let mut boundaries = self.epoch_boundaries.write().map_err(|_| 
            Error::LockError("Failed to acquire epoch boundaries lock for writing".to_string())
        )?;
        
        boundaries.insert(epoch, block_height);
        
        Ok(())
    }
    
    /// Update the archival policy
    pub fn set_archival_policy(&self, policy: ArchivalPolicy) -> Result<()> {
        let mut current_policy = self.archival_policy.write().map_err(|_| 
            Error::LockError("Failed to acquire policy lock for writing".to_string())
        )?;
        
        *current_policy = policy;
        
        Ok(())
    }
    
    /// Get the current archival policy
    pub fn get_archival_policy(&self) -> Result<ArchivalPolicy> {
        let policy = self.archival_policy.read().map_err(|_| 
            Error::LockError("Failed to acquire policy lock".to_string())
        )?;
        
        Ok(policy.clone())
    }
    
    /// Register a new register in the current epoch
    pub fn register_in_current_epoch(&self, register_id: ContentId) -> Result<()> {
        let current = self.current_epoch.read().map_err(|_| 
            Error::LockError("Failed to acquire epoch lock".to_string())
        )?;
        
        self.register_in_epoch(register_id, *current)
    }
    
    /// Register a register in a specific epoch
    pub fn register_in_epoch(&self, register_id: ContentId, epoch: EpochId) -> Result<()> {
        let mut registers = self.registers_per_epoch.write().map_err(|_| 
            Error::LockError("Failed to acquire registers lock for writing".to_string())
        )?;
        
        registers
            .entry(epoch)
            .or_insert_with(HashSet::new)
            .insert(register_id);
        
        Ok(())
    }
    
    /// Get all registers in a specific epoch
    pub fn get_registers_in_epoch(&self, epoch: EpochId) -> Result<HashSet<ContentId>> {
        let registers = self.registers_per_epoch.read().map_err(|_| 
            Error::LockError("Failed to acquire registers lock".to_string())
        )?;
        
        match registers.get(&epoch) {
            Some(set) => Ok(set.clone()),
            None => Ok(HashSet::new()),
        }
    }
    
    /// Check if an epoch is eligible for garbage collection
    pub fn is_epoch_eligible_for_gc(&self, epoch: EpochId) -> Result<bool> {
        let current = self.current_epoch.read().map_err(|_| 
            Error::LockError("Failed to acquire epoch lock".to_string())
        )?;
        
        let policy = self.archival_policy.read().map_err(|_| 
            Error::LockError("Failed to acquire policy lock".to_string())
        )?;
        
        let age = *current - epoch;
        
        Ok(age >= policy.prune_after as u64)
    }
    
    /// Get the epoch for a specific block height
    pub fn get_epoch_for_block(&self, block_height: BlockHeight) -> Result<EpochId> {
        let boundaries = self.epoch_boundaries.read().map_err(|_| 
            Error::LockError("Failed to acquire epoch boundaries lock".to_string())
        )?;
        
        // Find the highest epoch with a boundary <= block_height
        let mut current_epoch = 1; // Default to first epoch
        
        for (&epoch, &boundary) in boundaries.iter() {
            if boundary <= block_height && epoch > current_epoch {
                current_epoch = epoch;
            }
        }
        
        Ok(current_epoch)
    }
    
    /// Remove a register from epoch tracking (used during garbage collection)
    pub fn remove_register(&self, register_id: &ContentId, epoch: EpochId) -> Result<bool> {
        let mut registers = self.registers_per_epoch.write().map_err(|_| 
            Error::LockError("Failed to acquire registers lock for writing".to_string())
        )?;
        
        match registers.get_mut(&epoch) {
            Some(set) => Ok(set.remove(register_id)),
            None => Ok(false),
        }
    }
}

/// Thread-safe shared epoch manager
pub struct SharedEpochManager {
    /// Inner epoch manager
    inner: Arc<EpochManager>,
}

impl SharedEpochManager {
    /// Create a new shared epoch manager
    pub fn new() -> Self {
        Self {
            inner: Arc::new(EpochManager::new()),
        }
    }
    
    /// Create a new shared epoch manager with specified configuration
    pub fn with_config(
        starting_epoch: EpochId,
        starting_block: BlockHeight,
        policy: ArchivalPolicy,
    ) -> Self {
        Self {
            inner: Arc::new(EpochManager::with_config(
                starting_epoch,
                starting_block,
                policy,
            )),
        }
    }
    
    /// Get the inner epoch manager
    pub fn inner(&self) -> Arc<EpochManager> {
        self.inner.clone()
    }
    
    // Delegate methods to inner epoch manager
    
    /// Get the current epoch
    pub fn current_epoch(&self) -> Result<EpochId> {
        self.inner.current_epoch()
    }
    
    /// Get the epoch boundary (block height) for a specific epoch
    pub fn get_epoch_boundary(&self, epoch: EpochId) -> Result<Option<BlockHeight>> {
        self.inner.get_epoch_boundary(epoch)
    }
    
    /// Advance to the next epoch at a specific block height
    pub fn advance_epoch(&self, block_height: BlockHeight) -> Result<EpochId> {
        self.inner.advance_epoch(block_height)
    }
    
    /// Define a custom epoch boundary
    pub fn set_epoch_boundary(&self, epoch: EpochId, block_height: BlockHeight) -> Result<()> {
        self.inner.set_epoch_boundary(epoch, block_height)
    }
    
    /// Update the archival policy
    pub fn set_archival_policy(&self, policy: ArchivalPolicy) -> Result<()> {
        self.inner.set_archival_policy(policy)
    }
    
    /// Get the current archival policy
    pub fn get_archival_policy(&self) -> Result<ArchivalPolicy> {
        self.inner.get_archival_policy()
    }
    
    /// Register a new register in the current epoch
    pub fn register_in_current_epoch(&self, register_id: ContentId) -> Result<()> {
        self.inner.register_in_current_epoch(register_id)
    }
    
    /// Register a register in a specific epoch
    pub fn register_in_epoch(&self, register_id: ContentId, epoch: EpochId) -> Result<()> {
        self.inner.register_in_epoch(register_id, epoch)
    }
    
    /// Get all registers in a specific epoch
    pub fn get_registers_in_epoch(&self, epoch: EpochId) -> Result<HashSet<ContentId>> {
        self.inner.get_registers_in_epoch(epoch)
    }
    
    /// Check if an epoch is eligible for garbage collection
    pub fn is_epoch_eligible_for_gc(&self, epoch: EpochId) -> Result<bool> {
        self.inner.is_epoch_eligible_for_gc(epoch)
    }
    
    /// Get the epoch for a specific block height
    pub fn get_epoch_for_block(&self, block_height: BlockHeight) -> Result<EpochId> {
        self.inner.get_epoch_for_block(block_height)
    }
    
    /// Remove a register from epoch tracking (used during garbage collection)
    pub fn remove_register(&self, register_id: &ContentId, epoch: EpochId) -> Result<bool> {
        self.inner.remove_register(register_id, epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_epoch_manager_basic() {
        let manager = EpochManager::new();
        
        // Initial epoch should be 1
        assert_eq!(manager.current_epoch().unwrap(), 1);
        
        // Advance to epoch 2
        let new_epoch = manager.advance_epoch(100).unwrap();
        assert_eq!(new_epoch, 2);
        assert_eq!(manager.current_epoch().unwrap(), 2);
        
        // Verify boundary
        let boundary = manager.get_epoch_boundary(2).unwrap();
        assert_eq!(boundary, Some(100));
    }
    
    #[test]
    fn test_register_tracking() {
        let manager = EpochManager::new();
        
        // Register in current epoch (1)
        let reg_id1 = ContentId::new_unique();
        let reg_id2 = ContentId::new_unique();
        
        manager.register_in_current_epoch(reg_id1.clone()).unwrap();
        manager.register_in_current_epoch(reg_id2.clone()).unwrap();
        
        // Verify registers in epoch 1
        let registers = manager.get_registers_in_epoch(1).unwrap();
        assert_eq!(registers.len(), 2);
        assert!(registers.contains(&reg_id1));
        assert!(registers.contains(&reg_id2));
        
        // Advance to epoch 2
        manager.advance_epoch(100).unwrap();
        
        // Register in epoch 2
        let reg_id3 = ContentId::new_unique();
        manager.register_in_current_epoch(reg_id3.clone()).unwrap();
        
        // Verify registers in each epoch
        let registers1 = manager.get_registers_in_epoch(1).unwrap();
        let registers2 = manager.get_registers_in_epoch(2).unwrap();
        
        assert_eq!(registers1.len(), 2);
        assert_eq!(registers2.len(), 1);
        assert!(registers2.contains(&reg_id3));
    }
    
    #[test]
    fn test_archival_policy() {
        let manager = EpochManager::new();
        
        // Default policy
        let default_policy = manager.get_archival_policy().unwrap();
        assert_eq!(default_policy.keep_epochs, 2);
        assert_eq!(default_policy.prune_after, 3);
        
        // Set custom policy
        let custom_policy = ArchivalPolicy {
            keep_epochs: 5,
            prune_after: 10,
            summary_strategy: SummaryStrategy::SummarizeByAccount,
            archive_location: ArchiveLocation::LocalStorage("/tmp/archives".to_string()),
        };
        
        manager.set_archival_policy(custom_policy.clone()).unwrap();
        
        // Verify updated policy
        let updated_policy = manager.get_archival_policy().unwrap();
        assert_eq!(updated_policy.keep_epochs, 5);
        assert_eq!(updated_policy.prune_after, 10);
        
        // Test GC eligibility
        manager.advance_epoch(100).unwrap(); // Epoch 2
        manager.advance_epoch(200).unwrap(); // Epoch 3
        
        // Epoch 1 shouldn't be eligible yet (current is 3, age = 2, prune_after = 10)
        assert!(!manager.is_epoch_eligible_for_gc(1).unwrap());
        
        // Advance many epochs
        for i in 4..15 {
            manager.advance_epoch(i * 100).unwrap();
        }
        
        // Now epoch 1 should be eligible (current is 14, age = 13, prune_after = 10)
        assert!(manager.is_epoch_eligible_for_gc(1).unwrap());
    }
    
    #[test]
    fn test_epoch_by_block_height() {
        let manager = EpochManager::new();
        
        // Set up some epoch boundaries
        // Epoch 1: 0-99
        // Epoch 2: 100-199
        // Epoch 3: 200-299
        manager.set_epoch_boundary(1, 0).unwrap();
        manager.advance_epoch(100).unwrap(); // Epoch 2
        manager.advance_epoch(200).unwrap(); // Epoch 3
        
        // Test block height to epoch mapping
        assert_eq!(manager.get_epoch_for_block(0).unwrap(), 1);
        assert_eq!(manager.get_epoch_for_block(50).unwrap(), 1);
        assert_eq!(manager.get_epoch_for_block(100).unwrap(), 2);
        assert_eq!(manager.get_epoch_for_block(150).unwrap(), 2);
        assert_eq!(manager.get_epoch_for_block(200).unwrap(), 3);
        assert_eq!(manager.get_epoch_for_block(250).unwrap(), 3);
        assert_eq!(manager.get_epoch_for_block(300).unwrap(), 3); // Beyond defined epochs defaults to latest
    }
    
    #[test]
    fn test_shared_epoch_manager() {
        let shared_manager = SharedEpochManager::new();
        
        // Test delegation to inner methods
        assert_eq!(shared_manager.current_epoch().unwrap(), 1);
        
        // Test advancing epoch
        shared_manager.advance_epoch(100).unwrap();
        assert_eq!(shared_manager.current_epoch().unwrap(), 2);
        
        // Test register tracking
        let reg_id = ContentId::new_unique();
        shared_manager.register_in_current_epoch(reg_id.clone()).unwrap();
        
        let registers = shared_manager.get_registers_in_epoch(2).unwrap();
        assert_eq!(registers.len(), 1);
        assert!(registers.contains(&reg_id));
    }
} 
