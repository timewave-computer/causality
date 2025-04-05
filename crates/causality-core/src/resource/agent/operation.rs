// operation.rs - Agent operation system
//
// This file defines the Operation struct and related types for executing
// operations on resources through agents.

use crate::resource::{ResourceId, Resource};
use crate::utils::content_addressing;
use crate::effect::{Effect, EffectContext, EffectOutcome, EffectType};
use crate::serialization::{Serializable, SerializationError};
use causality_types::ContentHash;
use anyhow::{Result, anyhow};

use super::types::{AgentId, AgentError};
use super::agent::Agent;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::marker::PhantomData;

use hex;

// Define the serde module for ContentHash compatibility
mod content_hash_serde {
    use serde::{Serializer, Deserialize, Deserializer};
    use causality_types::ContentHash;

    pub fn serialize<S>(hash: &ContentHash, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = hash.to_string();
        serializer.serialize_str(&hex_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ContentHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let hex_string: String = Deserialize::deserialize(deserializer)?;
        
        let parts: Vec<&str> = hex_string.split(':').collect();
        if parts.len() == 2 {
            let algorithm = parts[0];
            let hex_value = parts[1];
            if let Ok(bytes) = hex::decode(hex_value) {
                return Ok(ContentHash::new(algorithm, bytes));
            }
        }
        
        if let Ok(bytes) = hex::decode(&hex_string) {
            if !bytes.is_empty() {
                return Ok(ContentHash::new("blake3", bytes));
            }
        }
        
        Err(Error::custom(format!(
            "ContentHash must be in format 'algorithm:value' or a valid hex string"
        )))
    }
}

/// Simple identity identifier used in operations
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct IdentityId(String);

impl IdentityId {
    /// Create a new identity ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the underlying ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IdentityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A serializable representation of an effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInfo {
    /// The type of effect
    pub effect_type: String,
    
    /// The target resource ID for this effect, if any
    pub target_resource_id: Option<ResourceId>,
    
    /// The resource type for this effect, if any
    pub resource_type: Option<String>,
    
    /// The name of the resource for this effect, if any
    pub resource_name: Option<String>,
    
    /// Parameters for this effect
    pub parameters: HashMap<String, String>,
    
    /// Metadata about this effect
    pub metadata: HashMap<String, String>,
}

impl EffectInfo {
    /// Create a new effect info
    pub fn new(effect_type: &str) -> Self {
        Self {
            effect_type: effect_type.to_string(),
            target_resource_id: None,
            resource_type: None,
            resource_name: None,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the target resource ID
    pub fn with_target_resource_id(mut self, resource_id: ResourceId) -> Self {
        self.target_resource_id = Some(resource_id);
        self
    }
    
    /// Set the resource type
    pub fn with_resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }
    
    /// Set the resource name
    pub fn with_resource_name(mut self, resource_name: &str) -> Self {
        self.resource_name = Some(resource_name.to_string());
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Represents an operation that can be performed by an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Operation {
    /// The identity that authored this operation
    pub identity: IdentityId,
    /// The type of operation
    pub operation_type: OperationType,
    /// The target resource of this operation
    pub target: ResourceId,
    /// The effects to apply when this operation is executed
    pub effects: Vec<EffectInfo>,
    /// The current state of the operation
    pub state: OperationState,
    /// IDs of previous operations this depends on
    pub previous_operations: Vec<OperationId>,
    /// Operation parameters
    pub parameters: HashMap<String, String>,
    /// Operation metadata
    pub metadata: HashMap<String, String>,
    /// The capability required for this operation
    pub capability: Option<String>,
}

impl Operation {
    /// Create a new operation
    pub fn new(
        identity: IdentityId,
        operation_type: OperationType,
        target: ResourceId,
        effects: Vec<Box<dyn Effect>>,
    ) -> Self {
        let effect_infos = effects_to_info(&effects);
        Self {
            identity,
            operation_type,
            target,
            effects: effect_infos,
            state: OperationState::Pending,
            previous_operations: Vec::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
            capability: None,
        }
    }

    /// Add an effect to this operation
    pub fn add_effect(&mut self, effect_info: EffectInfo) {
        self.effects.push(effect_info);
    }

    /// Add a dependency on a previous operation
    pub fn add_dependency(&mut self, operation_id: OperationId) {
        self.previous_operations.push(operation_id);
    }

    /// Set the state of the operation
    pub fn set_state(&mut self, state: OperationState) {
        self.state = state;
    }

    /// Get a specific parameter
    pub fn get_parameter(&self, name: &str) -> Option<&String> {
        self.parameters.get(name)
    }

    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Returns the target resource ID for this operation
    pub fn target_resource_id(&self) -> Option<&ResourceId> {
        Some(&self.target)
    }

    /// Returns the required capabilities for this operation
    pub fn required_capabilities(&self) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Add capabilities from the operation itself
        if let Some(ref cap) = self.capability {
            capabilities.push(cap.clone());
        }
        
        // Add capabilities from effects
        for effect in &self.effects {
            if let Some(ref resource_type) = effect.resource_type {
                // For each effect type, add a specific capability
                let capability = match effect.effect_type.as_str() {
                    "read" => format!("{}.read", resource_type),
                    "write" => format!("{}.write", resource_type),
                    "create" => format!("{}.create", resource_type),
                    "delete" => format!("{}.delete", resource_type),
                    custom => format!("{}.{}", resource_type, custom),
                };
                
                if !capabilities.contains(&capability) {
                    capabilities.push(capability);
                }
            }
        }
        
        capabilities
    }

    /// Returns true if this operation has effects
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }
    
    /// Generate a unique ID for this operation
    pub fn id(&self) -> Result<OperationId, SerializationError> {
        let hash = content_addressing::hash_object(self)
            .map_err(|e| SerializationError::SerializationFailed(e))?;
        Ok(OperationId::new(&hash))
    }
    
    /// Convert from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        serde_json::from_slice(bytes).map_err(|e| SerializationError::DeserializationFailed(e.to_string()))
    }
}

impl content_addressing::ContentAddressed for Operation {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        content_addressing::hash_object(self)
            .map_err(|e| anyhow!("Content addressing error: {}", e))
    }
}

/// Create an EffectInfo from an Effect
pub fn effect_info_from_effect<T: Effect + ?Sized>(effect: Box<T>) -> EffectInfo {
    let effect_type = match effect.effect_type() {
        EffectType::Read => "read".to_string(),
        EffectType::Write => "write".to_string(),
        EffectType::Create => "create".to_string(), 
        EffectType::Delete => "delete".to_string(),
        EffectType::Custom(ref name) => name.clone(),
    };
    
    let mut info = EffectInfo::new(&effect_type);
    
    // Add description as metadata
    info = info.with_metadata("description", &effect.description());
    
    // Try to extract resource ID if possible
    // This would need access to the Effect interface
    
    info
}

// Create a vector of EffectInfo from a vector of Effects 
pub fn effects_to_info(effects: &[Box<dyn Effect>]) -> Vec<EffectInfo> {
    effects.iter().map(|effect| {
        // Use the type from the original effect
        let effect_type = match effect.effect_type() {
            EffectType::Read => "read".to_string(),
            EffectType::Write => "write".to_string(),
            EffectType::Create => "create".to_string(), 
            EffectType::Delete => "delete".to_string(),
            EffectType::Custom(ref name) => name.clone(),
        };
        
        // Create a new EffectInfo
        let mut info = EffectInfo::new(&effect_type);
        
        // Add description
        info = info.with_metadata("description", &effect.description());
        
        // Use the properties from the original effect instead of trying to clone it
        info
    }).collect()
}

/// Define operation type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Create a new resource
    Create,
    /// Update an existing resource
    Update,
    /// Delete a resource
    Delete,
    /// Authorize access to a resource
    Authorize,
    /// Revoke access to a resource
    Revoke,
    /// Custom operation type
    Custom(String),
}

/// Define operation state
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationState {
    /// The operation is pending execution
    Pending,
    /// The operation is being executed
    InProgress,
    /// The operation completed successfully
    Completed,
    /// The operation failed
    Failed(String),
}

/// Define operation ID
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId {
    /// The hex string representation of the content hash
    pub hash_hex: String,
}

impl OperationId {
    /// Create a new operation ID from a content hash
    pub fn new(hash: &ContentHash) -> Self {
        Self {
            hash_hex: hash.to_hex(),
        }
    }
    
    /// Get the content hash for this operation ID
    pub fn content_hash(&self) -> Result<ContentHash, String> {
        let bytes = hex::decode(&self.hash_hex).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err(format!("Invalid hash length: {}", bytes.len()));
        }
        
        Ok(ContentHash::new("blake3", bytes))
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "op:{}", self.hash_hex)
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self {
            hash_hex: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        }
    }
}

/// Operation error types
#[derive(Error, Debug)]
pub enum OperationError {
    /// Failed to execute the operation
    #[error("Failed to execute operation: {0}")]
    ExecutionFailed(String),
    
    /// The operation is invalid
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Agent error during operation processing
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// General error during operation processing
    #[error("Operation error: {0}")]
    Other(#[from] anyhow::Error),
}

/// View of operation data for content addressing
#[derive(Serialize, Deserialize)]
struct OperationContentView {
    agent_id: AgentId,
    target_resource_id: ResourceId,
    operation_type: OperationType,
    parameters: HashMap<String, String>,
    effect_descriptions: Vec<String>,
    required_capability_ids: Vec<String>,
    metadata: HashMap<String, String>,
}

// Custom implementation for content addressing
impl OperationContentView {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        content_addressing::hash_object(self)
            .map_err(|e| anyhow::anyhow!("Content addressing error: {}", e))
    }
}

/// Context for operation execution
#[derive(Clone, Debug)]
pub struct OperationContext {
    /// Context information as key-value pairs
    context: HashMap<String, String>,
}

impl OperationContext {
    /// Create a new operation context
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }
    
    /// Add context information
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
    
    /// Get context information
    pub fn get(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
}

impl Default for OperationContext {
    fn default() -> Self {
        Self::new()
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

/// Status of operation execution
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation completed successfully
    Success,
    /// Operation failed
    Failed(String),
    /// Operation was cancelled
    Cancelled,
}

/// Builder for creating operations
#[derive(Default)]
pub struct OperationBuilder {
    /// Identity that will perform the operation
    identity: Option<IdentityId>,
    /// Target resource
    target_resource_id: Option<ResourceId>,
    /// Operation type
    operation_type: Option<OperationType>,
    /// Operation parameters
    parameters: HashMap<String, String>,
    /// Effects to be executed
    effects: Vec<Box<dyn Effect>>,
    /// Required capabilities
    required_capabilities: Vec<Box<Capability<dyn Resource>>>,
    /// Operation metadata
    metadata: HashMap<String, String>,
}

impl OperationBuilder {
    /// Create a new operation builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the identity
    pub fn with_identity(mut self, identity: IdentityId) -> Self {
        self.identity = Some(identity);
        self
    }
    
    /// Set the target resource
    pub fn with_target(mut self, target: ResourceId) -> Self {
        self.target_resource_id = Some(target);
        self
    }
    
    /// Set the operation type
    pub fn with_operation_type(mut self, operation_type: OperationType) -> Self {
        self.operation_type = Some(operation_type);
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Add an effect
    pub fn with_effect(mut self, effect: Box<dyn Effect>) -> Self {
        self.effects.push(effect);
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability_id: impl Into<String>) -> Self {
        // Create a simpler capability representation that doesn't rely on the trait
        let cap_string = capability_id.into();
        let capability = Box::new(Capability {
            id: cap_string.clone(),
            grants: "auto".to_string(), // Auto-determine grants based on capability ID
            origin: "user".to_string(),
            content_hash: content_addressing::hash_string(&cap_string),
            _phantom: PhantomData::<dyn Resource>,
        });
        
        self.required_capabilities.push(capability);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Build the operation
    pub fn build(self) -> Result<Operation, OperationError> {
        // Ensure required fields are set
        let operation_type = self.operation_type.ok_or_else(|| 
            OperationError::InvalidOperation("Operation type is required".to_string()))?;
            
        let target_resource_id = self.target_resource_id.ok_or_else(|| 
            OperationError::InvalidOperation("Target resource ID is required".to_string()))?;
        
        let identity = self.identity.unwrap_or_default();
        
        // Create the operation
        let mut operation = Operation {
            identity,
            operation_type,
            target: target_resource_id,
            effects: Vec::new(),
            state: OperationState::Pending,
            previous_operations: Vec::new(),
            parameters: self.parameters,
            metadata: self.metadata,
            capability: None,
        };
        
        // Add effects - convert them to EffectInfo
        for effect in self.effects {
            let effect_info = effect_info_from_effect(effect);
            operation.add_effect(effect_info);
        }
        
        // Add capability effects
        for capability in self.required_capabilities {
            // Create an EffectInfo for the capability
            let cap_name = capability.id().to_string();
            let mut effect_info = EffectInfo::new("capability");
            effect_info = effect_info.with_parameter("capability", &cap_name);
            operation.add_effect(effect_info);
        }
        
        Ok(operation)
    }
}

/// A basic operation configuration for agents
#[derive(Debug)]
pub struct OperationConfig {
    /// The operation ID
    pub id: OperationId,
    
    /// The operation type
    pub operation_type: String,
    
    /// The target resource ID
    pub target_id: ResourceId,
    
    /// Operation parameters
    pub parameters: HashMap<String, String>,
    
    /// Required capabilities for this operation
    pub required_capabilities: Vec<String>,
    
    /// Whether this operation requires authorization
    pub requires_authorization: bool,
}

impl Clone for OperationConfig {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            operation_type: self.operation_type.clone(),
            target_id: self.target_id.clone(),
            parameters: self.parameters.clone(),
            required_capabilities: self.required_capabilities.clone(),
            requires_authorization: self.requires_authorization,
        }
    }
}

/// Generic operation definition for agent operations
#[derive(Debug)]
pub struct OperationDefinition {
    /// The operation ID
    pub id: OperationId,
    
    /// The operation type
    pub operation_type: String,
    
    /// The target resource ID
    pub target_id: ResourceId,
    
    /// Operation parameters
    pub parameters: HashMap<String, String>,
    
    /// Required capabilities for this operation
    pub required_capabilities: Vec<String>,
    
    /// Whether this operation requires authorization
    pub requires_authorization: bool,
    
    /// The handler for this operation type
    pub handler: String,
}

impl Clone for OperationDefinition {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            operation_type: self.operation_type.clone(),
            target_id: self.target_id.clone(),
            parameters: self.parameters.clone(),
            required_capabilities: self.required_capabilities.clone(),
            requires_authorization: self.requires_authorization,
            handler: self.handler.clone(),
        }
    }
}

/// Represents required capabilities to perform this operation
#[derive(Debug)]
pub struct RequiredCapabilities {
    /// The capabilities required for this operation
    pub required_capabilities: Vec<Box<Capability<dyn Resource>>>,
}

impl Clone for RequiredCapabilities {
    fn clone(&self) -> Self {
        let cloned_capabilities = self.required_capabilities.iter()
            .map(|cap| {
                // Create a new capability with the same properties
                Box::new(Capability::new(
                    cap.id(),
                    &cap.grants,
                    Some(&cap.origin),
                ))
            })
            .collect();
        
        Self {
            required_capabilities: cloned_capabilities
        }
    }
}

/// Simple capability representation for operations
#[derive(Debug)]
pub struct Capability<T: ?Sized> {
    /// The ID of this capability
    pub id: String,
    /// The grants this capability provides
    pub grants: String,
    /// The origin of this capability
    pub origin: String,
    /// The content hash of this capability
    pub content_hash: ContentHash,
    /// Phantom data to indicate what resource this capability is for
    pub _phantom: PhantomData<T>,
}

impl<T: ?Sized> Capability<T> {
    /// Create a new capability
    pub fn new(id: &str, grants: &str, origin: Option<&str>) -> Self {
        let origin = origin.unwrap_or("system").to_string();
        let content_hash = content_addressing::hash_string(&format!("{}:{}:{}", id, grants, origin));
        
        Self {
            id: id.to_string(),
            grants: grants.to_string(),
            origin,
            content_hash,
            _phantom: PhantomData,
        }
    }
    
    /// Get the ID of this capability
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl<T: ?Sized> Clone for Capability<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            content_hash: self.content_hash.clone(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    // Mock Effect for testing
    #[derive(Debug, Clone)]
    struct MockEffect {
        id: String,
        type_id: String,
    }
    
    impl MockEffect {
        fn new() -> Self {
            Self {
                id: "mock-effect-1".to_string(),
                type_id: "mock-effect-type".to_string(),
            }
        }
    }
    
    #[async_trait::async_trait]
    impl Effect for MockEffect {
        fn effect_type(&self) -> EffectType {
            EffectType::Custom(self.type_id.clone())
        }
        
        fn description(&self) -> String {
            format!("Mock effect with ID: {}", self.id)
        }
        
        async fn execute(&self, _context: &dyn EffectContext) -> crate::effect::EffectResult<EffectOutcome> {
            Ok(EffectOutcome::success(HashMap::new()))
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    #[test]
    fn test_operation_creation() {
        // Create test data
        let agent_id = create_test_agent_id("test-agent");
        let identity_id = IdentityId::new(format!("{}", agent_id));
        let target_id = create_test_resource_id("test-resource");
        let capability_id = "test.capability";
        let effect = Box::new(MockEffect::new()) as Box<dyn Effect>;
        
        // Create an operation with the builder
        let operation = OperationBuilder::new()
            .with_identity(identity_id.clone())
            .with_target(target_id.clone())
            .with_operation_type(OperationType::Update)  // Use Update instead of Read
            .with_parameter("param1", "value1")
            .with_effect(effect)
            .with_capability(capability_id)
            .with_metadata("context", "test")
            .build()
            .unwrap();
        
        // Verify the operation
        assert_eq!(operation.identity, identity_id);
        assert_eq!(operation.target, target_id);
        assert_eq!(operation.operation_type, OperationType::Update);
        assert_eq!(operation.get_parameter("param1"), Some(&"value1".to_string()));
        assert_eq!(operation.effects.len(), 2);
        assert_eq!(operation.get_metadata("context"), Some(&"test".to_string()));
    }
    
    #[test]
    fn test_operation_content_addressing() {
        // Create test data
        let agent_id = create_test_agent_id("test-agent");
        let identity_id = IdentityId::new(format!("{}", agent_id));
        let target_id = create_test_resource_id("test-resource");
        
        // Create two identical operations
        let operation1 = OperationBuilder::new()
            .with_identity(identity_id.clone())
            .with_target(target_id.clone())
            .with_operation_type(OperationType::Update)  // Use Update instead of Read
            .build()
            .unwrap();
        
        let operation2 = OperationBuilder::new()
            .with_identity(identity_id.clone())
            .with_target(target_id.clone())
            .with_operation_type(OperationType::Update)  // Use Update instead of Read
            .build()
            .unwrap();
        
        // They should have the same ID
        let id1 = operation1.id().unwrap();
        let id2 = operation2.id().unwrap();
        assert_eq!(id1.hash_hex, id2.hash_hex);
        
        // Now create a different operation
        let operation3 = OperationBuilder::new()
            .with_identity(identity_id)
            .with_target(target_id)
            .with_operation_type(OperationType::Update)
            .with_parameter("param", "value")  // Add a parameter to make it different
            .build()
            .unwrap();
        
        // It should have a different ID
        let id3 = operation3.id().unwrap();
        assert_ne!(id1.hash_hex, id3.hash_hex);
    }
    
    // Helper function to create an agent ID
    fn create_test_agent_id(name: &str) -> AgentId {
        // Hash the name to create bytes
        let hash_bytes = blake3::hash(name.as_bytes()).as_bytes();
        AgentId::from_content_hash(hash_bytes, AgentType::User)
    }
    
    // Helper function to create a resource ID
    fn create_test_resource_id(name: &str) -> ResourceId {
        // Hash the name to create bytes for the content hash
        let hash_bytes = blake3::hash(name.as_bytes()).as_bytes().to_vec();
        ResourceId::new(ContentHash::new("blake3", hash_bytes))
    }
} 
