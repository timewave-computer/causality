// Resource manager implementation for TEL
//
// This module provides the ResourceManager which centralizes
// management of resources through the register-based model.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::tel::types::{ResourceId, Address, Domain, Timestamp, OperationId};
use crate::tel::error::{TelError, TelResult};
use crate::tel::resource::operations::{ResourceOperation, ResourceOperationType};
use crate::tel::resource::model::{
    Register, RegisterId, RegisterContents, RegisterState, 
    Resource, ResourceTimeData, ControllerLabel,
};

/// Statistics from garbage collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Number of registers processed
    pub processed: usize,
    /// Number of registers archived
    pub archived: usize,
    /// Number of registers deleted
    pub deleted: usize,
    /// Time taken for garbage collection (ms)
    pub time_taken_ms: u64,
}

/// Configuration for garbage collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionConfig {
    /// Max age before a register becomes eligible for archiving (ms)
    pub max_register_age_ms: u64,
    /// Max age for an archived register before deletion (ms)
    pub archive_retention_ms: u64,
    /// Maximum number of registers to process in one run
    pub max_registers_per_run: usize,
    /// Whether to automatically run GC when advancing epochs
    pub auto_gc_on_epoch_advance: bool,
}

impl Default for GarbageCollectionConfig {
    fn default() -> Self {
        Self {
            max_register_age_ms: 30 * 24 * 60 * 60 * 1000, // 30 days
            archive_retention_ms: 90 * 24 * 60 * 60 * 1000, // 90 days
            max_registers_per_run: 1000,
            auto_gc_on_epoch_advance: true,
        }
    }
}

/// Manages resources within the TEL system using the register-based model
#[derive(Debug)]
pub struct ResourceManager {
    /// All registers tracked by the manager
    registers: RwLock<HashMap<RegisterId, Register>>,
    /// Operations log
    operations: RwLock<Vec<(OperationId, ResourceOperation)>>,
    /// Garbage collection configuration
    gc_config: RwLock<GarbageCollectionConfig>,
    /// Current epoch
    current_epoch: RwLock<u64>,
    /// Registers grouped by epoch
    registers_by_epoch: RwLock<HashMap<u64, HashSet<RegisterId>>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            registers: RwLock::new(HashMap::new()),
            operations: RwLock::new(Vec::new()),
            gc_config: RwLock::new(GarbageCollectionConfig::default()),
            current_epoch: RwLock::new(0),
            registers_by_epoch: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new register
    pub fn create_register(
        &self,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
    ) -> TelResult<RegisterId> {
        let register_id = RegisterId::new();
        let current_epoch = *self.current_epoch.read().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch lock".to_string()))?;
        
        let register = Register::new(
            register_id,
            owner,
            domain,
            contents,
            current_epoch,
            None,
        );
        
        // Store the register
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        registers.insert(register_id, register);
        
        // Add to epoch tracking
        let mut registers_by_epoch = self.registers_by_epoch.write().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch tracking lock".to_string()))?;
        
        registers_by_epoch
            .entry(current_epoch)
            .or_insert_with(HashSet::new)
            .insert(register_id);
        
        Ok(register_id)
    }
    
    /// Get a register by ID
    pub fn get_register(&self, register_id: &RegisterId) -> TelResult<Register> {
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        registers.get(register_id)
            .cloned()
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))
    }
    
    /// Update a register's contents
    pub fn update_register(
        &self,
        register_id: &RegisterId,
        contents: RegisterContents,
    ) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot update register {:?} in state {:?}", register_id, register.state
            )));
        }
        
        register.contents = contents;
        register.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// Delete a register
    pub fn delete_register(&self, register_id: &RegisterId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_active() && !register.is_locked() && !register.is_frozen() {
            return Err(TelError::ResourceError(format!(
                "Cannot delete register {:?} in state {:?}", register_id, register.state
            )));
        }
        
        // Mark for deletion
        register.state = RegisterState::PendingDeletion;
        register.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// Transfer a register to a new owner
    pub fn transfer_register(
        &self,
        register_id: &RegisterId,
        from: &Address,
        to: Address,
    ) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        // Check ownership
        if &register.owner != from {
            return Err(TelError::AuthorizationError(format!(
                "Register {:?} is not owned by {:?}", register_id, from
            )));
        }
        
        if !register.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot transfer register {:?} in state {:?}", register_id, register.state
            )));
        }
        
        // Transfer ownership
        register.owner = to;
        register.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// Lock a register
    pub fn lock_register(&self, register_id: &RegisterId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot lock register {:?} in state {:?}", register_id, register.state
            )));
        }
        
        register.state = RegisterState::Locked;
        register.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// Unlock a register
    pub fn unlock_register(&self, register_id: &RegisterId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_locked() {
            return Err(TelError::ResourceError(format!(
                "Cannot unlock register {:?} in state {:?}", register_id, register.state
            )));
        }
        
        register.state = RegisterState::Active;
        register.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// Apply a resource operation
    pub fn apply_operation(&self, operation: ResourceOperation) -> TelResult<OperationId> {
        let operation_id = OperationId(Uuid::new_v4());
        
        // Process the operation based on type
        match operation.operation_type {
            ResourceOperationType::Create => {
                // Check if target register exists
                let registers = self.registers.read().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                if registers.contains_key(&operation.target) {
                    return Err(TelError::ResourceError(format!(
                        "Register {:?} already exists", operation.target
                    )));
                }
                
                // Create the register
                drop(registers);
                let current_epoch = *self.current_epoch.read().map_err(|_| 
                    TelError::InternalError("Failed to acquire epoch lock".to_string()))?;
                
                // Get contents from inputs
                let contents = operation.inputs.get(0)
                    .ok_or_else(|| TelError::ResourceError(
                        "Create operation must have contents as input".to_string()
                    ))?
                    .clone();
                
                let register = Register::new(
                    operation.target,
                    operation.initiator,
                    operation.domain,
                    contents,
                    current_epoch,
                    None,
                );
                
                // Store the register
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                registers.insert(operation.target, register);
                
                // Add to epoch tracking
                let mut registers_by_epoch = self.registers_by_epoch.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire epoch tracking lock".to_string()))?;
                
                registers_by_epoch
                    .entry(current_epoch)
                    .or_insert_with(HashSet::new)
                    .insert(operation.target);
            },
            ResourceOperationType::Update => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Verify ownership
                if register.owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot update register {:?} in state {:?}",
                        operation.target, register.state
                    )));
                }
                
                // Get contents from inputs
                let contents = operation.inputs.get(0)
                    .ok_or_else(|| TelError::ResourceError(
                        "Update operation must have contents as input".to_string()
                    ))?
                    .clone();
                
                // Update register
                register.contents = contents;
                register.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                // Add operation to history
                register.history.push(operation_id);
            },
            ResourceOperationType::Delete => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Verify ownership
                if register.owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() && !register.is_locked() && !register.is_frozen() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot delete register {:?} in state {:?}",
                        operation.target, register.state
                    )));
                }
                
                // Mark for deletion
                register.state = RegisterState::PendingDeletion;
                register.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                // Add operation to history
                register.history.push(operation_id);
            },
            ResourceOperationType::Transfer => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Verify ownership
                if register.owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot transfer register {:?} in state {:?}",
                        operation.target, register.state
                    )));
                }
                
                // Get recipient from inputs
                let recipient = operation.parameters.get("recipient")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TelError::ResourceError(
                        "Transfer operation must specify recipient in parameters".to_string()
                    ))?;
                
                // Parse recipient address
                let to_address = recipient.parse().map_err(|_| 
                    TelError::ParseError(format!("Invalid recipient address: {}", recipient)))?;
                
                // Transfer ownership
                register.owner = to_address;
                register.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                // Add operation to history
                register.history.push(operation_id);
            },
            ResourceOperationType::Lock => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Verify ownership
                if register.owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot lock register {:?} in state {:?}",
                        operation.target, register.state
                    )));
                }
                
                // Lock register
                register.state = RegisterState::Locked;
                register.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                // Add operation to history
                register.history.push(operation_id);
            },
            ResourceOperationType::Unlock => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Verify ownership
                if register.owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_locked() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot unlock register {:?} in state {:?}",
                        operation.target, register.state
                    )));
                }
                
                // Unlock register
                register.state = RegisterState::Active;
                register.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                // Add operation to history
                register.history.push(operation_id);
            },
            _ => {
                return Err(TelError::ResourceError(format!(
                    "Unsupported operation type: {:?}", operation.operation_type
                )));
            }
        }
        
        // Log the operation
        let mut operations = self.operations.write().map_err(|_| 
            TelError::InternalError("Failed to acquire operations lock".to_string()))?;
        
        operations.push((operation_id, operation));
        
        Ok(operation_id)
    }
    
    /// Configure garbage collection
    pub fn configure_gc(&self, config: GarbageCollectionConfig) -> TelResult<()> {
        let mut gc_config = self.gc_config.write().map_err(|_| 
            TelError::InternalError("Failed to acquire GC config lock".to_string()))?;
        
        *gc_config = config;
        
        Ok(())
    }
    
    /// Run garbage collection
    pub fn run_garbage_collection(&self) -> TelResult<CollectionStats> {
        let start_time = Instant::now();
        let mut processed = 0;
        let mut archived = 0;
        let mut deleted = 0;
        
        // Get GC config
        let gc_config = self.gc_config.read().map_err(|_| 
            TelError::InternalError("Failed to acquire GC config lock".to_string()))?;
        
        let max_age_ms = gc_config.max_register_age_ms;
        let archive_retention_ms = gc_config.archive_retention_ms;
        let max_registers = gc_config.max_registers_per_run;
        
        // Get all registers
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        // Current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Find registers that need processing
        let mut register_ids_to_process = Vec::new();
        for (id, register) in registers.iter() {
            if register.is_active() {
                let age = now - register.created_at;
                if age > max_age_ms {
                    register_ids_to_process.push(*id);
                }
            } else if register.is_pending_deletion() {
                let age = now - register.updated_at;
                if age > archive_retention_ms {
                    register_ids_to_process.push(*id);
                }
            }
            
            // Limit the number of registers processed
            if register_ids_to_process.len() >= max_registers {
                break;
            }
        }
        
        // Process registers
        for id in register_ids_to_process {
            let register = registers.get_mut(&id).unwrap();
            processed += 1;
            
            if register.is_active() {
                // Mark for deletion
                register.state = RegisterState::PendingDeletion;
                register.updated_at = now;
                archived += 1;
            } else if register.is_pending_deletion() {
                // Convert to tombstone
                register.state = RegisterState::Tombstone;
                register.updated_at = now;
                register.contents = RegisterContents::Empty;
                deleted += 1;
            }
        }
        
        // Calculate time taken
        let time_taken = start_time.elapsed();
        let time_taken_ms = time_taken.as_millis() as u64;
        
        Ok(CollectionStats {
            processed,
            archived,
            deleted,
            time_taken_ms,
        })
    }
    
    /// Advance to the next epoch
    pub fn advance_epoch(&self) -> TelResult<u64> {
        let mut current_epoch = self.current_epoch.write().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch lock".to_string()))?;
        
        *current_epoch += 1;
        
        // Automatic garbage collection if enabled
        let gc_config = self.gc_config.read().map_err(|_| 
            TelError::InternalError("Failed to acquire GC config lock".to_string()))?;
        
        if gc_config.auto_gc_on_epoch_advance {
            drop(gc_config);
            let _ = self.run_garbage_collection();
        }
        
        Ok(*current_epoch)
    }
    
    /// Get the current epoch
    pub fn current_epoch(&self) -> TelResult<u64> {
        let current_epoch = self.current_epoch.read().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch lock".to_string()))?;
        
        Ok(*current_epoch)
    }
    
    /// Query registers by owner
    pub fn query_registers_by_owner(&self, owner: &Address) -> TelResult<Vec<RegisterId>> {
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let mut result = Vec::new();
        for (id, register) in registers.iter() {
            if &register.owner == owner && register.is_active() {
                result.push(*id);
            }
        }
        
        Ok(result)
    }
    
    /// Query registers by domain
    pub fn query_registers_by_domain(&self, domain: &Domain) -> TelResult<Vec<RegisterId>> {
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let mut result = Vec::new();
        for (id, register) in registers.iter() {
            if &register.domain == domain && register.is_active() {
                result.push(*id);
            }
        }
        
        Ok(result)
    }
    
    /// Generate a unique register ID
    fn generate_register_id(&self) -> RegisterId {
        RegisterId::new()
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
} 