// Resource node implementation for the Temporal Effect Graph
// This file defines the ResourceNode struct, which represents a resource node
// in the TEG.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use causality_types::{ContentHash, ContentAddressed, HashError, HashOutput, test_content_hash};
use crate::{ResourceId, DomainId};
use crate::effect_node::ParameterValue;
use serde_json::Value;

/// Enumeration of resource states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ResourceState {
    /// Resource is active and available
    Active,
    /// Resource is frozen (can be read but not modified)
    Frozen,
    /// Resource is locked (cannot be read or modified by other effects)
    Locked,
    /// Resource is inactive (does not exist or is deleted)
    Inactive,
    /// Resource state is defined by a custom value
    Custom(String),
}

/// Represents a resource node in the Temporal Effect Graph
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceNode {
    /// Unique identifier for this resource
    pub id: ResourceId,
    
    /// The type of resource
    pub resource_type: String,
    
    /// Current state of the resource
    pub state: ResourceState,
    
    /// Additional metadata about the resource
    pub metadata: HashMap<String, ParameterValue>,
    
    /// Domain in which this resource exists
    pub domain_id: DomainId,
    
    /// Content hash for this resource node
    pub content_hash: ContentHash,
}

impl ResourceNode {
    /// Create a new resource node builder
    pub fn builder() -> ResourceNodeBuilder {
        ResourceNodeBuilder::new()
    }
    
    /// Check if the resource is in the active state
    pub fn is_active(&self) -> bool {
        matches!(self.state, ResourceState::Active)
    }
    
    /// Check if the resource is in the frozen state
    pub fn is_frozen(&self) -> bool {
        matches!(self.state, ResourceState::Frozen)
    }
    
    /// Check if the resource is in the locked state
    pub fn is_locked(&self) -> bool {
        matches!(self.state, ResourceState::Locked)
    }
    
    /// Check if the resource is in the inactive state
    pub fn is_inactive(&self) -> bool {
        matches!(self.state, ResourceState::Inactive)
    }
    
    /// Get a metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&ParameterValue> {
        self.metadata.get(key)
    }
    
    /// Get the resource identifier
    pub fn resource_id(&self) -> &ResourceId {
        &self.id
    }
    
    /// Get the resource type
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    /// Get the resource state
    pub fn state(&self) -> &ResourceState {
        &self.state
    }
    
    /// Get the resource metadata
    pub fn metadata(&self) -> &HashMap<String, ParameterValue> {
        &self.metadata
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl ContentAddressed for ResourceNode {
    fn content_hash(&self) -> Result<causality_types::HashOutput, HashError> {
        // We need to create a copy without the content_hash field to avoid circular hashing
        let mut resource_for_hash = self.clone();
        // Reset the content hash to a default/empty value to avoid it affecting the hash
        resource_for_hash.content_hash = ContentHash::new("blake3", vec![0; 32]);
        
        // Serialize the resource to JSON bytes
        let serialized = serde_json::to_vec(&resource_for_hash)
            .map_err(|e| HashError::SerializationError(e.to_string()))?;
        
        // Calculate the hash of the serialized data
        let hash_output = causality_types::content_addressing::content_hash_from_bytes(&serialized);
        Ok(hash_output)
    }
    
    fn verify(&self, expected_hash: &causality_types::HashOutput) -> Result<bool, HashError> {
        let actual_hash = self.content_hash()?;
        Ok(actual_hash == *expected_hash)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        serde_json::to_vec(self).map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> where Self: Sized {
        serde_json::from_slice(bytes).map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Builder for creating Resource Nodes
pub struct ResourceNodeBuilder {
    id: Option<ResourceId>,
    resource_type: Option<String>,
    state: ResourceState,
    metadata: HashMap<String, ParameterValue>,
    domain_id: Option<DomainId>,
}

impl ResourceNodeBuilder {
    /// Create a new resource node builder
    pub fn new() -> Self {
        Self {
            id: None,
            resource_type: None,
            state: ResourceState::Inactive,
            metadata: HashMap::new(),
            domain_id: None,
        }
    }
    
    /// Set the resource ID
    pub fn id(mut self, id: impl Into<ResourceId>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Set the resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }
    
    /// Set the resource state
    pub fn state(mut self, state: ResourceState) -> Self {
        self.state = state;
        self
    }
    
    /// Set the resource to active state
    pub fn active(mut self) -> Self {
        self.state = ResourceState::Active;
        self
    }
    
    /// Set the resource to frozen state
    pub fn frozen(mut self) -> Self {
        self.state = ResourceState::Frozen;
        self
    }
    
    /// Set the resource to locked state
    pub fn locked(mut self) -> Self {
        self.state = ResourceState::Locked;
        self
    }
    
    /// Set the resource to inactive state
    pub fn inactive(mut self) -> Self {
        self.state = ResourceState::Inactive;
        self
    }
    
    /// Add a metadata entry
    pub fn metadata(mut self, key: impl Into<String>, value: ParameterValue) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
    
    /// Add a string metadata entry
    pub fn string_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata(key, ParameterValue::String(value.into()))
    }
    
    /// Set the domain ID
    pub fn domain(mut self, domain_id: impl Into<DomainId>) -> Self {
        self.domain_id = Some(domain_id.into());
        self
    }
    
    /// Build the resource node
    pub fn build(self) -> Result<ResourceNode, String> {
        let id = self.id.ok_or_else(|| "Resource ID is required".to_string())?;
        let resource_type = self.resource_type.ok_or_else(|| "Resource type is required".to_string())?;
        let domain_id = self.domain_id.ok_or_else(|| "Domain ID is required".to_string())?;
        
        // Use test_content_hash instead of creating a placeholder
        let content_hash = test_content_hash();
        
        Ok(ResourceNode {
            id,
            resource_type,
            state: self.state,
            metadata: self.metadata,
            domain_id,
            content_hash,
        })
    }
}
