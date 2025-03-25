// Resource tracking in TEL
// Original file: src/tel/resource/tracking.rs

// Resource tracking module for TEL
//
// This module provides mechanisms for tracking resource state
// and managing resource lifecycle through the register-based model.
// Migrated to use the unified ResourceRegister model.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_tel::{Address, DomainId, Timestamp};
use crate::crypto::ContentId;
use causality_tel::{TelError, TelResult};
use causality_telel::Register;
use super::operations::{ResourceOperation, OperationId, ResourceOperationType};

/// Extension wrapper for tracking operation history and additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRegistryEntry {
    /// The underlying register implementation (which uses ResourceRegister)
    register: Register,
    /// History of operations on this register
    operation_history: Vec<OperationId>,
    /// Additional tracking metadata not stored in the register
    tracking_metadata: HashMap<String, Value>,
}

impl ResourceRegistryEntry {
    /// Create a new registry entry from a Register
    pub fn new(register: Register) -> Self {
        Self {
            register,
            operation_history: Vec::new(),
            tracking_metadata: HashMap::new(),
        }
    }
    
    /// Get the underlying register
    pub fn register(&self) -> &Register {
        &self.register
    }
    
    /// Get the underlying register (mutable)
    pub fn register_mut(&mut self) -> &mut Register {
        &mut self.register
    }
    
    /// Get the operation history
    pub fn operation_history(&self) -> &[OperationId] {
        &self.operation_history
    }
    
    /// Add an operation to the history
    pub fn add_operation(&mut self, operation_id: OperationId) {
        self.operation_history.push(operation_id);
    }
    
    /// Get tracking metadata
    pub fn tracking_metadata(&self) -> &HashMap<String, Value> {
        &self.tracking_metadata
    }
    
    /// Add tracking metadata
    pub fn add_tracking_metadata(&mut self, key: String, value: Value) {
        self.tracking_metadata.insert(key, value);
    }
    
    /// Get the resource ID
    pub fn id(&self) -> &ContentId {
        // Access the content ID via the underlying register
        // Using register_id() to get the RegisterId which contains ContentId
        let register_id = self.register.id();
        &register_id.0
    }
    
    /// Get the owner from metadata
    pub fn owner(&self) -> Option<String> {
        let metadata = self.register.metadata();
        metadata.get("tel_owner")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }
    
    /// Get the domain from metadata
    pub fn domain(&self) -> Option<String> {
        let metadata = self.register.metadata();
        metadata.get("tel_domain")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }
    
    /// Check if the register is active
    pub fn is_active(&self) -> bool {
        self.register.is_active()
    }
    
    /// Check if the register is locked
    pub fn is_locked(&self) -> bool {
        self.register.is_locked()
    }
    
    /// Check if the register is frozen
    pub fn is_frozen(&self) -> bool {
        self.register.is_frozen()
    }
    
    /// Check if the register is consumed (deleted)
    pub fn is_consumed(&self) -> bool {
        self.register.is_consumed()
    }
    
    /// Check if the register is archived
    pub fn is_archived(&self) -> bool {
        self.register.is_archived()
    }
    
    /// Get creation time from metadata
    pub fn created_at(&self) -> Option<u64> {
        let metadata = self.register.metadata();
        metadata.get("tel_created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
    }
    
    /// Get updated time from metadata
    pub fn updated_at(&self) -> Option<u64> {
        let metadata = self.register.metadata();
        metadata.get("tel_updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
    }
}

/// Tracks resource state and validates operations using the unified ResourceRegister model
#[derive(Debug)]
pub struct ResourceTracker {
    /// Map of registers being tracked
    registers: RwLock<HashMap<ContentId, ResourceRegistryEntry>>,
    /// Map of operations that have been applied
    operations: RwLock<HashMap<OperationId, ResourceOperation>>,
    /// Map of locked registers to their expiry times
    locks: RwLock<HashMap<ContentId, Timestamp>>,
}

impl ResourceTracker {
    /// Create a new resource tracker
    pub fn new() -> Self {
        Self {
            registers: RwLock::new(HashMap::new()),
            operations: RwLock::new(HashMap::new()),
            locks: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new resource
    pub fn register_resource(&self, register: Register) -> TelResult<()> {
        let register_id = *register.id().0;
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        if registers.contains_key(&register_id) {
            return Err(TelError::ResourceError(format!(
                "Resource already exists: {:?}", register_id
            )));
        }
        
        registers.insert(register_id, ResourceRegistryEntry::new(register));
        Ok(())
    }
    
    /// Get a resource register
    pub fn get_resource(&self, resource_id: &ContentId) -> TelResult<ResourceRegistryEntry> {
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        registers.get(resource_id)
            .cloned()
            .ok_or_else(|| TelError::ResourceNotFound(format!("{:?}", resource_id)))
    }
    
    /// Update a resource register
    pub fn update_resource(&self, register: Register) -> TelResult<()> {
        let register_id = *register.id().0;
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        if !registers.contains_key(&register_id) {
            return Err(TelError::ResourceNotFound(format!("{:?}", register_id)));
        }
        
        // Get the existing entry to preserve history and tracking metadata
        let existing_entry = registers.get(&register_id).unwrap();
        let mut updated_entry = ResourceRegistryEntry::new(register);
        
        // Copy operation history and tracking metadata
        updated_entry.operation_history = existing_entry.operation_history.clone();
        updated_entry.tracking_metadata = existing_entry.tracking_metadata.clone();
        
        registers.insert(register_id, updated_entry);
        Ok(())
    }
    
    /// Delete a resource
    pub fn delete_resource(&self, resource_id: &ContentId) -> TelResult<()> {
        let mut registers = self.registers.write().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        let entry = registers.get_mut(resource_id)
            .ok_or_else(|| TelError::ResourceNotFound(format!("{:?}", resource_id)))?;
        
        // Mark the register as consumed (deleted)
        entry.register_mut().consume()?;
        
        // In a real implementation, we might keep the entry but mark it as consumed
        // instead of removing it entirely
        
        Ok(())
    }
    
    /// Apply an operation to resources
    pub fn apply_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        // First, validate the operation
        self.validate_operation(&operation)?;
        
        // Record the operation
        {
            let mut operations = self.operations.write().map_err(|_| 
                TelError::InternalError("Failed to acquire operations lock".to_string()))?;
            operations.insert(operation.operation_id.clone(), operation.clone());
        }
        
        // Apply the operation based on its type
        match operation.operation_type {
            ResourceOperationType::Create => {
                self.apply_create_operation(operation)
            },
            ResourceOperationType::Update => {
                self.apply_update_operation(operation)
            },
            ResourceOperationType::Delete => {
                self.apply_delete_operation(operation)
            },
            ResourceOperationType::Transfer => {
                self.apply_transfer_operation(operation)
            },
            ResourceOperationType::Lock => {
                self.apply_lock_operation(operation)
            },
            ResourceOperationType::Unlock => {
                self.apply_unlock_operation(operation)
            },
            _ => {
                // For other operations, we would implement specific handling
                Err(TelError::UnsupportedOperation(format!(
                    "Operation type {:?} not yet implemented", operation.operation_type
                )))
            }
        }
    }
    
    /// Validate an operation before applying it
    fn validate_operation(&self, operation: &ResourceOperation) -> TelResult<()> {
        // Check if the operation has already been applied
        {
            let operations = self.operations.read().map_err(|_| 
                TelError::InternalError("Failed to acquire operations lock".to_string()))?;
            
            if operations.contains_key(&operation.operation_id) {
                return Err(TelError::ValidationError(format!(
                    "Operation already applied: {:?}", operation.operation_id
                )));
            }
        }
        
        // Validate resources exist and are in the correct state
        let registers = self.registers.read().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        // Get the target resource ID from the operation
        let target_id = &operation.target;
        
        // Check if the resource exists (except for create operations)
        if operation.operation_type != ResourceOperationType::Create {
            let entry = registers.get(target_id).ok_or_else(|| 
                TelError::ResourceNotFound(format!("{:?}", target_id)))?;
                
            // Check resource state
            if entry.is_consumed() {
                return Err(TelError::ResourceError(format!(
                    "Resource {:?} has been consumed/deleted", target_id
                )));
            }
            
            if entry.is_archived() {
                return Err(TelError::ResourceError(format!(
                    "Resource {:?} has been archived", target_id
                )));
            }
            
            // For most operations, the resource should be active
            if operation.operation_type != ResourceOperationType::Unlock && entry.is_locked() {
                // Get lock info from tracking metadata
                let lock_info = entry.tracking_metadata.get("lock_info")
                    .and_then(|v| v.as_object())
                    .map(|obj| format!("{:?}", obj))
                    .unwrap_or_else(|| "Unknown lock".to_string());
                    
                return Err(TelError::ResourceError(format!(
                    "Resource {:?} is locked: {}", target_id, lock_info
                )));
            }
            
            // For unlock operations, the resource must be locked
            if operation.operation_type == ResourceOperationType::Unlock && !entry.is_locked() {
                return Err(TelError::ResourceError(format!(
                    "Resource {:?} is not locked", target_id
                )));
            }
            
            // Check if resource is frozen (for operations that can't be applied to frozen resources)
            if ![ResourceOperationType::Unlock].contains(&operation.operation_type) && entry.is_frozen() {
                return Err(TelError::ResourceError(format!(
                    "Resource {:?} is frozen", target_id
                )));
            }
        } else {
            // For create operations, check the resource doesn't already exist
            if registers.contains_key(target_id) {
                return Err(TelError::ValidationError(format!(
                    "Resource {:?} already exists", target_id
                )));
            }
        }
        
        // Additional operation-specific validation could be added here
        
        Ok(())
    }
    
    // Implementation of operation handlers
    
    /// Apply a create operation
    fn apply_create_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = operation.target;
        let contents = operation.payload.as_ref()
            .ok_or_else(|| TelError::ValidationError(
                "Create operation must include payload".to_string()
            ))?;
            
        // Check if resource already exists
        {
            let registers = self.registers.read().map_err(|_| 
                TelError::InternalError("Failed to acquire resource lock".to_string()))?;
                
            if registers.contains_key(&resource_id) {
                return Err(TelError::ValidationError(format!(
                    "Resource already exists: {:?}", resource_id
                )));
            }
        }
        
        // Get current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Create metadata with owner, domain, timestamps
        let mut metadata = operation.metadata.clone();
        metadata.insert("tel_owner".to_string(), Value::String(operation.initiator.to_string()));
        metadata.insert("tel_domain".to_string(), Value::String(operation.domain.to_string()));
        metadata.insert("tel_created_at".to_string(), Value::String(now.to_string()));
        metadata.insert("tel_updated_at".to_string(), Value::String(now.to_string()));
        
        // Create a new register using the ResourceRegister model
        let register = Register::new_with_id(resource_id, contents.clone(), metadata);
        
        // Add the operation to history
        let mut entry = ResourceRegistryEntry::new(register.clone());
        entry.add_operation(operation.operation_id.clone());
        
        // Register the resource
        {
            let mut registers = self.registers.write().map_err(|_| 
                TelError::InternalError("Failed to acquire resource lock".to_string()))?;
                
            registers.insert(resource_id, entry);
        }
        
        Ok(())
    }
    
    /// Apply an update operation
    fn apply_update_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = operation.target;
        let contents = operation.payload.as_ref()
            .ok_or_else(|| TelError::ValidationError(
                "Update operation must include payload".to_string()
            ))?;
        
        let mut entry = self.get_resource(&resource_id)?;
        
        // Check ownership
        if entry.owner().map(|o| o != operation.initiator.to_string()).unwrap_or(true) {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        // Get current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Update the register with new contents
        entry.register_mut().update(contents.clone())?;
        
        // Update operation history and metadata
        entry.add_operation(operation.operation_id);
        entry.add_tracking_metadata("tel_updated_at".to_string(), Value::String(now.to_string()));
        
        // Update the resource in the tracker
        self.update_resource(entry.register().clone())
    }
    
    /// Apply a delete operation
    fn apply_delete_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = operation.target;
        
        let entry = self.get_resource(&resource_id)?;
        
        // Check ownership
        if entry.owner().map(|o| o != operation.initiator.to_string()).unwrap_or(true) {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        // For delete operations using the unified model, we consume the register
        // This corresponds to the RegisterOperationType::Consume
        self.delete_resource(&resource_id)
    }
    
    /// Apply a transfer operation
    fn apply_transfer_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = operation.target;
        let new_owner = operation.metadata.get("new_owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TelError::ValidationError(
                "Transfer operation must include new_owner in metadata".to_string()
            ))?;
        
        let mut entry = self.get_resource(&resource_id)?;
        
        // Check ownership
        if entry.owner().map(|o| o != operation.initiator.to_string()).unwrap_or(true) {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        // Get current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Update owner in register metadata
        let mut metadata = entry.register().metadata().clone();
        metadata.insert("tel_owner".to_string(), Value::String(new_owner.to_string()));
        metadata.insert("tel_updated_at".to_string(), Value::String(now.to_string()));
        
        // Create a new register with the updated metadata
        let mut updated_register = entry.register().clone();
        updated_register.update_metadata(metadata)?;
        
        // Update operation history
        entry.add_operation(operation.operation_id);
        
        // Update the resource in the tracker
        self.update_resource(updated_register)
    }
    
    /// Apply a lock operation
    fn apply_lock_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = operation.target;
        let expiry = operation.metadata.get("expiry")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| {
                // Default lock expiry (15 minutes)
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64 + 15 * 60 * 1000
            });
        
        let mut entry = self.get_resource(&resource_id)?;
        
        // Check ownership or authorization
        if entry.owner().map(|o| o != operation.initiator.to_string()).unwrap_or(true) {
            // Check if initiator has authorization to lock
            let is_authorized = operation.metadata.get("authorized_by")
                .and_then(|v| v.as_str())
                .map(|auth| auth == "system" || auth == "admin")
                .unwrap_or(false);
                
            if !is_authorized {
                return Err(TelError::AuthorizationError(format!(
                    "Initiator {:?} is not authorized to lock resource {:?}", 
                    operation.initiator, resource_id
                )));
            }
        }
        
        // Store lock information in tracking metadata
        let lock_info = serde_json::json!({
            "operation_id": operation.operation_id,
            "expiry": expiry,
            "locked_by": operation.initiator
        });
        
        entry.add_tracking_metadata("lock_info".to_string(), lock_info);
        
        // Lock the register
        entry.register_mut().lock()?;
        
        // Track the lock expiry
        {
            let mut locks = self.locks.write().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
            locks.insert(resource_id, expiry);
        }
        
        // Update operation history
        entry.add_operation(operation.operation_id);
        
        // Update the resource in the tracker
        self.update_resource(entry.register().clone())
    }
    
    /// Apply an unlock operation
    fn apply_unlock_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = operation.target;
        
        let mut entry = self.get_resource(&resource_id)?;
        
        // Get lock info
        let lock_info = entry.tracking_metadata().get("lock_info")
            .and_then(|v| v.as_object())
            .ok_or_else(|| TelError::ResourceError(format!(
                "Resource {:?} lock information not found", resource_id
            )))?;
        
        // Check if the initiator is the owner, the one who locked it, or system/admin
        let is_owner = entry.owner().map(|o| o == operation.initiator.to_string()).unwrap_or(false);
        let locked_by = lock_info.get("locked_by").and_then(|v| v.as_str()).unwrap_or("");
        let is_locker = locked_by == operation.initiator.to_string();
        let is_admin = operation.metadata.get("authorized_by")
            .and_then(|v| v.as_str())
            .map(|auth| auth == "system" || auth == "admin")
            .unwrap_or(false);
            
        if !is_owner && !is_locker && !is_admin {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not authorized to unlock resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        // Remove lock tracking metadata
        entry.tracking_metadata.remove("lock_info");
        
        // Unlock the register
        entry.register_mut().unlock()?;
        
        // Remove the lock expiry tracking
        {
            let mut locks = self.locks.write().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
            locks.remove(&resource_id);
        }
        
        // Update operation history
        entry.add_operation(operation.operation_id);
        
        // Update the resource in the tracker
        self.update_resource(entry.register().clone())
    }
    
    /// Check for expired locks and clear them
    pub fn cleanup_expired_locks(&self) -> TelResult<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        let mut expired_resources = Vec::new();
        
        // Find expired locks
        {
            let locks = self.locks.read().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
                
            for (resource_id, expiry) in locks.iter() {
                if *expiry <= now {
                    expired_resources.push(*resource_id);
                }
            }
        }
        
        // Update resources with expired locks
        for resource_id in &expired_resources {
            if let Ok(mut state) = self.get_resource(resource_id) {
                if let ResourceStatus::Locked { expiry, .. } = state.status {
                    if expiry <= now {
                        state.status = ResourceStatus::Active;
                        state.updated_at = now;
                        let _ = self.update_resource(state);
                    }
                }
            }
        }
        
        // Remove expired locks
        {
            let mut locks = self.locks.write().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
                
            for resource_id in &expired_resources {
                locks.remove(resource_id);
            }
        }
        
        Ok(expired_resources.len())
    }

    /// Check and expire locks that have passed their expiry time
    pub fn check_expired_locks(&self) -> TelResult<Vec<ContentId>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        let mut expired_locks = Vec::new();
        
        // Find expired locks
        {
            let locks = self.locks.read().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
                
            for (resource_id, expiry) in locks.iter() {
                if *expiry <= now {
                    expired_locks.push(*resource_id);
                }
            }
        }
        
        // Unlock resources with expired locks
        for resource_id in &expired_locks {
            // Create a system unlock operation
            let operation_id = OperationId::new();
            let operation = ResourceOperation {
                operation_id,
                operation_type: ResourceOperationType::Unlock,
                initiator: "system".to_string(),
                domain: "system".to_string(),
                target: *resource_id,
                payload: None,
                metadata: serde_json::json!({
                    "authorized_by": "system",
                    "reason": "lock expired"
                }).as_object().cloned().unwrap_or_default(),
                timestamp: now,
            };
            
            // Apply the unlock operation
            match self.apply_unlock_operation(operation) {
                Ok(_) => {
                    // Successfully unlocked
                },
                Err(err) => {
                    // Log the error but continue with other expired locks
                    eprintln!("Failed to unlock expired lock for resource {:?}: {:?}", 
                              resource_id, err);
                }
            }
        }
        
        Ok(expired_locks)
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
} 
