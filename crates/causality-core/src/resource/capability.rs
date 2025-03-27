// Resource capability system
//
// This file defines the capability-based access control system for resources.

use std::collections::HashMap;
use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use super::types::{ResourceId, ResourceType};
use super::interface::ResourceOperation;

/// Resource capability
///
/// A capability represents the ability to perform certain operations on a resource.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceCapability {
    /// Target resource ID or pattern
    pub target: CapabilityTarget,
    
    /// Allowed operations
    pub operations: Vec<ResourceOperation>,
    
    /// Delegation allowed
    pub delegable: bool,
    
    /// Expiration time (None means never expires)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    
    /// Capability constraints
    #[serde(default)]
    pub constraints: Vec<CapabilityConstraint>,
    
    /// Capability attestation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<CapabilityAttestation>,
}

impl ResourceCapability {
    /// Create a new resource capability
    pub fn new(
        target: CapabilityTarget,
        operations: Vec<ResourceOperation>,
        delegable: bool,
    ) -> Self {
        Self {
            target,
            operations,
            delegable,
            expires_at: None,
            constraints: Vec::new(),
            attestation: None,
        }
    }
    
    /// Add an expiration time to this capability
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add a constraint to this capability
    pub fn with_constraint(mut self, constraint: CapabilityConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Set the attestation for this capability
    pub fn with_attestation(mut self, attestation: CapabilityAttestation) -> Self {
        self.attestation = Some(attestation);
        self
    }
    
    /// Check if this capability allows a specific operation on a resource
    pub fn allows(
        &self,
        resource_id: &ResourceId,
        resource_type: Option<&ResourceType>,
        operation: &ResourceOperation,
        context: Option<&HashMap<String, serde_json::Value>>,
        timestamp: u64,
    ) -> bool {
        // Check if the capability matches the target
        if !self.target.matches(resource_id, resource_type) {
            return false;
        }
        
        // Check if the operation is allowed
        if !self.operations.contains(operation) && !self.operations.contains(&ResourceOperation::All) {
            return false;
        }
        
        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if timestamp >= expires_at {
                return false;
            }
        }
        
        // Check constraints
        if !self.constraints.is_empty() {
            if let Some(context) = context {
                for constraint in &self.constraints {
                    if !constraint.evaluate(context) {
                        return false;
                    }
                }
            } else {
                // If context is required but not provided, deny
                return false;
            }
        }
        
        true
    }
    
    /// Check if this capability is valid (has attestation if required)
    pub fn is_valid(&self) -> bool {
        // If attestation is required but not present, it's invalid
        // In a real system, we would also verify the attestation signature
        self.attestation.is_some()
    }
}

/// Capability target
///
/// Defines what resources this capability applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum CapabilityTarget {
    /// Specific resource
    Specific(ResourceId),
    
    /// Resources of a specific type
    OfType(ResourceType),
    
    /// Resources matching a pattern
    Pattern(String),
    
    /// All resources
    All,
}

impl CapabilityTarget {
    /// Check if this target matches a specific resource
    pub fn matches(&self, resource_id: &ResourceId, resource_type: Option<&ResourceType>) -> bool {
        match self {
            CapabilityTarget::Specific(id) => resource_id == id,
            CapabilityTarget::OfType(target_type) => {
                if let Some(rt) = resource_type {
                    rt.is_compatible_with(target_type)
                } else {
                    false
                }
            },
            CapabilityTarget::Pattern(pattern) => {
                // Simple pattern matching for now
                if let Some(name) = resource_id.name() {
                    name.contains(pattern)
                } else {
                    resource_id.hash.to_string().contains(pattern)
                }
            },
            CapabilityTarget::All => true,
        }
    }
}

impl Display for CapabilityTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityTarget::Specific(id) => write!(f, "specific:{}", id),
            CapabilityTarget::OfType(resource_type) => write!(f, "type:{}", resource_type),
            CapabilityTarget::Pattern(pattern) => write!(f, "pattern:{}", pattern),
            CapabilityTarget::All => write!(f, "all"),
        }
    }
}

/// Capability constraint
///
/// A constraint that must be satisfied for a capability to be valid.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum CapabilityConstraint {
    /// Requires a specific attribute to be present
    RequireAttribute { key: String },
    
    /// Requires a specific attribute to have a specific value
    RequireAttributeValue { key: String, value: serde_json::Value },
    
    /// Requires a timestamp to be before a specific time
    RequireBefore { timestamp: u64 },
    
    /// Requires a timestamp to be after a specific time
    RequireAfter { timestamp: u64 },
    
    /// Requires all constraints to be satisfied
    And(Vec<CapabilityConstraint>),
    
    /// Requires at least one constraint to be satisfied
    Or(Vec<CapabilityConstraint>),
}

impl CapabilityConstraint {
    /// Evaluate this constraint with the given context
    pub fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> bool {
        match self {
            CapabilityConstraint::RequireAttribute { key } => {
                context.contains_key(key)
            },
            CapabilityConstraint::RequireAttributeValue { key, value } => {
                if let Some(v) = context.get(key) {
                    v == value
                } else {
                    false
                }
            },
            CapabilityConstraint::RequireBefore { timestamp } => {
                if let Some(ts) = context.get("timestamp") {
                    if let Some(ts) = ts.as_u64() {
                        ts < *timestamp
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CapabilityConstraint::RequireAfter { timestamp } => {
                if let Some(ts) = context.get("timestamp") {
                    if let Some(ts) = ts.as_u64() {
                        ts > *timestamp
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CapabilityConstraint::And(constraints) => {
                constraints.iter().all(|c| c.evaluate(context))
            },
            CapabilityConstraint::Or(constraints) => {
                constraints.iter().any(|c| c.evaluate(context))
            },
        }
    }
}

/// Capability attestation
///
/// An attestation that proves the validity of a capability.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CapabilityAttestation {
    /// Principal who issued this capability
    pub issuer: String,
    
    /// Signature by the issuer
    pub signature: Vec<u8>,
    
    /// Timestamp of issuance
    pub issued_at: u64,
    
    /// Additional attestation metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CapabilityAttestation {
    /// Create a new capability attestation
    pub fn new(
        issuer: impl Into<String>,
        signature: Vec<u8>,
        issued_at: u64,
    ) -> Self {
        Self {
            issuer: issuer.into(),
            signature,
            issued_at,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to this attestation
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Capability set
///
/// A collection of capabilities for a principal.
#[derive(Debug, Clone, Default, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CapabilitySet {
    /// Capabilities in this set
    pub capabilities: Vec<ResourceCapability>,
}

impl CapabilitySet {
    /// Create a new empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }
    
    /// Create a new capability set with initial capabilities
    pub fn with_capabilities(capabilities: Vec<ResourceCapability>) -> Self {
        Self { capabilities }
    }
    
    /// Add a capability to this set
    pub fn add(&mut self, capability: ResourceCapability) {
        self.capabilities.push(capability);
    }
    
    /// Check if this set allows an operation on a resource
    pub fn allows(
        &self,
        resource_id: &ResourceId,
        resource_type: Option<&ResourceType>,
        operation: &ResourceOperation,
        context: Option<&HashMap<String, serde_json::Value>>,
        timestamp: u64,
    ) -> bool {
        self.capabilities.iter().any(|cap| {
            cap.allows(resource_id, resource_type, operation, context, timestamp)
        })
    }
    
    /// Get all capabilities for a specific resource
    pub fn for_resource(&self, resource_id: &ResourceId) -> Vec<&ResourceCapability> {
        self.capabilities.iter().filter(|cap| {
            matches!(cap.target, CapabilityTarget::Specific(ref id) if id == resource_id)
                || matches!(cap.target, CapabilityTarget::All)
        }).collect()
    }
} 