// committee.rs - Committee agent implementation
//
// This file implements the specialized CommitteeAgent type, representing a multi-agent 
// decision-making body for validating external facts and messages.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::ResourceError;
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::serialization::{Serializable, DeserializationError};
use crate::effect::Effect;

use super::types::{AgentId, AgentType, AgentState, AgentRelationship, AgentError};
use super::agent::{Agent, AgentImpl};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Committee-specific error types
#[derive(Error, Debug)]
pub enum CommitteeAgentError {
    /// Base agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Member error
    #[error("Member error: {0}")]
    MemberError(String),
    
    /// Decision error
    #[error("Decision error: {0}")]
    DecisionError(String),
    
    /// Domain error
    #[error("Domain error: {0}")]
    DomainError(String),
    
    /// Other error
    #[error("Committee error: {0}")]
    Other(String),
}

/// Committee member information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitteeMember {
    /// Agent ID of the member
    pub agent_id: AgentId,
    
    /// Member role within the committee
    pub role: MemberRole,
    
    /// Weight of the member in voting decisions
    pub voting_weight: u32,
    
    /// Public key for verification
    pub public_key: String,
    
    /// Whether the member is active
    pub active: bool,
}

/// Member role within a committee
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemberRole {
    /// Committee leader
    Leader,
    
    /// Committee validator
    Validator,
    
    /// Committee observer (non-voting)
    Observer,
}

/// Configuration for a committee
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitteeConfig {
    /// Domain this committee validates
    pub domain: String,
    
    /// Quorum percentage required for decisions (0-100)
    pub quorum_percentage: u8,
    
    /// Maximum size of the committee
    pub max_size: usize,
    
    /// Minimum number of votes required
    pub min_votes: usize,
    
    /// Protocol version
    pub protocol_version: String,
}

/// Decision made by the committee
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitteeDecision {
    /// Decision ID
    pub decision_id: String,
    
    /// Topic of the decision
    pub topic: String,
    
    /// Description of the decision
    pub description: String,
    
    /// Votes for this decision
    pub votes: HashMap<AgentId, Vote>,
    
    /// Timestamp of the decision
    pub timestamp: u64,
    
    /// Result of the decision
    pub result: Option<DecisionResult>,
}

/// Vote in a committee decision
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    /// Agent ID that cast the vote
    pub agent_id: AgentId,
    
    /// Vote value
    pub value: VoteValue,
    
    /// Signature of the vote
    pub signature: Vec<u8>,
    
    /// Timestamp of the vote
    pub timestamp: u64,
}

/// Possible vote values
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteValue {
    /// Vote in favor
    Approve,
    
    /// Vote against
    Reject,
    
    /// Abstain from voting
    Abstain,
}

/// Result of a committee decision
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionResult {
    /// Decision was approved
    Approved,
    
    /// Decision was rejected
    Rejected,
    
    /// Not enough votes to reach a decision
    Inconclusive,
}

/// Committee agent that specializes the base agent for committee-specific functionality
#[derive(Clone, Debug)]
pub struct CommitteeAgent {
    /// Base agent implementation
    base: AgentImpl,
    
    /// Committee configuration
    config: CommitteeConfig,
    
    /// Committee members
    members: Vec<CommitteeMember>,
    
    /// Past decisions
    decisions: Vec<CommitteeDecision>,
    
    /// Active decision IDs
    active_decisions: HashSet<String>,
}

impl CommitteeAgent {
    /// Create a new committee agent
    pub fn new(
        base_agent: AgentImpl,
        config: CommitteeConfig,
        members: Option<Vec<CommitteeMember>>,
    ) -> Result<Self, CommitteeAgentError> {
        // Verify that the base agent has the correct type
        if base_agent.agent_type() != &AgentType::Committee {
            return Err(CommitteeAgentError::Other(
                "Base agent must have Committee agent type".to_string()
            ));
        }
        
        Ok(Self {
            base: base_agent,
            config,
            members: members.unwrap_or_default(),
            decisions: Vec::new(),
            active_decisions: HashSet::new(),
        })
    }
    
    /// Get the committee configuration
    pub fn config(&self) -> &CommitteeConfig {
        &self.config
    }
    
    /// Update the committee configuration
    pub async fn update_config(&mut self, config: CommitteeConfig) -> Result<(), CommitteeAgentError> {
        self.config = config;
        
        // Update the content hash
        self.base.set_metadata("config_updated", &chrono::Utc::now().to_rfc3339()).await
            .map_err(CommitteeAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Get the committee members
    pub fn members(&self) -> &[CommitteeMember] {
        &self.members
    }
    
    /// Add a member to the committee
    pub async fn add_member(&mut self, member: CommitteeMember) -> Result<(), CommitteeAgentError> {
        // Check if the committee is already at max size
        if self.members.len() >= self.config.max_size {
            return Err(CommitteeAgentError::MemberError(
                format!("Committee is already at maximum size of {}", self.config.max_size)
            ));
        }
        
        // Check if the member is already in the committee
        if self.members.iter().any(|m| m.agent_id == member.agent_id) {
            return Err(CommitteeAgentError::MemberError(
                format!("Member {} is already in the committee", member.agent_id)
            ));
        }
        
        // Add the member
        self.members.push(member);
        
        // Update the content hash
        self.base.set_metadata("member_added", &chrono::Utc::now().to_rfc3339()).await
            .map_err(CommitteeAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Remove a member from the committee
    pub async fn remove_member(&mut self, agent_id: &AgentId) -> Result<(), CommitteeAgentError> {
        // Find the member's index
        let position = self.members.iter().position(|m| &m.agent_id == agent_id)
            .ok_or_else(|| CommitteeAgentError::MemberError(
                format!("Member {} is not in the committee", agent_id)
            ))?;
        
        // Remove the member
        self.members.remove(position);
        
        // Update the content hash
        self.base.set_metadata("member_removed", &chrono::Utc::now().to_rfc3339()).await
            .map_err(CommitteeAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Start a new decision
    pub async fn start_decision(
        &mut self,
        topic: String,
        description: String,
    ) -> Result<String, CommitteeAgentError> {
        // Generate a decision ID
        let decision_id = crate::id_utils::generate_decision_id();
        
        // Create a new decision
        let decision = CommitteeDecision {
            decision_id: decision_id.clone(),
            topic,
            description,
            votes: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            result: None,
        };
        
        // Add the decision
        self.decisions.push(decision);
        self.active_decisions.insert(decision_id.clone());
        
        // Update the content hash
        self.base.set_metadata("decision_started", &chrono::Utc::now().to_rfc3339()).await
            .map_err(CommitteeAgentError::AgentError)?;
        
        Ok(decision_id)
    }
    
    /// Cast a vote in a decision
    pub async fn cast_vote(
        &mut self,
        decision_id: &str,
        agent_id: AgentId,
        value: VoteValue,
        signature: Vec<u8>,
    ) -> Result<(), CommitteeAgentError> {
        // Check if this is an active decision
        if !self.active_decisions.contains(decision_id) {
            return Err(CommitteeAgentError::DecisionError(
                format!("Decision {} is not active", decision_id)
            ));
        }
        
        // Check if the agent is a committee member
        let member = self.members.iter().find(|m| m.agent_id == agent_id)
            .ok_or_else(|| CommitteeAgentError::MemberError(
                format!("Agent {} is not a committee member", agent_id)
            ))?;
        
        // Check if the member is active
        if !member.active {
            return Err(CommitteeAgentError::MemberError(
                format!("Member {} is not active", agent_id)
            ));
        }
        
        // Find the decision
        let decision = self.decisions.iter_mut()
            .find(|d| d.decision_id == decision_id)
            .ok_or_else(|| CommitteeAgentError::DecisionError(
                format!("Decision {} not found", decision_id)
            ))?;
        
        // Create the vote
        let vote = Vote {
            agent_id: agent_id.clone(),
            value,
            signature,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        // Add the vote
        decision.votes.insert(agent_id, vote);
        
        // Check if we have enough votes to finalize the decision
        self.check_decision_state(decision_id).await?;
        
        // Update the content hash
        self.base.set_metadata("vote_cast", &chrono::Utc::now().to_rfc3339()).await
            .map_err(CommitteeAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Check if a decision has reached a conclusion
    async fn check_decision_state(&mut self, decision_id: &str) -> Result<(), CommitteeAgentError> {
        // Find the decision
        let decision_index = self.decisions.iter().position(|d| d.decision_id == decision_id)
            .ok_or_else(|| CommitteeAgentError::DecisionError(
                format!("Decision {} not found", decision_id)
            ))?;
        
        let decision = &self.decisions[decision_index];
        
        // Calculate the total voting weight
        let mut total_weight = 0;
        let mut approve_weight = 0;
        let mut reject_weight = 0;
        
        for (agent_id, vote) in &decision.votes {
            // Find the member's weight
            if let Some(member) = self.members.iter().find(|m| &m.agent_id == agent_id) {
                match vote.value {
                    VoteValue::Approve => approve_weight += member.voting_weight,
                    VoteValue::Reject => reject_weight += member.voting_weight,
                    VoteValue::Abstain => {}, // Abstentions don't count towards any side
                }
                total_weight += member.voting_weight;
            }
        }
        
        // Calculate the total available weight
        let available_weight: u32 = self.members.iter()
            .filter(|m| m.active)
            .map(|m| m.voting_weight)
            .sum();
        
        // Check if we've reached the quorum
        let quorum_threshold = (available_weight as f64 * (self.config.quorum_percentage as f64 / 100.0)) as u32;
        
        let mut result = None;
        
        if total_weight >= quorum_threshold && decision.votes.len() >= self.config.min_votes {
            // We've reached quorum, determine the result
            if approve_weight > reject_weight {
                result = Some(DecisionResult::Approved);
            } else {
                result = Some(DecisionResult::Rejected);
            }
            
            // Remove from active decisions
            self.active_decisions.remove(decision_id);
        }
        
        // Update the decision result if it has changed
        if result != self.decisions[decision_index].result {
            self.decisions[decision_index].result = result;
            
            // Update the content hash
            self.base.set_metadata("decision_updated", &chrono::Utc::now().to_rfc3339()).await
                .map_err(CommitteeAgentError::AgentError)?;
        }
        
        Ok(())
    }
    
    /// Get a decision by ID
    pub fn get_decision(&self, decision_id: &str) -> Option<&CommitteeDecision> {
        self.decisions.iter().find(|d| d.decision_id == decision_id)
    }
    
    /// Get all decisions
    pub fn decisions(&self) -> &[CommitteeDecision] {
        &self.decisions
    }
    
    /// Get active decisions
    pub fn active_decisions(&self) -> Vec<&CommitteeDecision> {
        self.decisions.iter()
            .filter(|d| self.active_decisions.contains(&d.decision_id))
            .collect()
    }
    
    /// Check if a decision has been approved
    pub fn is_decision_approved(&self, decision_id: &str) -> Result<bool, CommitteeAgentError> {
        let decision = self.get_decision(decision_id)
            .ok_or_else(|| CommitteeAgentError::DecisionError(
                format!("Decision {} not found", decision_id)
            ))?;
        
        Ok(decision.result == Some(DecisionResult::Approved))
    }
    
    /// Validate a fact observed in the committee's domain
    pub async fn validate_fact(
        &mut self,
        fact: &str,
        signatures: Vec<(AgentId, Vec<u8>)>,
    ) -> Result<bool, CommitteeAgentError> {
        // Start a new decision for this fact
        let decision_id = self.start_decision(
            "Fact Validation".to_string(),
            format!("Validate fact: {}", fact),
        ).await?;
        
        // Process the signatures as votes
        for (agent_id, signature) in signatures {
            self.cast_vote(
                &decision_id,
                agent_id,
                VoteValue::Approve,
                signature,
            ).await?;
        }
        
        // Check if the decision has been approved
        self.is_decision_approved(&decision_id)
    }
    
    /// Get the domain this committee validates
    pub fn domain(&self) -> &str {
        &self.config.domain
    }
}

// Delegate the Agent trait methods to the base agent
#[async_trait]
impl Agent for CommitteeAgent {
    fn agent_id(&self) -> &AgentId {
        self.base.agent_id()
    }
    
    fn agent_type(&self) -> &AgentType {
        self.base.agent_type()
    }
    
    fn state(&self) -> &AgentState {
        self.base.state()
    }
    
    async fn set_state(&mut self, state: AgentState) -> Result<(), AgentError> {
        self.base.set_state(state).await
    }
    
    async fn add_capability(&mut self, capability: Capability) -> Result<(), AgentError> {
        self.base.add_capability(capability).await
    }
    
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError> {
        self.base.remove_capability(capability_id).await
    }
    
    fn has_capability(&self, capability_id: &str) -> bool {
        self.base.has_capability(capability_id)
    }
    
    fn capabilities(&self) -> Vec<Capability> {
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
impl crate::resource::Resource for CommitteeAgent {
    fn id(&self) -> crate::resource_types::ResourceId {
        self.base.id().clone()
    }
    
    fn resource_type(&self) -> crate::resource_types::ResourceType {
        self.base.resource_type()
    }
    
    fn state(&self) -> crate::resource::ResourceState {
        match self.base.state() {
            AgentState::Active => crate::resource::ResourceState::Active,
            AgentState::Inactive => crate::resource::ResourceState::Frozen,
            AgentState::Suspended { .. } => crate::resource::ResourceState::Locked,
        }
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.base.metadata().get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> crate::resource::ResourceResult<()> {
        *self.base.metadata_mut().entry(key.to_string()).or_insert_with(String::new) = value.to_string();
        Ok(())
    }
    
    fn clone_resource(&self) -> Box<dyn crate::resource::Resource> {
        Box::new(self.clone())
    }
}

/// Builder for creating committee agents
pub struct CommitteeAgentBuilder {
    /// Base agent builder
    base_builder: super::agent::AgentBuilder,
    
    /// Committee configuration
    config: Option<CommitteeConfig>,
    
    /// Committee members
    members: Vec<CommitteeMember>,
}

impl CommitteeAgentBuilder {
    /// Create a new committee agent builder
    pub fn new() -> Self {
        Self {
            base_builder: super::agent::AgentBuilder::new().agent_type(AgentType::Committee),
            config: None,
            members: Vec::new(),
        }
    }
    
    /// Set the agent state
    pub fn state(mut self, state: AgentState) -> Self {
        self.base_builder = self.base_builder.state(state);
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.base_builder = self.base_builder.with_capability(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
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
    
    /// Set the committee configuration
    pub fn with_config(mut self, config: CommitteeConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Set the domain
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        let mut config = self.config.unwrap_or_else(|| CommitteeConfig {
            domain: String::new(),
            quorum_percentage: 67,  // Default to 2/3 majority
            max_size: 21,          // Default to 21 members
            min_votes: 3,          // Default to 3 minimum votes
            protocol_version: "1.0".to_string(),
        });
        
        config.domain = domain.into();
        self.config = Some(config);
        self
    }
    
    /// Set the quorum percentage
    pub fn with_quorum_percentage(mut self, percentage: u8) -> Self {
        let mut config = self.config.clone().unwrap_or_else(|| CommitteeConfig {
            domain: String::new(),
            quorum_percentage: 67,
            max_size: 21,
            min_votes: 3,
            protocol_version: "1.0".to_string(),
        });
        
        // Ensure percentage is between 0 and 100
        let clamped = percentage.min(100);
        config.quorum_percentage = clamped;
        self.config = Some(config);
        self
    }
    
    /// Add a committee member
    pub fn with_member(mut self, member: CommitteeMember) -> Self {
        self.members.push(member);
        self
    }
    
    /// Build the committee agent
    pub fn build(self) -> Result<CommitteeAgent, CommitteeAgentError> {
        // Build the base agent
        let base_agent = self.base_builder.build()
            .map_err(CommitteeAgentError::AgentError)?;
        
        // Create the committee agent
        let config = self.config.ok_or_else(|| {
            CommitteeAgentError::Other("Committee configuration is required".to_string())
        })?;
        
        CommitteeAgent::new(
            base_agent,
            config,
            Some(self.members),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_committee_creation() {
        // Create a committee agent
        let committee = CommitteeAgentBuilder::new()
            .state(AgentState::Active)
            .with_domain("test-domain")
            .with_quorum_percentage(67)
            .build()
            .unwrap();
        
        // Check the agent type
        assert_eq!(committee.agent_type(), &AgentType::Committee);
        
        // Check the domain
        assert_eq!(committee.domain(), "test-domain");
    }
    
    #[tokio::test]
    async fn test_committee_membership() {
        // Create a committee agent
        let mut committee = CommitteeAgentBuilder::new()
            .state(AgentState::Active)
            .with_domain("test-domain")
            .build()
            .unwrap();
        
        // Create a member
        let member = CommitteeMember {
            agent_id: AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::Validator),
            role: MemberRole::Validator,
            voting_weight: 1,
            public_key: "public-key".to_string(),
            active: true,
        };
        
        // Initially no members
        assert_eq!(committee.members().len(), 0);
        
        // Add a member
        committee.add_member(member.clone()).await.unwrap();
        
        // Check members
        assert_eq!(committee.members().len(), 1);
        assert_eq!(&committee.members()[0].agent_id, &member.agent_id);
        
        // Remove the member
        committee.remove_member(&member.agent_id).await.unwrap();
        
        // Check members again
        assert_eq!(committee.members().len(), 0);
    }
    
    #[tokio::test]
    async fn test_committee_decisions() {
        // Create a committee agent
        let mut committee = CommitteeAgentBuilder::new()
            .state(AgentState::Active)
            .with_domain("test-domain")
            .with_config(CommitteeConfig {
                domain: "test-domain".to_string(),
                quorum_percentage: 60,
                max_size: 5,
                min_votes: 2,
                protocol_version: "1.0".to_string(),
            })
            .build()
            .unwrap();
        
        // Create two members
        let member1 = CommitteeMember {
            agent_id: AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::Leader),
            role: MemberRole::Leader,
            voting_weight: 2,
            public_key: "public-key-1".to_string(),
            active: true,
        };
        
        let member2 = CommitteeMember {
            agent_id: AgentId::from_content_hash(crate::crypto::ContentHash::create_from_bytes("test2".as_bytes()).unwrap().as_bytes(), AgentType::Validator),
            role: MemberRole::Validator,
            voting_weight: 1,
            public_key: "public-key-2".to_string(),
            active: true,
        };
        
        // Add members
        committee.add_member(member1.clone()).await.unwrap();
        committee.add_member(member2.clone()).await.unwrap();
        
        // Start a decision
        let decision_id = committee.start_decision(
            "Test Decision".to_string(),
            "This is a test decision".to_string(),
        ).await.unwrap();
        
        // Check active decisions
        assert_eq!(committee.active_decisions().len(), 1);
        
        // Cast votes
        committee.cast_vote(
            &decision_id,
            member1.agent_id.clone(),
            VoteValue::Approve,
            vec![1, 2, 3], // Mock signature
        ).await.unwrap();
        
        committee.cast_vote(
            &decision_id,
            member2.agent_id.clone(),
            VoteValue::Approve,
            vec![4, 5, 6], // Mock signature
        ).await.unwrap();
        
        // Check if the decision was approved
        assert!(committee.is_decision_approved(&decision_id).unwrap());
        
        // Check active decisions again
        assert_eq!(committee.active_decisions().len(), 0);
    }
} 