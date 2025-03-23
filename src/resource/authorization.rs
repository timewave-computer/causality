// Resource Authorization Module
//
// This module provides a capability-based authorization system for resource operations.
// It validates that operations on resources are only performed by entities with
// the appropriate capabilities, and enforces authorization based on the resource lifecycle state.

use std::sync::Arc;
use std::collections::HashMap;

use crate::resource::{ResourceId, CapabilityId};
use crate::resource::capability::{Capability, CapabilityType, Right};
use crate::resource::lifecycle_manager::{ResourceRegisterLifecycleManager, RegisterOperationType};
use crate::error::{Error, Result};
use crate::resource::capability_system::AuthorizationService;

/// Service for authorizing resource operations
pub struct ResourceAuthorizationService {
    /// Lifecycle manager for accessing resource states
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
    
    /// Authorization service for capability validation
    auth_service: AuthorizationService,
    
    /// Cache of capabilities for faster lookup
    capability_cache: HashMap<CapabilityId, Capability>,
}

impl ResourceAuthorizationService {
    /// Create a new resource authorization service
    pub fn new(lifecycle_manager: Arc<ResourceRegisterLifecycleManager>) -> Self {
        Self {
            auth_service: AuthorizationService::new(lifecycle_manager.clone()),
            lifecycle_manager,
            capability_cache: HashMap::new(),
        }
    }
    
    /// Register a capability with the authorization service
    pub fn register_capability(&mut self, capability: Capability) {
        self.capability_cache.insert(capability.id.clone(), capability);
    }
    
    /// Revoke a capability
    pub fn revoke_capability(&mut self, capability_id: &CapabilityId) -> Option<Capability> {
        self.capability_cache.remove(capability_id)
    }
    
    /// Get a capability by ID
    pub fn get_capability(&self, capability_id: &CapabilityId) -> Option<&Capability> {
        self.capability_cache.get(capability_id)
    }
    
    /// Check if an entity has permission to perform an operation on a resource
    pub fn has_permission(
        &self,
        entity_id: &str,
        resource_id: &ResourceId,
        operation_type: RegisterOperationType,
    ) -> Result<bool> {
        // Get all capabilities held by this entity for this resource
        let entity_capabilities: Vec<&Capability> = self.capability_cache
            .values()
            .filter(|cap| cap.holder == entity_id && cap.resource_id == *resource_id)
            .collect();
        
        if entity_capabilities.is_empty() {
            return Ok(false);
        }
        
        // Extract capability IDs
        let capability_ids: Vec<CapabilityId> = entity_capabilities
            .iter()
            .map(|cap| cap.id.clone())
            .collect();
        
        // Validate operation using the authorization service
        self.auth_service.check_operation_allowed(
            resource_id,
            operation_type,
            &capability_ids,
        )
    }
    
    /// Authorize and execute an operation
    pub fn authorize_and_execute<F>(
        &self,
        entity_id: &str,
        resource_id: &ResourceId,
        operation_type: RegisterOperationType,
        operation: F,
    ) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        // Check if the entity has permission
        if !self.has_permission(entity_id, resource_id, operation_type)? {
            return Err(Error::PermissionDenied(format!(
                "Entity {} does not have permission to perform {:?} on resource {}",
                entity_id, operation_type, resource_id
            )));
        }
        
        // Execute the operation
        operation()
    }
    
    /// Create a capability for an entity to perform operations on a resource
    pub fn create_capability(
        &mut self,
        resource_id: ResourceId,
        capability_type: CapabilityType,
        holder: String,
        delegated_from: Option<CapabilityId>,
    ) -> Result<Capability> {
        // Generate a unique ID for the capability
        let capability_id = format!("cap-{}-{}", resource_id, uuid::Uuid::new_v4());
        
        // Create the capability
        let capability = Capability {
            id: capability_id,
            resource_id,
            capability_type,
            holder,
            delegated_from,
        };
        
        // Register the capability
        self.register_capability(capability.clone());
        
        Ok(capability)
    }
    
    /// Delegate a capability to another entity
    pub fn delegate_capability(
        &mut self,
        original_capability_id: &CapabilityId,
        new_holder: String,
        delegated_rights: Option<Vec<Right>>,
    ) -> Result<Capability> {
        // Get the original capability
        let original_capability = self.get_capability(original_capability_id)
            .ok_or_else(|| Error::NotFound(format!("Capability {} not found", original_capability_id)))?
            .clone();
        
        // Determine the capability type for the delegation
        let new_capability_type = match delegated_rights {
            Some(rights) => CapabilityType::Custom(rights),
            None => original_capability.capability_type.clone(),
        };
        
        // Create the delegated capability
        self.create_capability(
            original_capability.resource_id,
            new_capability_type,
            new_holder,
            Some(original_capability_id.clone()),
        )
    }
    
    /// Get all capabilities for a resource
    pub fn get_capabilities_for_resource(&self, resource_id: &ResourceId) -> Vec<&Capability> {
        self.capability_cache
            .values()
            .filter(|cap| cap.resource_id == *resource_id)
            .collect()
    }
    
    /// Get all capabilities held by an entity
    pub fn get_capabilities_for_entity(&self, entity_id: &str) -> Vec<&Capability> {
        self.capability_cache
            .values()
            .filter(|cap| cap.holder == entity_id)
            .collect()
    }
    
    /// Check if a capability is a delegation
    pub fn is_delegation(&self, capability_id: &CapabilityId) -> bool {
        self.capability_cache.get(capability_id)
            .map(|cap| cap.delegated_from.is_some())
            .unwrap_or(false)
    }
    
    /// Validate a delegated capability chain
    pub fn validate_delegation_chain(&self, capability_id: &CapabilityId) -> Result<bool> {
        let mut current = match self.capability_cache.get(capability_id) {
            Some(cap) => cap,
            None => return Ok(false),
        };
        
        // If not delegated, it's valid
        if current.delegated_from.is_none() {
            return Ok(true);
        }
        
        // Check the delegation chain
        let mut visited = vec![capability_id.clone()];
        
        while let Some(delegated_from) = &current.delegated_from {
            // Check for circular delegation
            if visited.contains(delegated_from) {
                return Ok(false);
            }
            
            // Get the parent capability
            current = match self.capability_cache.get(delegated_from) {
                Some(cap) => cap,
                None => return Ok(false), // Parent capability not found
            };
            
            visited.push(delegated_from.clone());
        }
        
        // If we reached a root capability, the chain is valid
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_delegation() {
        // This would test capability delegation functionality
    }
    
    #[test]
    fn test_authorization_validation() {
        // This would test the authorization validation logic
    }
} 