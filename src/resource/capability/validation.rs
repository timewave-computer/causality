//! Enhanced capability validation logic using the new capability interfaces
//!
//! This module provides improved validation capabilities for the capability system,
//! including delegation chain verification, constraint enforcement, and time-based validation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::address::Address;
use crate::resource::{
    ResourceId, RegisterId, Quantity, Right,
    capability::{
        CapabilityRepository, Capability, CapabilityType, CapabilityId,
        CapabilityChain, CapabilityProof, DelegationConstraint,
        TemporalConstraint, QuantityConstraint, ExclusivityConstraint
    }
};
use crate::error::{Error, Result};

/// Enhanced capability validator
pub struct CapabilityValidator {
    /// Repository for capability access
    repository: Arc<dyn CapabilityRepository>,
}

impl CapabilityValidator {
    /// Create a new capability validator
    pub fn new(repository: Arc<dyn CapabilityRepository>) -> Self {
        Self { repository }
    }
    
    /// Validate a capability ID
    pub fn validate_capability(
        &self,
        capability_id: &CapabilityId,
        user_address: &Address,
    ) -> Result<CapabilityValidationResult> {
        // Get the capability
        let capability = self.repository.get_capability(capability_id)
            .map_err(|e| Error::CapabilityError(format!("Failed to get capability: {}", e)))?
            .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", capability_id)))?;
        
        // Check if the capability is owned by the user
        if &capability.owner != user_address {
            return Ok(CapabilityValidationResult {
                valid: false,
                reason: Some("Capability is not owned by the user".to_string()),
                capability_chain: None,
            });
        }
        
        // Check if the capability is revoked
        if let Some(revocation) = &capability.revocation {
            return Ok(CapabilityValidationResult {
                valid: false,
                reason: Some(format!("Capability has been revoked: {}", revocation.reason)),
                capability_chain: None,
            });
        }
        
        // Validate the delegation chain
        self.validate_delegation_chain(capability_id)
    }
    
    /// Validate a delegation chain for a capability
    pub fn validate_delegation_chain(
        &self,
        capability_id: &CapabilityId,
    ) -> Result<CapabilityValidationResult> {
        // Build the capability chain
        let chain = self.build_capability_chain(capability_id)?;
        
        // For root capabilities, there's no delegation to validate
        if chain.len() == 1 {
            return Ok(CapabilityValidationResult {
                valid: true,
                reason: None,
                capability_chain: Some(chain),
            });
        }
        
        // Check each link in the chain
        for i in 1..chain.len() {
            let parent = &chain[i-1];
            let child = &chain[i];
            
            // Check if parent is revoked
            if let Some(revocation) = &parent.revocation {
                return Ok(CapabilityValidationResult {
                    valid: false,
                    reason: Some(format!("Parent capability has been revoked: {}", revocation.reason)),
                    capability_chain: Some(chain),
                });
            }
            
            // Validate delegation constraints
            if let Some(delegation) = &child.delegation {
                if let Some(constraints) = &delegation.constraints {
                    let validation = self.validate_constraints(constraints, child)?;
                    if !validation.valid {
                        return Ok(validation);
                    }
                }
            }
        }
        
        // All checks passed
        Ok(CapabilityValidationResult {
            valid: true,
            reason: None,
            capability_chain: Some(chain),
        })
    }
    
    /// Build a capability chain for a capability
    pub fn build_capability_chain(
        &self,
        capability_id: &CapabilityId,
    ) -> Result<CapabilityChain> {
        let mut chain = Vec::new();
        let mut current_id = capability_id.clone();
        
        // Prevent infinite loops
        let mut visited = HashSet::new();
        visited.insert(current_id.clone());
        
        // Build the chain from the current capability up to the root
        loop {
            // Get the current capability
            let capability = self.repository.get_capability(&current_id)
                .map_err(|e| Error::CapabilityError(format!("Failed to get capability: {}", e)))?
                .ok_or_else(|| Error::NotFound(format!("Capability not found: {}", current_id)))?;
            
            // Add to chain
            chain.push(capability.clone());
            
            // If this is a root capability, we're done
            if capability.capability_type == CapabilityType::Root || capability.delegation.is_none() {
                break;
            }
            
            // Get the parent capability ID
            if let Some(ref delegation) = capability.delegation {
                current_id = delegation.parent_capability_id.clone();
                
                // Check for cycles
                if !visited.insert(current_id.clone()) {
                    return Err(Error::CapabilityError(format!("Cycle detected in capability chain")));
                }
            } else {
                break;
            }
        }
        
        // Reverse the chain to start with the root
        chain.reverse();
        
        Ok(chain)
    }
    
    /// Validate delegation constraints
    pub fn validate_constraints(
        &self,
        constraints: &Vec<DelegationConstraint>,
        capability: &Capability,
    ) -> Result<CapabilityValidationResult> {
        // Check each constraint
        for constraint in constraints {
            match constraint {
                DelegationConstraint::Temporal(temporal) => {
                    if !self.validate_temporal_constraint(temporal)? {
                        return Ok(CapabilityValidationResult {
                            valid: false,
                            reason: Some(format!("Temporal constraint violated")),
                            capability_chain: None,
                        });
                    }
                },
                DelegationConstraint::Quantity(quantity) => {
                    if !self.validate_quantity_constraint(quantity, capability)? {
                        return Ok(CapabilityValidationResult {
                            valid: false,
                            reason: Some(format!("Quantity constraint violated")),
                            capability_chain: None,
                        });
                    }
                },
                DelegationConstraint::Exclusivity(exclusivity) => {
                    if !self.validate_exclusivity_constraint(exclusivity, capability)? {
                        return Ok(CapabilityValidationResult {
                            valid: false,
                            reason: Some(format!("Exclusivity constraint violated")),
                            capability_chain: None,
                        });
                    }
                },
            }
        }
        
        // All constraints pass
        Ok(CapabilityValidationResult {
            valid: true,
            reason: None,
            capability_chain: None,
        })
    }
    
    /// Validate a temporal constraint
    pub fn validate_temporal_constraint(
        &self,
        constraint: &TemporalConstraint,
    ) -> Result<bool> {
        let now = SystemTime::now();
        
        // Check start time
        if let Some(start_time) = constraint.start_time {
            if now < start_time {
                return Ok(false); // Not yet valid
            }
        }
        
        // Check end time
        if let Some(end_time) = constraint.end_time {
            if now > end_time {
                return Ok(false); // No longer valid
            }
        }
        
        // Check duration
        if let Some(duration) = constraint.duration {
            if let Some(creation_time) = constraint.creation_time {
                let expiration = creation_time + duration;
                if now > expiration {
                    return Ok(false); // Duration expired
                }
            }
        }
        
        Ok(true)
    }
    
    /// Validate a quantity constraint
    pub fn validate_quantity_constraint(
        &self,
        constraint: &QuantityConstraint,
        capability: &Capability,
    ) -> Result<bool> {
        // Get the resource quantity from the capability
        let quantity = capability.resource_info.get("quantity")
            .and_then(|q| q.parse::<u64>().ok())
            .unwrap_or(0);
        
        // Check against the maximum
        if let Some(max) = constraint.maximum {
            if quantity > max {
                return Ok(false);
            }
        }
        
        // Check against the minimum
        if let Some(min) = constraint.minimum {
            if quantity < min {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Validate an exclusivity constraint
    pub fn validate_exclusivity_constraint(
        &self,
        constraint: &ExclusivityConstraint,
        capability: &Capability,
    ) -> Result<bool> {
        // If this isn't exclusive, it's valid
        if !constraint.exclusive {
            return Ok(true);
        }
        
        // For exclusive capabilities, check if there are other delegations
        // from this same parent for the same resource and right
        if let Some(ref delegation) = capability.delegation {
            let parent_id = &delegation.parent_capability_id;
            
            // Find all capabilities delegated from this parent
            let delegated = self.repository.find_delegated_capabilities(parent_id)
                .map_err(|e| Error::CapabilityError(format!("Failed to find delegated capabilities: {}", e)))?;
            
            // Check if there are any other active capabilities for the same resource/right
            for other_id in delegated {
                // Skip this capability
                if other_id == capability.id {
                    continue;
                }
                
                // Get the other capability
                if let Ok(Some(other)) = self.repository.get_capability(&other_id) {
                    // Skip revoked capabilities
                    if other.revocation.is_some() {
                        continue;
                    }
                    
                    // Check for same resource and right
                    if other.resource_id == capability.resource_id && other.right == capability.right {
                        return Ok(false); // Violates exclusivity
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    /// Create a capability proof
    pub fn create_capability_proof(
        &self,
        capability_id: &CapabilityId,
        user_address: &Address,
    ) -> Result<CapabilityProof> {
        // Validate the capability
        let validation = self.validate_capability(capability_id, user_address)?;
        
        if !validation.valid {
            return Err(Error::CapabilityError(format!(
                "Cannot create proof for invalid capability: {}",
                validation.reason.unwrap_or_else(|| "Unknown reason".to_string())
            )));
        }
        
        // Get the capability chain
        let chain = validation.capability_chain.ok_or_else(|| {
            Error::CapabilityError("Missing capability chain in validation result".to_string())
        })?;
        
        // Create the proof
        let proof = CapabilityProof {
            capability_id: capability_id.clone(),
            owner: user_address.clone(),
            resource_id: chain[0].resource_id.clone(), // Root resource ID
            right: chain[0].right,                     // Root right
            created_at: SystemTime::now(),
            chain_length: chain.len() as u32,
            verification_hash: self.compute_chain_hash(&chain),
        };
        
        Ok(proof)
    }
    
    /// Compute a hash for a capability chain (simplified hash for demo)
    fn compute_chain_hash(&self, chain: &CapabilityChain) -> String {
        // In a real implementation, this would be a cryptographic hash
        // For demo purposes, we'll just concatenate IDs
        let mut hash_input = String::new();
        for capability in chain {
            hash_input.push_str(&capability.id.to_string());
        }
        
        format!("HASH[{}]", hash_input)
    }
    
    /// Verify a capability proof
    pub fn verify_capability_proof(
        &self,
        proof: &CapabilityProof,
    ) -> Result<bool> {
        // First, ensure the capability exists
        let capability = self.repository.get_capability(&proof.capability_id)
            .map_err(|e| Error::CapabilityError(format!("Failed to get capability: {}", e)))?;
        
        if capability.is_none() {
            return Ok(false);
        }
        
        // Check that the proof owner matches the capability owner
        let capability = capability.unwrap();
        if capability.owner != proof.owner {
            return Ok(false);
        }
        
        // Build the chain to verify against
        let chain = self.build_capability_chain(&proof.capability_id)?;
        
        // Check chain length
        if chain.len() as u32 != proof.chain_length {
            return Ok(false);
        }
        
        // Compute the hash and compare
        let computed_hash = self.compute_chain_hash(&chain);
        if computed_hash != proof.verification_hash {
            return Ok(false);
        }
        
        // Verify the capability is still valid
        let validation = self.validate_capability_chain(&proof.capability_id)?;
        
        Ok(validation.valid)
    }
}

/// Result of capability validation
#[derive(Debug, Clone)]
pub struct CapabilityValidationResult {
    /// Whether the capability is valid
    pub valid: bool,
    
    /// Reason for invalidity (if any)
    pub reason: Option<String>,
    
    /// The capability chain (if built)
    pub capability_chain: Option<CapabilityChain>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::capability::{MockCapabilityRepository, AuthorizationService};
    use std::time::{Duration, SystemTime};
    
    fn create_test_setup() -> (
        CapabilityValidator,
        Arc<MockCapabilityRepository>,
        Address,
        ResourceId,
        Right,
    ) {
        // Create test components
        let repo = Arc::new(MockCapabilityRepository::new());
        let validator = CapabilityValidator::new(repo.clone());
        
        // Test data
        let address = Address::new("test-user");
        let resource_id = ResourceId::new("test-resource");
        let right = Right::Read;
        
        (validator, repo, address, resource_id, right)
    }
    
    #[test]
    fn test_validate_root_capability() -> Result<()> {
        let (validator, repo, address, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let capability = Capability::new_root(
            address.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let capability_id = capability.id.clone();
        repo.store_capability(capability)?;
        
        // Validate
        let result = validator.validate_capability(&capability_id, &address)?;
        assert!(result.valid);
        
        Ok(())
    }
    
    #[test]
    fn test_validate_delegation_chain() -> Result<()> {
        let (validator, repo, address, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let root = Capability::new_root(
            address.clone(),
            resource_id.clone(),
            right,
            "Root capability".to_string(),
        );
        
        let root_id = root.id.clone();
        repo.store_capability(root)?;
        
        // Create a delegate
        let delegate = Address::new("delegate-user");
        
        // Create auth service for delegation
        let auth_service = AuthorizationService::new(repo.clone());
        
        // Create delegation
        let delegation = auth_service.delegate_capability(
            crate::resource::capability::Delegation::new(
                address.clone(),
                delegate.clone(),
                resource_id.clone(),
                right,
                None,
            )
        )?;
        
        // Validate the delegation
        let result = validator.validate_delegation_chain(&delegation)?;
        assert!(result.valid);
        assert!(result.capability_chain.is_some());
        assert_eq!(result.capability_chain.unwrap().len(), 2); // Root + delegation
        
        Ok(())
    }
    
    #[test]
    fn test_temporal_constraint() -> Result<()> {
        let (validator, repo, address, resource_id, right) = create_test_setup();
        
        // Test expired constraint
        let expired = TemporalConstraint {
            start_time: None,
            end_time: Some(SystemTime::now() - Duration::from_secs(3600)), // 1 hour ago
            duration: None,
            creation_time: None,
        };
        
        assert!(!validator.validate_temporal_constraint(&expired)?);
        
        // Test future constraint
        let future = TemporalConstraint {
            start_time: Some(SystemTime::now() + Duration::from_secs(3600)), // 1 hour from now
            end_time: None,
            duration: None,
            creation_time: None,
        };
        
        assert!(!validator.validate_temporal_constraint(&future)?);
        
        // Test valid constraint
        let valid = TemporalConstraint {
            start_time: Some(SystemTime::now() - Duration::from_secs(3600)), // 1 hour ago
            end_time: Some(SystemTime::now() + Duration::from_secs(3600)),   // 1 hour from now
            duration: None,
            creation_time: None,
        };
        
        assert!(validator.validate_temporal_constraint(&valid)?);
        
        Ok(())
    }
    
    #[test]
    fn test_capability_proof() -> Result<()> {
        let (validator, repo, address, resource_id, right) = create_test_setup();
        
        // Create a root capability
        let capability = Capability::new_root(
            address.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let capability_id = capability.id.clone();
        repo.store_capability(capability)?;
        
        // Create a proof
        let proof = validator.create_capability_proof(&capability_id, &address)?;
        
        // Verify the proof
        assert!(validator.verify_capability_proof(&proof)?);
        
        Ok(())
    }
} 