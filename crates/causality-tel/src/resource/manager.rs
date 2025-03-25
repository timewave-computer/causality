// Resource manager for TEL
// Original file: src/tel/resource/model/manager.rs

// Resource manager implementation for TEL
//
// This module provides the ResourceManager which centralizes
// management of resources through the register-based model.
// Migration note: This file is being migrated to use the unified ResourceRegister model.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crypto;

use causality_tel::{ResourceId, Address, Domain, Timestamp, OperationId};
use causality_tel::{TelError, TelResult};
use causality_tel::{ResourceOperation, ResourceOperationType};
use causality_telel::{
    Register, ContentId, RegisterContents, RegisterState, 
    Resource, ResourceTimeData, ControllerLabel,
};
use crate::resource::{ResourceRegister, StateVisibility};

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
    registers: RwLock<HashMap<ContentId, Register>>,
    /// Operations log
    operations: RwLock<Vec<(OperationId, ResourceOperation)>>,
    /// Garbage collection configuration
    gc_config: RwLock<GarbageCollectionConfig>,
    /// Current epoch
    current_epoch: RwLock<u64>,
    /// Registers grouped by epoch
    registers_by_epoch: RwLock<HashMap<u64, HashSet<ContentId>>>,
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
    ) -> TelResult<ContentId> {
        let register_id = ContentId::new();
        let current_epoch = *self.current_epoch.read().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch lock".to_string()))?;
        
        // Create metadata with owner and domain information
        let mut metadata = HashMap::new();
        metadata.insert("tel_owner".to_string(), serde_json::Value::String(owner.clone()));
        metadata.insert("tel_domain".to_string(), serde_json::Value::String(domain.clone()));
        metadata.insert("tel_epoch".to_string(), serde_json::Value::Number(serde_json::Number::from(current_epoch)));
        
        // Create a new Register instance
        let register = Register::new(
            register_id.into(), // Convert ContentId to RegisterId
            contents,
            Some(metadata),
            RegisterState::Active,
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
    pub fn get_register(&self, register_id: &ContentId) -> TelResult<Register> {
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
        register_id: &ContentId,
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
                "Cannot update register {:?} in state {:?}", register_id, register.state()
            )));
        }
        
        // Update the register contents
        register.update_contents(contents)?;
        
        Ok(())
    }
    
    /// Delete a register
    pub fn delete_register(&self, register_id: &ContentId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_active() && !register.is_locked() && !register.is_frozen() {
            return Err(TelError::ResourceError(format!(
                "Cannot delete register {:?} in state {:?}", register_id, register.state()
            )));
        }
        
        // Mark for consuming (equivalent to deletion in unified model)
        register.consume()?;
        
        Ok(())
    }
    
    /// Transfer a register to a new owner
    pub fn transfer_register(
        &self,
        register_id: &ContentId,
        from: &Address,
        to: Address,
    ) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        // Get metadata to check ownership
        let metadata = register.metadata();
        let owner = metadata.get("tel_owner")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        
        // Check ownership
        if owner != from {
            return Err(TelError::AuthorizationError(format!(
                "Register {:?} is not owned by {:?}", register_id, from
            )));
        }
        
        if !register.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot transfer register {:?} in state {:?}", register_id, register.state()
            )));
        }
        
        // Update owner in metadata
        let mut updated_metadata = metadata.clone();
        updated_metadata.insert("tel_owner".to_string(), serde_json::Value::String(to));
        register.update_metadata(updated_metadata)?;
        
        Ok(())
    }
    
    /// Lock a register
    pub fn lock_register(&self, register_id: &ContentId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot lock register {:?} in state {:?}", register_id, register.state()
            )));
        }
        
        // Lock the register
        register.lock()?;
        
        Ok(())
    }
    
    /// Unlock a register
    pub fn unlock_register(&self, register_id: &ContentId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get_mut(register_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Register {:?} not found", register_id
            )))?;
        
        if !register.is_locked() {
            return Err(TelError::ResourceError(format!(
                "Cannot unlock register {:?} in state {:?}", register_id, register.state()
            )));
        }
        
        // Unlock the register
        register.unlock()?;
        
        Ok(())
    }
    
    /// Apply a resource operation
    pub fn apply_operation(&self, operation: ResourceOperation) -> TelResult<OperationId> {
        // Generate a unique operation ID based on operation content and time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
            
        // Combine operation details and timestamp for unique hash input
        let operation_data = format!(
            "operation-{}-{}-{}-{}",
            operation.operation_type,
            operation.target,
            operation.resource_id.map_or("none".to_string(), |id| id.to_string()),
            now
        );
        
        // Generate a content ID
        let hasher = crypto::hash::HashFactory::default().create_hasher().unwrap();
        let hash = hasher.hash(operation_data.as_bytes());
        let content_id = crypto::hash::ContentId::from(hash);
        
        // Create an OperationId from the content_id
        let operation_id = OperationId::from_content_id(&content_id);
        
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
                
                // Create metadata with owner and domain information
                let mut metadata = HashMap::new();
                metadata.insert("tel_owner".to_string(), serde_json::Value::String(operation.initiator.clone()));
                metadata.insert("tel_domain".to_string(), serde_json::Value::String(operation.domain.clone()));
                metadata.insert("tel_epoch".to_string(), serde_json::Value::Number(serde_json::Number::from(current_epoch)));
                metadata.insert("tel_operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                
                // Create a new Register instance
                let register = Register::new(
                    operation.target.into(), // Convert ContentId to RegisterId
                    contents,
                    Some(metadata),
                    RegisterState::Active,
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
                
                // Get metadata to check ownership
                let metadata = register.metadata();
                let owner = metadata.get("tel_owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                
                // Verify ownership
                if owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot update register {:?} in state {:?}",
                        operation.target, register.state()
                    )));
                }
                
                // Get contents from inputs
                let contents = operation.inputs.get(0)
                    .ok_or_else(|| TelError::ResourceError(
                        "Update operation must have contents as input".to_string()
                    ))?
                    .clone();
                
                // Update register contents
                register.update_contents(contents)?;
                
                // Update operation history
                let mut updated_metadata = metadata.clone();
                updated_metadata.insert("tel_operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                register.update_metadata(updated_metadata)?;
            },
            ResourceOperationType::Delete => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Get metadata to check ownership
                let metadata = register.metadata();
                let owner = metadata.get("tel_owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                
                // Verify ownership
                if owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() && !register.is_locked() && !register.is_frozen() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot delete register {:?} in state {:?}",
                        operation.target, register.state()
                    )));
                }
                
                // Mark for consuming (equivalent to deletion in unified model)
                register.consume()?;
                
                // Update operation history
                let mut updated_metadata = metadata.clone();
                updated_metadata.insert("tel_operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                register.update_metadata(updated_metadata)?;
            },
            ResourceOperationType::Transfer => {
                // Check if register exists and initiator is owner
                let mut registers = self.registers.write().map_err(|_| 
                    TelError::InternalError("Failed to acquire registers lock".to_string()))?;
                
                let register = registers.get_mut(&operation.target)
                    .ok_or_else(|| TelError::ResourceError(format!(
                        "Register {:?} not found", operation.target
                    )))?;
                
                // Get metadata to check ownership
                let metadata = register.metadata();
                let owner = metadata.get("tel_owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                
                // Verify ownership
                if owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot transfer register {:?} in state {:?}",
                        operation.target, register.state()
                    )));
                }
                
                // Get new owner from inputs
                let new_owner = operation.outputs.get(0)
                    .ok_or_else(|| TelError::ResourceError(
                        "Transfer operation must have new owner as output".to_string()
                    ))?;
                
                // Get new owner address
                let new_owner_address = match new_owner {
                    RegisterContents::String(addr) => addr.clone(),
                    _ => return Err(TelError::ResourceError(
                        "New owner must be a string address".to_string()
                    )),
                };
                
                // Update owner in metadata
                let mut updated_metadata = metadata.clone();
                updated_metadata.insert("tel_owner".to_string(), serde_json::Value::String(new_owner_address));
                updated_metadata.insert("tel_operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                register.update_metadata(updated_metadata)?;
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
                let metadata = register.metadata();
                let owner = metadata.get("tel_owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                
                if owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_active() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot lock register {:?} in state {:?}",
                        operation.target, register.state()
                    )));
                }
                
                // Lock the register
                register.lock()?;
                
                // Update operation history
                let mut updated_metadata = metadata.clone();
                updated_metadata.insert("tel_operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                register.update_metadata(updated_metadata)?;
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
                let metadata = register.metadata();
                let owner = metadata.get("tel_owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                
                if owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Operation initiator {:?} is not the owner of register {:?}",
                        operation.initiator, operation.target
                    )));
                }
                
                // Check register state
                if !register.is_locked() {
                    return Err(TelError::ResourceError(format!(
                        "Cannot unlock register {:?} in state {:?}",
                        operation.target, register.state()
                    )));
                }
                
                // Unlock the register
                register.unlock()?;
                
                // Update operation history
                let mut updated_metadata = metadata.clone();
                updated_metadata.insert("tel_operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                register.update_metadata(updated_metadata)?;
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
    pub fn query_registers_by_owner(&self, owner: &Address) -> TelResult<Vec<ContentId>> {
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
    pub fn query_registers_by_domain(&self, domain: &Domain) -> TelResult<Vec<ContentId>> {
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
    fn generate_register_id(&self) -> ContentId {
        ContentId::new()
    }
    
    /// Get a resource register by ID
    pub fn get_resource_register(&self, resource_id: &ContentId) -> TelResult<ResourceRegister> {
        // Get the register
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let register = registers.get(resource_id)
            .ok_or_else(|| TelError::ResourceError(format!(
                "Resource register {:?} not found", resource_id
            )))?;
        
        // Convert Register to ResourceRegister
        Ok(register.resource_register().clone())
    }
    
    /// Update a resource register
    pub fn update_resource_register(&self, resource_id: &ContentId, resource_register: ResourceRegister) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        // Check if register exists
        if !registers.contains_key(resource_id) {
            return Err(TelError::ResourceError(format!(
                "Resource register {:?} not found", resource_id
            )));
        }
        
        // Update the register with the new resource register
        let register = Register::from_resource_register(resource_register);
        registers.insert(*resource_id, register);
        
        Ok(())
    }
    
    /// Get all resource registers
    pub fn get_all_resource_registers(&self) -> TelResult<Vec<ResourceRegister>> {
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let mut result = Vec::new();
        for register in registers.values() {
            result.push(register.resource_register().clone());
        }
        
        Ok(result)
    }
    
    /// Get resource registers by domain
    pub fn get_resource_registers_by_domain(&self, domain: &Domain) -> TelResult<Vec<ResourceRegister>> {
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire registers lock".to_string()))?;
        
        let mut result = Vec::new();
        for register in registers.values() {
            // Check if register belongs to the specified domain
            // We need to extract the domain from the metadata
            if let Some(reg_domain) = register.resource_register().metadata.get("domain") {
                if reg_domain == &domain.to_string() {
                    result.push(register.resource_register().clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Clear all resource registers (for testing or snapshot restoration)
    pub fn clear_all_resource_registers(&self) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers write lock".to_string()))?;
        
        registers.clear();
        
        // Also clear epoch data
        let mut registers_by_epoch = self.registers_by_epoch.write().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch data write lock".to_string()))?;
        
        registers_by_epoch.clear();
        
        Ok(())
    }
    
    /// Restore a resource register from a snapshot
    pub fn restore_resource_register(&self, resource_register: &ResourceRegister) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire registers write lock".to_string()))?;
        
        // Create a Register from the ResourceRegister
        let register = Register::from_resource_register(resource_register.clone());
        
        // Add to the collection
        registers.insert(resource_register.id, register);
        
        // Add to epoch tracking
        let mut registers_by_epoch = self.registers_by_epoch.write().map_err(|_| 
            TelError::InternalError("Failed to acquire epoch data write lock".to_string()))?;
        
        let current_epoch = *self.current_epoch.read().map_err(|_| 
            TelError::InternalError("Failed to acquire current epoch lock".to_string()))?;
        
        registers_by_epoch
            .entry(current_epoch)
            .or_insert_with(HashSet::new)
            .insert(resource_register.id);
        
        Ok(())
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
} 
