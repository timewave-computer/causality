// Capability system implementation
// Original file: src/resource/capability_system.rs

// Rigorous Capability System
//
// This module implements a rigorous capability model that integrates with the unified
// resource register architecture, providing secure capability-based authorization.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::resource::{ContentId, CapabilityId, ResourceRegister, Right, CapabilityType};
use causality_resource_manager::ResourceRegisterLifecycleManager;
use causality_resource::RelationshipTracker;
use causality_types::Address;
use causality_types::{Error, Result};

/// A formal definition of a capability within the rigorous capability model
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RigorousCapability {
    /// Unique ID of this capability
    pub id: CapabilityId,
    
    /// Resource ID this capability applies to
    pub resource_id: ContentId,
    
    /// Rights granted by this capability
    pub rights: HashSet<Right>,
    
    /// Delegated from another capability (chain of trust)
    pub delegated_from: Option<CapabilityId>,
    
    /// The address that issued this capability
    pub issuer: Address,
    
    /// The address that owns this capability
    pub owner: Address,
    
    /// Optional expiration timestamp
    pub expires_at: Option<u64>,
    
    /// Optional revocation identifier 
    pub revocation_id: Option<String>,
    
    /// Whether the capability can be delegated further
    pub delegatable: bool,
    
    /// Constraints on the capability's use
    pub constraints: Vec<CapabilityConstraint>,
    
    /// Cryptographic proof of validity (signature, etc.)
    pub proof: Option<CapabilityProof>,
}

/// Constraints that limit how a capability can be used
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CapabilityConstraint {
    /// Limit capability to specific operations
    Operations(Vec<String>),
    
    /// Limit capability to specific domains
    Domains(Vec<String>),
    
    /// Limit capability to a time window
    TimeWindow { start: u64, end: u64 },
    
    /// Limit capability to a maximum number of uses
    MaxUses(u32),
    
    /// Limit capability to a maximum quantity for fungible resources
    MaxQuantity(u128),
    
    /// Require additional authentication factors
    RequireAuthFactor(AuthenticationFactor),
    
    /// Custom constraint with arbitrary logic
    Custom {
        name: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Authentication factors that can be required for capability use
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AuthenticationFactor {
    /// Require signature with specific key
    Signature,
    
    /// Require multi-signature with a threshold
    MultiSignature { threshold: u32, keys: Vec<String> },
    
    /// Require a specific authentication method
    Method(String),
    
    /// Require a secret knowledge token
    SecretToken,
}

/// Cryptographic proof of capability validity
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CapabilityProof {
    /// Type of proof
    pub proof_type: ProofType,
    
    /// Proof data (signatures, certificate, etc.)
    pub data: Vec<u8>,
    
    /// Metadata about the proof
    pub metadata: HashMap<String, String>,
}

/// Types of cryptographic proofs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProofType {
    /// ED25519 signature
    Ed25519Signature,
    
    /// ECDSA signature
    EcdsaSignature,
    
    /// Multi-signature
    MultiSignature,
    
    /// ZK proof
    ZkProof,
    
    /// Certificate-based proof
    Certificate,
    
    /// Custom proof type
    Custom(String),
}

/// Status of a capability validation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    /// Capability is valid and can be used
    Valid,
    
    /// Capability is invalid
    Invalid { reason: String },
    
    /// Capability is expired
    Expired,
    
    /// Capability is revoked
    Revoked { revocation_id: String },
    
    /// Capability exists but is missing required proof
    MissingProof,
    
    /// Capability use would exceed constraints
    ConstraintViolated { constraint: String },
}

/// Interface for the rigorous capability system
#[async_trait]
pub trait CapabilitySystem: Send + Sync {
    /// Create a new capability
    async fn create_capability(&self, capability: RigorousCapability) -> Result<CapabilityId>;
    
    /// Get a capability by ID
    async fn get_capability(&self, id: &CapabilityId) -> Result<RigorousCapability>;
    
    /// Validate a capability
    async fn validate_capability(&self, id: &CapabilityId) -> Result<CapabilityStatus>;
    
    /// Check if a capability grants specific rights
    async fn check_capability_rights(&self, id: &CapabilityId, rights: &[Right]) -> Result<bool>;
    
    /// Delegate a capability from one entity to another
    async fn delegate_capability(&self, 
        from_id: &CapabilityId, 
        to_address: &Address,
        rights: &[Right],
        constraints: Vec<CapabilityConstraint>,
        delegatable: bool,
    ) -> Result<CapabilityId>;
    
    /// Revoke a capability
    async fn revoke_capability(&self, id: &CapabilityId) -> Result<()>;
    
    /// Get all capabilities for a resource
    async fn get_capabilities_for_resource(&self, resource_id: &ContentId) -> Result<Vec<RigorousCapability>>;
    
    /// Get all capabilities owned by an address
    async fn get_capabilities_for_owner(&self, owner: &Address) -> Result<Vec<RigorousCapability>>;
    
    /// Check if a capability is valid for a specific operation
    async fn can_perform_operation(&self, 
        id: &CapabilityId, 
        operation: &str,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<bool>;
    
    /// Consume a use of the capability (for max uses constraint)
    async fn consume_capability_use(&self, id: &CapabilityId) -> Result<()>;
}

/// Implementation of the capability system using the unified resource architecture
pub struct UnifiedCapabilitySystem {
    capabilities: RwLock<HashMap<CapabilityId, RigorousCapability>>,
    capability_uses: RwLock<HashMap<CapabilityId, u32>>,
    revocations: RwLock<HashSet<String>>,
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
    relationship_tracker: Arc<RelationshipTracker>,
}

impl UnifiedCapabilitySystem {
    /// Create a new unified capability system
    pub fn new(
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
    ) -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
            capability_uses: RwLock::new(HashMap::new()),
            revocations: RwLock::new(HashSet::new()),
            lifecycle_manager,
            relationship_tracker,
        }
    }
    
    /// Check if a capability is expired
    fn is_expired(&self, capability: &RigorousCapability) -> bool {
        if let Some(expires_at) = capability.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            return now > expires_at;
        }
        false
    }
    
    /// Check if a capability is revoked
    fn is_revoked(&self, capability: &RigorousCapability) -> bool {
        if let Some(revocation_id) = &capability.revocation_id {
            let revocations = self.revocations.read().unwrap();
            return revocations.contains(revocation_id);
        }
        false
    }
    
    /// Validate constraints on a capability
    fn validate_constraints(&self, 
        capability: &RigorousCapability,
        operation: Option<&str>,
        parameters: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<CapabilityStatus> {
        // Check each constraint
        for constraint in &capability.constraints {
            match constraint {
                CapabilityConstraint::MaxUses(max) => {
                    let uses = self.capability_uses.read().unwrap()
                        .get(&capability.id)
                        .copied()
                        .unwrap_or(0);
                    
                    if uses >= *max {
                        return Ok(CapabilityStatus::ConstraintViolated { 
                            constraint: format!("Maximum uses ({}) exceeded", max)
                        });
                    }
                },
                CapabilityConstraint::TimeWindow { start, end } => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    if now < *start || now > *end {
                        return Ok(CapabilityStatus::ConstraintViolated { 
                            constraint: format!("Time window ({}-{}) violated at {}", start, end, now)
                        });
                    }
                },
                CapabilityConstraint::Operations(allowed_ops) => {
                    if let Some(op) = operation {
                        if !allowed_ops.iter().any(|allowed| allowed == op) {
                            return Ok(CapabilityStatus::ConstraintViolated { 
                                constraint: format!("Operation '{}' not allowed", op)
                            });
                        }
                    }
                },
                CapabilityConstraint::Domains(allowed_domains) => {
                    if let Some(params) = parameters {
                        if let Some(domain) = params.get("domain").and_then(|v| v.as_str()) {
                            if !allowed_domains.iter().any(|allowed| allowed == domain) {
                                return Ok(CapabilityStatus::ConstraintViolated { 
                                    constraint: format!("Domain '{}' not allowed", domain)
                                });
                            }
                        }
                    }
                },
                CapabilityConstraint::MaxQuantity(max) => {
                    if let Some(params) = parameters {
                        if let Some(quantity) = params.get("quantity").and_then(|v| v.as_u64()) {
                            if quantity as u128 > *max {
                                return Ok(CapabilityStatus::ConstraintViolated { 
                                    constraint: format!("Quantity {} exceeds maximum {}", quantity, max)
                                });
                            }
                        }
                    }
                },
                // Other constraints would be checked here
                _ => { /* Skip other constraints for now */ }
            }
        }
        
        Ok(CapabilityStatus::Valid)
    }
}

#[async_trait]
impl CapabilitySystem for UnifiedCapabilitySystem {
    async fn create_capability(&self, capability: RigorousCapability) -> Result<CapabilityId> {
        let id = capability.id.clone();
        let mut capabilities = self.capabilities.write().unwrap();
        capabilities.insert(id.clone(), capability);
        Ok(id)
    }
    
    async fn get_capability(&self, id: &CapabilityId) -> Result<RigorousCapability> {
        let capabilities = self.capabilities.read().unwrap();
        capabilities.get(id)
            .cloned()
            .ok_or_else(|| Error::ResourceNotFound(format!("Capability not found: {}", id)))
    }
    
    async fn validate_capability(&self, id: &CapabilityId) -> Result<CapabilityStatus> {
        let capability = self.get_capability(id).await?;
        
        // Check if it's revoked
        if self.is_revoked(&capability) {
            return Ok(CapabilityStatus::Revoked { 
                revocation_id: capability.revocation_id.unwrap_or_default() 
            });
        }
        
        // Check if it's expired
        if self.is_expired(&capability) {
            return Ok(CapabilityStatus::Expired);
        }
        
        // Check for required proof
        if capability.proof.is_none() {
            return Ok(CapabilityStatus::MissingProof);
        }
        
        // Validate constraints (no specific operation here)
        self.validate_constraints(&capability, None, None)
    }
    
    async fn check_capability_rights(&self, id: &CapabilityId, rights: &[Right]) -> Result<bool> {
        // Get the capability
        let capability = self.get_capability(id).await?;
        
        // Check if the capability is valid
        let status = self.validate_capability(id).await?;
        if status != CapabilityStatus::Valid {
            return Ok(false);
        }
        
        // Check if it has all the required rights
        let has_all_rights = rights.iter().all(|right| capability.rights.contains(right));
        Ok(has_all_rights)
    }
    
    async fn delegate_capability(&self, 
        from_id: &CapabilityId, 
        to_address: &Address,
        rights: &[Right],
        constraints: Vec<CapabilityConstraint>,
        delegatable: bool,
    ) -> Result<CapabilityId> {
        // Get the parent capability
        let parent = self.get_capability(from_id).await?;
        
        // Verify the parent capability is valid
        let status = self.validate_capability(from_id).await?;
        if status != CapabilityStatus::Valid {
            return Err(Error::Unauthorized(format!(
                "Cannot delegate from invalid capability: {:?}", status
            )));
        }
        
        // Verify the parent is delegatable
        if !parent.delegatable {
            return Err(Error::Unauthorized(
                "Source capability is not delegatable".to_string()
            ));
        }
        
        // Verify all rights are present in the parent
        for right in rights {
            if !parent.rights.contains(right) {
                return Err(Error::Unauthorized(format!(
                    "Cannot delegate right {:?} that is not in parent capability", right
                )));
            }
        }
        
        // Create the new capability
        let child_id = CapabilityId::from(format!("{}-delegated-{}", from_id, to_address));
        let mut rights_set = HashSet::new();
        rights_set.extend(rights.iter().cloned());
        
        let child = RigorousCapability {
            id: child_id.clone(),
            resource_id: parent.resource_id.clone(),
            rights: rights_set,
            delegated_from: Some(from_id.clone()),
            issuer: parent.owner.clone(), // Current owner becomes the issuer
            owner: to_address.clone(),
            expires_at: parent.expires_at, // Inherit expiration
            revocation_id: Some(format!("rev-{}", child_id)),
            delegatable,
            constraints, // Use the provided constraints
            proof: None, // No proof yet
        };
        
        // Store the new capability
        self.create_capability(child).await
    }
    
    async fn revoke_capability(&self, id: &CapabilityId) -> Result<()> {
        // Get the capability to check if it exists
        let capability = self.get_capability(id).await?;
        
        // Add the revocation ID to the revocation set
        if let Some(revocation_id) = &capability.revocation_id {
            let mut revocations = self.revocations.write().unwrap();
            revocations.insert(revocation_id.clone());
        } else {
            // If no explicit revocation ID, use the capability ID
            let mut revocations = self.revocations.write().unwrap();
            revocations.insert(id.clone());
        }
        
        Ok(())
    }
    
    async fn get_capabilities_for_resource(&self, resource_id: &ContentId) -> Result<Vec<RigorousCapability>> {
        let capabilities = self.capabilities.read().unwrap();
        let mut result = Vec::new();
        
        for capability in capabilities.values() {
            if &capability.resource_id == resource_id {
                result.push(capability.clone());
            }
        }
        
        Ok(result)
    }
    
    async fn get_capabilities_for_owner(&self, owner: &Address) -> Result<Vec<RigorousCapability>> {
        let capabilities = self.capabilities.read().unwrap();
        let mut result = Vec::new();
        
        for capability in capabilities.values() {
            if &capability.owner == owner {
                result.push(capability.clone());
            }
        }
        
        Ok(result)
    }
    
    async fn can_perform_operation(&self, 
        id: &CapabilityId, 
        operation: &str,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Get the capability
        let capability = self.get_capability(id).await?;
        
        // Check if the capability is basically valid
        let status = self.validate_capability(id).await?;
        if status != CapabilityStatus::Valid {
            return Ok(false);
        }
        
        // Check constraints specifically for this operation
        let constraint_check = self.validate_constraints(
            &capability, 
            Some(operation), 
            Some(parameters)
        )?;
        
        // Return true only if constraints pass
        Ok(constraint_check == CapabilityStatus::Valid)
    }
    
    async fn consume_capability_use(&self, id: &CapabilityId) -> Result<()> {
        // First check if capability exists
        self.get_capability(id).await?;
        
        // Increment the usage count
        let mut capability_uses = self.capability_uses.write().unwrap();
        let count = capability_uses.entry(id.clone()).or_insert(0);
        *count += 1;
        
        Ok(())
    }
}

/// Validator that integrates with the resource lifecycle manager
pub struct CapabilityValidator {
    capability_system: Arc<dyn CapabilitySystem>,
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
}

impl CapabilityValidator {
    pub fn new(
        capability_system: Arc<dyn CapabilitySystem>,
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>
    ) -> Self {
        Self {
            capability_system,
            lifecycle_manager,
        }
    }
    
    pub async fn validate_operation(
        &self,
        address: &Address,
        resource_id: &ContentId,
        operation: &str,
        parameters: &HashMap<String, serde_json::Value>,
        required_rights: &[Right],
    ) -> Result<bool> {
        // Get the resource from the lifecycle manager to verify it exists
        let resource = self.lifecycle_manager.get_register(resource_id)?;
        
        // Find capabilities for this owner and resource
        let capabilities = self.capability_system.get_capabilities_for_resource(resource_id).await?;
        let owner_capabilities: Vec<RigorousCapability> = capabilities.into_iter()
            .filter(|cap| &cap.owner == address)
            .collect();
        
        // Check if any capability has the required rights and can perform this operation
        for cap in owner_capabilities {
            // Check if this capability has all required rights
            let has_rights = required_rights.iter().all(|right| cap.rights.contains(right));
            if !has_rights {
                continue;
            }
            
            // Validate for this specific operation
            if self.capability_system.can_perform_operation(
                &cap.id, 
                operation, 
                parameters
            ).await? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

/// Service that provides authorization services for resources
pub struct AuthorizationService {
    capability_system: Arc<dyn CapabilitySystem>,
}

impl AuthorizationService {
    /// Create a new authorization service
    pub fn new(capability_system: Arc<dyn CapabilitySystem>) -> Self {
        Self {
            capability_system,
        }
    }
    
    /// Check if an address is authorized to perform an action on a resource
    pub async fn check_authorization(
        &self,
        address: &Address,
        resource_id: &ContentId,
        required_rights: &[Right],
    ) -> Result<bool> {
        // Find capabilities for this owner and resource
        let capabilities = self.capability_system.get_capabilities_for_resource(resource_id).await?;
        let owner_capabilities: Vec<RigorousCapability> = capabilities.into_iter()
            .filter(|cap| &cap.owner == address)
            .collect();
        
        // Check if any capability has the required rights
        for cap in owner_capabilities {
            // Check capability status
            let status = self.capability_system.validate_capability(&cap.id).await?;
            if status != CapabilityStatus::Valid {
                continue;
            }
            
            // Check if this capability has all required rights
            let has_rights = required_rights.iter().all(|right| cap.rights.contains(right));
            if has_rights {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Authorize an operation on a resource
    pub async fn authorize_operation(
        &self,
        address: &Address,
        resource_id: &ContentId,
        operation: &str,
        parameters: &HashMap<String, serde_json::Value>,
        required_rights: &[Right],
    ) -> Result<bool> {
        // Find capabilities for this owner and resource
        let capabilities = self.capability_system.get_capabilities_for_resource(resource_id).await?;
        let owner_capabilities: Vec<RigorousCapability> = capabilities.into_iter()
            .filter(|cap| &cap.owner == address)
            .collect();
        
        // Check if any capability has the required rights and can perform this operation
        for cap in owner_capabilities {
            // Check if this capability has all required rights
            let has_rights = required_rights.iter().all(|right| cap.rights.contains(right));
            if !has_rights {
                continue;
            }
            
            // Validate for this specific operation
            if self.capability_system.can_perform_operation(
                &cap.id, 
                operation, 
                parameters
            ).await? {
                // Consume a use of the capability
                let _ = self.capability_system.consume_capability_use(&cap.id).await;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

/// Service that validates capabilities against resource lifecycle states
pub struct AuthorizationService {
    /// Reference to the lifecycle manager
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
}

impl AuthorizationService {
    /// Create a new authorization service
    pub fn new(lifecycle_manager: Arc<ResourceRegisterLifecycleManager>) -> Self {
        Self {
            lifecycle_manager,
        }
    }
    
    /// Check if an operation is allowed based on the resource state and provided capabilities
    pub fn check_operation_allowed(
        &self,
        resource_id: &ContentId,
        operation_type: RegisterOperationType,
        capability_ids: &[CapabilityId],
    ) -> Result<bool> {
        // Get the resource state
        let state = self.lifecycle_manager.get_state(resource_id)?;
        
        // Check if the resource state allows the operation
        if !Self::is_operation_allowed_for_state(state, operation_type) {
            return Ok(false);
        }
        
        // Check if the capabilities allow the operation
        // For now, we just check that at least one capability exists
        // In a more complete implementation, we would validate the specific rights
        // of each capability against the required rights for the operation
        if capability_ids.is_empty() {
            return Ok(false);
        }
        
        // In a real implementation, this would check each capability to ensure
        // it grants the right to perform the operation
        // For now, we just return true if the state allows it and capabilities exist
        Ok(true)
    }
    
    /// Determine if an operation is allowed for a given resource state
    fn is_operation_allowed_for_state(state: RegisterState, operation: RegisterOperationType) -> bool {
        match (state, operation) {
            // Initial state: allows registration and activation
            (RegisterState::Initial, RegisterOperationType::Register) => true,
            (RegisterState::Initial, RegisterOperationType::Activate) => true,
            (RegisterState::Initial, _) => false,
            
            // Active state: allows most operations except registration (already registered)
            (RegisterState::Active, RegisterOperationType::Register) => false,
            (RegisterState::Active, RegisterOperationType::Activate) => false, // Already active
            (RegisterState::Active, _) => true,
            
            // Locked state: only allows unlocking and reading
            (RegisterState::Locked, RegisterOperationType::Unlock) => true,
            (RegisterState::Locked, RegisterOperationType::Read) => true,
            (RegisterState::Locked, _) => false,
            
            // Frozen state: only allows unfreezing and reading
            (RegisterState::Frozen, RegisterOperationType::Unfreeze) => true,
            (RegisterState::Frozen, RegisterOperationType::Read) => true,
            (RegisterState::Frozen, _) => false,
            
            // Consumed state: allows no operations (resource is consumed)
            (RegisterState::Consumed, _) => false,
            
            // Pending state: allows activation, cancellation and reading
            (RegisterState::Pending, RegisterOperationType::Activate) => true,
            (RegisterState::Pending, RegisterOperationType::Cancel) => true,
            (RegisterState::Pending, RegisterOperationType::Read) => true,
            (RegisterState::Pending, _) => false,
        }
    }
    
    /// Check if a capability grants a specific right for a resource
    pub fn does_capability_grant_right(
        &self,
        capability: &Capability,
        resource_id: &ContentId,
        right: Right,
    ) -> bool {
        // Check if the capability is for this resource
        if capability.resource_id != *resource_id {
            return false;
        }
        
        // Check the capability type
        match capability.capability_type {
            CapabilityType::Read => matches!(right, Right::Read),
            CapabilityType::Write => matches!(right, Right::Write | Right::Update),
            CapabilityType::Execute => matches!(right, Right::Execute),
            CapabilityType::Delegate => matches!(right, Right::Delegate),
            CapabilityType::Admin => true, // Admin can do anything
            CapabilityType::Custom(ref rights) => rights.contains(&right),
        }
    }
    
    /// Validate if a set of capabilities allows a specific operation
    pub fn validate_capabilities_for_operation(
        &self,
        capabilities: &[Capability],
        resource_id: &ContentId,
        operation_type: RegisterOperationType,
    ) -> Result<bool> {
        // Get the resource state
        let state = self.lifecycle_manager.get_state(resource_id)?;
        
        // Check if the operation is allowed for this state
        if !Self::is_operation_allowed_for_state(state, operation_type) {
            return Ok(false);
        }
        
        // Determine what rights are needed for this operation
        let required_rights = Self::rights_for_operation(operation_type);
        
        // Check if any capability grants the required rights
        for capability in capabilities {
            let mut has_all_rights = true;
            for right in &required_rights {
                if !self.does_capability_grant_right(capability, resource_id, *right) {
                    has_all_rights = false;
                    break;
                }
            }
            
            if has_all_rights {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Determine what rights are required for an operation
    fn rights_for_operation(operation: RegisterOperationType) -> Vec<Right> {
        match operation {
            RegisterOperationType::Register => vec![Right::Create],
            RegisterOperationType::Activate => vec![Right::Update],
            RegisterOperationType::Deactivate => vec![Right::Update],
            RegisterOperationType::Lock => vec![Right::Update],
            RegisterOperationType::Unlock => vec![Right::Update],
            RegisterOperationType::Freeze => vec![Right::Update],
            RegisterOperationType::Unfreeze => vec![Right::Update],
            RegisterOperationType::Consume => vec![Right::Delete],
            RegisterOperationType::Update => vec![Right::Update],
            RegisterOperationType::Read => vec![Right::Read],
            RegisterOperationType::Cancel => vec![Right::Update],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operation_allowed_for_state() {
        // Test initial state
        assert!(AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Initial, 
            RegisterOperationType::Register
        ));
        
        assert!(AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Initial, 
            RegisterOperationType::Activate
        ));
        
        assert!(!AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Initial, 
            RegisterOperationType::Update
        ));
        
        // Test active state
        assert!(AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Active, 
            RegisterOperationType::Update
        ));
        
        assert!(AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Active, 
            RegisterOperationType::Lock
        ));
        
        assert!(!AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Active, 
            RegisterOperationType::Register
        ));
        
        // Test locked state
        assert!(AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Locked, 
            RegisterOperationType::Unlock
        ));
        
        assert!(AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Locked, 
            RegisterOperationType::Read
        ));
        
        assert!(!AuthorizationService::is_operation_allowed_for_state(
            RegisterState::Locked, 
            RegisterOperationType::Update
        ));
    }
} 
