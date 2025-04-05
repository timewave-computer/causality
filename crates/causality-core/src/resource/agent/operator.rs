// operator.rs - Operator agent implementation
//
// This file implements the specialized OperatorAgent type, representing system operators.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::{Resource, ResourceState, ResourceResult};
use crate::resource::operation::Capability;

use super::types::{AgentId, AgentType, AgentState, AgentError, AgentRelationship};
use super::agent::{Agent, AgentImpl};

use async_trait::async_trait;
use thiserror::Error;

/// Operator-specific error types
#[derive(Error, Debug)]
pub enum OperatorAgentError {
    /// Base agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Other error
    #[error("Operator error: {0}")]
    Other(String),
}

/// Operator agent that specializes the base agent for operator-specific functionality
#[derive(Clone, Debug)]
pub struct OperatorAgent {
    /// Base agent implementation
    base: AgentImpl,
}

impl OperatorAgent {
    /// Create a new operator agent
    pub fn new(base_agent: AgentImpl) -> Result<Self, OperatorAgentError> {
        // Verify that the base agent has the correct type
        if base_agent.agent_type() != &AgentType::Operator {
            return Err(OperatorAgentError::Other(
                "Base agent must have Operator agent type".to_string()
            ));
        }
        
        Ok(Self {
            base: base_agent,
        })
    }
}

// Delegate the Agent trait methods to the base agent
#[async_trait]
impl Agent for OperatorAgent {
    fn agent_id(&self) -> &AgentId {
        self.base.agent_id()
    }
    
    fn agent_type(&self) -> &AgentType {
        self.base.agent_type()
    }
    
    fn state(&self) -> &AgentState {
        Agent::state(&self.base)
    }
    
    async fn set_state(&mut self, state: AgentState) -> Result<(), AgentError> {
        self.base.set_state(state).await
    }
    
    async fn add_capability(&mut self, capability: Capability<Box<dyn Resource>>) -> Result<(), AgentError> {
        self.base.add_capability(capability).await
    }
    
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError> {
        self.base.remove_capability(capability_id).await
    }
    
    fn has_capability(&self, capability_id: &str) -> bool {
        self.base.has_capability(capability_id)
    }
    
    fn capabilities(&self) -> Vec<Capability<Box<dyn Resource>>> {
        self.base.capabilities()
    }
    
    async fn add_relationship(&mut self, relationship: AgentRelationship) -> Result<(), AgentError> {
        self.base.add_relationship(relationship).await
    }
    
    async fn remove_relationship(&mut self, target_id: &ResourceId) -> Result<(), AgentError> {
        self.base.remove_relationship(target_id).await
    }
    
    fn relationships(&self) -> Vec<AgentRelationship> {
        self.base.relationships()
    }
    
    fn get_relationship(&self, target_id: &ResourceId) -> Option<&AgentRelationship> {
        self.base.get_relationship(target_id)
    }
    
    fn clone_agent(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}

// Implement the Resource trait through delegation to the base agent
impl Resource for OperatorAgent {
    fn id(&self) -> ResourceId {
        self.base.id().clone()
    }
    
    fn resource_type(&self) -> ResourceType {
        ResourceType::new("Agent", "1.0")
    }
    
    fn state(&self) -> ResourceState {
        match Agent::state(&self.base) {
            &AgentState::Active => ResourceState::Active,
            &AgentState::Inactive => ResourceState::Created,
            &AgentState::Suspended { .. } => ResourceState::Locked,
        }
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.base.get_metadata(key)
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()> {
        self.base.set_metadata(key, value)
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

/// Builder for creating operator agents
pub struct OperatorAgentBuilder {
    /// Base agent builder
    base_builder: super::agent::AgentBuilder,
}

impl OperatorAgentBuilder {
    /// Create a new operator agent builder
    pub fn new() -> Self {
        Self {
            base_builder: super::agent::AgentBuilder::new().agent_type(AgentType::Operator),
        }
    }
    
    /// Set the agent state
    pub fn state(mut self, state: AgentState) -> Self {
        self.base_builder = self.base_builder.state(state);
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: Capability<Box<dyn Resource>>) -> Self {
        self.base_builder = self.base_builder.with_capability(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability<Box<dyn Resource>>>) -> Self {
        self.base_builder = self.base_builder.with_capabilities(capabilities);
        self
    }
    
    /// Add a relationship
    pub fn with_relationship(mut self, relationship: AgentRelationship) -> Self {
        self.base_builder = self.base_builder.with_relationship(relationship);
        self
    }
    
    /// Add multiple relationships
    pub fn with_relationships(mut self, relationships: Vec<AgentRelationship>) -> Self {
        self.base_builder = self.base_builder.with_relationships(relationships);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.base_builder = self.base_builder.with_metadata(key, value);
        self
    }
    
    /// Build the operator agent
    pub fn build(self) -> Result<OperatorAgent, OperatorAgentError> {
        // Build the base agent
        let base_agent = self.base_builder.build()
            .map_err(OperatorAgentError::AgentError)?;
        
        // Create the operator agent
        OperatorAgent::new(base_agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_operator_creation() {
        // Create an operator agent
        let operator = OperatorAgentBuilder::new()
            .state(AgentState::Active)
            .build()
            .unwrap();
        
        // Check the agent type
        assert_eq!(operator.agent_type(), &AgentType::Operator);
    }
} 