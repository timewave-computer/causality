use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::resource::{
    Register, RegisterId, RegisterOperation, 
    ResourceManager, TransitionSystem
};
use crate::address::{Address, AddressGenerator};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::crypto::Signature;

/// Errors that can occur when working with capabilities
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("Capability has expired")]
    Expired,
    
    #[error("Capability has been revoked")]
    Revoked,
    
    #[error("Invalid capability signature")]
    InvalidSignature,
    
    #[error("Operation not permitted by capability rights")]
    OperationNotPermitted,
    
    #[error("Resource type mismatch: expected {expected}, got {actual}")]
    ResourceTypeMismatch { expected: String, actual: String },
    
    #[error("Restriction violated: {0}")]
    RestrictionViolated(String),
    
    #[error("Invalid delegated capability: {0}")]
    InvalidDelegation(String),
    
    #[error("Missing required capability parameter: {0}")]
    MissingParameter(String),
    
    #[error("Capability validation failed: {0}")]
    ValidationFailed(String),
}

/// Result type for capability operations
pub type CapabilityResult<T> = Result<T, CapabilityError>;

/// Rights that can be granted by a capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Right {
    /// Right to read the resource
    Read,
    
    /// Right to write to the resource
    Write,
    
    /// Right to delete the resource
    Delete,
    
    /// Right to transfer ownership of the resource
    Transfer,
    
    /// Right to delegate capabilities on the resource
    Delegate,
    
    /// Right to execute a specific operation on the resource
    Execute(String),
    
    /// Custom right with a named identifier
    Custom(String),
}

/// Restrictions that can be applied to a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restrictions {
    /// Map of restriction names to their values
    restrictions: HashMap<String, String>,
}

impl Restrictions {
    /// Create a new empty set of restrictions
    pub fn new() -> Self {
        Self {
            restrictions: HashMap::new(),
        }
    }
    
    /// Add a restriction with a value
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.restrictions.insert(name.into(), value.into());
        self
    }
    
    /// Get the value of a restriction
    pub fn get(&self, name: &str) -> Option<&str> {
        self.restrictions.get(name).map(|s| s.as_str())
    }
    
    /// Check if a restriction exists
    pub fn has(&self, name: &str) -> bool {
        self.restrictions.contains_key(name)
    }
    
    /// Check if this restriction set contains all restrictions in another set
    pub fn contains_all(&self, other: &Restrictions) -> bool {
        other.restrictions.iter().all(|(k, v)| {
            self.restrictions.get(k).map_or(false, |my_v| my_v == v)
        })
    }
    
    /// Check if this restriction set is more restrictive than another
    pub fn is_more_restrictive_than(&self, other: &Restrictions) -> bool {
        // Has all the other restrictions
        if !self.contains_all(other) {
            return false;
        }
        
        // And possibly additional restrictions or more restrictive values
        self.restrictions.len() >= other.restrictions.len()
    }
    
    /// Check if all restrictions are satisfied by the provided values
    pub fn are_satisfied_by(&self, values: &HashMap<String, String>) -> CapabilityResult<()> {
        for (name, constraint) in &self.restrictions {
            if let Some(value) = values.get(name) {
                // Simple string equality check for now
                if value != constraint {
                    return Err(CapabilityError::RestrictionViolated(format!(
                        "Restriction {} violated: expected {}, got {}",
                        name, constraint, value
                    )));
                }
            } else {
                return Err(CapabilityError::MissingParameter(name.clone()));
            }
        }
        
        Ok(())
    }
}

impl Default for Restrictions {
    fn default() -> Self {
        Self::new()
    }
}

/// Identifier for a capability
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CapabilityId(Uuid);

impl CapabilityId {
    /// Create a new random capability ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create a deterministic capability ID from resource and rights
    pub fn from_resource(
        resource_id: &str,
        resource_type: &str,
        rights: &[Right],
    ) -> Self {
        let mut buf = Vec::new();
        buf.extend_from_slice(resource_id.as_bytes());
        buf.extend_from_slice(b"::");
        buf.extend_from_slice(resource_type.as_bytes());
        buf.extend_from_slice(b"::");
        
        // Sort rights for deterministic ordering
        let mut rights_str: Vec<String> = rights.iter()
            .map(|r| format!("{:?}", r))
            .collect();
        rights_str.sort();
        
        for right in rights_str {
            buf.extend_from_slice(right.as_bytes());
            buf.extend_from_slice(b",");
        }
        
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &buf);
        Self(uuid)
    }
}

impl fmt::Debug for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CapabilityId({})", self.0)
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A capability that grants specific rights to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapability {
    /// Unique identifier for this capability
    id: CapabilityId,
    
    /// ID of the resource this capability grants access to
    resource_id: String,
    
    /// Type of the resource
    resource_type: String,
    
    /// The address of the issuer of this capability
    issuer: Address,
    
    /// The address of the holder of this capability
    holder: Address,
    
    /// Rights granted by this capability
    rights: Vec<Right>,
    
    /// Restrictions on the use of this capability
    restrictions: Restrictions,
    
    /// Optional expiration time for this capability
    expires_at: Option<SystemTime>,
    
    /// Optional signature by the issuer
    signature: Option<Signature>,
    
    /// Optional parent capability ID if this was delegated
    parent_id: Option<CapabilityId>,
    
    /// Revocation status
    revoked: bool,
}

impl ResourceCapability {
    /// Create a new capability
    pub fn new(
        resource_id: impl Into<String>,
        resource_type: impl Into<String>,
        issuer: Address,
        holder: Address,
        rights: Vec<Right>,
    ) -> Self {
        let resource_id = resource_id.into();
        let resource_type = resource_type.into();
        
        Self {
            id: CapabilityId::from_resource(&resource_id, &resource_type, &rights),
            resource_id,
            resource_type,
            issuer,
            holder,
            rights,
            restrictions: Restrictions::new(),
            expires_at: None,
            signature: None,
            parent_id: None,
            revoked: false,
        }
    }
    
    /// Get the capability ID
    pub fn id(&self) -> &CapabilityId {
        &self.id
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }
    
    /// Get the resource type
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    /// Get the issuer
    pub fn issuer(&self) -> &Address {
        &self.issuer
    }
    
    /// Get the holder
    pub fn holder(&self) -> &Address {
        &self.holder
    }
    
    /// Get the rights
    pub fn rights(&self) -> &[Right] {
        &self.rights
    }
    
    /// Check if the capability grants a specific right
    pub fn has_right(&self, right: &Right) -> bool {
        self.rights.contains(right)
    }
    
    /// Get the restrictions
    pub fn restrictions(&self) -> &Restrictions {
        &self.restrictions
    }
    
    /// Add a restriction to this capability
    pub fn add_restriction(
        &mut self, 
        name: impl Into<String>, 
        value: impl Into<String>
    ) -> &mut Self {
        self.restrictions.add(name, value);
        self
    }
    
    /// Set expiration time for this capability
    pub fn with_expiration(&mut self, expires_at: SystemTime) -> &mut Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Set expiration to a duration from now
    pub fn expires_in(&mut self, duration: Duration) -> &mut Self {
        let expires_at = SystemTime::now() + duration;
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Check if the capability has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expires_at {
            return SystemTime::now() > expiry;
        }
        false
    }
    
    /// Check if the capability has been revoked
    pub fn is_revoked(&self) -> bool {
        self.revoked
    }
    
    /// Revoke this capability
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
    
    /// Sign this capability with the issuer's private key (implementation placeholder)
    pub fn sign(&mut self, _private_key: &[u8]) -> &mut Self {
        // In a real implementation, this would create a signature using the private key
        // For now, we just set a placeholder signature
        self.signature = Some(Signature::new_random());
        self
    }
    
    /// Verify this capability's signature (implementation placeholder)
    pub fn verify_signature(&self) -> CapabilityResult<()> {
        // In a real implementation, this would verify the signature
        // using the issuer's public key
        if self.signature.is_none() {
            return Err(CapabilityError::InvalidSignature);
        }
        
        // Assume signature is valid for now
        Ok(())
    }
    
    /// Validate that this capability is valid for use
    pub fn validate(&self) -> CapabilityResult<()> {
        // Check if revoked
        if self.is_revoked() {
            return Err(CapabilityError::Revoked);
        }
        
        // Check if expired
        if self.is_expired() {
            return Err(CapabilityError::Expired);
        }
        
        // Verify signature
        self.verify_signature()?;
        
        Ok(())
    }
    
    /// Delegate a subset of this capability to another holder
    pub fn delegate(
        &self,
        new_holder: Address,
        rights: Vec<Right>,
        restrictions: Option<Restrictions>,
    ) -> CapabilityResult<Self> {
        // Validate this capability first
        self.validate()?;
        
        // Check if we have delegation right
        if !self.has_right(&Right::Delegate) {
            return Err(CapabilityError::OperationNotPermitted);
        }
        
        // Ensure all delegated rights are owned by this capability
        for right in &rights {
            if !self.has_right(right) {
                return Err(CapabilityError::OperationNotPermitted);
            }
        }
        
        // Create new capability
        let mut delegated = Self {
            id: CapabilityId::new(),
            resource_id: self.resource_id.clone(),
            resource_type: self.resource_type.clone(),
            issuer: self.holder.clone(), // Current holder becomes issuer
            holder: new_holder,
            rights,
            restrictions: restrictions.unwrap_or_else(|| self.restrictions.clone()),
            expires_at: self.expires_at,
            signature: None,
            parent_id: Some(self.id.clone()),
            revoked: false,
        };
        
        // Ensure delegated capability has at least as restrictive constraints
        if !delegated.restrictions.is_more_restrictive_than(&self.restrictions) {
            return Err(CapabilityError::InvalidDelegation(
                "Delegated capability must be at least as restrictive as parent".into()
            ));
        }
        
        Ok(delegated)
    }
    
    /// Attenuate this capability by reducing its rights and/or adding restrictions
    pub fn attenuate(
        &self,
        rights: Vec<Right>,
        additional_restrictions: Option<Restrictions>,
    ) -> CapabilityResult<Self> {
        // Validate this capability first
        self.validate()?;
        
        // Ensure all attenuated rights are owned by this capability
        for right in &rights {
            if !self.has_right(right) {
                return Err(CapabilityError::OperationNotPermitted);
            }
        }
        
        // Create new capability with same holder but reduced rights
        let mut attenuated = Self {
            id: CapabilityId::new(),
            resource_id: self.resource_id.clone(),
            resource_type: self.resource_type.clone(),
            issuer: self.issuer.clone(),
            holder: self.holder.clone(),
            rights,
            restrictions: self.restrictions.clone(),
            expires_at: self.expires_at,
            signature: None,
            parent_id: Some(self.id.clone()),
            revoked: false,
        };
        
        // Add additional restrictions if provided
        if let Some(additional) = additional_restrictions {
            for (k, v) in additional.restrictions {
                attenuated.restrictions.add(k, v);
            }
        }
        
        Ok(attenuated)
    }
}

/// A reference to a capability that can be passed around safely
#[derive(Debug, Clone)]
pub struct CapabilityRef {
    /// The capability ID
    id: CapabilityId,
    
    /// The capability itself, wrapped in an Arc for shared ownership
    capability: Arc<ResourceCapability>,
}

impl CapabilityRef {
    /// Create a new capability reference
    pub fn new(capability: ResourceCapability) -> Self {
        let id = capability.id().clone();
        Self {
            id,
            capability: Arc::new(capability),
        }
    }
    
    /// Get the capability ID
    pub fn id(&self) -> &CapabilityId {
        &self.id
    }
    
    /// Get a reference to the capability
    pub fn capability(&self) -> &ResourceCapability {
        &self.capability
    }
    
    /// Create a capability reference from an existing Arc<ResourceCapability>
    pub fn from_arc(capability: Arc<ResourceCapability>) -> Self {
        let id = capability.id().clone();
        Self { id, capability }
    }
}

/// Repository for tracking and validating capabilities
#[derive(Debug, Default)]
pub struct CapabilityRepository {
    /// Map of capability IDs to capabilities
    capabilities: HashMap<CapabilityId, CapabilityRef>,
    
    /// Map of resource IDs to capabilities that target them
    resource_capabilities: HashMap<String, Vec<CapabilityId>>,
    
    /// Map of holder addresses to capabilities they hold
    holder_capabilities: HashMap<Address, Vec<CapabilityId>>,
    
    /// Map of revoked capability IDs
    revoked_capabilities: Vec<CapabilityId>,
}

impl CapabilityRepository {
    /// Create a new empty capability repository
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            resource_capabilities: HashMap::new(),
            holder_capabilities: HashMap::new(),
            revoked_capabilities: Vec::new(),
        }
    }
    
    /// Register a new capability
    pub fn register(&mut self, capability: ResourceCapability) -> CapabilityRef {
        let capability_ref = CapabilityRef::new(capability);
        let cap = capability_ref.capability();
        let id = cap.id().clone();
        
        // Add to main map
        self.capabilities.insert(id.clone(), capability_ref.clone());
        
        // Add to resource map
        self.resource_capabilities
            .entry(cap.resource_id().to_string())
            .or_default()
            .push(id.clone());
        
        // Add to holder map
        self.holder_capabilities
            .entry(cap.holder().clone())
            .or_default()
            .push(id.clone());
        
        capability_ref
    }
    
    /// Get a capability by ID
    pub fn get(&self, id: &CapabilityId) -> Option<CapabilityRef> {
        self.capabilities.get(id).cloned()
    }
    
    /// Get all capabilities for a resource
    pub fn get_for_resource(&self, resource_id: &str) -> Vec<CapabilityRef> {
        if let Some(ids) = self.resource_capabilities.get(resource_id) {
            ids.iter()
                .filter_map(|id| self.capabilities.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all capabilities held by an address
    pub fn get_for_holder(&self, holder: &Address) -> Vec<CapabilityRef> {
        if let Some(ids) = self.holder_capabilities.get(holder) {
            ids.iter()
                .filter_map(|id| self.capabilities.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Revoke a capability
    pub fn revoke(&mut self, id: &CapabilityId) -> CapabilityResult<()> {
        if let Some(capability_ref) = self.capabilities.get_mut(id) {
            // We need to get a mutable reference to the actual capability
            // This is a limitation of the current design - Arc prevents mutation
            // In a real implementation, we would track revocation separately
            
            // Instead, we'll just add it to our revoked list
            self.revoked_capabilities.push(id.clone());
            
            Ok(())
        } else {
            Err(CapabilityError::ValidationFailed("Capability not found".into()))
        }
    }
    
    /// Check if a capability is revoked
    pub fn is_revoked(&self, id: &CapabilityId) -> bool {
        self.revoked_capabilities.contains(id)
    }
    
    /// Validate a capability
    pub fn validate(&self, id: &CapabilityId) -> CapabilityResult<CapabilityRef> {
        // Check if it exists
        let capability_ref = self.get(id).ok_or_else(|| {
            CapabilityError::ValidationFailed("Capability not found".into())
        })?;
        
        // Check if it's revoked in our repository
        if self.is_revoked(id) {
            return Err(CapabilityError::Revoked);
        }
        
        // Run the capability's own validation
        capability_ref.capability().validate()?;
        
        Ok(capability_ref)
    }
} 