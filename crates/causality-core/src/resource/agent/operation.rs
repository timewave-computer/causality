// operation.rs - Agent operation system
//
// This file defines the Operation struct and related types for executing
// operations on resources through agents.

use crate::resource::{ResourceId, ResourceError, ResourceType};
use crate::capability::Capability;
use crate::effect::{Effect, EffectId, EffectTypeId, EffectContext, EffectOutcome};
use crate::serialization::{Serializable, DeserializationError};
use crate::crypto::ContentHash;

use super::types::{AgentId, AgentType, AgentState, AgentError};
use super::agent::Agent;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Operation to be executed by an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Operation {
    /// Unique ID for the operation
    id: OperationId,
    
    /// Agent that initiated the operation
    agent_id: AgentId,
    
    /// Target resource
    target_resource_id: ResourceId,
    
    /// Operation type
    operation_type: OperationType,
    
    /// Operation parameters
    parameters: HashMap<String, String>,
    
    /// Effects to be executed as part of this operation
    effects: Vec<Box<dyn Effect>>,
    
    /// Required capabilities for this operation
    required_capabilities: Vec<Capability>,
    
    /// Operation metadata
    metadata: HashMap<String, String>,
}

impl Operation {
    /// Create a new operation
    pub fn new(
        agent_id: AgentId,
        target_resource_id: ResourceId,
        operation_type: OperationType,
        parameters: HashMap<String, String>,
        effects: Vec<Box<dyn Effect>>,
        required_capabilities: Vec<Capability>,
        metadata: HashMap<String, String>,
    ) -> Result<Self, OperationError> {
        let mut operation = Self {
            id: OperationId::default(), // Placeholder - will be updated
            agent_id,
            target_resource_id,
            operation_type,
            parameters,
            effects,
            required_capabilities,
            metadata,
        };
        
        // Compute the content hash and set the ID
        let hash = operation.compute_content_hash()?;
        operation.id = OperationId::new(hash);
        
        Ok(operation)
    }
    
    /// Get the operation ID
    pub fn id(&self) -> &OperationId {
        &self.id
    }
    
    /// Get the agent ID
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    
    /// Get the target resource ID
    pub fn target_resource_id(&self) -> &ResourceId {
        &self.target_resource_id
    }
    
    /// Get the operation type
    pub fn operation_type(&self) -> &OperationType {
        &self.operation_type
    }
    
    /// Get the operation parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
    
    /// Get a specific parameter
    pub fn get_parameter(&self, name: &str) -> Option<&String> {
        self.parameters.get(name)
    }
    
    /// Get the effects
    pub fn effects(&self) -> &[Box<dyn Effect>] {
        &self.effects
    }
    
    /// Get the required capabilities
    pub fn required_capabilities(&self) -> &[Capability] {
        &self.required_capabilities
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Compute the content hash for this operation
    fn compute_content_hash(&self) -> Result<ContentHash, OperationError> {
        // Create a view of the operation for content addressing
        let view = OperationContentView {
            agent_id: self.agent_id.clone(),
            target_resource_id: self.target_resource_id.clone(),
            operation_type: self.operation_type.clone(),
            parameters: self.parameters.clone(),
            effect_ids: self.effects.iter().map(|e| e.id().clone()).collect(),
            required_capability_ids: self.required_capabilities.iter().map(|c| c.id().to_string()).collect(),
            metadata: self.metadata.clone(),
        };
        
        // Compute the content hash
        let hash = view.content_hash()
            .map_err(|e| OperationError::SerializationError(e.to_string()))?;
        
        Ok(hash)
    }
}

/// View of operation data for content addressing
#[derive(Serialize, Deserialize)]
struct OperationContentView {
    agent_id: AgentId,
    target_resource_id: ResourceId,
    operation_type: OperationType,
    parameters: HashMap<String, String>,
    effect_ids: Vec<EffectId>,
    required_capability_ids: Vec<String>,
    metadata: HashMap<String, String>,
}

// Custom implementation for content addressing
impl OperationContentView {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        ContentHash::for_object(self)
    }
}

/// Unique ID for an operation
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId {
    /// The content hash of the operation
    hash: ContentHash,
}

impl OperationId {
    /// Create a new operation ID
    pub fn new(hash: ContentHash) -> Self {
        Self { hash }
    }
    
    /// Get the content hash
    pub fn hash(&self) -> &ContentHash {
        &self.hash
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self {
            hash: ContentHash::default(),
        }
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "op:{}", self.hash)
    }
}

/// Type of operation
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Create a resource
    Create,
    
    /// Read a resource
    Read,
    
    /// Update a resource
    Update,
    
    /// Delete a resource
    Delete,
    
    /// Transfer a resource
    Transfer,
    
    /// Custom operation
    Custom(String),
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Read => write!(f, "read"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::Transfer => write!(f, "transfer"),
            Self::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Context for operation execution
#[derive(Clone, Debug)]
pub struct OperationContext {
    /// The effect context to use for execution
    effect_context: Arc<dyn EffectContext>,
    
    /// Additional context parameters
    parameters: HashMap<String, String>,
}

impl OperationContext {
    /// Create a new operation context
    pub fn new(effect_context: Arc<dyn EffectContext>) -> Self {
        Self {
            effect_context,
            parameters: HashMap::new(),
        }
    }
    
    /// Get the effect context
    pub fn effect_context(&self) -> &Arc<dyn EffectContext> {
        &self.effect_context
    }
    
    /// Add a parameter
    pub fn add_parameter(&mut self, key: &str, value: &str) -> &mut Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Get a parameter
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }
}

/// Result of an operation
#[derive(Clone, Debug)]
pub struct OperationResult {
    /// The operation ID
    operation_id: OperationId,
    
    /// Status of the operation
    status: OperationStatus,
    
    /// Effect outcomes
    effect_outcomes: Vec<EffectOutcome>,
    
    /// Result data
    data: HashMap<String, String>,
}

impl OperationResult {
    /// Create a new operation result
    pub fn new(
        operation_id: OperationId,
        status: OperationStatus,
        effect_outcomes: Vec<EffectOutcome>,
        data: HashMap<String, String>,
    ) -> Self {
        Self {
            operation_id,
            status,
            effect_outcomes,
            data,
        }
    }
    
    /// Get the operation ID
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    
    /// Get the status
    pub fn status(&self) -> &OperationStatus {
        &self.status
    }
    
    /// Get the effect outcomes
    pub fn effect_outcomes(&self) -> &[EffectOutcome] {
        &self.effect_outcomes
    }
    
    /// Get the result data
    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }
    
    /// Get a specific data value
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    
    /// Check if the operation was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, OperationStatus::Success)
    }
}

/// Status of an operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation was successful
    Success,
    
    /// Operation failed
    Failure(String),
    
    /// Operation is pending
    Pending,
    
    /// Operation was cancelled
    Cancelled,
}

/// Error during operation execution
#[derive(Debug, Error)]
pub enum OperationError {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(#[from] ResourceError),
    
    /// Effect error
    #[error("Effect error: {0}")]
    EffectError(String),
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Missing capability
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Other error
    #[error("Operation error: {0}")]
    Other(String),
}

/// Builder for creating operations
pub struct OperationBuilder {
    agent_id: Option<AgentId>,
    target_resource_id: Option<ResourceId>,
    operation_type: Option<OperationType>,
    parameters: HashMap<String, String>,
    effects: Vec<Box<dyn Effect>>,
    required_capabilities: Vec<Capability>,
    metadata: HashMap<String, String>,
}

impl OperationBuilder {
    /// Create a new operation builder
    pub fn new() -> Self {
        Self {
            agent_id: None,
            target_resource_id: None,
            operation_type: None,
            parameters: HashMap::new(),
            effects: Vec::new(),
            required_capabilities: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the agent ID
    pub fn agent_id(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
    
    /// Set the target resource ID
    pub fn target_resource(mut self, resource_id: ResourceId) -> Self {
        self.target_resource_id = Some(resource_id);
        self
    }
    
    /// Set the operation type
    pub fn operation_type(mut self, operation_type: OperationType) -> Self {
        self.operation_type = Some(operation_type);
        self
    }
    
    /// Set a parameter
    pub fn parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add multiple parameters
    pub fn parameters(mut self, params: HashMap<String, String>) -> Self {
        self.parameters.extend(params);
        self
    }
    
    /// Add an effect
    pub fn add_effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }
    
    /// Add a required capability
    pub fn require_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
    
    /// Set metadata
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Build the operation
    pub fn build(self) -> Result<Operation, OperationError> {
        // Check that we have the required fields
        let agent_id = self.agent_id
            .ok_or_else(|| OperationError::InvalidOperation("Agent ID is required".to_string()))?;
        
        let target_resource_id = self.target_resource_id
            .ok_or_else(|| OperationError::InvalidOperation("Target resource ID is required".to_string()))?;
        
        let operation_type = self.operation_type
            .ok_or_else(|| OperationError::InvalidOperation("Operation type is required".to_string()))?;
        
        // Create the operation
        Operation::new(
            agent_id,
            target_resource_id,
            operation_type,
            self.parameters,
            self.effects,
            self.required_capabilities,
            self.metadata,
        )
    }
}

impl Default for OperationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityBuilder;
    
    // Mock Effect for testing
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct MockEffect {
        id: EffectId,
        type_id: EffectTypeId,
    }
    
    impl MockEffect {
        fn new() -> Self {
            Self {
                id: EffectId::default(),
                type_id: EffectTypeId::default(),
            }
        }
    }
    
    impl Effect for MockEffect {
        fn id(&self) -> &EffectId {
            &self.id
        }
        
        fn type_id(&self) -> EffectTypeId {
            self.type_id.clone()
        }
        
        fn name(&self) -> String {
            "MockEffect".to_string()
        }
        
        fn clone_effect(&self) -> Box<dyn Effect> {
            Box::new(self.clone())
        }
    }
    
    #[test]
    fn test_operation_creation() {
        // Create an agent ID
        let resource_id = ResourceId::new(ResourceType::Agent, vec![1, 2, 3, 4]);
        let agent_id = AgentId::new(resource_id, AgentType::User);
        
        // Create a target resource ID
        let target_id = ResourceId::new(ResourceType::Document, vec![5, 6, 7, 8]);
        
        // Create an operation
        let mut params = HashMap::new();
        params.insert("param1".to_string(), "value1".to_string());
        
        let capability = CapabilityBuilder::new()
            .id("capability1")
            .action("read")
            .build();
        
        let effect = Box::new(MockEffect::new()) as Box<dyn Effect>;
        
        let operation = OperationBuilder::new()
            .agent_id(agent_id)
            .target_resource(target_id.clone())
            .operation_type(OperationType::Read)
            .parameters(params.clone())
            .add_effect(effect)
            .require_capability(capability.clone())
            .metadata("context", "test")
            .build()
            .unwrap();
        
        // Verify the operation
        assert_eq!(operation.agent_id().resource_id(), &resource_id);
        assert_eq!(operation.target_resource_id(), &target_id);
        assert_eq!(operation.operation_type(), &OperationType::Read);
        assert_eq!(operation.get_parameter("param1"), Some(&"value1".to_string()));
        assert_eq!(operation.effects().len(), 1);
        assert_eq!(operation.required_capabilities().len(), 1);
        assert_eq!(operation.get_metadata("context"), Some(&"test".to_string()));
    }
    
    #[test]
    fn test_operation_content_addressing() {
        // Create an agent ID
        let resource_id = ResourceId::new(ResourceType::Agent, vec![1, 2, 3, 4]);
        let agent_id = AgentId::new(resource_id, AgentType::User);
        
        // Create a target resource ID
        let target_id = ResourceId::new(ResourceType::Document, vec![5, 6, 7, 8]);
        
        // Create two identical operations
        let operation1 = OperationBuilder::new()
            .agent_id(agent_id.clone())
            .target_resource(target_id.clone())
            .operation_type(OperationType::Read)
            .build()
            .unwrap();
        
        let operation2 = OperationBuilder::new()
            .agent_id(agent_id.clone())
            .target_resource(target_id.clone())
            .operation_type(OperationType::Read)
            .build()
            .unwrap();
        
        // They should have the same ID
        assert_eq!(operation1.id(), operation2.id());
        
        // Now create a different operation
        let operation3 = OperationBuilder::new()
            .agent_id(agent_id)
            .target_resource(target_id)
            .operation_type(OperationType::Update)
            .build()
            .unwrap();
        
        // It should have a different ID
        assert_ne!(operation1.id(), operation3.id());
    }
} 