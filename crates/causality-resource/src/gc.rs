// Resource garbage collection
// Original file: src/resource/garbage_collection.rs

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use causality_types::{Error, Result};
use crate::resource::{
    Register, ContentId, RegisterState, ArchiveReference,
    SharedEpochManager, SharedArchiveManager, EpochId,
};

/// Configuration for garbage collection policy
#[derive(Debug, Clone)]
pub struct GarbageCollectionConfig {
    /// Number of epochs to retain before considering for garbage collection
    pub retention_epochs: usize,
    
    /// Whether to perform automatic garbage collection when advancing epochs
    pub auto_gc_on_epoch_advance: bool,
    
    /// Minimum age of a register before it can be garbage collected (in seconds)
    pub min_age_seconds: Option<u64>,
    
    /// Whether to require registers to be archived before garbage collection
    pub require_archived: bool,
    
    /// Whether to delete archives when garbage collecting
    pub delete_archives: bool,
    
    /// Custom predicate for determining if a register can be garbage collected
    pub custom_gc_predicate: Option<Arc<dyn Fn(&Register) -> bool + Send + Sync>>,
}

impl Default for GarbageCollectionConfig {
    fn default() -> Self {
        Self {
            retention_epochs: 2,
            auto_gc_on_epoch_advance: false,
            min_age_seconds: Some(86400), // 1 day
            require_archived: true,
            delete_archives: false,
            custom_gc_predicate: None,
        }
    }
}

/// Manager for garbage collection of registers
pub struct GarbageCollectionManager {
    /// Configuration for garbage collection policy
    config: GarbageCollectionConfig,
    
    /// Epoch manager reference for epoch-based garbage collection
    epoch_manager: Option<SharedEpochManager>,
    
    /// Archive manager reference for archive deletion during garbage collection
    archive_manager: Option<SharedArchiveManager>,
    
    /// Map of register IDs that have been garbage collected to their deletion time
    deleted_registers: HashMap<ContentId, SystemTime>,
}

impl GarbageCollectionManager {
    /// Create a new garbage collection manager with the given configuration
    pub fn new(
        config: GarbageCollectionConfig,
        epoch_manager: Option<SharedEpochManager>,
        archive_manager: Option<SharedArchiveManager>,
    ) -> Self {
        Self {
            config,
            epoch_manager,
            archive_manager,
            deleted_registers: HashMap::new(),
        }
    }
    
    /// Create a new garbage collection manager with default configuration
    pub fn with_default_config(
        epoch_manager: Option<SharedEpochManager>,
        archive_manager: Option<SharedArchiveManager>,
    ) -> Self {
        Self::new(
            GarbageCollectionConfig::default(),
            epoch_manager,
            archive_manager,
        )
    }
    
    /// Check if a register is eligible for garbage collection
    pub fn is_eligible_for_gc(&self, register: &Register) -> bool {
        // Skip if the register is not in a final state
        if !matches!(register.state, RegisterState::Archived | RegisterState::Consumed) {
            return false;
        }
        
        // If archived state is required, check that condition
        if self.config.require_archived && register.state != RegisterState::Archived {
            return false;
        }
        
        // Check epoch-based retention policy
        if let Some(epoch_manager) = &self.epoch_manager {
            if let Some(register_epoch) = register.epoch {
                let current_epoch = epoch_manager.get_current_epoch();
                if current_epoch <= register_epoch + self.config.retention_epochs {
                    return false;
                }
            }
        }
        
        // Check age-based retention policy
        if let Some(min_age_secs) = self.config.min_age_seconds {
            if let Some(modified_at) = register.modified_at {
                let age = SystemTime::now()
                    .duration_since(modified_at)
                    .unwrap_or(Duration::from_secs(0));
                
                if age.as_secs() < min_age_secs {
                    return false;
                }
            }
        }
        
        // Check custom predicate if provided
        if let Some(predicate) = &self.config.custom_gc_predicate {
            if !predicate(register) {
                return false;
            }
        }
        
        true
    }
    
    /// Mark a register as garbage collected
    pub fn mark_as_collected(&mut self, register_id: &ContentId) {
        self.deleted_registers.insert(register_id.clone(), SystemTime::now());
    }
    
    /// Check if a register has been garbage collected
    pub fn is_collected(&self, register_id: &ContentId) -> bool {
        self.deleted_registers.contains_key(register_id)
    }
    
    /// Delete an archive if configured to do so
    pub fn delete_archive_if_configured(&self, archive_ref: &ArchiveReference) -> Result<bool> {
        if !self.config.delete_archives {
            return Ok(false);
        }
        
        if let Some(archive_manager) = &self.archive_manager {
            archive_manager.delete_archive(archive_ref)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Get all registers eligible for garbage collection by epoch
    pub fn get_eligible_registers_by_epoch(&self, 
        registers: &HashMap<ContentId, Register>,
        target_epoch: EpochId
    ) -> Vec<ContentId> {
        registers.iter()
            .filter(|(_, register)| {
                register.epoch == Some(target_epoch) && self.is_eligible_for_gc(register)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Get number of epochs to retain before garbage collection
    pub fn get_retention_epochs(&self) -> usize {
        self.config.retention_epochs
    }
    
    /// Check if auto garbage collection on epoch advance is enabled
    pub fn is_auto_gc_on_epoch_advance(&self) -> bool {
        self.config.auto_gc_on_epoch_advance
    }
    
    /// Get minimum age (in seconds) for garbage collection eligibility
    pub fn get_min_age_seconds(&self) -> Option<u64> {
        self.config.min_age_seconds
    }
    
    /// Check if registers must be archived before garbage collection
    pub fn is_archive_required(&self) -> bool {
        self.config.require_archived
    }
    
    /// Check if archives should be deleted when garbage collecting
    pub fn should_delete_archives(&self) -> bool {
        self.config.delete_archives
    }
    
    /// Update the garbage collection configuration
    pub fn update_config(&mut self, config: GarbageCollectionConfig) {
        self.config = config;
    }
    
    /// Get a reference to the garbage collection configuration
    pub fn get_config(&self) -> &GarbageCollectionConfig {
        &self.config
    }
    
    /// Get a list of all garbage collected register IDs
    pub fn get_collected_register_ids(&self) -> Vec<ContentId> {
        self.deleted_registers.keys().cloned().collect()
    }
    
    /// Get the time when a register was garbage collected
    pub fn get_collection_time(&self, register_id: &ContentId) -> Option<SystemTime> {
        self.deleted_registers.get(register_id).cloned()
    }
    
    /// Clear the garbage collection history
    pub fn clear_collection_history(&mut self) {
        self.deleted_registers.clear();
    }
    
    /// Retain collection history for only specified register IDs
    pub fn retain_collection_history(&mut self, register_ids: &HashSet<ContentId>) {
        self.deleted_registers.retain(|id, _| register_ids.contains(id));
    }
    
    /// Get the number of records in the garbage collection history
    pub fn collection_history_size(&self) -> usize {
        self.deleted_registers.len()
    }
}

/// Thread-safe shared garbage collection manager
pub struct SharedGarbageCollectionManager {
    /// Inner garbage collection manager wrapped in Arc and Mutex for thread safety
    inner: Arc<Mutex<GarbageCollectionManager>>,
}

impl SharedGarbageCollectionManager {
    /// Create a new shared garbage collection manager with the given configuration
    pub fn new(
        config: GarbageCollectionConfig,
        epoch_manager: Option<SharedEpochManager>,
        archive_manager: Option<SharedArchiveManager>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(GarbageCollectionManager::new(
                config,
                epoch_manager,
                archive_manager,
            ))),
        }
    }
    
    /// Create a new shared garbage collection manager with default configuration
    pub fn with_default_config(
        epoch_manager: Option<SharedEpochManager>,
        archive_manager: Option<SharedArchiveManager>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(GarbageCollectionManager::with_default_config(
                epoch_manager,
                archive_manager,
            ))),
        }
    }
    
    /// Check if a register is eligible for garbage collection
    pub fn is_eligible_for_gc(&self, register: &Register) -> Result<bool> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.is_eligible_for_gc(register))
    }
    
    /// Mark a register as garbage collected
    pub fn mark_as_collected(&self, register_id: &ContentId) -> Result<()> {
        let mut gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        gc_manager.mark_as_collected(register_id);
        Ok(())
    }
    
    /// Check if a register has been garbage collected
    pub fn is_collected(&self, register_id: &ContentId) -> Result<bool> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.is_collected(register_id))
    }
    
    /// Delete an archive if configured to do so
    pub fn delete_archive_if_configured(&self, archive_ref: &ArchiveReference) -> Result<bool> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        gc_manager.delete_archive_if_configured(archive_ref)
    }
    
    /// Get all registers eligible for garbage collection by epoch
    pub fn get_eligible_registers_by_epoch(
        &self,
        registers: &HashMap<ContentId, Register>,
        target_epoch: EpochId
    ) -> Result<Vec<ContentId>> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.get_eligible_registers_by_epoch(registers, target_epoch))
    }
    
    /// Get number of epochs to retain before garbage collection
    pub fn get_retention_epochs(&self) -> Result<usize> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.get_retention_epochs())
    }
    
    /// Check if auto garbage collection on epoch advance is enabled
    pub fn is_auto_gc_on_epoch_advance(&self) -> Result<bool> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.is_auto_gc_on_epoch_advance())
    }
    
    /// Update the garbage collection configuration
    pub fn update_config(&self, config: GarbageCollectionConfig) -> Result<()> {
        let mut gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        gc_manager.update_config(config);
        Ok(())
    }
    
    /// Get a list of all garbage collected register IDs
    pub fn get_collected_register_ids(&self) -> Result<Vec<ContentId>> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.get_collected_register_ids())
    }
    
    /// Get the time when a register was garbage collected
    pub fn get_collection_time(&self, register_id: &ContentId) -> Result<Option<SystemTime>> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.get_collection_time(register_id))
    }
    
    /// Clear the garbage collection history
    pub fn clear_collection_history(&self) -> Result<()> {
        let mut gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        gc_manager.clear_collection_history();
        Ok(())
    }
    
    /// Get the number of records in the garbage collection history
    pub fn collection_history_size(&self) -> Result<usize> {
        let gc_manager = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(gc_manager.collection_history_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::{RegisterContents, RegisterMetadata};
    use causality_types::{Address, Domain};
    use std::collections::HashMap;
    
    #[test]
    fn test_gc_eligibility() {
        // Create a GC manager with default config
        let gc_manager = GarbageCollectionManager::with_default_config(None, None);
        
        // Create a register that's active
        let active_register = Register {
            register_id: ContentId::new_v4(),
            owner: Address::new("owner"),
            domain: Domain::new("domain"),
            contents: RegisterContents::with_string("test"),
            metadata: RegisterMetadata::new(),
            state: RegisterState::Active,
            created_at: SystemTime::now(),
            modified_at: Some(SystemTime::now()),
            epoch: Some(0),
            archive_reference: None,
        };
        
        // Check that an active register is not eligible
        assert!(!gc_manager.is_eligible_for_gc(&active_register));
        
        // Create a register that's consumed but too new
        let now = SystemTime::now();
        let consumed_register = Register {
            register_id: ContentId::new_v4(),
            owner: Address::new("owner"),
            domain: Domain::new("domain"),
            contents: RegisterContents::with_string("test"),
            metadata: RegisterMetadata::new(),
            state: RegisterState::Consumed,
            created_at: now,
            modified_at: Some(now),
            epoch: Some(0),
            archive_reference: None,
        };
        
        // Should not be eligible due to age
        assert!(!gc_manager.is_eligible_for_gc(&consumed_register));
        
        // Create a register that's archived but in current epoch
        let archived_register = Register {
            register_id: ContentId::new_v4(),
            owner: Address::new("owner"),
            domain: Domain::new("domain"),
            contents: RegisterContents::with_string("test"),
            metadata: RegisterMetadata::new(),
            state: RegisterState::Archived,
            created_at: SystemTime::now(),
            modified_at: Some(SystemTime::now().checked_sub(Duration::from_secs(100000)).unwrap()),
            epoch: Some(0),
            archive_reference: Some(ArchiveReference {
                epoch: 0,
                archive_hash: "hash".to_string(),
            }),
        };
        
        // Should not be eligible due to being in current epoch (with default retention of 2)
        assert!(!gc_manager.is_eligible_for_gc(&archived_register));
        
        // Create a custom config with no retention epochs and no min age
        let custom_config = GarbageCollectionConfig {
            retention_epochs: 0,
            min_age_seconds: None,
            ..Default::default()
        };
        
        let custom_gc_manager = GarbageCollectionManager::new(custom_config, None, None);
        
        // Now the archived register should be eligible with custom config
        assert!(custom_gc_manager.is_eligible_for_gc(&archived_register));
    }
    
    #[test]
    fn test_gc_collection_tracking() {
        // Create a GC manager
        let mut gc_manager = GarbageCollectionManager::with_default_config(None, None);
        
        // Create some register IDs
        let register_id1 = ContentId::new_v4();
        let register_id2 = ContentId::new_v4();
        
        // Initially nothing is collected
        assert!(!gc_manager.is_collected(&register_id1));
        assert!(!gc_manager.is_collected(&register_id2));
        
        // Mark one as collected
        gc_manager.mark_as_collected(&register_id1);
        
        // Verify tracking
        assert!(gc_manager.is_collected(&register_id1));
        assert!(!gc_manager.is_collected(&register_id2));
        
        // Check collection time
        assert!(gc_manager.get_collection_time(&register_id1).is_some());
        assert!(gc_manager.get_collection_time(&register_id2).is_none());
        
        // Get collected IDs
        let collected_ids = gc_manager.get_collected_register_ids();
        assert_eq!(collected_ids.len(), 1);
        assert!(collected_ids.contains(&register_id1));
        
        // Clear collection history
        gc_manager.clear_collection_history();
        assert!(!gc_manager.is_collected(&register_id1));
        assert_eq!(gc_manager.collection_history_size(), 0);
    }
    
    #[test]
    fn test_shared_gc_manager() {
        // Create a shared GC manager
        let shared_gc_manager = SharedGarbageCollectionManager::with_default_config(None, None);
        
        // Create a register ID
        let register_id = ContentId::new_v4();
        
        // Initially not collected
        assert!(!shared_gc_manager.is_collected(&register_id).unwrap());
        
        // Mark as collected
        shared_gc_manager.mark_as_collected(&register_id).unwrap();
        
        // Should now be collected
        assert!(shared_gc_manager.is_collected(&register_id).unwrap());
        
        // Get collection time
        assert!(shared_gc_manager.get_collection_time(&register_id).unwrap().is_some());
        
        // Clear collection history
        shared_gc_manager.clear_collection_history().unwrap();
        
        // Should no longer be marked as collected
        assert!(!shared_gc_manager.is_collected(&register_id).unwrap());
    }
    
    #[test]
    fn test_custom_gc_predicate() {
        // Create a custom predicate that only allows registers with certain metadata
        let predicate = Arc::new(|register: &Register| -> bool {
            register.metadata.get("can_gc").map_or(false, |v| v == "true")
        });
        
        // Create config with custom predicate
        let config = GarbageCollectionConfig {
            min_age_seconds: None,
            retention_epochs: 0,
            custom_gc_predicate: Some(predicate),
            ..Default::default()
        };
        
        let gc_manager = GarbageCollectionManager::new(config, None, None);
        
        // Create a register with the required metadata
        let mut eligible_metadata = RegisterMetadata::new();
        eligible_metadata.insert("can_gc".to_string(), "true".to_string());
        
        let eligible_register = Register {
            register_id: ContentId::new_v4(),
            owner: Address::new("owner"),
            domain: Domain::new("domain"),
            contents: RegisterContents::with_string("test"),
            metadata: eligible_metadata,
            state: RegisterState::Archived,
            created_at: SystemTime::now(),
            modified_at: Some(SystemTime::now()),
            epoch: Some(0),
            archive_reference: Some(ArchiveReference {
                epoch: 0,
                archive_hash: "hash".to_string(),
            }),
        };
        
        // Create a register without the required metadata
        let ineligible_register = Register {
            register_id: ContentId::new_v4(),
            owner: Address::new("owner"),
            domain: Domain::new("domain"),
            contents: RegisterContents::with_string("test"),
            metadata: RegisterMetadata::new(),
            state: RegisterState::Archived,
            created_at: SystemTime::now(),
            modified_at: Some(SystemTime::now()),
            epoch: Some(0),
            archive_reference: Some(ArchiveReference {
                epoch: 0,
                archive_hash: "hash".to_string(),
            }),
        };
        
        // Check eligibility
        assert!(gc_manager.is_eligible_for_gc(&eligible_register));
        assert!(!gc_manager.is_eligible_for_gc(&ineligible_register));
    }
} 
