// authorization.rs - Authorization system for agent capability verification
//
// This file implements the authorization system for verifying that agents have
// the necessary capabilities to perform operations.

use crate::resource::types::{ResourceId, ResourceType};
use crate::utils::content_addressing;
use crate::resource::Resource;
use crate::resource::operation::Capability;
use causality_types::ContentHash;
use crate::effect::Effect;
use crate::serialization::Serializable;
use anyhow;

use super::types::{AgentId, AgentError};
use super::operation::{Operation, OperationId, OperationError};
use super::agent::Agent;

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use crate::capability::IdentityId;

/// Constraint on an authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConstraint {
    /// Constraint type
    pub constraint_type: String,
    
    /// Constraint parameters
    pub parameters: HashMap<String, String>,
    
    /// Whether this constraint is required
    pub required: bool,
}

// A serializable representation of a capability for content addressing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableCapability {
    id: String,
    grants: Vec<String>,
    origin: Option<String>,
    content_hash: Option<String>,
}

/// A struct to contain wrapped capability information that can be cloned
/// without requiring the dyn Resource to implement Clone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// The resource identifier
    pub id: ResourceId,
    /// The capability grants
    pub grants: Vec<String>,
    /// The identity id of origin
    pub identity_id: String,
    /// The content hash
    pub content_hash: Option<ContentHash>,
}

impl CapabilityInfo {
    pub fn from_capability(cap: &Capability<Box<dyn Resource>>) -> Self {
        // Create a serializable representation for content addressing
        let serializable = SerializableCapability {
            id: cap.id.clone(),
            grants: cap.grants.clone(),
            origin: cap.origin.clone(),
            content_hash: cap.content_hash.clone(),
        };
        
        CapabilityInfo {
            id: ResourceId::new(content_addressing::hash_string(&cap.id)),
            grants: cap.grants.clone(),
            identity_id: cap.origin.clone().unwrap_or_default(),
            content_hash: Some(content_addressing::hash_object(&serializable).unwrap()),
        }
    }
    
    pub fn content_hash(&self) -> &ContentHash {
        self.content_hash.as_ref().unwrap()
    }
}

/// Authorization for an agent operation
#[derive(Debug, Clone)]
pub struct Authorization {
    /// The agent ID that is authorizing
    pub agent_id: AgentId,
    
    /// The operation being authorized
    pub operation_id: OperationId,
    
    /// The capabilities being granted for this operation (using the cloneable wrapper)
    pub capabilities: Vec<CapabilityInfo>,
    
    /// Any constraints on this authorization
    pub constraints: Vec<AuthorizationConstraint>,
    
    /// Expiration time (if any)
    pub expires_at: Option<u64>,
    
    /// Metadata for this authorization
    pub metadata: HashMap<String, String>,
}

/// Proposed authorization
#[derive(Debug, Clone)]
pub struct AuthorizationProposal {
    /// The agent ID that is proposing authorization
    pub agent_id: AgentId,
    
    /// The operation being authorized
    pub operation_id: OperationId,
    
    /// The capabilities being proposed for this operation (using the cloneable wrapper)
    pub capabilities: Vec<CapabilityInfo>,
    
    /// Any constraints on this authorization
    pub constraints: Vec<AuthorizationConstraint>,
    
    /// Expiration time (if any)
    pub expires_at: Option<u64>,
    
    /// Metadata for this proposal
    pub metadata: HashMap<String, String>,
}

impl Authorization {
    /// Create a new authorization
    pub fn new(
        agent_id: AgentId,
        operation_id: OperationId,
        capabilities: Vec<Capability<Box<dyn Resource>>>,
        constraints: Vec<AuthorizationConstraint>,
        expires_at: Option<u64>,
        metadata: HashMap<String, String>,
    ) -> Result<Self, AuthorizationError> {
        // Convert the capabilities to CapabilityInfo objects
        let capability_infos = capabilities.iter().map(|cap| {
            // Create a serializable representation for content addressing
            let serializable = SerializableCapability {
                id: cap.id.clone(),
                grants: cap.grants.clone(),
                origin: cap.origin.clone(),
                content_hash: cap.content_hash.clone(),
            };
            
            CapabilityInfo {
                id: ResourceId::new(content_addressing::hash_string(&cap.id)),
                grants: cap.grants.clone(),
                identity_id: cap.origin.clone().unwrap_or_default(),
                content_hash: Some(content_addressing::hash_object(&serializable).unwrap()),
            }
        }).collect();
        
        Ok(Self {
            agent_id,
            operation_id,
            capabilities: capability_infos,
            constraints,
            expires_at,
            metadata,
        })
    }
    
    /// Get the agent ID
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    
    /// Get the operation ID
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    
    /// Get the capabilities
    pub fn capabilities(&self) -> &[CapabilityInfo] {
        &self.capabilities
    }
    
    /// Get the constraints
    pub fn constraints(&self) -> &[AuthorizationConstraint] {
        &self.constraints
    }
    
    /// Get the expires_at
    pub fn expires_at(&self) -> Option<u64> {
        self.expires_at
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Compute the content hash of this authorization
    fn compute_content_hash(&self) -> Result<ContentHash, AuthorizationError> {
        // Converting to a serializable view
        let view = AuthorizationContentView {
            agent_id: self.agent_id.clone(),
            operation_id: self.operation_id.clone(),
            capabilities: self.capabilities.clone(),
            constraints: self.constraints.clone(),
            expires_at: self.expires_at,
            metadata: self.metadata.clone(),
        };
        
        view.content_hash().map_err(|e| {
            AuthorizationError::SerializationError(e.to_string())
        })
    }
    
    /// Get the content hash
    pub fn content_hash(&self) -> Result<ContentHash, AuthorizationError> {
        self.compute_content_hash()
    }
    
    /// Verify this authorization
    pub fn verify(&self) -> Result<bool, AuthorizationError> {
        // In a real implementation, verify the signature or other proof mechanism
        // For now, just check that the proof is not empty
        Ok(!self.constraints.is_empty())
    }

    pub fn authorized_for(&self, _capability: &Capability<Box<dyn Resource>>, resource_id: &ResourceId) -> bool {
        self.capabilities.iter().any(|c| c.id == *resource_id)
    }
}

/// View of authorization data for content addressing
#[derive(Serialize, Deserialize)]
struct AuthorizationContentView {
    agent_id: AgentId,
    operation_id: OperationId,
    capabilities: Vec<CapabilityInfo>,
    constraints: Vec<AuthorizationConstraint>,
    expires_at: Option<u64>,
    metadata: HashMap<String, String>,
}

// Custom implementation for content addressing
impl AuthorizationContentView {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        content_addressing::hash_object(self)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

/// Authorization error types
#[derive(Error, Debug)]
pub enum AuthorizationError {
    /// Missing required capability
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    /// Unauthorized operation
    #[error("Unauthorized operation: {0}")]
    UnauthorizedOperation(String),
    
    /// Invalid authorization
    #[error("Invalid authorization: {0}")]
    InvalidAuthorization(String),
    
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Operation error
    #[error("Operation error: {0}")]
    OperationError(#[from] OperationError),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Other error
    #[error("Authorization error: {0}")]
    Other(String),
}

/// Capability verifier for operations
pub struct CapabilityVerifier {
    registry: Arc<CapabilityRegistry>,
}

impl CapabilityVerifier {
    /// Create a new capability verifier
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self { registry }
    }
    
    /// Verify that an agent has the capabilities required for an operation
    pub async fn verify_capabilities<A: Agent + Send + Sync>(
        &self,
        agent: &A,
        operation: &Operation,
    ) -> Result<Authorization, AuthorizationError> {
        // Get the required capabilities for the operation
        let required_capabilities = operation.required_capabilities();
        
        // Create a vector to store the granted capabilities
        let mut granted_capabilities = Vec::new();
        
        // Check if the agent has all required capabilities
        for cap_id in required_capabilities {
            if let Some(cap_def) = agent.capabilities().iter().find(|c| c.id == cap_id) {
                // Create a capability for this operation
                let capability = Capability::new(
                    String::from(&cap_id as &str),
                    cap_def.grants.clone(),
                    cap_def.origin.clone(),
                    cap_def.content_hash.clone(),
                );
                granted_capabilities.push(capability);
            } else {
                return Err(AuthorizationError::MissingCapability(String::from(&cap_id as &str)));
            }
        }
        
        // Validate capabilities against the registry
        let target_id = match operation.target_resource_id() {
            Some(id) => id,
            None => return Err(AuthorizationError::Other("Operation has no target resource".to_string())),
        };
        
        let valid_capabilities = self.registry.validate_capabilities(
            granted_capabilities,
            target_id,
        )?;
        
        // Create operation ID, handling potential errors
        let op_id = match operation.id() {
            Ok(id) => id,
            Err(e) => return Err(AuthorizationError::SerializationError(e.to_string())),
        };
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("verifier".to_string(), "system".to_string());
        
        // Create and return the authorization
        Authorization::new(
            agent.agent_id().clone(),
            op_id,
            valid_capabilities,
            vec![],
            None,
            metadata,
        )
    }
    
    /// Check if an agent has a specific capability
    fn has_capability<A: Agent>(&self, agent: &A, capability: &Capability<Box<dyn Resource>>) -> bool {
        agent.capabilities().iter().any(|c| c.id() == capability.id())
    }
}

/// Registry for capabilities
pub struct CapabilityRegistry {
    capabilities: HashMap<String, CapabilityDefinition>,
}

/// Definition of a capability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    /// ID of the capability
    id: String,
    
    /// Name of the capability
    name: String,
    
    /// Description of the capability
    description: String,
    
    /// Resource types this capability applies to
    resource_types: Vec<ResourceType>,
    
    /// Allowed actions
    allowed_actions: Vec<String>,
    
    /// Whether this capability can be delegated
    delegatable: bool,
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }
    
    /// Register a capability definition
    pub fn register_capability(&mut self, definition: CapabilityDefinition) -> Result<(), AuthorizationError> {
        if self.capabilities.contains_key(&definition.id) {
            return Err(AuthorizationError::Other(format!(
                "Capability with ID {} already exists",
                definition.id
            )));
        }
        
        self.capabilities.insert(definition.id.clone(), definition);
        Ok(())
    }
    
    /// Get a capability definition
    pub fn get_capability(&self, id: &str) -> Option<&CapabilityDefinition> {
        self.capabilities.get(id)
    }
    
    /// Validate capabilities for a specific resource
    pub fn validate_capabilities(
        &self,
        capabilities: Vec<Capability<Box<dyn Resource>>>,
        _resource_id: &ResourceId,
    ) -> Result<Vec<Capability<Box<dyn Resource>>>, AuthorizationError> {
        // In a more complete implementation, we would check each capability against the resource
        // For now, just return the capabilities as-is
        Ok(capabilities)
    }
}

/// Authorization service for handling capability verification
pub struct AuthorizationService {
    capability_verifier: CapabilityVerifier,
}

impl AuthorizationService {
    /// Create a new authorization service
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            capability_verifier: CapabilityVerifier::new(registry),
        }
    }
    
    /// Authorize an operation for an agent
    pub async fn authorize<A: Agent + Send + Sync>(
        &self,
        agent: &A,
        operation: &Operation,
    ) -> Result<Authorization, AuthorizationError> {
        // Verify that the agent has the required capabilities
        let authorization = self.capability_verifier.verify_capabilities(agent, operation).await?;
        
        // In a production system, you would log the authorization attempt here
        
        Ok(authorization)
    }
}

/// Authorization profile for an agent
#[derive(Debug)]
pub struct AgentAuthorization {
    /// Identity ID of the agent
    pub identity: IdentityId,
    
    /// Capabilities granted to this agent
    pub capabilities: Vec<Box<Capability<Box<dyn Resource>>>>,
    
    /// Timestamp when this authorization was created
    pub timestamp: u64,
    
    /// Optional expiration time
    pub expires: Option<u64>,
    
    /// Content hash of this authorization
    pub hash: Option<ContentHash>,
}

impl Clone for AgentAuthorization {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            capabilities: self.capabilities.iter()
                .map(|cap| {
                    // Create a new capability with the same properties
                    Box::new(Capability {
                        id: cap.id.clone(),
                        grants: cap.grants.clone(),
                        origin: cap.origin.clone(),
                        content_hash: cap.content_hash.clone(),
                        phantom: std::marker::PhantomData,
                    })
                })
                .collect(),
            timestamp: self.timestamp,
            expires: self.expires,
            hash: self.hash.clone(),
        }
    }
}

// Do the same for AuthorizationRequest
#[derive(Debug)]
pub struct AuthorizationRequest {
    /// Identity requesting authorization
    pub identity: IdentityId,
    
    /// Capabilities being requested
    pub capabilities: Vec<Box<Capability<Box<dyn Resource>>>>,
    
    /// Request timestamp
    pub timestamp: u64,
    
    /// Hash of the request content
    pub hash: Option<ContentHash>,
}

impl Clone for AuthorizationRequest {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            capabilities: self.capabilities.iter()
                .map(|cap| {
                    // Create a new capability with the same properties
                    Box::new(Capability {
                        id: cap.id.clone(),
                        grants: cap.grants.clone(),
                        origin: cap.origin.clone(),
                        content_hash: cap.content_hash.clone(),
                        phantom: std::marker::PhantomData,
                    })
                })
                .collect(),
            timestamp: self.timestamp,
            hash: self.hash.clone(),
        }
    }
}

/// Check if an agent has capability for a specific operation
pub fn check_operation(
    agent: &dyn Agent,
    operation: &super::operation::Operation,
) -> Result<Authorization, AuthorizationError> {
    let capabilities = agent.capabilities();
    
    // Check if any capability allows operation
    for capability in capabilities {
        // Skip capability if it's not active
        // Use the metadata field as it's a standard HashMap in Operation
        let inactive_default = "inactive".to_string();
        let status = operation.metadata.get("status").unwrap_or(&inactive_default);
        if status != "active" {
            continue;
        }
        
        // Check if the capability allows the operation
        if capability_allows_operation(&capability, operation)? {
            // Get operation ID, handling potential error
            let op_id = match operation.id() {
                Ok(id) => id,
                Err(e) => return Err(AuthorizationError::SerializationError(
                    format!("Failed to get operation ID: {}", e)
                )),
            };
            
            // Create authorization
            return Ok(Authorization::new(
                agent.agent_id().clone(),
                op_id,
                vec![capability.clone()],
                Vec::new(), // No constraints
                None,       // No expiration
                HashMap::new(),
            )?);
        }
    }
    
    // No matching capability found
    Err(AuthorizationError::Other(
        format!("Agent is not authorized to perform operation: {:?}", operation.operation_type)
    ))
}

/// Check if a capability allows a specific operation
fn capability_allows_operation(
    _capability: &Capability<Box<dyn Resource>>,
    _operation: &super::operation::Operation,
) -> Result<bool, AuthorizationError> {
    // For now, always return true (placeholder implementation)
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::agent::agent::{Agent, AgentImpl, AgentBuilder};
    use crate::resource::agent::types::{AgentType, AgentState};
    use crate::resource::agent::operation::{Operation, OperationBuilder, OperationType};

    // Test helper to create a capability that doesn't require accessing private fields
    fn create_test_capability(id: &str) -> Capability<Box<dyn Resource>> {
        Capability {
            id: id.into(),
            grants: vec!["read".into()],
            origin: Some("system".into()),
            content_hash: None,
            phantom: PhantomData,
        }
    }

    // A basic test that just verifies the contract of the Authorization struct
    #[test]
    fn test_authorization_creation() {
        // Create agent_id using the public constructor
        let agent_id = AgentId::new(
            ResourceId::new(ContentHash::new("blake3", vec![1, 2, 3, 4])),
            AgentType::User,
        );
        
        let operation_id = ContentId::from_bytes(vec![5, 6, 7, 8]);
        
        let capabilities = vec![
            create_test_capability("read"),
        ];
        
        let authorization = Authorization::new(
            agent_id.clone(),
            operation_id.clone(),
            capabilities,
            vec![],
            None,
            HashMap::new(),
        ).unwrap();
        
        assert_eq!(authorization.agent_id(), &agent_id);
        assert_eq!(authorization.operation_id(), &operation_id);
        assert_eq!(authorization.capabilities().len(), 1);
    }
} 
