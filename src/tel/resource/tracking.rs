// Resource tracking module for TEL
//
// This module provides mechanisms for tracking resource state
// and managing resource lifecycle through the register-based model.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::tel::types::{ResourceId, Address, DomainId, Timestamp};
use crate::tel::error::{TelError, TelResult};
use super::operations::{ResourceOperation, OperationId, ResourceOperationType};

/// Status of a resource in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Resource is active and can be operated on
    Active,
    /// Resource is locked for a specific operation
    Locked {
        /// ID of the operation locking the resource
        operation_id: OperationId,
        /// When the lock expires
        expiry: Timestamp,
    },
    /// Resource is frozen (operations suspended)
    Frozen {
        /// Reason for freezing
        reason: String,
        /// Authority that froze the resource
        authority: Address,
    },
    /// Resource is marked for deletion
    PendingDeletion {
        /// When the resource will be deleted
        scheduled_time: Timestamp,
    },
    /// Resource is deleted (tombstone)
    Tombstone {
        /// When the resource was deleted
        deletion_time: Timestamp,
        /// Hash of the resource before deletion
        content_hash: [u8; 32],
    },
}

/// State of a resource in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    /// Resource identifier
    pub resource_id: ResourceId,
    /// Current status
    pub status: ResourceStatus,
    /// Address of the owner
    pub owner: Address,
    /// Domain this resource belongs to
    pub domain: DomainId,
    /// Raw contents of the resource
    pub contents: Vec<u8>,
    /// Creation time
    pub created_at: Timestamp,
    /// Last updated time
    pub updated_at: Timestamp,
    /// History of operations on this resource
    pub operation_history: Vec<OperationId>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Tracks resource state and validates operations
#[derive(Debug)]
pub struct ResourceTracker {
    /// Map of resources being tracked
    resources: RwLock<HashMap<ResourceId, ResourceState>>,
    /// Map of operations that have been applied
    operations: RwLock<HashMap<OperationId, ResourceOperation>>,
    /// Map of locked resources to their expiry times
    locks: RwLock<HashMap<ResourceId, Timestamp>>,
}

impl ResourceTracker {
    /// Create a new resource tracker
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
            operations: RwLock::new(HashMap::new()),
            locks: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new resource
    pub fn register_resource(&self, state: ResourceState) -> TelResult<()> {
        let mut resources = self.resources.write().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        if resources.contains_key(&state.resource_id) {
            return Err(TelError::ResourceError(format!(
                "Resource already exists: {:?}", state.resource_id
            )));
        }
        
        resources.insert(state.resource_id, state);
        Ok(())
    }
    
    /// Get the state of a resource
    pub fn get_resource(&self, resource_id: &ResourceId) -> TelResult<ResourceState> {
        let resources = self.resources.read().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        resources.get(resource_id)
            .cloned()
            .ok_or_else(|| TelError::ResourceNotFound(format!("{:?}", resource_id)))
    }
    
    /// Update a resource's state
    pub fn update_resource(&self, state: ResourceState) -> TelResult<()> {
        let mut resources = self.resources.write().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        if !resources.contains_key(&state.resource_id) {
            return Err(TelError::ResourceNotFound(format!("{:?}", state.resource_id)));
        }
        
        resources.insert(state.resource_id, state);
        Ok(())
    }
    
    /// Delete a resource
    pub fn delete_resource(&self, resource_id: &ResourceId) -> TelResult<()> {
        let mut resources = self.resources.write().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        if !resources.contains_key(resource_id) {
            return Err(TelError::ResourceNotFound(format!("{:?}", resource_id)));
        }
        
        // In a real implementation, we would mark as tombstone instead of removing
        resources.remove(resource_id);
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
            operations.insert(operation.operation_id, operation.clone());
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
        let resources = self.resources.read().map_err(|_| 
            TelError::InternalError("Failed to acquire resource lock".to_string()))?;
        
        for resource_id in &operation.resource_ids {
            match resources.get(resource_id) {
                Some(state) => {
                    // Check if resource is active
                    match state.status {
                        ResourceStatus::Active => {
                            // Resource is active, continue validation
                        },
                        ResourceStatus::Locked { operation_id, expiry } => {
                            return Err(TelError::ResourceError(format!(
                                "Resource {:?} is locked by operation {:?} until {}", 
                                resource_id, operation_id, expiry
                            )));
                        },
                        ResourceStatus::Frozen { reason, .. } => {
                            return Err(TelError::ResourceError(format!(
                                "Resource {:?} is frozen: {}", resource_id, reason
                            )));
                        },
                        ResourceStatus::PendingDeletion { .. } => {
                            return Err(TelError::ResourceError(format!(
                                "Resource {:?} is pending deletion", resource_id
                            )));
                        },
                        ResourceStatus::Tombstone { .. } => {
                            return Err(TelError::ResourceError(format!(
                                "Resource {:?} has been deleted", resource_id
                            )));
                        },
                    }
                },
                None => {
                    // For create operations, it's okay if the resource doesn't exist
                    if operation.operation_type != ResourceOperationType::Create {
                        return Err(TelError::ResourceNotFound(format!("{:?}", resource_id)));
                    }
                },
            }
        }
        
        // Additional operation-specific validation could be added here
        
        Ok(())
    }
    
    // Implementation of operation handlers
    
    /// Apply a create operation
    fn apply_create_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let inputs = &operation.inputs;
        let contents = inputs.contents.as_ref().ok_or_else(|| 
            TelError::ValidationError("Create operation requires contents".to_string()))?;
            
        let resource_id = if !operation.resource_ids.is_empty() {
            operation.resource_ids[0]
        } else {
            return Err(TelError::ValidationError("No resource ID provided".to_string()));
        };
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        let state = ResourceState {
            resource_id,
            status: ResourceStatus::Active,
            owner: operation.initiator.clone(),
            domain: operation.domain.clone(),
            contents: contents.clone(),
            created_at: now,
            updated_at: now,
            operation_history: vec![operation.operation_id],
            metadata: operation.metadata.clone(),
        };
        
        self.register_resource(state)
    }
    
    /// Apply an update operation
    fn apply_update_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let inputs = &operation.inputs;
        let contents = inputs.contents.as_ref().ok_or_else(|| 
            TelError::ValidationError("Update operation requires contents".to_string()))?;
            
        let resource_id = if !operation.resource_ids.is_empty() {
            operation.resource_ids[0]
        } else {
            return Err(TelError::ValidationError("No resource ID provided".to_string()));
        };
        
        let mut state = self.get_resource(&resource_id)?;
        
        // Check ownership
        if state.owner != operation.initiator {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Update the state
        state.contents = contents.clone();
        state.updated_at = now;
        state.operation_history.push(operation.operation_id);
        
        self.update_resource(state)
    }
    
    /// Apply a delete operation
    fn apply_delete_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = if !operation.resource_ids.is_empty() {
            operation.resource_ids[0]
        } else {
            return Err(TelError::ValidationError("No resource ID provided".to_string()));
        };
        
        let state = self.get_resource(&resource_id)?;
        
        // Check ownership
        if state.owner != operation.initiator {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        self.delete_resource(&resource_id)
    }
    
    /// Apply a transfer operation
    fn apply_transfer_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let inputs = &operation.inputs;
        let target_address = inputs.target_address.as_ref().ok_or_else(|| 
            TelError::ValidationError("Transfer operation requires target address".to_string()))?;
            
        let resource_id = if !operation.resource_ids.is_empty() {
            operation.resource_ids[0]
        } else {
            return Err(TelError::ValidationError("No resource ID provided".to_string()));
        };
        
        let mut state = self.get_resource(&resource_id)?;
        
        // Check ownership
        if state.owner != operation.initiator {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Update ownership
        state.owner = target_address.clone();
        state.updated_at = now;
        state.operation_history.push(operation.operation_id);
        
        self.update_resource(state)
    }
    
    /// Apply a lock operation
    fn apply_lock_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = if !operation.resource_ids.is_empty() {
            operation.resource_ids[0]
        } else {
            return Err(TelError::ValidationError("No resource ID provided".to_string()));
        };
        
        // Get expiry time from parameters
        let expiry = match operation.inputs.parameters.get("expiry") {
            Some(Value::Number(n)) => {
                if let Some(expiry) = n.as_u64() {
                    expiry
                } else {
                    return Err(TelError::ValidationError("Invalid expiry time".to_string()));
                }
            },
            _ => {
                // Default to 5 minutes from now
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64 + 5 * 60 * 1000
            }
        };
        
        let mut state = self.get_resource(&resource_id)?;
        
        // Check ownership
        if state.owner != operation.initiator {
            return Err(TelError::AuthorizationError(format!(
                "Initiator {:?} is not the owner of resource {:?}", 
                operation.initiator, resource_id
            )));
        }
        
        // Lock the resource
        state.status = ResourceStatus::Locked {
            operation_id: operation.operation_id,
            expiry,
        };
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        state.updated_at = now;
        state.operation_history.push(operation.operation_id);
        
        // Record the lock
        {
            let mut locks = self.locks.write().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
            locks.insert(resource_id, expiry);
        }
        
        self.update_resource(state)
    }
    
    /// Apply an unlock operation
    fn apply_unlock_operation(&self, operation: ResourceOperation) -> TelResult<()> {
        let resource_id = if !operation.resource_ids.is_empty() {
            operation.resource_ids[0]
        } else {
            return Err(TelError::ValidationError("No resource ID provided".to_string()));
        };
        
        let mut state = self.get_resource(&resource_id)?;
        
        // Check that the resource is actually locked
        match state.status {
            ResourceStatus::Locked { operation_id, .. } => {
                // Check if the unlock is authorized
                // In a real implementation, we would check if the unlock operation
                // is authorized by the operation that locked the resource
                
                // For now, just check ownership
                if state.owner != operation.initiator {
                    return Err(TelError::AuthorizationError(format!(
                        "Initiator {:?} is not the owner of resource {:?}", 
                        operation.initiator, resource_id
                    )));
                }
            },
            _ => {
                return Err(TelError::ValidationError(format!(
                    "Resource {:?} is not locked", resource_id
                )));
            }
        }
        
        // Unlock the resource
        state.status = ResourceStatus::Active;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        state.updated_at = now;
        state.operation_history.push(operation.operation_id);
        
        // Remove the lock
        {
            let mut locks = self.locks.write().map_err(|_| 
                TelError::InternalError("Failed to acquire locks".to_string()))?;
            locks.remove(&resource_id);
        }
        
        self.update_resource(state)
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
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
} 