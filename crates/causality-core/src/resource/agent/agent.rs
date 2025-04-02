// agent.rs - Core Agent trait and implementation

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::interface::{ResourceState, ResourceResult};
use crate::resource::Resource;
use crate::utils::content_addressing;
use crate::serialization::{SerializationError, Serializable, Serializer};
use causality_error::Error as CoreError;
use crate::resource::operation::Capability;

use super::types::{AgentId, AgentType, AgentState, AgentRelationship, AgentError, SerializableAgentRelationship};
use super::authorization::{Authorization, AuthorizationError};
use super::operation::Operation;

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::any::Any;
use causality_types::ContentId;
use anyhow::Result;
use causality_types::ContentHash;

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
    async fn add_capability(&mut self, capability: Capability<Box<dyn Resource>>) -> Result<(), AgentError>;
    
    /// Remove a capability from the agent
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError>;
    
    /// Check if the agent has a capability
    fn has_capability(&self, capability_id: &str) -> bool;
    
    /// Get all capabilities
    fn capabilities(&self) -> Vec<Capability<Box<dyn Resource>>>;
    
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

/// Serializable wrapper for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableCapability {
    id: String,
    capability_type: String,
    metadata: HashMap<String, String>,
}

impl SerializableCapability {
    fn from_capability(capability: &Capability<Box<dyn Resource>>) -> Self {
        Self {
            id: capability.id().to_string(),
            capability_type: "capability".to_string(),
            metadata: HashMap::new(),
        }
    }
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
    
    /// The agent's capabilities (serialized as IDs only)
    #[serde(skip)]
    capabilities: Vec<Capability<Box<dyn Resource>>>,
    
    /// Serializable capability data
    serializable_capabilities: Vec<SerializableCapability>,
    
    /// Relationships with other resources
    relationships: Vec<SerializableAgentRelationship>,
    
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
        initial_capabilities: Option<Vec<Capability<Box<dyn Resource>>>>,
        initial_relationships: Option<Vec<AgentRelationship>>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Self, AgentError> {
        // Create a temporary ID with default content hash
        let temp_hash = content_addressing::default_content_hash();
        let resource_id = ResourceId::new(temp_hash);
        
        // Create the agent ID
        let agent_id = AgentId::new(resource_id.clone(), agent_type.clone());
        
        // Convert relationships to serializable form
        let serializable_relationships = initial_relationships
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.into())
            .collect();
            
        // Convert capabilities to serializable form
        let capabilities = initial_capabilities.unwrap_or_default();
        let serializable_capabilities = capabilities
            .iter()
            .map(|c| SerializableCapability::from_capability(c))
            .collect();
        
        // Create the agent
        let mut agent = Self {
            resource_id,
            agent_id,
            agent_type,
            state: initial_state.unwrap_or_default(),
            capabilities,
            serializable_capabilities,
            relationships: serializable_relationships,
            metadata: metadata.unwrap_or_default(),
            content_hash: None,
        };
        
        // Generate the content hash
        let hash = agent.compute_content_hash()?;
        agent.content_hash = Some(hash.clone());
        
        // Update the resource ID with the content hash
        agent.resource_id = ResourceId::new(hash);
        
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
            serializable_capabilities: self.serializable_capabilities.clone(),
            relationships: self.relationships.clone(),
            metadata: self.metadata.clone(),
        };
        
        // Compute the content hash
        content_view.content_hash()
            .map_err(|e| AgentError::SerializationError(e.to_string()))
    }
}

/// View of agent data for content addressing
#[derive(Serialize, Deserialize)]
struct AgentContentView {
    agent_type: AgentType,
    state: AgentState,
    serializable_capabilities: Vec<SerializableCapability>,
    relationships: Vec<SerializableAgentRelationship>,
    metadata: HashMap<String, String>,
}

// Custom implementation for content addressing
impl AgentContentView {
    fn content_hash(&self) -> anyhow::Result<ContentHash> {
        content_addressing::hash_object(self)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl crate::resource::Resource for AgentImpl {
    fn id(&self) -> ResourceId {
        self.agent_id.resource_id().clone()
    }
    
    fn resource_type(&self) -> ResourceType {
        ResourceType::new("Agent", "1.0")
    }
    
    fn state(&self) -> ResourceState {
        match self.state {
            AgentState::Active => ResourceState::Active,
            AgentState::Inactive => ResourceState::Created,
            AgentState::Suspended { .. } => ResourceState::Frozen,
        }
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> causality_types::Result<(), crate::resource::ResourceError> {
        self.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn clone_resource(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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
    
    async fn add_capability(&mut self, capability: Capability<Box<dyn Resource>>) -> Result<(), AgentError> {
        // Check if the capability already exists
        if self.capabilities.iter().any(|c| c.id() == capability.id()) {
            return Err(AgentError::Other(format!(
                "Capability with ID {} already exists", capability.id()
            )));
        }
        
        // Add the capability
        self.capabilities.push(capability.clone());
        
        // Add to serializable capabilities
        self.serializable_capabilities.push(SerializableCapability::from_capability(&capability));
        
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
        
        // Find and remove from serializable capabilities
        if let Some(pos) = self.serializable_capabilities.iter()
            .position(|c| c.id == capability_id) {
            self.serializable_capabilities.remove(pos);
        }
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    fn has_capability(&self, capability_id: &str) -> bool {
        self.capabilities.iter().any(|c| c.id() == capability_id)
    }
    
    fn capabilities(&self) -> Vec<Capability<Box<dyn Resource>>> {
        self.capabilities.clone()
    }
    
    async fn add_relationship(&mut self, relationship: AgentRelationship) -> Result<(), AgentError> {
        // Check if the relationship already exists
        if self.relationships.iter().any(|r| {
            let r = AgentRelationship::from(r.clone());
            r.target_resource_id() == relationship.target_resource_id()
        }) {
            return Err(AgentError::Other(format!(
                "Relationship with target resource ID {} already exists", 
                relationship.target_resource_id()
            )));
        }
        
        // Add the relationship
        self.relationships.push(relationship.into());
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    async fn remove_relationship(&mut self, target_id: &ResourceId) -> Result<(), AgentError> {
        // Find the relationship
        let position = self.relationships.iter().position(|r| {
            let r = AgentRelationship::from(r.clone());
            r.target_resource_id() == target_id
        }).ok_or_else(|| AgentError::Other(format!(
            "Relationship with target resource ID {} not found", target_id
        )))?;
        
        // Remove the relationship
        self.relationships.remove(position);
        
        // Recompute the content hash
        let hash = self.compute_content_hash()?;
        self.content_hash = Some(hash);
        
        Ok(())
    }
    
    fn relationships(&self) -> Vec<AgentRelationship> {
        self.relationships.iter()
            .map(|r| AgentRelationship::from(r.clone()))
            .collect()
    }
    
    fn get_relationship(&self, target_id: &ResourceId) -> Option<&AgentRelationship> {
        None
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
    capabilities: Vec<Capability<Box<dyn Resource>>>,
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
    pub fn with_capability(mut self, capability: Capability<Box<dyn Resource>>) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability<Box<dyn Resource>>>) -> Self {
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
    use crate::resource::agent::types::{RelationshipType, AgentType, AgentState};
    
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
        let test_hash = content_addressing::hash_bytes(&[1, 2, 3, 4]);
        let target_id = ResourceId::new(test_hash);
        
        // Add a relationship
        let relationship = AgentRelationship::new(
            RelationshipType::Owns,
            target_id.clone(),
            vec![],
            HashMap::new(),
        );
        
        agent.add_relationship(relationship.clone()).await.unwrap();
        
        // Check if the relationship was added
        assert_eq!(agent.relationships().len(), 1);
        let retrieved = agent.relationships().first().unwrap(); // Use this since get_relationship now returns None
        assert_eq!(retrieved.relationship_type(), &RelationshipType::Owns);
        
        // Remove the relationship
        agent.remove_relationship(&target_id).await.unwrap();
        assert_eq!(agent.relationships().len(), 0);
        assert!(agent.get_relationship(&target_id).is_none());
    }
} 
