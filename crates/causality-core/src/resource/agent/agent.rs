// agent.rs - Core agent implementation
//
// This file implements the Agent trait which forms the foundation
// for all agent components in the system.

use crate::resource::{Resource, ResourceResult, ResourceState};
use crate::resource_types::{ResourceId, ResourceType};
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::effect::Effect;

use super::types::{AgentId, AgentType, AgentState, AgentRelationship, AgentError};

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Agent trait defining the core functionality for all agent types
/// 
/// This trait extends the Resource trait and adds agent-specific functionality.
#[async_trait]
pub trait Agent: Resource + Send + Sync {
    /// Get the agent ID
    fn agent_id(&self) -> &AgentId;
    
    /// Get the agent type
    fn agent_type(&self) -> &AgentType;
    
    /// Get the agent state
    fn state(&self) -> &AgentState;
    
    /// Set the agent state
    async fn set_state(&mut self, state: AgentState) -> Result<(), AgentError>;
    
    /// Add a capability to the agent
    async fn add_capability(&mut self, capability: Capability<Resource>) -> Result<(), AgentError>;
    
    /// Remove a capability from the agent
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError>;
    
    /// Check if the agent has a capability
    fn has_capability(&self, capability_id: &str) -> bool;
    
    /// Get all capabilities
    fn capabilities(&self) -> Vec<Capability<Resource>>;
    
    /// Add a relationship with another resource
    async fn add_relationship(&mut self, relationship: AgentRelationship) -> Result<(), AgentError>;
    
    /// Remove a relationship
    async fn remove_relationship(&mut self, target_id: &ResourceId) -> Result<(), AgentError>;
    
    /// Get all relationships
    fn relationships(&self) -> Vec<AgentRelationship>;
    
    /// Get a specific relationship
    fn get_relationship(&self, target_id: &ResourceId) -> Option<&AgentRelationship>;
    
    /// Clone the agent
    fn clone_agent(&self) -> Box<dyn Agent>;
}

/// Base implementation of an agent
/// 
/// This struct implements both the Agent and Resource traits
/// and provides the foundation for specialized agent types.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentImpl {
    /// The resource ID
    resource_id: ResourceId,
    
    /// The agent ID
    agent_id: AgentId,
    
    /// The agent type
    agent_type: AgentType,
    
    /// The agent state
    state: AgentState,
    
    /// The agent's capabilities
    capabilities: Vec<Capability<Resource>>,
    
    /// Relationships with other resources
    relationships: Vec<AgentRelationship>,
    
    /// Metadata
    metadata: HashMap<String, String>,
    
    /// Content hash of the agent
    #[serde(skip)]
    content_hash: Option<ContentHash>,
}

impl AgentImpl {
    /// Create a new agent
    pub fn new(
        agent_type: AgentType,
        initial_state: Option<AgentState>,
        initial_capabilities: Option<Vec<Capability<Resource>>>,
        initial_relationships: Option<Vec<AgentRelationship>>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Self, AgentError> {
        // Create a temporary ID for the resource
        let resource_id = ResourceId::new(ResourceType::Agent, vec![]);
        
        // Create the agent ID
        let agent_id = AgentId::new(resource_id.clone(), agent_type.clone());
        
        // Create the agent
        let mut agent = Self {
            resource_id,
            agent_id,
            agent_type,
            state: initial_state.unwrap_or_default(),
            capabilities: initial_capabilities.unwrap_or_default(),
            relationships: initial_relationships.unwrap_or_default(),
            metadata: metadata.unwrap_or_default(),
            content_hash: None,
        };
        
        // Generate the content hash
        let hash = agent.compute_content_hash()?;
        agent.content_hash = Some(hash.clone());
        
        // Update the resource ID with the content hash
        let resource_bytes = hash.as_bytes().to_vec();
        agent.resource_id = ResourceId::new(ResourceType::Agent, resource_bytes);
        
        // Update the agent ID with the new resource ID
        agent.agent_id = AgentId::new(agent.resource_id.clone(), agent.agent_type.clone());
        
        Ok(agent)
    }
    
    /// Compute the content hash of the agent
    fn compute_content_hash(&self) -> Result<ContentHash, AgentError> {
        // Create a serializable view of the agent for content addressing
        let content_view = AgentContentView {
            agent_type: self.agent_type.clone(),
            state: self.state.clone(),
            capabilities: self.capabilities.clone(),
            relationships: self.relationships.clone(),
            metadata: self.metadata.clone(),
        };
        
        // Compute the content hash
        let hash = content_view.content_hash()
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;
        
        Ok(hash)
    }
}

/// View of agent data for content addressing
#[derive(Serialize, Deserialize)]
struct AgentContentView {
    agent_type: AgentType,
    state: AgentState,
    capabilities: Vec<Capability<Resource>>,
    relationships: Vec<AgentRelationship>,
    metadata: HashMap<String, String>,
}

// Custom implementation for content addressing
impl AgentContentView {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        ContentHash::for_object(self)
    }
}

impl crate::resource::Resource for AgentImpl {
    fn id(&self) -> crate::resource_types::ResourceId {
        self.resource_id.clone()
    }
    
    fn resource_type(&self) -> crate::resource_types::ResourceType {
        crate::resource_types::ResourceType::new("agent", "1.0")
    }
    
    fn state(&self) -> crate::resource::ResourceState {
        match &self.state {
            AgentState::Active => crate::resource::ResourceState::Active,
            AgentState::Inactive => crate::resource::ResourceState::Frozen,
            AgentState::Suspended { .. } => crate::resource::ResourceState::Locked,
        }
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> crate::resource::ResourceResult<()> {
        self.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn clone_resource(&self) -> Box<dyn crate::resource::Resource> {
        Box::new(self.clone())
    }
}

#[async_trait]
impl Agent for AgentImpl {
    fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    
    fn agent_type(&self) -> &AgentType {
        &self.agent_type
    }
    
    fn state(&self) -> &AgentState {
        &self.state
    }
    
    async fn set_state(&mut self, state: AgentState) -> Result<(), AgentError> {
        self.state = state;
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    async fn add_capability(&mut self, capability: Capability<Resource>) -> Result<(), AgentError> {
        // Check if the capability already exists
        if self.capabilities.iter().any(|c| c.id() == capability.id()) {
            return Err(AgentError::Other(format!(
                "Capability with ID {} already exists", capability.id()
            )));
        }
        
        // Add the capability
        self.capabilities.push(capability);
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError> {
        // Find the capability
        let position = self.capabilities.iter()
            .position(|c| c.id() == capability_id)
            .ok_or_else(|| AgentError::Other(format!(
                "Capability with ID {} not found", capability_id
            )))?;
        
        // Remove the capability
        self.capabilities.remove(position);
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    fn has_capability(&self, capability_id: &str) -> bool {
        self.capabilities.iter().any(|c| c.id() == capability_id)
    }
    
    fn capabilities(&self) -> Vec<Capability<Resource>> {
        self.capabilities.clone()
    }
    
    async fn add_relationship(&mut self, relationship: AgentRelationship) -> Result<(), AgentError> {
        // Check if the relationship already exists
        if self.relationships.iter().any(|r| r.target_resource_id() == relationship.target_resource_id()) {
            return Err(AgentError::Other(format!(
                "Relationship with target {} already exists", relationship.target_resource_id()
            )));
        }
        
        // Add the relationship
        self.relationships.push(relationship);
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    async fn remove_relationship(&mut self, target_id: &ResourceId) -> Result<(), AgentError> {
        // Find the relationship
        let position = self.relationships.iter()
            .position(|r| r.target_resource_id() == target_id)
            .ok_or_else(|| AgentError::Other(format!(
                "Relationship with target {} not found", target_id
            )))?;
        
        // Remove the relationship
        self.relationships.remove(position);
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    fn relationships(&self) -> Vec<AgentRelationship> {
        self.relationships.clone()
    }
    
    fn get_relationship(&self, target_id: &ResourceId) -> Option<&AgentRelationship> {
        self.relationships.iter()
            .find(|r| r.target_resource_id() == target_id)
    }
    
    fn clone_agent(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}

impl AgentImpl {
    /// Get the content hash
    pub fn content_hash(&self) -> anyhow::Result<ContentHash> {
        if let Some(hash) = &self.content_hash {
            Ok(hash.clone())
        } else {
            self.compute_content_hash()
                .map_err(|e| anyhow::anyhow!("Failed to compute content hash: {}", e))
        }
    }
}

/// Agent builder for creating agents
pub struct AgentBuilder {
    agent_type: AgentType,
    state: AgentState,
    capabilities: Vec<Capability<Resource>>,
    relationships: Vec<AgentRelationship>,
    metadata: HashMap<String, String>,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new() -> Self {
        Self {
            agent_type: AgentType::User, // Default to user agent
            state: AgentState::Inactive,
            capabilities: Vec::new(),
            relationships: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the agent type
    pub fn agent_type(mut self, agent_type: AgentType) -> Self {
        self.agent_type = agent_type;
        self
    }
    
    /// Set the initial state
    pub fn state(mut self, state: AgentState) -> Self {
        self.state = state;
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: Capability<Resource>) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability<Resource>>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }
    
    /// Add a relationship
    pub fn with_relationship(mut self, relationship: AgentRelationship) -> Self {
        self.relationships.push(relationship);
        self
    }
    
    /// Add multiple relationships
    pub fn with_relationships(mut self, relationships: Vec<AgentRelationship>) -> Self {
        self.relationships.extend(relationships);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Build the agent
    pub fn build(self) -> Result<AgentImpl, AgentError> {
        AgentImpl::new(
            self.agent_type,
            Some(self.state),
            Some(self.capabilities),
            Some(self.relationships),
            Some(self.metadata),
        )
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_creation() {
        let agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        assert_eq!(agent.agent_type(), &AgentType::User);
        assert_eq!(agent.state(), &AgentState::Active);
        assert!(agent.capabilities().is_empty());
        assert!(agent.relationships().is_empty());
    }
    
    #[test]
    fn test_content_addressing() {
        let agent1 = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        let agent2 = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        // Same content should produce same hash
        let hash1 = agent1.content_hash().unwrap();
        let hash2 = agent2.content_hash().unwrap();
        assert_eq!(hash1, hash2);
        
        // Different content should produce different hash
        let agent3 = AgentBuilder::new()
            .agent_type(AgentType::Committee)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        let hash3 = agent3.content_hash().unwrap();
        assert_ne!(hash1, hash3);
    }
    
    #[tokio::test]
    async fn test_capability_management() {
        let mut agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .build()
            .unwrap();
        
        // Add a capability
        let capability = Capability::new("test", "read", None);
        agent.add_capability(capability.clone()).await.unwrap();
        
        // Check if the agent has the capability
        assert!(agent.has_capability("test"));
        assert_eq!(agent.capabilities().len(), 1);
        
        // Remove the capability
        agent.remove_capability("test").await.unwrap();
        assert!(!agent.has_capability("test"));
        assert_eq!(agent.capabilities().len(), 0);
    }
    
    #[tokio::test]
    async fn test_relationship_management() {
        let mut agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .build()
            .unwrap();
        
        // Create a target resource ID
        let target_id = ResourceId::new(ResourceType::Document, vec![1, 2, 3, 4]);
        
        // Add a relationship
        let relationship = AgentRelationship::new(
            super::types::RelationshipType::Owns,
            target_id.clone(),
            vec![],
            HashMap::new(),
        );
        
        agent.add_relationship(relationship.clone()).await.unwrap();
        
        // Check if the relationship was added
        assert_eq!(agent.relationships().len(), 1);
        let retrieved = agent.get_relationship(&target_id).unwrap();
        assert_eq!(retrieved.relationship_type(), &super::types::RelationshipType::Owns);
        
        // Remove the relationship
        agent.remove_relationship(&target_id).await.unwrap();
        assert_eq!(agent.relationships().len(), 0);
        assert!(agent.get_relationship(&target_id).is_none());
    }
} 