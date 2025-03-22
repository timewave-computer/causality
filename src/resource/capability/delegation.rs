//! Capability delegation and revocation system
//!
//! This module provides enhanced functionality for delegating and revoking capabilities
//! with built-in constraint enforcement, verification, and security features.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::address::Address;
use crate::resource::{
    ResourceId, RegisterId, Quantity, Right,
    capability::{
        CapabilityRepository, Capability, CapabilityType, CapabilityId,
        CapabilityChain, Delegation, Revocation, DelegationConstraint,
        TemporalConstraint, QuantityConstraint, ExclusivityConstraint,
        validation::{CapabilityValidator, CapabilityValidationResult}
    }
};
use crate::error::{Error, Result};

/// Enhanced delegation manager for capabilities
pub struct DelegationManager {
    /// Repository for capability storage
    repository: Arc<dyn CapabilityRepository>,
    
    /// Validator for capability validation
    validator: CapabilityValidator,
}

impl DelegationManager {
    /// Create a new delegation manager
    pub fn new(repository: Arc<dyn CapabilityRepository>) -> Self {
        let validator = CapabilityValidator::new(repository.clone());
        Self { repository, validator }
    }
    
    /// Delegate a capability with enhanced security and constraints
    ///
    /// This method creates a new capability delegated from an existing one,
    /// with optional constraints on the delegation.
    pub fn delegate_capability(
        &self,
        delegator: &Address,
        delegatee: &Address,
        parent_capability_id: &CapabilityId,
        constraints: Option<Vec<DelegationConstraint>>,
        purpose: Option<String>,
    ) -> Result<CapabilityId> {
        // 1. Validate the parent capability
        let validation = self.validator.validate_capability(parent_capability_id, delegator)?;
        
        if !validation.valid {
            let reason = validation.reason.unwrap_or_else(|| "Unknown validation failure".to_string());
            return Err(Error::CapabilityError(
                format!("Cannot delegate invalid capability: {}", reason)
            ));
        }
        
        // 2. Get the parent capability
        let parent = self.repository.get_capability(parent_capability_id)?
            .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", parent_capability_id)))?;
        
        // 3. Check if parent is delegatable (not all capabilities can be delegated)
        if let Some(delegatable) = parent.attributes.get("delegatable") {
            if delegatable == "false" {
                return Err(Error::CapabilityError(
                    format!("Capability is not delegatable: {}", parent_capability_id)
                ));
            }
        }
        
        // 4. Check if the delegatee already has this capability
        // This prevents duplicate delegations
        let existing = self.repository.find_capabilities_for_resource(
            delegatee,
            &parent.resource_id,
            Some(parent.right),
        )?;
        
        if !existing.is_empty() {
            // Optionally could return the existing capability instead of error
            return Err(Error::CapabilityError(
                format!("Delegatee already has a capability for this resource and right")
            ));
        }
        
        // 5. Create the delegation info
        let delegation_info = Delegation {
            parent_capability_id: parent_capability_id.clone(),
            delegator: delegator.clone(),
            purpose: purpose.unwrap_or_else(|| "General delegation".to_string()),
            constraints,
            delegated_at: SystemTime::now(),
        };
        
        // 6. Create the new capability
        let new_capability = Capability {
            id: CapabilityId::new_unique(),
            owner: delegatee.clone(),
            resource_id: parent.resource_id.clone(),
            right: parent.right,
            capability_type: CapabilityType::Delegated,
            delegation: Some(delegation_info),
            revocation: None,
            attributes: HashMap::new(),
            resource_info: parent.resource_info.clone(),
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
        };
        
        // 7. Store the new capability
        self.repository.store_capability(new_capability.clone())?;
        
        // 8. Return the new capability ID
        Ok(new_capability.id)
    }
    
    /// Create a delegation chain
    ///
    /// This method creates a chain of delegations from an existing capability,
    /// passing through multiple delegates in sequence.
    pub fn create_delegation_chain(
        &self,
        starting_capability_id: &CapabilityId,
        owner: &Address,
        delegates: &[Address],
        constraints: Option<Vec<DelegationConstraint>>,
        purpose: Option<String>,
    ) -> Result<Vec<CapabilityId>> {
        if delegates.is_empty() {
            return Ok(vec![]);
        }
        
        let mut capability_ids = Vec::new();
        let mut current_owner = owner.clone();
        let mut current_capability_id = starting_capability_id.clone();
        
        // Create the delegation chain
        for delegate in delegates {
            // Delegate the current capability to the next delegate
            current_capability_id = self.delegate_capability(
                &current_owner,
                delegate,
                &current_capability_id,
                constraints.clone(),
                purpose.clone(),
            )?;
            
            capability_ids.push(current_capability_id.clone());
            current_owner = delegate.clone();
        }
        
        Ok(capability_ids)
    }
    
    /// Revoke a capability
    ///
    /// This method revokes a capability, preventing its further use.
    /// Optionally cascades the revocation to all derived capabilities.
    pub fn revoke_capability(
        &self,
        capability_id: &CapabilityId,
        revoker: &Address,
        reason: String,
        cascade: bool,
    ) -> Result<()> {
        // 1. Check that the capability exists
        let mut capability = self.repository.get_capability(capability_id)?
            .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", capability_id)))?;
        
        // 2. Check authorization to revoke
        // The revoker must either:
        // - Own the capability
        // - Be the original delegator (if this is a delegated capability)
        // - Have admin rights (not implemented here)
        let authorized = if capability.owner == *revoker {
            true
        } else if let Some(ref delegation) = capability.delegation {
            delegation.delegator == *revoker
        } else {
            false
        };
        
        if !authorized {
            return Err(Error::Unauthorized(
                format!("Not authorized to revoke capability: {}", capability_id)
            ));
        }
        
        // 3. Check if already revoked
        if capability.revocation.is_some() {
            return Ok(()); // Already revoked, nothing to do
        }
        
        // 4. Create revocation information
        let revocation = Revocation {
            revoked_by: revoker.clone(),
            reason,
            revoked_at: SystemTime::now(),
        };
        
        // 5. Update the capability with revocation info
        capability.revocation = Some(revocation);
        capability.last_updated = SystemTime::now();
        
        // 6. Store the updated capability
        self.repository.store_capability(capability)?;
        
        // 7. If cascade is true, revoke all derived capabilities
        if cascade {
            let delegated = self.repository.find_delegated_capabilities(capability_id)?;
            
            for delegated_id in delegated {
                self.revoke_capability(
                    &delegated_id,
                    revoker,
                    "Cascading revocation from parent capability".to_string(),
                    true, // Continue cascading
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Transfer a capability to a new owner
    ///
    /// This method transfers ownership of a capability to a new address.
    pub fn transfer_capability(
        &self,
        capability_id: &CapabilityId,
        current_owner: &Address,
        new_owner: &Address,
    ) -> Result<()> {
        // 1. Validate the capability
        let validation = self.validator.validate_capability(capability_id, current_owner)?;
        
        if !validation.valid {
            let reason = validation.reason.unwrap_or_else(|| "Unknown validation failure".to_string());
            return Err(Error::CapabilityError(
                format!("Cannot transfer invalid capability: {}", reason)
            ));
        }
        
        // 2. Get the capability
        let mut capability = self.repository.get_capability(capability_id)?
            .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", capability_id)))?;
        
        // 3. Check if the capability is transferable
        if let Some(transferable) = capability.attributes.get("transferable") {
            if transferable == "false" {
                return Err(Error::CapabilityError(
                    format!("Capability is not transferable: {}", capability_id)
                ));
            }
        }
        
        // 4. Update ownership
        capability.owner = new_owner.clone();
        capability.last_updated = SystemTime::now();
        
        // 5. Store the updated capability
        self.repository.store_capability(capability)?;
        
        Ok(())
    }
    
    /// Create a template from an existing capability
    ///
    /// This method creates a template from an existing capability that can be
    /// used to create new capabilities with the same properties.
    pub fn create_capability_template(
        &self,
        capability_id: &CapabilityId,
        owner: &Address,
        template_name: String,
    ) -> Result<CapabilityId> {
        // 1. Validate the capability
        let validation = self.validator.validate_capability(capability_id, owner)?;
        
        if !validation.valid {
            let reason = validation.reason.unwrap_or_else(|| "Unknown validation failure".to_string());
            return Err(Error::CapabilityError(
                format!("Cannot create template from invalid capability: {}", reason)
            ));
        }
        
        // 2. Get the source capability
        let source = self.repository.get_capability(capability_id)?
            .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", capability_id)))?;
        
        // 3. Create a new template capability
        let mut attributes = source.attributes.clone();
        attributes.insert("is_template".to_string(), "true".to_string());
        attributes.insert("template_name".to_string(), template_name);
        
        let template = Capability {
            id: CapabilityId::new_unique(),
            owner: owner.clone(),
            resource_id: source.resource_id.clone(),
            right: source.right,
            capability_type: CapabilityType::Template,
            delegation: None,  // Templates don't have delegation info
            revocation: None,
            attributes,
            resource_info: source.resource_info.clone(),
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
        };
        
        // 4. Store the template
        self.repository.store_capability(template.clone())?;
        
        // 5. Return the template ID
        Ok(template.id)
    }
    
    /// Create a capability from a template
    ///
    /// This method creates a new capability based on a template.
    pub fn create_from_template(
        &self,
        template_id: &CapabilityId,
        template_owner: &Address,
        new_owner: &Address,
        resource_id: Option<ResourceId>,
    ) -> Result<CapabilityId> {
        // 1. Validate the template
        let validation = self.validator.validate_capability(template_id, template_owner)?;
        
        if !validation.valid {
            let reason = validation.reason.unwrap_or_else(|| "Unknown validation failure".to_string());
            return Err(Error::CapabilityError(
                format!("Cannot use invalid template: {}", reason)
            ));
        }
        
        // 2. Get the template
        let template = self.repository.get_capability(template_id)?
            .ok_or_else(|| Error::NotFound(format!("Template not found: {}", template_id)))?;
        
        // 3. Verify this is actually a template
        if template.capability_type != CapabilityType::Template {
            return Err(Error::CapabilityError(
                format!("Capability is not a template: {}", template_id)
            ));
        }
        
        // 4. Create the new capability from the template
        let new_capability = Capability {
            id: CapabilityId::new_unique(),
            owner: new_owner.clone(),
            resource_id: resource_id.unwrap_or_else(|| template.resource_id.clone()),
            right: template.right,
            capability_type: CapabilityType::Root, // New capabilities from templates are roots
            delegation: None,
            revocation: None,
            attributes: template.attributes.clone(),
            resource_info: template.resource_info.clone(),
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
        };
        
        // 5. Store the new capability
        self.repository.store_capability(new_capability.clone())?;
        
        // 6. Return the new capability ID
        Ok(new_capability.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::capability::MockCapabilityRepository;
    
    fn create_test_setup() -> (
        DelegationManager,
        Arc<MockCapabilityRepository>,
        Address,
        ResourceId,
        Right,
    ) {
        let repo = Arc::new(MockCapabilityRepository::new());
        let manager = DelegationManager::new(repo.clone());
        
        let owner = Address::new("test-owner");
        let resource_id = ResourceId::new("test-resource");
        let right = Right::Read;
        
        (manager, repo, owner, resource_id, right)
    }
    
    #[test]
    fn test_delegate_capability() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let mut root = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        // Make it delegatable
        root.attributes.insert("delegatable".to_string(), "true".to_string());
        
        let root_id = root.id.clone();
        repo.store_capability(root)?;
        
        // Create a delegate
        let delegate = Address::new("test-delegate");
        
        // Delegate the capability
        let delegated_id = manager.delegate_capability(
            &owner,
            &delegate,
            &root_id,
            None,
            Some("Test delegation".to_string()),
        )?;
        
        // Verify the delegated capability
        let delegated = repo.get_capability(&delegated_id)?.unwrap();
        
        assert_eq!(delegated.owner, delegate);
        assert_eq!(delegated.resource_id, resource_id);
        assert_eq!(delegated.right, right);
        assert_eq!(delegated.capability_type, CapabilityType::Delegated);
        assert!(delegated.delegation.is_some());
        
        let delegation = delegated.delegation.unwrap();
        assert_eq!(delegation.parent_capability_id, root_id);
        assert_eq!(delegation.delegator, owner);
        assert_eq!(delegation.purpose, "Test delegation");
        
        Ok(())
    }
    
    #[test]
    fn test_delegation_chain() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let mut root = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Root capability".to_string(),
        );
        
        // Make it delegatable
        root.attributes.insert("delegatable".to_string(), "true".to_string());
        
        let root_id = root.id.clone();
        repo.store_capability(root)?;
        
        // Create delegates
        let delegate1 = Address::new("delegate-1");
        let delegate2 = Address::new("delegate-2");
        let delegate3 = Address::new("delegate-3");
        
        // Create a delegation chain
        let chain_ids = manager.create_delegation_chain(
            &root_id,
            &owner,
            &[delegate1.clone(), delegate2.clone(), delegate3.clone()],
            None,
            Some("Chain delegation".to_string()),
        )?;
        
        // Verify the chain length
        assert_eq!(chain_ids.len(), 3);
        
        // Verify each capability in the chain
        let cap1 = repo.get_capability(&chain_ids[0])?.unwrap();
        assert_eq!(cap1.owner, delegate1);
        
        let cap2 = repo.get_capability(&chain_ids[1])?.unwrap();
        assert_eq!(cap2.owner, delegate2);
        
        let cap3 = repo.get_capability(&chain_ids[2])?.unwrap();
        assert_eq!(cap3.owner, delegate3);
        
        // Verify the delegation chain links
        let del1 = cap1.delegation.unwrap();
        assert_eq!(del1.parent_capability_id, root_id);
        
        let del2 = cap2.delegation.unwrap();
        assert_eq!(del2.parent_capability_id, chain_ids[0]);
        
        let del3 = cap3.delegation.unwrap();
        assert_eq!(del3.parent_capability_id, chain_ids[1]);
        
        Ok(())
    }
    
    #[test]
    fn test_revoke_capability() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let mut root = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        // Make it delegatable
        root.attributes.insert("delegatable".to_string(), "true".to_string());
        
        let root_id = root.id.clone();
        repo.store_capability(root)?;
        
        // Create a delegate
        let delegate = Address::new("test-delegate");
        
        // Delegate the capability
        let delegated_id = manager.delegate_capability(
            &owner,
            &delegate,
            &root_id,
            None,
            Some("Test delegation".to_string()),
        )?;
        
        // Revoke the delegated capability
        manager.revoke_capability(
            &delegated_id,
            &owner, // The delegator is revoking
            "Test revocation".to_string(),
            false,
        )?;
        
        // Verify the capability is revoked
        let revoked = repo.get_capability(&delegated_id)?.unwrap();
        assert!(revoked.revocation.is_some());
        
        let revocation = revoked.revocation.unwrap();
        assert_eq!(revocation.revoked_by, owner);
        assert_eq!(revocation.reason, "Test revocation");
        
        Ok(())
    }
    
    #[test]
    fn test_capability_template() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let root = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let root_id = root.id.clone();
        repo.store_capability(root)?;
        
        // Create a template
        let template_id = manager.create_capability_template(
            &root_id,
            &owner,
            "Test Template".to_string(),
        )?;
        
        // Verify the template
        let template = repo.get_capability(&template_id)?.unwrap();
        assert_eq!(template.capability_type, CapabilityType::Template);
        assert_eq!(template.attributes.get("is_template").unwrap(), "true");
        assert_eq!(template.attributes.get("template_name").unwrap(), "Test Template");
        
        // Create a capability from the template
        let new_owner = Address::new("new-owner");
        let new_resource = ResourceId::new("new-resource");
        
        let new_id = manager.create_from_template(
            &template_id,
            &owner,
            &new_owner,
            Some(new_resource.clone()),
        )?;
        
        // Verify the new capability
        let new_cap = repo.get_capability(&new_id)?.unwrap();
        assert_eq!(new_cap.owner, new_owner);
        assert_eq!(new_cap.resource_id, new_resource);
        assert_eq!(new_cap.right, right);
        assert_eq!(new_cap.capability_type, CapabilityType::Root);
        
        Ok(())
    }
} 