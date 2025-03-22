// One-Time Use Register System
//
// This module brings together the components for the one-time use register system:
// - Register lifecycle stages
// - Nullifier mechanism
// - Transition system
// - Register versioning
// - Epoch management
//
// The system provides a complete implementation of the register lifecycle
// described in ADR-006: ZK-Based Register System for Domain Adapters.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::error::{Error, Result};
use crate::types::{Domain, Address};
use crate::resource::register::{
    RegisterId, RegisterContents, Register, BlockHeight, RegisterOperation, OperationType
};
use crate::resource::lifecycle::{
    RegisterState, TransitionReason, StateTransition, RegisterLifecycleManager
};
use crate::resource::nullifier::{
    RegisterNullifier, NullifierStatus, SharedNullifierRegistry
};
use crate::resource::transition::{
    TransitionType, RegisterTransition, TransitionObserver, SharedTransitionSystem
};
use crate::resource::versioning::{
    SchemaVersion, VersionMigration, MigrationRegistry, SharedMigrationRegistry
};
use crate::resource::epoch::{
    EpochId, EpochManager, SharedEpochManager, ArchivalPolicy
};
use crate::resource::summarization::{
    SharedSummaryManager, SummaryStrategy, SummaryRecord
};
use crate::resource::archival::{
    SharedArchiveManager, CompressionFormat, ArchiveReference
};
use crate::resource::{
    SharedGarbageCollectionManager, GarbageCollectionConfig
};
use crate::domain::{DomainId, DomainRegistry, DomainAdapter};
use crate::domain::fact::{FactQuery};
use crate::log::fact_types::FactType;
use crate::domain::{Transaction, TransactionId, TransactionReceipt};
use crate::domain::map::map::{TimeMap, TimeMapEntry, SharedTimeMap};
use crate::log::fact::{FactLogger, FactMetadata};
use crate::resource::fact_observer::RegisterFactObserver;

/// Configuration for the one-time register system
#[derive(Debug, Clone)]
pub struct OneTimeRegisterConfig {
    /// Current block height
    pub current_block_height: BlockHeight,
    
    /// Number of blocks after which pending nullifiers expire
    pub nullifier_timeout: u64,
    
    /// Initial observers for register transitions
    pub initial_observers: Vec<Arc<dyn TransitionObserver>>,
    
    /// Migration registry for version migrations
    pub migration_registry: Option<SharedMigrationRegistry>,
    
    /// Epoch manager for lifecycle management
    pub epoch_manager: Option<SharedEpochManager>,
    
    /// Summary manager for register summarization
    pub summary_manager: Option<SharedSummaryManager>,
    
    /// Archive manager for register archival
    pub archive_manager: Option<SharedArchiveManager>,
    
    /// Shared garbage collection manager for register garbage collection
    pub gc_manager: Option<SharedGarbageCollectionManager>,
    
    /// Domain registry for domain adapter integration
    pub domain_registry: Option<Arc<DomainRegistry>>,

    /// Shared time map for cross-domain time synchronization
    pub time_map: Option<SharedTimeMap>,
    
    /// Fact logger for tracking register operations as facts
    pub fact_logger: Option<Arc<FactLogger>>,
    
    /// Observer name for fact logging
    pub fact_observer_name: Option<String>,
}

impl Default for OneTimeRegisterConfig {
    fn default() -> Self {
        Self {
            current_block_height: 0,
            nullifier_timeout: 100, // Default timeout of 100 blocks
            initial_observers: Vec::new(),
            migration_registry: None,
            epoch_manager: None,
            summary_manager: None,
            archive_manager: None,
            gc_manager: None,
            domain_registry: None,
            time_map: None,
            fact_logger: None,
            fact_observer_name: None,
        }
    }
}

/// One-time register system
///
/// Manages registers that can only be consumed once
pub struct OneTimeRegisterSystem {
    /// Registry for nullifiers
    nullifier_registry: SharedNullifierRegistry,
    
    /// System for managing transitions
    transition_system: SharedTransitionSystem,
    
    /// Migration registry for register version migrations
    migration_registry: SharedMigrationRegistry,
    
    /// Epoch manager for lifecycle management
    epoch_manager: Option<SharedEpochManager>,
    
    /// Summary manager for register summarization
    summary_manager: Option<SharedSummaryManager>,
    
    /// Archive manager for register archival
    archive_manager: Option<SharedArchiveManager>,
    
    /// Garbage collection manager for register garbage collection
    gc_manager: Option<SharedGarbageCollectionManager>,
    
    /// Domain registry for domain adapter integration
    domain_registry: Option<Arc<DomainRegistry>>,
    
    /// Mapping from register IDs to domain IDs
    register_to_domain: RwLock<HashMap<RegisterId, DomainId>>,
    
    /// Mapping from domain IDs to register IDs
    domain_to_registers: RwLock<HashMap<DomainId, HashSet<RegisterId>>>,

    /// Shared time map for cross-domain time synchronization
    time_map: Option<SharedTimeMap>,
    
    /// Fact observer for logging register operations as facts
    fact_observer: Option<Arc<RegisterFactObserver>>,
}

impl OneTimeRegisterSystem {
    /// Create a new one-time register system
    pub fn new(config: OneTimeRegisterConfig) -> Result<Self> {
        // Create the nullifier registry
        let nullifier_registry = SharedNullifierRegistry::new(
            config.current_block_height,
            config.nullifier_timeout,
        );
        
        // Create the transition system
        let transition_system = SharedTransitionSystem::new(
            config.current_block_height,
        );
        
        // Add initial observers
        for observer in config.initial_observers {
            transition_system.add_observer(observer)?;
        }
        
        let migration_registry = config.migration_registry
            .unwrap_or_else(|| SharedMigrationRegistry::new());
            
        let epoch_manager = config.epoch_manager;
            
        let summary_manager = config.summary_manager;
            
        let archive_manager = config.archive_manager;
        
        let gc_manager = config.gc_manager;
        
        // Initialize the system
        let mut system = Self {
            nullifier_registry,
            transition_system,
            migration_registry,
            epoch_manager,
            summary_manager,
            archive_manager,
            gc_manager,
            domain_registry: config.domain_registry,
            register_to_domain: RwLock::new(HashMap::new()),
            domain_to_registers: RwLock::new(HashMap::new()),
            time_map: config.time_map,
            fact_observer: None,
        };
        
        // If we have both epoch and archive managers but no garbage collection manager,
        // create a default one
        if system.gc_manager.is_none() && 
           system.epoch_manager.is_some() && 
           system.archive_manager.is_some() {
            system.gc_manager = Some(SharedGarbageCollectionManager::with_default_config(
                system.epoch_manager.clone(),
                system.archive_manager.clone(),
            ));
        }
        
        // Set up fact observer if logger is provided
        if let Some(logger) = config.fact_logger {
            let observer_name = config.fact_observer_name
                .unwrap_or_else(|| "register-system".to_string());
            
            // Create observer with domain ID from logger
            let domain_id = logger.domain_id().clone();
            
            // Create and store the fact observer
            let fact_observer = Arc::new(RegisterFactObserver::new(
                logger,
                observer_name,
                domain_id,
            ));
            
            system.fact_observer = Some(fact_observer);
        }
        
        Ok(system)
    }
    
    /// Update the current block height
    pub fn update_block_height(&self, block_height: BlockHeight) -> Result<()> {
        self.nullifier_registry.update_block_height(block_height)?;
        self.transition_system.update_block_height(block_height)?;
        
        // Check if we need to advance the epoch
        let current_epoch_boundary = self.get_current_epoch_boundary()?;
        
        if let Some(boundary) = current_epoch_boundary {
            if block_height >= boundary + 100 { // Advance every 100 blocks by default
                self.advance_epoch()?;
            }
        }
        
        Ok(())
    }
    
    /// Get the current epoch
    pub fn get_current_epoch(&self) -> Result<EpochId> {
        if let Some(epoch_manager) = &self.epoch_manager {
            epoch_manager.current_epoch()
        } else {
            Err(Error::ManagerNotAvailable("Epoch manager not available".to_string()))
        }
    }
    
    /// Get the block height boundary for the current epoch
    pub fn get_current_epoch_boundary(&self) -> Result<Option<BlockHeight>> {
        if let Some(epoch_manager) = &self.epoch_manager {
            let current_epoch = self.get_current_epoch()?;
            epoch_manager.get_epoch_boundary(current_epoch)
        } else {
            Ok(None)
        }
    }
    
    /// Advance to the next epoch
    pub fn advance_epoch(&mut self) -> Result<EpochId> {
        if let Some(epoch_manager) = &self.epoch_manager {
            let old_epoch = self.get_current_epoch()?;
            let new_epoch = epoch_manager.advance_epoch(old_epoch)?;
            
            // Auto-GC if configured
            let _ = self.auto_gc_on_epoch_advance(old_epoch);
            
            Ok(new_epoch)
        } else {
            Err(Error::ManagerNotAvailable("Epoch manager not available".to_string()))
        }
    }
    
    /// Add a transition observer
    pub fn add_observer(&self, observer: Arc<dyn TransitionObserver>) -> Result<()> {
        self.transition_system.add_observer(observer)
    }
    
    /// Create a register
    pub fn create_register(
        &self,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        transaction_id: &str,
    ) -> Result<Register> {
        // Generate a unique register ID
        let register_id = RegisterId::new_unique();
        
        // Get the current block height
        let current_block_height = self.get_current_block_height()?;
        
        // Create the register
        let register = Register::new(
            register_id,
            owner,
            domain,
            contents,
            HashMap::new(),
            current_block_height,
            current_block_height,
        );
        
        // Get the nullifier registry
        let mut nullifier_registry = self.nullifier_registry.write()
            .map_err(|_| Error::LockError("Failed to acquire nullifier_registry lock".to_string()))?;
        
        // Create a nullifier for the register
        let nullifier = RegisterNullifier::new(register.register_id.clone(), current_block_height);
        nullifier_registry.insert(
            register.register_id.clone(),
            nullifier
        );
        
        // Add the register to the transition system
        let mut transition_system = self.transition_system.write()
            .map_err(|_| Error::LockError("Failed to acquire transition_system lock".to_string()))?;
        
        let transition = RegisterTransition::new(
            register.register_id.clone(),
            None, // No previous state
            register.state.clone(),
            TransitionType::Creation,
            transaction_id.to_string(),
            "Initial creation".to_string(),
            current_block_height,
        );
        
        transition_system.entry(register.register_id.clone())
            .or_insert_with(Vec::new)
            .push(transition);
        
        // Log the register creation as a fact
        self.log_register_creation(&register, transaction_id)?;
        
        Ok(register)
    }
    
    /// Consume a register
    pub fn consume_register(
        &self,
        register: &mut Register,
        transaction_id: &str,
        successors: Vec<RegisterId>,
    ) -> Result<RegisterTransition> {
        // Get the current block height
        let current_block_height = self.get_current_block_height()?;
        
        // Verify the register is in a state that can be consumed
        if register.state != RegisterState::Active && register.state != RegisterState::Locked {
            return Err(Error::InvalidState(format!(
                "Register {} is in state {}, cannot be consumed",
                register.register_id, register.state
            )));
        }
        
        // Update the register state
        let previous_state = register.state.clone();
        register.state = RegisterState::Consumed;
        register.updated_at = current_block_height;
        
        // Record the transition
        let mut transition_system = self.transition_system.write()
            .map_err(|_| Error::LockError("Failed to acquire transition_system lock".to_string()))?;
        
        let transition = RegisterTransition::new(
            register.register_id.clone(),
            Some(previous_state.clone()),
            register.state.clone(),
            TransitionType::Consumption,
            transaction_id.to_string(),
            "Register consumed".to_string(),
            current_block_height,
        );
        
        transition_system.entry(register.register_id.clone())
            .or_insert_with(Vec::new)
            .push(transition.clone());
        
        // Mark the nullifier as spent
        let mut nullifier_registry = self.nullifier_registry.write()
            .map_err(|_| Error::LockError("Failed to acquire nullifier_registry lock".to_string()))?;
        
        if let Some(nullifier) = nullifier_registry.get_mut(&register.register_id) {
            nullifier.mark_as_spent(current_block_height);
            
            // Log the register consumption as a fact
            let nullifier_hash = nullifier.nullifier_hash().to_string();
            self.log_register_consumption(register, transaction_id, &nullifier_hash, successors)?;
        }
        
        // Log state change
        self.log_register_state_change(register, previous_state, "Register consumed", transaction_id)?;
        
        Ok(transition)
    }
    
    /// Archive a register
    pub fn archive_register(
        &self,
        register_id: &RegisterId,
    ) -> Result<ArchiveReference> {
        // Retrieve the register
        let register = self.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register not found: {}", register_id)))?;
            
        // Check that the register is not already archived
        if register.state == RegisterState::Archived {
            return Err(Error::InvalidOperation(format!(
                "Register {} is already archived", register_id
            )));
        }
        
        // Get current epoch and block height
        let epoch = self.get_current_epoch()?;
        let block_height = self.nullifier_registry.current_block_height();
        
        // Archive the register
        let archive_ref = self.archive_manager.as_ref()
            .unwrap()
            .archive_register(&register, epoch, block_height)?;
        
        // Update the register to archived state
        let mut updated_register = register.clone();
        updated_register.state = RegisterState::Archived;
        updated_register.archive_reference = Some(archive_ref.clone());
        
        // Update the local register
        self.update_register(&updated_register)?;
        
        Ok(archive_ref)
    }
    
    /// Retrieve a register from archive
    pub fn retrieve_from_archive(
        &self,
        archive_ref: &ArchiveReference,
    ) -> Result<Option<Register>> {
        self.archive_manager.as_ref()
            .unwrap()
            .retrieve_register(archive_ref)
    }
    
    /// Verify an archive exists and is intact
    pub fn verify_archive(
        &self,
        archive_ref: &ArchiveReference,
    ) -> Result<bool> {
        self.archive_manager.as_ref()
            .unwrap()
            .verify_archive(archive_ref)
    }
    
    /// Archive all registers in a specified epoch
    pub fn archive_epoch(&self, epoch: EpochId) -> Result<Vec<ArchiveReference>> {
        // Get all registers for the epoch
        let registers = self.get_registers_for_epoch(epoch)?;
        
        // Skip if no registers
        if registers.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut archive_refs = Vec::new();
        let current_block_height = self.nullifier_registry.current_block_height();
        
        // Archive each register
        for register in registers {
            // Skip already archived registers
            if register.state == RegisterState::Archived {
                continue;
            }
            
            // Archive the register
            let archive_ref = self.archive_manager.as_ref()
                .unwrap()
                .archive_register(
                    &register, 
                    epoch, 
                    current_block_height
                )?;
            
            // Update the register to archived state
            let mut updated_register = register.clone();
            updated_register.state = RegisterState::Archived;
            updated_register.archive_reference = Some(archive_ref.clone());
            
            // Update the local register
            self.update_register(&updated_register)?;
            
            archive_refs.push(archive_ref);
        }
        
        Ok(archive_refs)
    }
    
    /// Get archive manager
    pub fn archive_manager(&self) -> &SharedArchiveManager {
        self.archive_manager.as_ref().unwrap()
    }
    
    /// Convert a register to a summary
    pub fn summarize_registers(
        &self,
        register: &mut Register,
        summarized_registers: Vec<RegisterId>,
        summary_method: &str,
    ) -> Result<RegisterTransition> {
        let transition = self.transition_system.summarize_registers(
            register,
            summarized_registers,
            summary_method,
        )?;
        
        // Register the summary in the current epoch
        if let Some(epoch_manager) = &self.epoch_manager {
            epoch_manager.register_in_current_epoch(register.register_id.clone())?;
        }
        
        Ok(transition)
    }
    
    /// Lock a register
    pub fn lock_register(
        &self,
        register: &mut Register,
        reason: &str,
    ) -> Result<RegisterTransition> {
        // Get the current block height
        let current_block_height = self.get_current_block_height()?;
        
        // Verify the register is in a state that can be locked
        if register.state != RegisterState::Active {
            return Err(Error::InvalidState(format!(
                "Register {} is in state {}, cannot be locked",
                register.register_id, register.state
            )));
        }
        
        // Update the register state
        let previous_state = register.state.clone();
        register.state = RegisterState::Locked;
        register.updated_at = current_block_height;
        
        // Record the transition
        let mut transition_system = self.transition_system.write()
            .map_err(|_| Error::LockError("Failed to acquire transition_system lock".to_string()))?;
        
        let transaction_id = format!("lock-{}", uuid::Uuid::new_v4());
        let description = format!("Register locked: {}", reason);
        
        let transition = RegisterTransition::new(
            register.register_id.clone(),
            Some(previous_state.clone()),
            register.state.clone(),
            TransitionType::Locking,
            transaction_id.clone(),
            description,
            current_block_height,
        );
        
        transition_system.entry(register.register_id.clone())
            .or_insert_with(Vec::new)
            .push(transition.clone());
        
        // Log the register lock as a fact
        self.log_register_lock(register, reason, &transaction_id)?;
        
        // Log state change
        self.log_register_state_change(register, previous_state, reason, &transaction_id)?;
        
        Ok(transition)
    }
    
    /// Unlock a register
    pub fn unlock_register(
        &self,
        register: &mut Register,
        reason: &str,
    ) -> Result<RegisterTransition> {
        // Get the current block height
        let current_block_height = self.get_current_block_height()?;
        
        // Verify the register is in a state that can be unlocked
        if register.state != RegisterState::Locked {
            return Err(Error::InvalidState(format!(
                "Register {} is in state {}, cannot be unlocked",
                register.register_id, register.state
            )));
        }
        
        // Update the register state
        let previous_state = register.state.clone();
        register.state = RegisterState::Active;
        register.updated_at = current_block_height;
        
        // Record the transition
        let mut transition_system = self.transition_system.write()
            .map_err(|_| Error::LockError("Failed to acquire transition_system lock".to_string()))?;
        
        let transaction_id = format!("unlock-{}", uuid::Uuid::new_v4());
        let description = format!("Register unlocked: {}", reason);
        
        let transition = RegisterTransition::new(
            register.register_id.clone(),
            Some(previous_state.clone()),
            register.state.clone(),
            TransitionType::Unlocking,
            transaction_id.clone(),
            description,
            current_block_height,
        );
        
        transition_system.entry(register.register_id.clone())
            .or_insert_with(Vec::new)
            .push(transition.clone());
        
        // Log the register unlock as a fact
        self.log_register_unlock(register, reason, &transaction_id)?;
        
        // Log state change
        self.log_register_state_change(register, previous_state, reason, &transaction_id)?;
        
        Ok(transition)
    }
    
    /// Freeze a register
    pub fn freeze_register(
        &self,
        register: &mut Register,
        reason: &str,
    ) -> Result<RegisterTransition> {
        self.transition_system.change_register_state(
            register,
            RegisterState::Frozen,
            TransitionReason::UserAction(reason.to_string()),
        )
    }
    
    /// Unfreeze a register
    pub fn unfreeze_register(
        &self,
        register: &mut Register,
        reason: &str,
    ) -> Result<RegisterTransition> {
        self.transition_system.change_register_state(
            register,
            RegisterState::Active,
            TransitionReason::UserAction(reason.to_string()),
        )
    }
    
    /// Mark a register for deletion
    pub fn mark_register_for_deletion(
        &self,
        register: &mut Register,
        reason: &str,
    ) -> Result<RegisterTransition> {
        self.transition_system.change_register_state(
            register,
            RegisterState::PendingDeletion,
            TransitionReason::UserAction(reason.to_string()),
        )
    }
    
    /// Delete a register
    pub fn delete_register(
        &self,
        register: &mut Register,
        reason: &str,
    ) -> Result<RegisterTransition> {
        self.transition_system.change_register_state(
            register,
            RegisterState::Tombstone,
            TransitionReason::UserAction(reason.to_string()),
        )
    }
    
    /// Check if a register has a nullifier
    pub fn has_nullifier(&self, register_id: &RegisterId) -> Result<bool> {
        self.nullifier_registry.has_nullifier(register_id)
    }
    
    /// Get the nullifier for a register
    pub fn get_nullifier_for_register(
        &self,
        register_id: &RegisterId,
    ) -> Result<Option<RegisterNullifier>> {
        self.nullifier_registry.get_nullifier_for_register(register_id)
    }
    
    /// Get the status of a nullifier
    pub fn get_nullifier_status(
        &self,
        nullifier_hash: &crate::util::hash::Hash256,
    ) -> Result<Option<NullifierStatus>> {
        self.nullifier_registry.get_nullifier_status(nullifier_hash)
    }
    
    /// Get the transition history for a register
    pub fn get_transition_history(
        &self,
        register_id: &RegisterId,
    ) -> Result<Option<Vec<RegisterTransition>>> {
        self.transition_system.get_transition_history(register_id)
    }
    
    /// Validate that an operation is permitted for a register's current state
    pub fn validate_operation(
        &self,
        register: &Register,
        operation: &RegisterOperation,
    ) -> Result<()> {
        match register.state {
            // Active registers can have any operation
            RegisterState::Active => Ok(()),
            
            // Locked registers can only be unlocked or viewed
            RegisterState::Locked => match operation.op_type {
                OperationType::UpdateRegister => {
                    // Here we would check if this is an unlock operation
                    Err(Error::InvalidOperation("Register is locked".to_string()))
                }
                _ => Ok(()),
            },
            
            // Frozen registers cannot be modified
            RegisterState::Frozen => match operation.op_type {
                OperationType::UpdateRegister | OperationType::DeleteRegister => {
                    Err(Error::InvalidOperation("Register is frozen".to_string()))
                }
                _ => Ok(()),
            },
            
            // Consumed registers cannot be used
            RegisterState::Consumed => {
                Err(Error::InvalidOperation("Register has been consumed".to_string()))
            }
            
            // PendingConsumption registers can only be fully consumed
            RegisterState::PendingConsumption => {
                Err(Error::InvalidOperation(
                    "Register is pending consumption".to_string()
                ))
            }
            
            // Archived registers can only be viewed or deleted
            RegisterState::Archived => match operation.op_type {
                OperationType::UpdateRegister => {
                    Err(Error::InvalidOperation("Register is archived".to_string()))
                }
                _ => Ok(()),
            },
            
            // Summary registers can only be viewed or archived
            RegisterState::Summary => match operation.op_type {
                OperationType::UpdateRegister => {
                    Err(Error::InvalidOperation("Register is a summary".to_string()))
                }
                _ => Ok(()),
            },
            
            // PendingDeletion registers can only be fully deleted or undeleted
            RegisterState::PendingDeletion => match operation.op_type {
                OperationType::DeleteRegister => Ok(()),
                _ => Err(Error::InvalidOperation(
                    "Register is pending deletion".to_string()
                )),
            },
            
            // Tombstone registers cannot be modified
            RegisterState::Tombstone => {
                Err(Error::InvalidOperation("Register is a tombstone".to_string()))
            }
            
            // Error state registers cannot be used
            RegisterState::Error => {
                Err(Error::InvalidOperation("Register is in an error state".to_string()))
            }
        }
    }
    
    /// Migrate a register to a new schema version
    pub fn migrate_register_version(
        &self,
        register: &mut Register,
        to_version: &SchemaVersion,
    ) -> Result<()> {
        // Migrate the register
        let migrated = self.migration_registry.migrate_register(register, to_version)?;
        
        // Update the register
        *register = migrated;
        
        // Increment version counter
        register.version += 1;
        
        // Update timestamp
        register.updated_at = self.nullifier_registry.current_block_height()?;
        
        Ok(())
    }
    
    /// Register a new version migration
    pub fn register_migration(&self, migration: VersionMigration) -> Result<()> {
        self.migration_registry.register_migration(migration)
    }
    
    /// Get current schema version of a register
    pub fn get_register_schema_version(&self, register: &Register) -> Result<SchemaVersion> {
        match register.metadata.get("schema_version") {
            Some(version_str) => {
                // Parse version string (format: schema_id-vX.Y.Z)
                let parts: Vec<&str> = version_str.split('-').collect();
                if parts.len() != 2 || !parts[1].starts_with('v') {
                    return Err(Error::InvalidInput(format!(
                        "Invalid version format: {}", version_str
                    )));
                }
                
                let schema_id = parts[0].to_string();
                let version_parts: Vec<&str> = parts[1][1..].split('.').collect();
                if version_parts.len() != 3 {
                    return Err(Error::InvalidInput(format!(
                        "Invalid version number format: {}", parts[1]
                    )));
                }
                
                let major = version_parts[0].parse::<u16>().map_err(|_| 
                    Error::InvalidInput(format!("Invalid major version: {}", version_parts[0]))
                )?;
                
                let minor = version_parts[1].parse::<u16>().map_err(|_| 
                    Error::InvalidInput(format!("Invalid minor version: {}", version_parts[1]))
                )?;
                
                let patch = version_parts[2].parse::<u16>().map_err(|_| 
                    Error::InvalidInput(format!("Invalid patch version: {}", version_parts[2]))
                )?;
                
                Ok(SchemaVersion::new(major, minor, patch, &schema_id))
            },
            None => Err(Error::InvalidInput(
                "Register does not have a schema version".to_string()
            )),
        }
    }
    
    /// Set schema version for a register
    pub fn set_register_schema_version(
        &self,
        register: &mut Register,
        version: &SchemaVersion,
    ) -> Result<()> {
        register.metadata.insert(
            "schema_version".to_string(),
            version.to_string(),
        );
        
        Ok(())
    }
    
    /// Create a register with specific schema version
    pub fn create_register_with_version(
        &self,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        transaction_id: &str,
        version: &SchemaVersion,
    ) -> Result<Register> {
        let mut register = self.create_register(owner, domain, contents, transaction_id)?;
        
        // Set the schema version
        register.metadata.insert(
            "schema_version".to_string(),
            version.to_string(),
        );
        
        Ok(register)
    }
    
    /// Get registers from a specific epoch
    pub fn get_registers_in_epoch(&self, epoch: EpochId) -> Result<std::collections::HashSet<RegisterId>> {
        if let Some(epoch_manager) = &self.epoch_manager {
            epoch_manager.get_registers_in_epoch(epoch)
        } else {
            Ok(std::collections::HashSet::new())
        }
    }
    
    /// Get the epoch for a specific block height
    pub fn get_epoch_for_block(&self, block_height: BlockHeight) -> Result<EpochId> {
        if let Some(epoch_manager) = &self.epoch_manager {
            epoch_manager.get_epoch_for_block(block_height)
        } else {
            Err(Error::ManagerNotAvailable("Epoch manager not available".to_string()))
        }
    }
    
    /// Check if an epoch is eligible for garbage collection
    pub fn is_epoch_eligible_for_gc(&self, epoch: EpochId) -> Result<bool> {
        if let Some(gc_manager) = &self.gc_manager {
            gc_manager.is_epoch_eligible_for_gc(epoch)
        } else {
            Ok(false)
        }
    }
    
    /// Update the archival policy
    pub fn set_archival_policy(&self, policy: ArchivalPolicy) -> Result<()> {
        if let Some(epoch_manager) = &self.epoch_manager {
            epoch_manager.set_archival_policy(policy)
        } else {
            Err(Error::ManagerNotAvailable("Epoch manager not available".to_string()))
        }
    }
    
    /// Get the current archival policy
    pub fn get_archival_policy(&self) -> Result<ArchivalPolicy> {
        if let Some(epoch_manager) = &self.epoch_manager {
            epoch_manager.get_archival_policy()
        } else {
            Err(Error::ManagerNotAvailable("Epoch manager not available".to_string()))
        }
    }
    
    /// Generate summaries for registers in the specified epoch
    pub fn generate_summaries_for_epoch(
        &self,
        epoch: EpochId,
        strategy_name: &str,
    ) -> Result<Vec<Register>> {
        // Get all registers for the epoch
        let registers = self.get_registers_for_epoch(epoch)?;
        
        // Skip if no registers
        if registers.is_empty() {
            return Ok(Vec::new());
        }
        
        // Get current block height
        let block_height = self.nullifier_registry.current_block_height();
        
        // Generate summaries
        let summaries = self.summary_manager.as_ref()
            .unwrap()
            .generate_summaries(
                &registers,
                strategy_name,
                epoch,
                block_height,
            )?;
        
        // Add the summary registers to our system
        for summary in &summaries {
            self.add_summary_register(summary.clone())?;
        }
        
        Ok(summaries)
    }
    
    /// Add a summary register to the system
    pub fn add_summary_register(&self, summary: Register) -> Result<()> {
        // Validate the register is a summary
        if summary.state != RegisterState::Summary {
            return Err(Error::InvalidInput(format!(
                "Register {} is not a summary register", summary.register_id
            )));
        }
        
        // Create a summary record if it doesn't exist
        let maybe_record = self.summary_manager.as_ref()
            .unwrap()
            .get_summary_record(&summary.register_id)?;
        
        if maybe_record.is_none() {
            // Extract summary record from register metadata
            let record = SummaryRecord::from_metadata(
                summary.register_id.clone(),
                &summary.metadata,
                summary.domain.clone(),
            )?;
            
            // Add to summary manager
            self.summary_manager.as_ref()
                .unwrap()
                .add_summary_record(record)?;
        }
        
        // Update summarized registers to point to this summary
        for summarized_id in &summary.summarizes {
            if let Some(mut register) = self.get_register(summarized_id)? {
                // Add this summary to the register's summarized_by field
                if register.summarized_by.is_none() {
                    register.summarized_by = Some(summary.register_id.clone());
                    
                    // Update the register
                    self.update_register(&register)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Verify a summary register
    pub fn verify_summary(
        &self,
        summary_id: &RegisterId,
    ) -> Result<bool> {
        // Get the summary register
        let summary = self.get_register(summary_id)?
            .ok_or_else(|| Error::NotFound(format!("Summary register not found: {}", summary_id)))?;
            
        // Get the summarized registers
        let mut summarized_registers = Vec::new();
        
        for id in &summary.summarizes {
            if let Some(register) = self.get_register(id)? {
                summarized_registers.push(register);
            }
        }
        
        // Verify the summary
        self.summary_manager.as_ref()
            .unwrap()
            .verify_summary(&summary, &summarized_registers)
    }
    
    /// Register a new summary strategy
    pub fn register_summary_strategy(
        &self,
        strategy: Arc<dyn SummaryStrategy>,
    ) -> Result<()> {
        self.summary_manager.as_ref()
            .unwrap()
            .register_strategy(strategy)
    }
    
    /// Get a summary strategy by name
    pub fn get_summary_strategy(
        &self,
        name: &str,
    ) -> Result<Arc<dyn SummaryStrategy>> {
        self.summary_manager.as_ref()
            .unwrap()
            .get_strategy(name)
    }
    
    /// Check if a register is eligible for garbage collection
    pub fn is_eligible_for_gc(&self, register_id: &RegisterId) -> Result<bool> {
        // Check if we have a garbage collection manager
        if let Some(gc_manager) = &self.gc_manager {
            // Get the register
            if let Some(register) = self.get_register(register_id)? {
                // Check if the register is eligible for garbage collection
                return gc_manager.is_eligible_for_gc(&register);
            }
        }
        
        // If no garbage collection manager or register not found, not eligible
        Ok(false)
    }
    
    /// Garbage collect a register, removing it from the register system
    pub fn garbage_collect_register(&mut self, register_id: &RegisterId) -> Result<bool> {
        // Check if the register exists and is eligible for garbage collection
        if !self.is_eligible_for_gc(register_id)? {
            return Ok(false);
        }
        
        // Get the register's archive reference for potential archive deletion
        let archive_ref = self.get_register(register_id)?
            .and_then(|r| r.archive_reference.clone());
        
        // Remove the register from the register system
        let removed = self.get_register(register_id)?.is_some();
        
        if removed {
            // If we have a garbage collection manager, mark the register as collected
            if let Some(gc_manager) = &self.gc_manager {
                gc_manager.mark_as_collected(register_id)?;
                
                // If the register has an archive reference and we're configured to delete archives,
                // delete the archive
                if let Some(ref_val) = archive_ref {
                    let _ = gc_manager.delete_archive_if_configured(&ref_val)?;
                }
            }
        }
        
        Ok(removed)
    }
    
    /// Garbage collect all eligible registers in the specified epoch
    pub fn garbage_collect_epoch(&mut self, epoch_id: EpochId) -> Result<Vec<RegisterId>> {
        // Check if we have a garbage collection manager
        if let Some(gc_manager) = &self.gc_manager {
            // Get all eligible registers in the specified epoch
            let eligible_ids = gc_manager.get_eligible_registers_by_epoch(
                &self.get_registers_in_epoch(epoch_id)?, 
                epoch_id
            )?;
            
            // Garbage collect each eligible register
            let mut collected_ids = Vec::new();
            for id in eligible_ids {
                if self.garbage_collect_register(&id)? {
                    collected_ids.push(id);
                }
            }
            
            Ok(collected_ids)
        } else {
            // If no garbage collection manager, nothing to collect
            Ok(Vec::new())
        }
    }
    
    /// Garbage collect all eligible registers
    pub fn garbage_collect_all_eligible(&mut self) -> Result<Vec<RegisterId>> {
        // Check if we have a garbage collection manager and epoch manager
        if let (Some(gc_manager), Some(epoch_manager)) = (&self.gc_manager, &self.epoch_manager) {
            let current_epoch = epoch_manager.get_current_epoch();
            let retention_epochs = gc_manager.get_retention_epochs()?;
            
            // Calculate the oldest epoch that should be retained
            let oldest_retained_epoch = if current_epoch > retention_epochs {
                current_epoch - retention_epochs
            } else {
                0
            };
            
            // Collect registers from all epochs older than the retention limit
            let mut collected_ids = Vec::new();
            for epoch in 0..oldest_retained_epoch {
                let epoch_collected = self.garbage_collect_epoch(epoch)?;
                collected_ids.extend(epoch_collected);
            }
            
            Ok(collected_ids)
        } else {
            // If no garbage collection manager or epoch manager, nothing to collect
            Ok(Vec::new())
        }
    }
    
    /// Check if a register has been garbage collected
    pub fn is_garbage_collected(&self, register_id: &RegisterId) -> Result<bool> {
        if let Some(gc_manager) = &self.gc_manager {
            gc_manager.is_collected(register_id)
        } else {
            Ok(false)
        }
    }
    
    /// Get the time when a register was garbage collected
    pub fn get_garbage_collection_time(&self, register_id: &RegisterId) -> Result<Option<std::time::SystemTime>> {
        if let Some(gc_manager) = &self.gc_manager {
            gc_manager.get_collection_time(register_id)
        } else {
            Ok(None)
        }
    }
    
    /// Update the garbage collection configuration
    pub fn update_gc_config(&self, config: GarbageCollectionConfig) -> Result<()> {
        if let Some(gc_manager) = &self.gc_manager {
            gc_manager.update_config(config)
        } else {
            Err(Error::ManagerNotAvailable("Garbage Collection manager not available".to_string()))
        }
    }
    
    /// Auto-GC on epoch advance if configured to do so
    fn auto_gc_on_epoch_advance(&mut self, old_epoch: EpochId) -> Result<Vec<RegisterId>> {
        // Check if we have a garbage collection manager
        if let Some(gc_manager) = &self.gc_manager {
            // Check if auto-GC is enabled
            if gc_manager.is_auto_gc_on_epoch_advance()? {
                // Garbage collect the old epoch
                return self.garbage_collect_epoch(old_epoch);
            }
        }
        
        Ok(Vec::new())
    }
    
    /// Associate a register with a domain
    pub fn associate_register(&self, register_id: &RegisterId, domain_id: &DomainId) -> Result<()> {
        // Check if domain registry is configured
        if self.domain_registry.is_none() {
            return Err(Error::ConfigurationError("Domain registry not configured".to_string()));
        }
        
        // Check if domain exists
        let domain_registry = self.domain_registry.as_ref().unwrap();
        if !domain_registry.has_domain(domain_id)? {
            return Err(Error::NotFound(format!("Domain {} not found", domain_id)));
        }
        
        // Update register-to-domain mapping
        let mut reg_to_domain = self.register_to_domain.write()
            .map_err(|_| Error::LockError("Failed to acquire register_to_domain lock".to_string()))?;
        
        reg_to_domain.insert(register_id.clone(), domain_id.clone());
        
        // Update domain-to-registers mapping
        let mut domain_to_regs = self.domain_to_registers.write()
            .map_err(|_| Error::LockError("Failed to acquire domain_to_registers lock".to_string()))?;
        
        domain_to_regs.entry(domain_id.clone())
            .or_insert_with(HashSet::new)
            .insert(register_id.clone());
        
        // Log domain integration
        let transaction_id = format!("domain-integration-{}", uuid::Uuid::new_v4());
        self.log_register_domain_integration(register_id, domain_id, &transaction_id)?;
        
        Ok(())
    }
    
    /// Get the domain ID associated with a register
    pub fn get_domain_for_register(&self, register_id: &RegisterId) -> Result<Option<DomainId>> {
        let register_to_domain = self.register_to_domain.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on register to domain mapping".into())
        })?;
        
        Ok(register_to_domain.get(register_id).cloned())
    }
    
    /// Get all registers associated with a domain
    pub fn get_registers_for_domain(&self, domain_id: &DomainId) -> Result<HashSet<RegisterId>> {
        let domain_to_registers = self.domain_to_registers.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on domain to registers mapping".into())
        })?;
        
        match domain_to_registers.get(domain_id) {
            Some(registers) => Ok(registers.clone()),
            None => Ok(HashSet::new()),
        }
    }
    
    /// Create a register in a specific domain
    pub fn create_register_in_domain(
        &self,
        domain_id: &DomainId,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        transaction_id: &str,
    ) -> Result<Register> {
        if self.domain_registry.is_none() {
            return Err(Error::NotSupported("Domain adapter support not configured".to_string()));
        }
        
        // Ensure the domain exists
        let domain_registry = self.domain_registry.as_ref().unwrap();
        domain_registry.get_adapter(domain_id)?;
        
        // Create the register
        let register = self.create_register(
            owner,
            domain,
            contents,
            transaction_id,
        )?;
        
        // Associate the register with the domain
        self.associate_register(&register.register_id, domain_id)?;
        
        Ok(register)
    }
    
    /// Consume a register through its associated domain
    pub fn consume_register_through_domain(
        &self,
        register_id: &RegisterId,
        transaction_id: &str,
    ) -> Result<TransactionReceipt> {
        if self.domain_registry.is_none() {
            return Err(Error::NotSupported("Domain adapter support not configured".to_string()));
        }
        
        // Get the domain ID
        let domain_id = self.get_domain_for_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("No domain associated with register {}", register_id)))?;
        
        // Get the domain adapter
        let domain_registry = self.domain_registry.as_ref().unwrap();
        let adapter = domain_registry.get_adapter(&domain_id)?;
        
        // Get the register
        let register = self.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Ensure register is in a valid state for consumption
        if register.state != RegisterState::Active && register.state != RegisterState::Locked {
            return Err(Error::InvalidState(format!(
                "Register {} is in state {:?} and cannot be consumed",
                register_id, register.state
            )));
        }
        
        // Prepare transaction data (using register contents as transaction data)
        let data = match register.contents.as_binary() {
            Some(binary) => binary.to_vec(),
            None => register.contents.to_string().into_bytes(),
        };
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("register_id".to_string(), register_id.to_string());
        metadata.insert("owner".to_string(), register.owner.to_string());
        
        // Create the transaction
        let transaction = Transaction {
            domain_id: domain_id.clone(),
            tx_type: "register_consume".to_string(),
            data,
            metadata,
        };
        
        // Submit the transaction
        let tx_id = tokio::runtime::Handle::current().block_on(async {
            adapter.submit_transaction(transaction).await
        })?;
        
        // Consume the register locally
        let mut register_mut = register.clone();
        self.consume_register(&mut register_mut, transaction_id)?;
        
        // Get the transaction receipt
        let receipt = tokio::runtime::Handle::current().block_on(async {
            adapter.get_transaction_receipt(&tx_id).await
        })?;
        
        Ok(receipt)
    }
    
    /// Observe a fact from a domain and store it in a register
    pub fn observe_fact_to_register(
        &self,
        domain_id: &DomainId,
        owner: Address,
        domain: Domain,
        fact_type: &str,
        parameters: HashMap<String, String>,
        transaction_id: &str,
    ) -> Result<Register> {
        if self.domain_registry.is_none() {
            return Err(Error::NotSupported("Domain adapter support not configured".to_string()));
        }
        
        // Get the domain adapter
        let domain_registry = self.domain_registry.as_ref().unwrap();
        let adapter = domain_registry.get_adapter(domain_id)?;
        
        // Create the fact query
        let query = FactQuery {
            domain_id: domain_id.clone(),
            fact_type: fact_type.to_string(),
            parameters,
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Observe the fact
        let fact = tokio::runtime::Handle::current().block_on(async {
            adapter.observe_fact(query).await
        })?;
        
        // Serialize the fact to JSON
        let fact_json = serde_json::to_string(&fact)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize fact: {}", e)))?;
        
        // Create a register with the fact data
        let contents = RegisterContents::with_json(fact_json);
        let register = self.create_register_in_domain(
            domain_id,
            owner,
            domain,
            contents,
            transaction_id,
        )?;
        
        Ok(register)
    }
    
    /// Execute a domain operation using a register
    pub fn execute_domain_operation(
        &self,
        register_id: &RegisterId,
        operation: &str,
        parameters: HashMap<String, String>,
        transaction_id: &str,
    ) -> Result<TransactionReceipt> {
        if self.domain_registry.is_none() {
            return Err(Error::NotSupported("Domain adapter support not configured".to_string()));
        }
        
        // Get the domain ID
        let domain_id = self.get_domain_for_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("No domain associated with register {}", register_id)))?;
        
        // Get the domain adapter
        let domain_registry = self.domain_registry.as_ref().unwrap();
        let adapter = domain_registry.get_adapter(&domain_id)?;
        
        // Get the register
        let register = self.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Prepare transaction data
        let mut data = Vec::new();
        
        // Append operation
        data.extend_from_slice(operation.as_bytes());
        
        // Append register contents
        match register.contents.as_binary() {
            Some(binary) => data.extend_from_slice(binary),
            None => data.extend_from_slice(register.contents.to_string().as_bytes()),
        }
        
        // Create metadata
        let mut metadata = parameters.clone();
        metadata.insert("register_id".to_string(), register_id.to_string());
        metadata.insert("operation".to_string(), operation.to_string());
        
        // Create the transaction
        let transaction = Transaction {
            domain_id: domain_id.clone(),
            tx_type: "register_operation".to_string(),
            data,
            metadata,
        };
        
        // Submit the transaction
        let tx_id = tokio::runtime::Handle::current().block_on(async {
            adapter.submit_transaction(transaction).await
        })?;
        
        // Get the transaction receipt
        let receipt = tokio::runtime::Handle::current().block_on(async {
            adapter.get_transaction_receipt(&tx_id).await
        })?;
        
        Ok(receipt)
    }
    
    /// Synchronize a register with its domain state
    pub fn sync_register_with_domain(&self, register_id: &RegisterId) -> Result<()> {
        if self.domain_registry.is_none() {
            return Err(Error::NotSupported("Domain adapter support not configured".to_string()));
        }
        
        // Get the domain ID
        let domain_id = self.get_domain_for_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("No domain associated with register {}", register_id)))?;
        
        // Get the domain adapter
        let domain_registry = self.domain_registry.as_ref().unwrap();
        let adapter = domain_registry.get_adapter(&domain_id)?;
        
        // Get the register
        let register = self.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Create a fact query to get the current state of the register in the domain
        let mut parameters = HashMap::new();
        parameters.insert("register_id".to_string(), register_id.to_string());
        
        let query = FactQuery {
            domain_id: domain_id.clone(),
            fact_type: "register_state".to_string(),
            parameters,
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Observe the fact
        let fact = tokio::runtime::Handle::current().block_on(async {
            match adapter.observe_fact(query).await {
                Ok(fact) => Ok(fact),
                Err(_) => Err(Error::ExternalError("Failed to observe register state from domain".to_string())),
            }
        })?;
        
        // Extract the domain state from the fact
        let domain_state = match fact {
            FactType::Json(json) => {
                json.get("state")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::ExternalError("Invalid register state fact from domain".to_string()))?
            },
            _ => return Err(Error::ExternalError("Unexpected fact type from domain".to_string())),
        };
        
        // Update the register state based on domain state
        let mut register_mut = register.clone();
        match domain_state {
            "active" => {
                if register.state != RegisterState::Active {
                    self.unlock_register(&mut register_mut, "domain-sync")?;
                }
            },
            "locked" => {
                if register.state != RegisterState::Locked {
                    self.lock_register(&mut register_mut, "domain-sync")?;
                }
            },
            "consumed" => {
                if register.state != RegisterState::Consumed {
                    self.consume_register(&mut register_mut, "domain-sync")?;
                }
            },
            "archived" => {
                if register.state != RegisterState::Archived {
                    self.archive_register(register_id)?;
                }
            },
            _ => {
                // Unknown state, leave as is
            }
        }
        
        Ok(())
    }
    
    /// Get the domain registry
    pub fn domain_registry(&self) -> Option<&Arc<DomainRegistry>> {
        self.domain_registry.as_ref()
    }
    
    /// Set the domain registry
    pub fn set_domain_registry(&mut self, domain_registry: Arc<DomainRegistry>) {
        self.domain_registry = Some(domain_registry);
    }

    /// Get the time map
    pub fn time_map(&self) -> Option<&SharedTimeMap> {
        self.time_map.as_ref()
    }

    /// Set the time map for this register system
    pub fn set_time_map(&mut self, time_map: SharedTimeMap) {
        self.time_map = Some(time_map);
    }

    /// Get the current time map entry for a specific domain
    pub fn get_time_for_domain(&self, domain_id: &DomainId) -> Result<Option<TimeMapEntry>> {
        if let Some(time_map) = &self.time_map {
            let map = time_map.get()?;
            let entry = map.entries.get(domain_id).cloned();
            Ok(entry)
        } else {
            Err(Error::NotSupported("Time map not configured".to_string()))
        }
    }

    /// Get a reference to the domain's latest block height from the time map
    pub fn get_domain_height(&self, domain_id: &DomainId) -> Result<Option<BlockHeight>> {
        if let Some(time_map) = &self.time_map {
            let map = time_map.get()?;
            let height = map.get_height(domain_id);
            Ok(height)
        } else {
            Err(Error::NotSupported("Time map not configured".to_string()))
        }
    }

    /// Synchronize registers with the time map
    /// 
    /// This method ensures that registers are synchronized with the latest domain state
    /// as recorded in the time map. It updates register metadata with time map information
    /// and verifies that register operations respect causal ordering.
    pub fn sync_with_time_map(&self) -> Result<HashMap<RegisterId, TimeMapEntry>> {
        if self.time_map.is_none() {
            return Err(Error::NotSupported("Time map not configured".to_string()));
        }

        let time_map = self.time_map.as_ref().unwrap().get()?;
        let result = HashMap::new();
        
        // Read the register-to-domain map
        let register_to_domain = self.register_to_domain.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on register to domain mapping".to_string())
        })?;
        
        // For each register associated with a domain, update register metadata
        for (register_id, domain_id) in register_to_domain.iter() {
            // Check if the domain has an entry in the time map
            if let Some(time_entry) = time_map.entries.get(domain_id) {
                // Attempt to get the register
                if let Some(mut register) = self.get_register(register_id)? {
                    // Update register metadata with time map information
                    register.metadata.insert("time_map_height".to_string(), time_entry.height.to_string());
                    register.metadata.insert("time_map_timestamp".to_string(), time_entry.timestamp.to_string());
                    
                    // Ensure register is updated to reflect time map state
                    match domain_id {
                        // Implementation specific to each domain type would go here
                        _ => {
                            // Generic update logic
                            self.sync_register_with_domain(register_id)?;
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }

    /// Verify that register operations respect causal ordering
    ///
    /// This method checks that a proposed register operation is consistent with the
    /// causal ordering defined by the time map.
    pub fn verify_causal_ordering(
        &self,
        register_id: &RegisterId,
        operation: &RegisterOperation,
    ) -> Result<bool> {
        if self.time_map.is_none() {
            return Err(Error::NotSupported("Time map not configured".to_string()));
        }

        // Get the domain associated with the register
        let domain_id = self.get_domain_for_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("No domain associated with register {}", register_id)))?;
        
        // Get the time map
        let time_map = self.time_map.as_ref().unwrap().get()?;
        
        // Get the time map entry for the domain
        let time_entry = time_map.entries.get(&domain_id)
            .ok_or_else(|| Error::NotFound(format!("No time map entry for domain {}", domain_id)))?;
        
        // Get the register
        let register = self.get_register(register_id)?
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Check operation specific constraints
        match operation.operation_type {
            OperationType::Create => {
                // For create operations, we need to ensure the register was created after the
                // time map entry's observation time
                let register_created_at = register.created_at;
                let time_entry_timestamp = time_entry.timestamp;
                
                // Register should not be created before the domain's observed state
                if register_created_at < time_entry_timestamp {
                    return Ok(false);
                }
            },
            OperationType::Update => {
                // For update operations, ensure the register's last update height is consistent
                // with the domain's height in the time map
                if let Some(last_updated_height) = register.metadata.get("time_map_height") {
                    if let Ok(height) = last_updated_height.parse::<u64>() {
                        // The update should be based on a height that's not in the future compared to time map
                        if height > time_entry.height.0 {
                            return Ok(false);
                        }
                    }
                }
            },
            OperationType::Consume => {
                // For consume operations, similar to update operations, ensure consistency
                // with time map's domain height
                if let Some(last_updated_height) = register.metadata.get("time_map_height") {
                    if let Ok(height) = last_updated_height.parse::<u64>() {
                        if height > time_entry.height.0 {
                            return Ok(false);
                        }
                    }
                }
            },
            // Handle other operation types as needed
            _ => {
                // Default validation for other operations
            }
        }
        
        // If all checks pass, the operation respects causal ordering
        Ok(true)
    }

    /// Register an observer for time map updates
    ///
    /// This method registers an observer that will be notified when the time map is updated.
    /// The observer can then take actions based on the updated time map, such as updating
    /// register state.
    pub fn register_time_map_observer<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&TimeMap) + Send + Sync + 'static
    {
        if let Some(time_map) = &self.time_map {
            // Implementation would depend on the specific SharedTimeMap implementation,
            // which should have a subscribe or add_observer method
            
            // For demonstration, assuming SharedTimeMap has a wrapper around TimeMapNotifier
            // time_map.subscribe(callback)?;
            
            // Since the actual implementation might vary, we'll return a stub for now
            Err(Error::NotImplemented("Time map observer registration not implemented yet".to_string()))
        } else {
            Err(Error::NotSupported("Time map not configured".to_string()))
        }
    }

    /// Update register with time map information
    ///
    /// This method updates a register's metadata with information from the time map.
    /// It is useful when creating or updating registers to ensure they have the latest
    /// time map information.
    pub fn update_register_with_time_info(&self, register: &mut Register) -> Result<()> {
        if self.time_map.is_none() {
            return Ok(());  // Not an error, just no time map to use
        }

        // Get the domain associated with the register
        let domain_id = match self.get_domain_for_register(&register.register_id)? {
            Some(id) => id,
            None => return Ok(()),  // No domain associated, nothing to update
        };
        
        // Get the time map
        let time_map = self.time_map.as_ref().unwrap().get()?;
        
        // Get the time map entry for the domain
        if let Some(time_entry) = time_map.entries.get(&domain_id) {
            // Update register metadata with time map information
            register.metadata.insert("time_map_height".to_string(), time_entry.height.to_string());
            register.metadata.insert("time_map_timestamp".to_string(), time_entry.timestamp.to_string());
            register.metadata.insert("time_map_hash".to_string(), hex::encode(&time_entry.hash));
            register.metadata.insert("time_map_version".to_string(), time_map.version.to_string());
            
            // Update the register's validity range based on the time map
            // This is just an example - adjust according to your specific requirements
            if register.validity.start == 0 {
                register.validity.start = time_entry.timestamp;
            }
        }
        
        Ok(())
    }

    /// Create a register with time map information
    ///
    /// This method is similar to create_register but also includes time map information
    /// in the register's metadata.
    pub fn create_register_with_time_info(
        &self,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        transaction_id: &str,
    ) -> Result<Register> {
        // Create the register first
        let mut register = self.create_register(
            owner,
            domain,
            contents,
            transaction_id,
        )?;
        
        // Update with time map information
        self.update_register_with_time_info(&mut register)?;
        
        Ok(register)
    }

    /// Modify the domain registry and ensure time map is updated
    pub fn set_domain_registry_and_sync_time(&mut self, domain_registry: Arc<DomainRegistry>) -> Result<()> {
        self.domain_registry = Some(domain_registry);
        
        // If we have a time map, ensure it's synchronized with the domain registry
        if let Some(time_map) = &self.time_map {
            // Get all domains from the registry
            let domains = self.domain_registry.as_ref().unwrap().list_domains()?;
            
            // For each domain, ensure we have a time map entry
            for domain_id in domains {
                // Get the domain adapter
                let adapter = self.domain_registry.as_ref().unwrap().get_adapter(&domain_id)?;
                
                // Get the current time map state from the domain
                let entry = tokio::runtime::Handle::current().block_on(async {
                    adapter.get_time_map().await
                })?;
                
                // Update the time map
                time_map.update_domain(entry)?;
            }
        }
        
        Ok(())
    }

    /// Get the fact observer
    pub fn fact_observer(&self) -> Option<&Arc<RegisterFactObserver>> {
        self.fact_observer.as_ref()
    }
    
    /// Set the fact observer
    pub fn set_fact_observer(&mut self, fact_observer: Arc<RegisterFactObserver>) {
        self.fact_observer = Some(fact_observer);
    }
    
    /// Log a register creation fact
    fn log_register_creation(&self, register: &Register, transaction_id: &str) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_creation(register, transaction_id)?;
        }
        Ok(())
    }
    
    /// Log a register update fact
    fn log_register_update(&self, register: &Register, previous_data: &[u8], transaction_id: &str) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_update(register, previous_data, transaction_id)?;
        }
        Ok(())
    }
    
    /// Log a register consumption fact
    fn log_register_consumption(
        &self, 
        register: &Register, 
        transaction_id: &str,
        nullifier: &str,
        successors: Vec<RegisterId>,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            let block_height = self.get_current_block_height()?;
            observer.observe_register_consumption(
                register, 
                transaction_id, 
                nullifier,
                successors,
                block_height,
            )?;
        }
        Ok(())
    }
    
    /// Log a register state change fact
    fn log_register_state_change(
        &self,
        register: &Register,
        previous_state: RegisterState,
        reason: &str,
        transaction_id: &str,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_state_change(
                register,
                previous_state,
                reason,
                transaction_id,
            )?;
        }
        Ok(())
    }
    
    /// Log a register ownership transfer fact
    fn log_register_ownership_transfer(
        &self,
        register: &Register,
        previous_owner: &str,
        transaction_id: &str,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_ownership_transfer(
                register,
                previous_owner,
                transaction_id,
            )?;
        }
        Ok(())
    }
    
    /// Log a register lock fact
    fn log_register_lock(
        &self,
        register: &Register,
        reason: &str,
        transaction_id: &str,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_lock(
                register,
                reason,
                transaction_id,
            )?;
        }
        Ok(())
    }
    
    /// Log a register unlock fact
    fn log_register_unlock(
        &self,
        register: &Register,
        reason: &str,
        transaction_id: &str,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_unlock(
                register,
                reason,
                transaction_id,
            )?;
        }
        Ok(())
    }
    
    /// Log a register nullifier creation fact
    fn log_register_nullifier_creation(
        &self,
        register_id: &RegisterId,
        nullifier: &str,
        transaction_id: &str,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            let block_height = self.get_current_block_height()?;
            observer.observe_register_nullifier_creation(
                register_id,
                nullifier,
                block_height,
                transaction_id,
            )?;
        }
        Ok(())
    }
    
    /// Log a register domain integration fact
    fn log_register_domain_integration(
        &self,
        register_id: &RegisterId,
        domain_id: &DomainId,
        transaction_id: &str,
    ) -> Result<()> {
        if let Some(observer) = &self.fact_observer {
            observer.observe_register_domain_integration(
                register_id,
                domain_id,
                transaction_id,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::transition::LoggingTransitionObserver;
    
    #[test]
    fn test_one_time_register_system() {
        // Create configuration
        let config = OneTimeRegisterConfig {
            current_block_height: 100,
            nullifier_timeout: 20,
            initial_observers: vec![
                Arc::new(LoggingTransitionObserver::new("test-observer".to_string()))
            ],
            migration_registry: None,
            epoch_manager: None,
            summary_manager: None,
            archive_manager: None,
            gc_manager: None,
            domain_registry: None,
            time_map: None,
            fact_logger: None,
            fact_observer_name: None,
        };
        
        // Create the system
        let system = OneTimeRegisterSystem::new(config).unwrap();
        
        // Create a register
        let mut register = system.create_register(
            Address::new("owner"),
            Domain::new("test-domain"),
            RegisterContents::empty(),
            "test-tx",
        ).unwrap();
        
        // Test locking a register
        let transition = system.lock_register(&mut register, "Test lock").unwrap();
        assert_eq!(register.state, RegisterState::Locked);
        
        // Test unlocking a register
        let transition = system.unlock_register(&mut register, "Test unlock").unwrap();
        assert_eq!(register.state, RegisterState::Active);
        
        // Test consuming a register
        let successor_id = RegisterId::new_unique();
        let transition = system.consume_register(
            &mut register,
            "test-tx-consume",
            vec![successor_id.clone()],
        ).unwrap();
        
        // Verify the register is consumed
        assert_eq!(register.state, RegisterState::Consumed);
        assert_eq!(register.consumed_by_tx, Some("test-tx-consume".to_string()));
        assert_eq!(register.successors, vec![successor_id]);
        
        // Verify the nullifier exists
        assert!(system.has_nullifier(&register.register_id).unwrap());
        
        // Get transition history
        let history = system.get_transition_history(&register.register_id).unwrap();
        assert!(history.is_some());
        assert_eq!(history.unwrap().len(), 3); // lock, unlock, consume
    }
    
    #[test]
    fn test_epoch_integration() {
        // Create a custom epoch manager for testing
        let epoch_manager = SharedEpochManager::new();
        
        // Create configuration with custom epoch manager
        let config = OneTimeRegisterConfig {
            current_block_height: 100,
            nullifier_timeout: 20,
            initial_observers: vec![],
            migration_registry: None,
            epoch_manager: Some(epoch_manager.clone()),
            summary_manager: None,
            archive_manager: None,
            gc_manager: None,
            domain_registry: None,
            time_map: None,
            fact_logger: None,
            fact_observer_name: None,
        };
        
        // Create the system
        let system = OneTimeRegisterSystem::new(config).unwrap();
        
        // Initial epoch should be 1
        assert_eq!(system.get_current_epoch().unwrap(), 1);
        
        // Create registers in epoch 1
        let reg1 = system.create_register(
            Address::new("owner1"),
            Domain::new("domain1"),
            RegisterContents::empty(),
            "tx1",
        ).unwrap();
        
        let reg2 = system.create_register(
            Address::new("owner2"),
            Domain::new("domain2"),
            RegisterContents::empty(),
            "tx2",
        ).unwrap();
        
        // Get registers in epoch 1
        let epoch1_regs = system.get_registers_in_epoch(1).unwrap();
        assert_eq!(epoch1_regs.len(), 2);
        assert!(epoch1_regs.contains(&reg1.register_id));
        assert!(epoch1_regs.contains(&reg2.register_id));
        
        // Advance to epoch 2
        system.update_block_height(200).unwrap();
        system.advance_epoch().unwrap();
        assert_eq!(system.get_current_epoch().unwrap(), 2);
        
        // Create register in epoch 2
        let reg3 = system.create_register(
            Address::new("owner3"),
            Domain::new("domain3"),
            RegisterContents::empty(),
            "tx3",
        ).unwrap();
        
        // Get registers in epoch 2
        let epoch2_regs = system.get_registers_in_epoch(2).unwrap();
        assert_eq!(epoch2_regs.len(), 1);
        assert!(epoch2_regs.contains(&reg3.register_id));
        
        // Consume a register from epoch 1, creating successor in epoch 2
        let mut reg1_mut = reg1.clone();
        let successor_id = RegisterId::new_unique();
        system.consume_register(
            &mut reg1_mut,
            "tx-consume",
            vec![successor_id.clone()],
        ).unwrap();
        
        // Check that successor is tracked in epoch 2
        let updated_epoch2_regs = system.get_registers_in_epoch(2).unwrap();
        assert!(updated_epoch2_regs.contains(&successor_id));
    }
} 