//! Integration between the new rigorous capability system and existing authorization code
//!
//! This module provides the bridge between the new capability-based authorization model
//! and the existing authorization systems, enabling a gradual migration path.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::fmt::Debug;

use crate::address::Address;
use crate::resource::{
    ResourceId, RegisterId, Quantity, Right,
    capability::{
        CapabilityRepository, Capability, CapabilityType, CapabilityId,
        CapabilityChain, CapabilityProof, AuthorizationService,
        Delegation, DelegationConstraint, CapabilityRevocation
    }
};
use crate::error::{Error, Result};

/// Integration layer between legacy authorization systems and the new capability system
pub struct CapabilityIntegration {
    /// The new capability repository
    capability_repo: Arc<dyn CapabilityRepository>,
    
    /// The new authorization service
    authorization_service: Arc<AuthorizationService>,
    
    /// Legacy authorization mappings
    legacy_authorizations: RwLock<HashMap<Address, HashSet<(ResourceId, Right)>>>,
}

impl CapabilityIntegration {
    /// Create a new capability integration layer
    pub fn new(
        capability_repo: Arc<dyn CapabilityRepository>,
        authorization_service: Arc<AuthorizationService>,
    ) -> Self {
        Self {
            capability_repo,
            authorization_service,
            legacy_authorizations: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a legacy authorization
    pub fn register_legacy_authorization(
        &self,
        address: &Address,
        resource_id: &ResourceId,
        right: Right,
    ) -> Result<()> {
        // Add to legacy mappings
        let mut authorizations = self.legacy_authorizations.write()
            .map_err(|e| Error::Internal(format!("Failed to acquire lock: {}", e)))?;
        
        authorizations
            .entry(address.clone())
            .or_insert_with(HashSet::new)
            .insert((resource_id.clone(), right));
        
        // Also create a root capability for this authorization
        let capability_id = self.create_authorization_capability(address, resource_id, right)?;
        
        Ok(())
    }
    
    /// Create a capability for a legacy authorization
    fn create_authorization_capability(
        &self,
        address: &Address,
        resource_id: &ResourceId,
        right: Right,
    ) -> Result<CapabilityId> {
        // Create a new root capability
        let capability = Capability::new_root(
            address.clone(),
            resource_id.clone(),
            right,
            "Legacy authorization".to_string(),
        );
        
        // Store in repository
        let capability_id = capability.id.clone();
        self.capability_repo.store_capability(capability)
            .map_err(|e| Error::CapabilityError(format!("Failed to store capability: {}", e)))?;
        
        Ok(capability_id)
    }
    
    /// Check if a legacy authorization exists
    pub fn has_legacy_authorization(
        &self,
        address: &Address,
        resource_id: &ResourceId,
        right: Right,
    ) -> Result<bool> {
        let authorizations = self.legacy_authorizations.read()
            .map_err(|e| Error::Internal(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(rights) = authorizations.get(address) {
            return Ok(rights.contains(&(resource_id.clone(), right)));
        }
        
        Ok(false)
    }
    
    /// Check if a capability exists (either via legacy or new system)
    pub fn has_capability(
        &self,
        address: &Address,
        resource_id: &ResourceId,
        right: Right,
    ) -> Result<bool> {
        // First check legacy authorizations
        if self.has_legacy_authorization(address, resource_id, right)? {
            return Ok(true);
        }
        
        // Then check new capability system
        let capabilities = self.authorization_service.find_capabilities(
            address,
            resource_id,
            right,
        ).map_err(|e| Error::CapabilityError(format!("Failed to find capabilities: {}", e)))?;
        
        Ok(!capabilities.is_empty())
    }
    
    /// Get capabilities for an address (from both legacy and new systems)
    pub fn get_capabilities_for_address(
        &self,
        address: &Address,
    ) -> Result<Vec<Capability>> {
        let mut capabilities = Vec::new();
        
        // Get legacy authorizations
        let authorizations = self.legacy_authorizations.read()
            .map_err(|e| Error::Internal(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(rights) = authorizations.get(address) {
            // Convert each legacy authorization to a capability
            for (resource_id, right) in rights {
                if let Ok(capability_id) = self.create_authorization_capability(
                    address, resource_id, *right
                ) {
                    if let Ok(Some(capability)) = self.capability_repo.get_capability(&capability_id) {
                        capabilities.push(capability);
                    }
                }
            }
        }
        
        // Get capabilities from new system
        let new_capabilities = self.authorization_service.get_capabilities_for_address(address)
            .map_err(|e| Error::CapabilityError(format!("Failed to get capabilities: {}", e)))?;
        
        // Combine results
        capabilities.extend(new_capabilities);
        
        Ok(capabilities)
    }
    
    /// Delegate a capability from one address to another
    pub fn delegate_capability(
        &self,
        from_address: &Address,
        to_address: &Address,
        resource_id: &ResourceId,
        right: Right,
        constraint: Option<DelegationConstraint>,
    ) -> Result<CapabilityId> {
        // Check if the source has the capability (either legacy or new)
        if !self.has_capability(from_address, resource_id, right)? {
            return Err(Error::CapabilityError(format!(
                "Address {} does not have capability for resource {} with right {:?}",
                from_address, resource_id, right
            )));
        }
        
        // Create delegation in the new system
        let delegation = Delegation::new(
            from_address.clone(),
            to_address.clone(),
            resource_id.clone(),
            right,
            constraint,
        );
        
        // Store delegation
        let capability_id = self.authorization_service.delegate_capability(delegation)
            .map_err(|e| Error::CapabilityError(format!("Failed to delegate capability: {}", e)))?;
        
        Ok(capability_id)
    }
    
    /// Revoke a capability
    pub fn revoke_capability(
        &self,
        revoker: &Address,
        capability_id: &CapabilityId,
    ) -> Result<()> {
        // Get the capability
        let capability = self.capability_repo.get_capability(capability_id)
            .map_err(|e| Error::CapabilityError(format!("Failed to get capability: {}", e)))?
            .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", capability_id)))?;
        
        // Create revocation
        let revocation = CapabilityRevocation::new(
            revoker.clone(),
            capability_id.clone(),
            "Explicit revocation".to_string(),
        );
        
        // Revoke the capability
        self.authorization_service.revoke_capability(revocation)
            .map_err(|e| Error::CapabilityError(format!("Failed to revoke capability: {}", e)))?;
        
        Ok(())
    }
    
    /// Convert a legacy authorization to a new capability
    pub fn convert_legacy_to_capability(
        &self,
        address: &Address,
        resource_id: &ResourceId,
        right: Right,
    ) -> Result<CapabilityId> {
        // Check if legacy authorization exists
        if !self.has_legacy_authorization(address, resource_id, right)? {
            return Err(Error::CapabilityError(format!(
                "No legacy authorization for address {} on resource {} with right {:?}",
                address, resource_id, right
            )));
        }
        
        // Create a new root capability
        let capability_id = self.create_authorization_capability(address, resource_id, right)?;
        
        // Remove from legacy system
        let mut authorizations = self.legacy_authorizations.write()
            .map_err(|e| Error::Internal(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(rights) = authorizations.get_mut(address) {
            rights.remove(&(resource_id.clone(), right));
            
            // Remove the address entry if it's now empty
            if rights.is_empty() {
                authorizations.remove(address);
            }
        }
        
        Ok(capability_id)
    }
    
    /// Convert all legacy authorizations for an address to new capabilities
    pub fn convert_all_legacy_authorizations(
        &self,
        address: &Address,
    ) -> Result<Vec<CapabilityId>> {
        let mut capability_ids = Vec::new();
        
        // Get legacy authorizations
        let mut authorizations = self.legacy_authorizations.write()
            .map_err(|e| Error::Internal(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(rights) = authorizations.get(address) {
            // Clone the rights to avoid borrowing issues
            let rights_clone: Vec<(ResourceId, Right)> = rights.iter()
                .map(|(r, right)| (r.clone(), *right))
                .collect();
            
            // Convert each authorization
            for (resource_id, right) in rights_clone {
                let capability_id = self.create_authorization_capability(
                    address, &resource_id, right
                )?;
                capability_ids.push(capability_id);
            }
            
            // Remove all legacy authorizations for this address
            authorizations.remove(address);
        }
        
        Ok(capability_ids)
    }
    
    /// Get the authorization service
    pub fn authorization_service(&self) -> Arc<AuthorizationService> {
        self.authorization_service.clone()
    }
    
    /// Get the capability repository
    pub fn capability_repository(&self) -> Arc<dyn CapabilityRepository> {
        self.capability_repo.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::capability::MockCapabilityRepository;
    
    fn create_test_setup() -> (
        CapabilityIntegration,
        Address,
        ResourceId,
        Right,
    ) {
        // Create test components
        let repo = Arc::new(MockCapabilityRepository::new());
        let auth_service = Arc::new(AuthorizationService::new(repo.clone()));
        
        // Create integration
        let integration = CapabilityIntegration::new(
            repo.clone(),
            auth_service,
        );
        
        // Test data
        let address = Address::new("test-user");
        let resource_id = ResourceId::new("test-resource");
        let right = Right::Read;
        
        (integration, address, resource_id, right)
    }
    
    #[test]
    fn test_legacy_authorization() -> Result<()> {
        let (integration, address, resource_id, right) = create_test_setup();
        
        // Register legacy authorization
        integration.register_legacy_authorization(&address, &resource_id, right)?;
        
        // Check it exists
        assert!(integration.has_legacy_authorization(&address, &resource_id, right)?);
        assert!(integration.has_capability(&address, &resource_id, right)?);
        
        // Check a non-existent authorization
        assert!(!integration.has_legacy_authorization(&address, &resource_id, Right::Write)?);
        
        Ok(())
    }
    
    #[test]
    fn test_delegation() -> Result<()> {
        let (integration, address, resource_id, right) = create_test_setup();
        
        // Register legacy authorization
        integration.register_legacy_authorization(&address, &resource_id, right)?;
        
        // Create another user
        let delegate = Address::new("delegate-user");
        
        // Delegate the capability
        let capability_id = integration.delegate_capability(
            &address,
            &delegate,
            &resource_id,
            right,
            None,
        )?;
        
        // Check delegate has capability
        assert!(integration.has_capability(&delegate, &resource_id, right)?);
        
        // Check the capability chain
        let capabilities = integration.get_capabilities_for_address(&delegate)?;
        assert!(!capabilities.is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_revocation() -> Result<()> {
        let (integration, address, resource_id, right) = create_test_setup();
        
        // Register legacy authorization
        integration.register_legacy_authorization(&address, &resource_id, right)?;
        
        // Create another user and delegate
        let delegate = Address::new("delegate-user");
        let capability_id = integration.delegate_capability(
            &address,
            &delegate,
            &resource_id,
            right,
            None,
        )?;
        
        // Revoke the capability
        integration.revoke_capability(&address, &capability_id)?;
        
        // Delegate should no longer have capability
        assert!(!integration.has_capability(&delegate, &resource_id, right)?);
        
        Ok(())
    }
    
    #[test]
    fn test_conversion() -> Result<()> {
        let (integration, address, resource_id, right) = create_test_setup();
        
        // Register multiple legacy authorizations
        integration.register_legacy_authorization(&address, &resource_id, Right::Read)?;
        integration.register_legacy_authorization(&address, &resource_id, Right::Write)?;
        
        // Convert one
        let capability_id = integration.convert_legacy_to_capability(
            &address, &resource_id, Right::Read
        )?;
        
        // Check state
        assert!(!integration.has_legacy_authorization(&address, &resource_id, Right::Read)?);
        assert!(integration.has_capability(&address, &resource_id, Right::Read)?);
        
        assert!(integration.has_legacy_authorization(&address, &resource_id, Right::Write)?);
        
        // Convert all remaining
        let capability_ids = integration.convert_all_legacy_authorizations(&address)?;
        assert_eq!(capability_ids.len(), 1); // Should be just the Write right
        
        // Check all legacy authorizations are gone
        assert!(!integration.has_legacy_authorization(&address, &resource_id, Right::Write)?);
        
        // But capabilities remain
        assert!(integration.has_capability(&address, &resource_id, Right::Read)?);
        assert!(integration.has_capability(&address, &resource_id, Right::Write)?);
        
        Ok(())
    }
} 