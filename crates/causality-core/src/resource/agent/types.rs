// types.rs - Core type definitions for the agent system
//
// This file defines the fundamental types used by the agent resource system,
// following the design outlined in ADR-032.

use crate::resource::{ResourceId, ResourceError};
use causality_error::Error as CoreError;
use crate::resource::operation::Capability;
use crate::resource::agent::operation::IdentityId;
use causality_types::ContentId;
use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::HashMap;

/// Unique identifier for an agent
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AgentId {
    /// The resource ID of the agent
    resource_id: ResourceId,
    
    /// The agent type
    agent_type: AgentType,
}

impl AgentId {
    /// Create a new agent ID
    pub fn new(resource_id: ResourceId, agent_type: AgentType) -> Self {
        Self {
            resource_id,
            agent_type,
        }
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    /// Get the agent type
    pub fn agent_type(&self) -> &AgentType {
        &self.agent_type
    }
    
    /// Create an agent ID from a content hash
    pub fn from_content_hash(hash: &[u8], agent_type: AgentType) -> Self {
        // Create ContentId from bytes
        let content_id = ContentId::from_bytes(hash);
        let resource_id = ResourceId::from_content_id(&content_id)
            .unwrap_or_else(|_| panic!("Failed to create ResourceId from ContentId"));
        
        AgentId {
            resource_id,
            agent_type,
        }
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.agent_type, self.resource_id)
    }
}

impl FromStr for AgentId {
    type Err = AgentError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(AgentError::InvalidAgentId(
                "Agent ID must be in the format 'type:resource_id'".to_string()
            ));
        }
        
        let agent_type = AgentType::from_str(parts[0])
            .map_err(|_| AgentError::InvalidAgentType(parts[0].to_string()))?;
        
        let resource_id = ResourceId::from_str(parts[1])
            .map_err(|e| AgentError::ResourceError(format!("Invalid resource ID: {}", e)))?;
        
        Ok(Self {
            resource_id,
            agent_type,
        })
    }
}

/// Types of agents in the system
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// Human user of the system
    User,
    
    /// Multi-agent decision-making body
    Committee,
    
    /// Automated system operator
    Operator,

    /// Leader of a committee
    Leader,

    /// Validator in a committee
    Validator,
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Committee => write!(f, "committee"),
            Self::Operator => write!(f, "operator"),
            Self::Leader => write!(f, "leader"),
            Self::Validator => write!(f, "validator"),
        }
    }
}

impl FromStr for AgentType {
    type Err = AgentError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Self::User),
            "committee" => Ok(Self::Committee),
            "operator" => Ok(Self::Operator),
            "leader" => Ok(Self::Leader),
            "validator" => Ok(Self::Validator),
            _ => Err(AgentError::InvalidAgentType(s.to_string())),
        }
    }
}

/// State of an agent resource
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is active and can perform operations
    Active,
    
    /// Agent is inactive and cannot perform operations
    Inactive,
    
    /// Agent is suspended and cannot perform operations
    Suspended {
        /// Reason for suspension
        reason: String,
        
        /// When the suspension occurred
        timestamp: u64,
    },
}

impl Default for AgentState {
    fn default() -> Self {
        Self::Inactive
    }
}

/// A relationship between an agent and another resource
#[derive(Debug, Clone)]
pub struct AgentRelationship {
    /// The type of relationship
    relationship_type: RelationshipType,
    
    /// The target resource ID
    target_resource_id: ResourceId,
    
    /// Capabilities granted by this relationship (stored as strings)
    capability_ids: Vec<String>,
    
    /// Additional metadata about the relationship
    metadata: HashMap<String, String>,
}

impl AgentRelationship {
    /// Create a new agent relationship
    pub fn new(
        relationship_type: RelationshipType,
        target_resource_id: ResourceId,
        capability_ids: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            relationship_type,
            target_resource_id,
            capability_ids,
            metadata,
        }
    }
    
    /// Get the relationship type
    pub fn relationship_type(&self) -> &RelationshipType {
        &self.relationship_type
    }
    
    /// Get the target resource ID
    pub fn target_resource_id(&self) -> &ResourceId {
        &self.target_resource_id
    }
    
    /// Get the capability IDs
    pub fn capability_ids(&self) -> &[String] {
        &self.capability_ids
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Types of relationships between agents and resources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Agent owns the resource
    Owns,
    
    /// Agent is a parent of the resource
    Parent,
    
    /// Agent is a child of the resource
    Child,
    
    /// Agent is a peer of the resource
    Peer,
    
    /// Agent delegates to the resource
    Delegate,
    
    /// Agent depends on the resource
    DependsOn,
    
    /// Custom relationship type
    Custom(String),
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owns => write!(f, "owns"),
            Self::Parent => write!(f, "parent"),
            Self::Child => write!(f, "child"),
            Self::Peer => write!(f, "peer"),
            Self::Delegate => write!(f, "delegate"),
            Self::DependsOn => write!(f, "depends_on"),
            Self::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl FromStr for RelationshipType {
    type Err = AgentError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owns" => Ok(Self::Owns),
            "parent" => Ok(Self::Parent),
            "child" => Ok(Self::Child),
            "peer" => Ok(Self::Peer),
            "delegate" => Ok(Self::Delegate),
            "depends_on" => Ok(Self::DependsOn),
            _ => {
                if s.starts_with("custom:") {
                    let name = s.trim_start_matches("custom:").to_string();
                    Ok(Self::Custom(name))
                } else {
                    Err(AgentError::InvalidRelationshipType(s.to_string()))
                }
            }
        }
    }
}

/// Errors that can occur in the agent system
#[derive(Debug, Error)]
pub enum AgentError {
    /// Invalid agent ID format
    #[error("Invalid agent ID: {0}")]
    InvalidAgentId(String),
    
    /// Invalid agent type
    #[error("Invalid agent type: {0}")]
    InvalidAgentType(String),
    
    /// Invalid relationship type
    #[error("Invalid relationship type: {0}")]
    InvalidRelationshipType(String),
    
    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    /// Agent already exists
    #[error("Agent already exists: {0}")]
    AgentAlreadyExists(String),
    
    /// Agent is inactive
    #[error("Agent is inactive: {0}")]
    AgentInactive(String),
    
    /// Agent is suspended
    #[error("Agent is suspended: {0}")]
    AgentSuspended(String),
    
    /// Operation not permitted
    #[error("Operation not permitted: {0}")]
    OperationNotPermitted(String),
    
    /// Missing capability
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Other errors
    #[error("Agent error: {0}")]
    Other(String),
}

impl From<ResourceError> for AgentError {
    fn from(err: ResourceError) -> Self {
        Self::ResourceError(err.to_string())
    }
}

impl From<CoreError> for AgentError {
    fn from(err: CoreError) -> Self {
        Self::SerializationError(err.to_string())
    }
}

pub struct AgentCapabilities {
    /// Agent's main identity 
    identity: IdentityId,
    
    /// Capabilities granted to this agent
    capabilities: Vec<Capability<dyn std::any::Any + Send + Sync>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAgentRelationship {
    /// The type of relationship
    pub relationship_type: RelationshipType,
    
    /// The target resource ID
    pub target_resource_id: ResourceId,
    
    /// Capabilities granted by this relationship (stored as strings)
    pub capability_ids: Vec<String>,
    
    /// Additional metadata about the relationship
    pub metadata: HashMap<String, String>,
}

impl SerializableAgentRelationship {
    /// Get the relationship type
    pub fn relationship_type(&self) -> &RelationshipType {
        &self.relationship_type
    }
    
    /// Get the target resource ID
    pub fn target_resource_id(&self) -> &ResourceId {
        &self.target_resource_id
    }
    
    /// Get the capability IDs
    pub fn capability_ids(&self) -> &[String] {
        &self.capability_ids
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl From<AgentRelationship> for SerializableAgentRelationship {
    fn from(r: AgentRelationship) -> Self {
        Self {
            relationship_type: r.relationship_type,
            target_resource_id: r.target_resource_id,
            capability_ids: r.capability_ids,
            metadata: r.metadata,
        }
    }
}

impl From<SerializableAgentRelationship> for AgentRelationship {
    fn from(s: SerializableAgentRelationship) -> Self {
        Self {
            relationship_type: s.relationship_type,
            target_resource_id: s.target_resource_id,
            capability_ids: s.capability_ids,
            metadata: s.metadata,
        }
    }
}

impl From<&SerializableAgentRelationship> for AgentRelationship {
    fn from(s: &SerializableAgentRelationship) -> Self {
        Self {
            relationship_type: s.relationship_type.clone(),
            target_resource_id: s.target_resource_id.clone(),
            capability_ids: s.capability_ids.clone(),
            metadata: s.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::content_addressing;
    
    #[test]
    fn test_agent_id_display_and_parse() {
        // Create a content ID for testing
        let content_id = ContentId::new("test-content-id");
        let resource_id = ResourceId::from_content_id(&content_id).unwrap();
        let agent_id = AgentId::new(resource_id, AgentType::User);
        
        // Test display
        let display_str = format!("{}", agent_id);
        
        // Test parsing
        let parsed = AgentId::from_str(&display_str).unwrap();
        assert_eq!(parsed.agent_type(), &AgentType::User);
        assert_eq!(parsed.resource_id(), agent_id.resource_id());
    }
    
    #[test]
    fn test_agent_state_default() {
        let state = AgentState::default();
        assert_eq!(state, AgentState::Inactive);
    }
    
    #[test]
    fn test_relationship_type_display_and_parse() {
        let relationship_type = RelationshipType::Owns;
        let relationship_type_str = relationship_type.to_string();
        let parsed_relationship_type = RelationshipType::from_str(&relationship_type_str).unwrap();
        
        assert_eq!(relationship_type, parsed_relationship_type);
        
        let custom_type = RelationshipType::Custom("test".to_string());
        let custom_type_str = custom_type.to_string();
        let parsed_custom_type = RelationshipType::from_str(&custom_type_str).unwrap();
        
        if let RelationshipType::Custom(name) = parsed_custom_type {
            assert_eq!(name, "test");
        } else {
            panic!("Expected RelationshipType::Custom");
        }
    }
} 