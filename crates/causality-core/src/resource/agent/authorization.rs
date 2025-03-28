// authorization.rs - Authorization system for agent capability verification
//
// This file implements the authorization system for verifying that agents have
// the necessary capabilities to perform operations.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::ResourceError;
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::effect::Effect;
use crate::serialization::{Serializable, DeserializationError};

use super::types::{AgentId, AgentError};
use super::operation::{Operation, OperationId, OperationType, OperationError};
use super::agent::Agent;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Authorization for an agent operation
#[derive(Debug, Clone)]
pub struct Authorization {
    /// The agent ID that is authorizing
    pub agent_id: AgentId,
    
    /// The operation being authorized
    pub operation_id: OperationId,
    
    /// The capabilities being granted for this operation
    pub capabilities: Vec<Capability<dyn Resource>>,
    
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
    
    /// The capabilities being proposed for this operation
    pub capabilities: Vec<Capability<dyn Resource>>,
    
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
        capabilities: Vec<Capability<dyn Resource>>,
        constraints: Vec<AuthorizationConstraint>,
        expires_at: Option<u64>,
        metadata: HashMap<String, String>,
    ) -> Result<Self, AuthorizationError> {
        let authorization = Self {
            agent_id,
            operation_id,
            capabilities,
            constraints,
            expires_at,
            metadata,
        };
        
        // Generate content hash
        let hash = authorization.compute_content_hash()?;
        
        Ok(authorization)
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
    pub fn capabilities(&self) -> &[Capability<dyn Resource>] {
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
        // Create a serializable view for content addressing
        let content_view = AuthorizationContentView {
            agent_id: self.agent_id.clone(),
            operation_id: self.operation_id.clone(),
            capabilities: self.capabilities.clone(),
            constraints: self.constraints.clone(),
            expires_at: self.expires_at,
            metadata: self.metadata.clone(),
        };
        
        // Compute the content hash
        let hash = content_view.content_hash()
            .map_err(|e| AuthorizationError::SerializationError(e.to_string()))?;
        
        Ok(hash)
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
}

/// View of authorization data for content addressing
#[derive(Serialize, Deserialize)]
struct AuthorizationContentView {
    agent_id: AgentId,
    operation_id: OperationId,
    capabilities: Vec<Capability<dyn Resource>>,
    constraints: Vec<AuthorizationConstraint>,
    expires_at: Option<u64>,
    metadata: HashMap<String, String>,
}

// Custom implementation for content addressing
impl AuthorizationContentView {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        ContentHash::for_object(self)
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
        
        // Check if the agent has all required capabilities
        for capability in required_capabilities {
            if !self.has_capability(agent, capability) {
                return Err(AuthorizationError::MissingCapability(format!(
                    "Agent {} is missing capability {}",
                    agent.agent_id(),
                    capability.id()
                )));
            }
        }
        
        // Validate capabilities against the registry
        let valid_capabilities = self.registry.validate_capabilities(
            agent.capabilities(),
            operation.target_resource_id(),
        )?;
        
        // Create a proof (in a real implementation, this would be a signature)
        let proof = vec![1, 2, 3, 4]; // Placeholder proof
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("verifier".to_string(), "system".to_string());
        
        // Create and return the authorization
        Authorization::new(
            agent.agent_id().clone(),
            operation.id().clone(),
            valid_capabilities,
            vec![],
            None,
            metadata,
        )
    }
    
    /// Check if an agent has a specific capability
    fn has_capability<A: Agent>(&self, agent: &A, capability: &Capability<dyn Resource>) -> bool {
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
        capabilities: Vec<Capability<dyn Resource>>,
        resource_id: &ResourceId,
    ) -> Result<Vec<Capability<dyn Resource>>, AuthorizationError> {
        // Filter out capabilities that are not valid for this resource
        let valid_capabilities = capabilities.into_iter()
            .filter(|capability| {
                // Check if the capability is registered
                if let Some(definition) = self.get_capability(capability.id()) {
                    // Check if the capability applies to this resource type
                    definition.resource_types.contains(&resource_id.resource_type())
                } else {
                    false
                }
            })
            .collect();
        
        Ok(valid_capabilities)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::agent::agent::{Agent, AgentImpl, AgentBuilder};
    use crate::resource::agent::types::{AgentType, AgentState};
    use crate::resource::agent::operation::{Operation, OperationBuilder, OperationType};
    
    #[tokio::test]
    async fn test_capability_verification() {
        // Create a capability registry
        let mut registry = CapabilityRegistry::new();
        
        // Register a capability definition
        let definition = CapabilityDefinition {
            id: "read".to_string(),
            name: "Read".to_string(),
            description: "Allows reading a resource".to_string(),
            resource_types: vec![ResourceType::new("Document", "1.0")],
            allowed_actions: vec!["read".to_string()],
            delegatable: true,
        };
        
        registry.register_capability(definition).unwrap();
        
        let registry_arc = Arc::new(registry);
        
        // Create a capability verifier
        let verifier = CapabilityVerifier::new(registry_arc);
        
        // Create an agent with the read capability
        let agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .with_capability(Capability::new("read", "read", None))
            .build()
            .unwrap();
        
        // Create a target resource ID
        let target_id = ResourceId::new(ContentHash::default());
        
        // Create an operation that requires the read capability
        let read_capability = Capability::new("read", "read", None);
        
        let operation = OperationBuilder::new()
            .agent_id(agent.agent_id().clone())
            .target_resource(target_id)
            .operation_type(OperationType::Read)
            .require_capability(read_capability)
            .build()
            .unwrap();
        
        // Verify the capabilities
        let authorization = verifier.verify_capabilities(&agent, &operation).await.unwrap();
        
        // Check the authorization
        assert_eq!(authorization.agent_id(), agent.agent_id());
        assert_eq!(authorization.operation_id(), operation.id());
        assert_eq!(authorization.capabilities().len(), 1);
        assert_eq!(authorization.capabilities()[0].id(), "read");
    }
    
    #[tokio::test]
    async fn test_unauthorized_operation() {
        // Create a capability registry
        let registry = CapabilityRegistry::new();
        let registry_arc = Arc::new(registry);
        
        // Create a capability verifier
        let verifier = CapabilityVerifier::new(registry_arc);
        
        // Create an agent without the required capability
        let agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        // Create a target resource ID
        let target_id = ResourceId::new(ContentHash::default());
        
        // Create an operation that requires the read capability
        let read_capability = Capability::new("read", "read", None);
        
        let operation = OperationBuilder::new()
            .agent_id(agent.agent_id().clone())
            .target_resource(target_id)
            .operation_type(OperationType::Read)
            .require_capability(read_capability)
            .build()
            .unwrap();
        
        // Attempt to verify capabilities (should fail)
        let result = verifier.verify_capabilities(&agent, &operation).await;
        
        // Check that verification failed
        assert!(result.is_err());
        if let Err(AuthorizationError::MissingCapability(_)) = result {
            // Expected error
        } else {
            panic!("Expected MissingCapability error, got {:?}", result);
        }
    }
} 