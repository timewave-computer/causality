// registry.rs - Agent registry for managing agent resources
//
// This file implements the agent registry for managing agents in the system.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::ResourceError;
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::serialization::{Serializable, DeserializationError};

use super::types::{AgentId, AgentType, AgentState, AgentError};
use super::agent::{Agent, AgentImpl};
use super::authorization::{Authorization, AuthorizationError, CapabilityRegistry};

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, RwLock, Mutex};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::RwLock as AsyncRwLock;

/// Agent registry error types
#[derive(Error, Debug)]
pub enum AgentRegistryError {
    /// Agent not found
    #[error("Agent not found: {0}")]
    NotFound(String),
    
    /// Agent already exists
    #[error("Agent already exists: {0}")]
    AlreadyExists(String),
    
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(#[from] AuthorizationError),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(#[from] ResourceError),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Lock error
    #[error("Lock error: {0}")]
    LockError(String),
    
    /// Other error
    #[error("Registry error: {0}")]
    Other(String),
}

/// Result type for agent registry operations
pub type AgentRegistryResult<T> = Result<T, AgentRegistryError>;

/// Agent registry for managing agents
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    /// Register a new agent
    async fn register_agent<A: Agent + Send + Sync + 'static>(&self, agent: A) -> AgentRegistryResult<AgentId>;
    
    /// Get an agent by ID
    async fn get_agent(&self, agent_id: &AgentId) -> AgentRegistryResult<Box<dyn Agent>>;
    
    /// Update an agent
    async fn update_agent<A: Agent + Send + Sync + 'static>(&self, agent: A) -> AgentRegistryResult<()>;
    
    /// Delete an agent
    async fn delete_agent(&self, agent_id: &AgentId) -> AgentRegistryResult<()>;
    
    /// List all agents
    async fn list_agents(&self) -> AgentRegistryResult<Vec<AgentId>>;
    
    /// List agents by type
    async fn list_agents_by_type(&self, agent_type: &AgentType) -> AgentRegistryResult<Vec<AgentId>>;
    
    /// Find agents with capability
    async fn find_agents_with_capability(&self, capability_id: &str) -> AgentRegistryResult<Vec<AgentId>>;
    
    /// Check if an agent exists
    async fn exists(&self, agent_id: &AgentId) -> AgentRegistryResult<bool>;
}

/// In-memory implementation of agent registry
pub struct InMemoryAgentRegistry {
    agents: AsyncRwLock<HashMap<AgentId, Box<dyn Agent>>>,
    capability_index: AsyncRwLock<HashMap<String, HashSet<AgentId>>>,
    type_index: AsyncRwLock<HashMap<AgentType, HashSet<AgentId>>>,
    capability_registry: Arc<CapabilityRegistry>,
}

impl InMemoryAgentRegistry {
    /// Create a new in-memory agent registry
    pub fn new(capability_registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            agents: AsyncRwLock::new(HashMap::new()),
            capability_index: AsyncRwLock::new(HashMap::new()),
            type_index: AsyncRwLock::new(HashMap::new()),
            capability_registry,
        }
    }
    
    /// Index an agent by its capabilities
    async fn index_by_capabilities(&self, agent: &dyn Agent) -> AgentRegistryResult<()> {
        let agent_id = agent.agent_id().clone();
        let capabilities = agent.capabilities();
        
        let mut capability_index = self.capability_index.write().await;
        
        for capability in capabilities {
            let capability_id = capability.id().to_string();
            let agents = capability_index.entry(capability_id).or_insert_with(HashSet::new);
            agents.insert(agent_id.clone());
        }
        
        Ok(())
    }
    
    /// Remove an agent from the capability index
    async fn remove_from_capability_index(&self, agent_id: &AgentId) -> AgentRegistryResult<()> {
        let mut capability_index = self.capability_index.write().await;
        
        for agents in capability_index.values_mut() {
            agents.remove(agent_id);
        }
        
        Ok(())
    }
    
    /// Index an agent by its type
    async fn index_by_type(&self, agent: &dyn Agent) -> AgentRegistryResult<()> {
        let agent_id = agent.agent_id().clone();
        let agent_type = agent.agent_type().clone();
        
        let mut type_index = self.type_index.write().await;
        let agents = type_index.entry(agent_type).or_insert_with(HashSet::new);
        agents.insert(agent_id);
        
        Ok(())
    }
    
    /// Remove an agent from the type index
    async fn remove_from_type_index(&self, agent_id: &AgentId) -> AgentRegistryResult<()> {
        let agent_type = {
            let agents = self.agents.read().await;
            let agent = agents.get(agent_id).ok_or_else(|| {
                AgentRegistryError::NotFound(format!("Agent not found: {}", agent_id))
            })?;
            agent.agent_type().clone()
        };
        
        let mut type_index = self.type_index.write().await;
        if let Some(agents) = type_index.get_mut(&agent_type) {
            agents.remove(agent_id);
        }
        
        Ok(())
    }
}

#[async_trait]
impl AgentRegistry for InMemoryAgentRegistry {
    async fn register_agent<A: Agent + Send + Sync + 'static>(&self, agent: A) -> AgentRegistryResult<AgentId> {
        let agent_id = agent.agent_id().clone();
        
        // Check if agent already exists
        let exists = self.exists(&agent_id).await?;
        if exists {
            return Err(AgentRegistryError::AlreadyExists(
                format!("Agent already exists: {}", agent_id)
            ));
        }
        
        // Store the agent
        let boxed_agent = Box::new(agent) as Box<dyn Agent>;
        
        // Index by capabilities
        self.index_by_capabilities(boxed_agent.as_ref()).await?;
        
        // Index by type
        self.index_by_type(boxed_agent.as_ref()).await?;
        
        // Add to main registry
        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), boxed_agent);
        
        Ok(agent_id)
    }
    
    async fn get_agent(&self, agent_id: &AgentId) -> AgentRegistryResult<Box<dyn Agent>> {
        let agents = self.agents.read().await;
        
        let agent = agents.get(agent_id).ok_or_else(|| {
            AgentRegistryError::NotFound(format!("Agent not found: {}", agent_id))
        })?;
        
        Ok(agent.clone_agent())
    }
    
    async fn update_agent<A: Agent + Send + Sync + 'static>(&self, agent: A) -> AgentRegistryResult<()> {
        let agent_id = agent.agent_id().clone();
        
        // Check if agent exists
        let exists = self.exists(&agent_id).await?;
        if !exists {
            return Err(AgentRegistryError::NotFound(
                format!("Agent not found: {}", agent_id)
            ));
        }
        
        // Remove from indexes
        self.remove_from_capability_index(&agent_id).await?;
        self.remove_from_type_index(&agent_id).await?;
        
        // Box the agent
        let boxed_agent = Box::new(agent) as Box<dyn Agent>;
        
        // Reindex
        self.index_by_capabilities(boxed_agent.as_ref()).await?;
        self.index_by_type(boxed_agent.as_ref()).await?;
        
        // Update in main registry
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, boxed_agent);
        
        Ok(())
    }
    
    async fn delete_agent(&self, agent_id: &AgentId) -> AgentRegistryResult<()> {
        // Check if agent exists
        let exists = self.exists(agent_id).await?;
        if !exists {
            return Err(AgentRegistryError::NotFound(
                format!("Agent not found: {}", agent_id)
            ));
        }
        
        // Remove from indexes
        self.remove_from_capability_index(agent_id).await?;
        self.remove_from_type_index(agent_id).await?;
        
        // Remove from main registry
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
        
        Ok(())
    }
    
    async fn list_agents(&self) -> AgentRegistryResult<Vec<AgentId>> {
        let agents = self.agents.read().await;
        let agent_ids = agents.keys().cloned().collect();
        Ok(agent_ids)
    }
    
    async fn list_agents_by_type(&self, agent_type: &AgentType) -> AgentRegistryResult<Vec<AgentId>> {
        let type_index = self.type_index.read().await;
        
        let agent_ids = match type_index.get(agent_type) {
            Some(agents) => agents.iter().cloned().collect(),
            None => Vec::new(),
        };
        
        Ok(agent_ids)
    }
    
    async fn find_agents_with_capability(&self, capability_id: &str) -> AgentRegistryResult<Vec<AgentId>> {
        let capability_index = self.capability_index.read().await;
        
        let agent_ids = match capability_index.get(capability_id) {
            Some(agents) => agents.iter().cloned().collect(),
            None => Vec::new(),
        };
        
        Ok(agent_ids)
    }
    
    async fn exists(&self, agent_id: &AgentId) -> AgentRegistryResult<bool> {
        let agents = self.agents.read().await;
        Ok(agents.contains_key(agent_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::agent::agent::AgentBuilder;
    
    #[tokio::test]
    async fn test_register_and_get_agent() {
        // Create a capability registry
        let capability_registry = Arc::new(CapabilityRegistry::new());
        
        // Create an agent registry
        let registry = InMemoryAgentRegistry::new(capability_registry);
        
        // Create an agent
        let agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .with_capability(Capability::new("read", "read", None))
            .build()
            .unwrap();
        
        let agent_id = agent.agent_id().clone();
        
        // Register the agent
        let registered_id = registry.register_agent(agent).await.unwrap();
        assert_eq!(registered_id, agent_id);
        
        // Get the agent
        let retrieved_agent = registry.get_agent(&agent_id).await.unwrap();
        assert_eq!(retrieved_agent.agent_id(), &agent_id);
    }
    
    #[tokio::test]
    async fn test_list_agents_by_type() {
        // Create a capability registry
        let capability_registry = Arc::new(CapabilityRegistry::new());
        
        // Create an agent registry
        let registry = InMemoryAgentRegistry::new(capability_registry);
        
        // Create a user agent
        let user_agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        // Create an operator agent
        let operator_agent = AgentBuilder::new()
            .agent_type(AgentType::Operator)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        // Register the agents
        registry.register_agent(user_agent).await.unwrap();
        registry.register_agent(operator_agent).await.unwrap();
        
        // List user agents
        let user_agents = registry.list_agents_by_type(&AgentType::User).await.unwrap();
        assert_eq!(user_agents.len(), 1);
        
        // List operator agents
        let operator_agents = registry.list_agents_by_type(&AgentType::Operator).await.unwrap();
        assert_eq!(operator_agents.len(), 1);
        
        // List committee agents (should be empty)
        let committee_agents = registry.list_agents_by_type(&AgentType::Committee).await.unwrap();
        assert_eq!(committee_agents.len(), 0);
    }
    
    #[tokio::test]
    async fn test_find_agents_with_capability() {
        // Create a capability registry
        let capability_registry = Arc::new(CapabilityRegistry::new());
        
        // Create an agent registry
        let registry = InMemoryAgentRegistry::new(capability_registry);
        
        // Create an agent with read capability
        let read_agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .with_capability(Capability::new("read", "read", None))
            .build()
            .unwrap();
        
        // Create an agent with write capability
        let write_agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .with_capability(Capability::new("write", "write", None))
            .build()
            .unwrap();
        
        // Register the agents
        registry.register_agent(read_agent).await.unwrap();
        registry.register_agent(write_agent).await.unwrap();
        
        // Find agents with read capability
        let read_agents = registry.find_agents_with_capability("read").await.unwrap();
        assert_eq!(read_agents.len(), 1);
        
        // Find agents with write capability
        let write_agents = registry.find_agents_with_capability("write").await.unwrap();
        assert_eq!(write_agents.len(), 1);
        
        // Find agents with execute capability (should be empty)
        let execute_agents = registry.find_agents_with_capability("execute").await.unwrap();
        assert_eq!(execute_agents.len(), 0);
    }
    
    #[tokio::test]
    async fn test_update_agent() {
        // Create a capability registry
        let capability_registry = Arc::new(CapabilityRegistry::new());
        
        // Create an agent registry
        let registry = InMemoryAgentRegistry::new(capability_registry);
        
        // Create an agent
        let mut agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        let agent_id = agent.agent_id().clone();
        
        // Register the agent
        registry.register_agent(agent.clone_agent()).await.unwrap();
        
        // Add a capability to the agent
        agent.add_capability(Capability::new("read", "read", None)).await.unwrap();
        
        // Update the agent
        registry.update_agent(agent).await.unwrap();
        
        // Get the updated agent
        let updated_agent = registry.get_agent(&agent_id).await.unwrap();
        
        // Check that the agent has the new capability
        assert!(updated_agent.has_capability("read"));
    }
    
    #[tokio::test]
    async fn test_delete_agent() {
        // Create a capability registry
        let capability_registry = Arc::new(CapabilityRegistry::new());
        
        // Create an agent registry
        let registry = InMemoryAgentRegistry::new(capability_registry);
        
        // Create an agent
        let agent = AgentBuilder::new()
            .agent_type(AgentType::User)
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        let agent_id = agent.agent_id().clone();
        
        // Register the agent
        registry.register_agent(agent).await.unwrap();
        
        // Verify the agent exists
        assert!(registry.exists(&agent_id).await.unwrap());
        
        // Delete the agent
        registry.delete_agent(&agent_id).await.unwrap();
        
        // Verify the agent no longer exists
        assert!(!registry.exists(&agent_id).await.unwrap());
    }
} 