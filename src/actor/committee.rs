// Committee Actor Module
//
// This module implements the Committee actor type for Causality.
// Committees are groups of actors that can verify facts, manage governance, and audit the system.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::{ContentId, ContentHash, TraceId, Timestamp};
use crate::actor::{
    Actor, ActorId, ActorType, ActorState, ActorInfo, 
    ActorRole, ActorCapability, Message, MessageCategory, MessagePayload
};

/// Committee decision rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionRule {
    /// Simple majority vote
    SimpleMajority,
    /// Qualified majority (e.g., 2/3)
    QualifiedMajority(u8),
    /// Unanimous decision
    Unanimous,
    /// Weighted voting
    Weighted,
    /// Custom rule
    Custom(String),
}

/// Committee vote on a decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    /// The member that cast the vote
    pub member_id: ActorId,
    /// The vote (true = yes, false = no)
    pub vote: bool,
    /// Vote weight (if using weighted voting)
    pub weight: Option<u64>,
    /// When the vote was cast
    pub timestamp: Timestamp,
    /// Any comments provided with the vote
    pub comments: Option<String>,
}

/// Decision record 
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    /// The decision ID
    pub id: String,
    /// Description of what is being decided
    pub description: String,
    /// The proposal being voted on
    pub proposal: String,
    /// When the decision was created
    pub created_at: Timestamp,
    /// When the decision closes
    pub closes_at: Option<Timestamp>,
    /// The decision result (if finalized)
    pub result: Option<bool>,
    /// When the decision was finalized
    pub finalized_at: Option<Timestamp>,
    /// All votes cast
    pub votes: Vec<Vote>,
    /// Decision rule applied
    pub rule: DecisionRule,
}

impl Decision {
    /// Create a new decision
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        proposal: impl Into<String>,
        rule: DecisionRule,
        closes_at: Option<Timestamp>,
    ) -> Self {
        Decision {
            id: id.into(),
            description: description.into(),
            proposal: proposal.into(),
            created_at: Timestamp::now(),
            closes_at,
            result: None,
            finalized_at: None,
            votes: Vec::new(),
            rule,
        }
    }
    
    /// Add a vote to this decision
    pub fn add_vote(&mut self, vote: Vote) -> Result<()> {
        // Check if decision is already finalized
        if self.result.is_some() {
            return Err(Error::InvalidState("Decision is already finalized".to_string()));
        }
        
        // Check if this member has already voted
        if self.votes.iter().any(|v| v.member_id == vote.member_id) {
            return Err(Error::InvalidState(format!(
                "Member {} has already voted",
                vote.member_id
            )));
        }
        
        self.votes.push(vote);
        
        Ok(())
    }
    
    /// Check if the decision is ready to be finalized
    pub fn is_ready_for_finalization(&self, total_members: usize) -> bool {
        // If already finalized, then it's not ready for finalization
        if self.result.is_some() {
            return false;
        }
        
        // If closing time has passed, it's ready for finalization
        if let Some(closes_at) = self.closes_at {
            if Timestamp::now() >= closes_at {
                return true;
            }
        }
        
        // Check based on decision rule
        match &self.rule {
            DecisionRule::SimpleMajority => {
                // Ready if more than 50% of members have voted
                self.votes.len() > total_members / 2
            },
            DecisionRule::QualifiedMajority(percentage) => {
                // Calculate the number of votes needed
                let required_votes = (total_members as f64 * (*percentage as f64 / 100.0)).ceil() as usize;
                self.votes.len() >= required_votes
            },
            DecisionRule::Unanimous => {
                // Ready if all members have voted
                self.votes.len() == total_members
            },
            DecisionRule::Weighted => {
                // Ready if all members have voted
                // For weighted voting, we typically need all votes in
                self.votes.len() == total_members
            },
            DecisionRule::Custom(_) => {
                // For custom rules, default to requiring all votes
                self.votes.len() == total_members
            },
        }
    }
    
    /// Finalize the decision
    pub fn finalize(&mut self, total_members: usize) -> Result<bool> {
        // Check if already finalized
        if self.result.is_some() {
            return Err(Error::InvalidState("Decision is already finalized".to_string()));
        }
        
        let result = match &self.rule {
            DecisionRule::SimpleMajority => {
                // Count yes votes
                let yes_votes = self.votes.iter().filter(|v| v.vote).count();
                yes_votes > self.votes.len() / 2
            },
            DecisionRule::QualifiedMajority(percentage) => {
                // Calculate the number of yes votes needed
                let required_yes = (self.votes.len() as f64 * (*percentage as f64 / 100.0)).ceil() as usize;
                let yes_votes = self.votes.iter().filter(|v| v.vote).count();
                yes_votes >= required_yes
            },
            DecisionRule::Unanimous => {
                // All votes must be yes
                self.votes.iter().all(|v| v.vote)
            },
            DecisionRule::Weighted => {
                // Sum the weights of all yes votes
                let yes_weight: u64 = self.votes.iter()
                    .filter(|v| v.vote)
                    .filter_map(|v| v.weight)
                    .sum();
                
                // Sum the weights of all votes
                let total_weight: u64 = self.votes.iter()
                    .filter_map(|v| v.weight)
                    .sum();
                
                // Yes if more than 50% of the weighted votes are yes
                yes_weight > total_weight / 2
            },
            DecisionRule::Custom(_) => {
                // For custom rules, default to simple majority
                let yes_votes = self.votes.iter().filter(|v| v.vote).count();
                yes_votes > self.votes.len() / 2
            },
        };
        
        self.result = Some(result);
        self.finalized_at = Some(Timestamp::now());
        
        Ok(result)
    }
}

/// Committee actor implementation
#[derive(Debug)]
pub struct Committee {
    /// Actor ID
    id: ActorId,
    /// Actor type
    actor_type: ActorType,
    /// Actor state
    state: RwLock<ActorState>,
    /// Actor information
    info: RwLock<ActorInfo>,
    /// Committee name
    name: String,
    /// Committee description
    description: Option<String>,
    /// Committee members
    members: RwLock<HashSet<ActorId>>,
    /// Decision rule
    decision_rule: RwLock<DecisionRule>,
    /// Active decisions
    decisions: RwLock<HashMap<String, Decision>>,
    /// Finalized decisions
    finalized_decisions: RwLock<HashMap<String, Decision>>,
}

impl Committee {
    /// Create a new committee
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        decision_rule: DecisionRule,
    ) -> Self {
        let id_str = id.into();
        let name_str = name.into();
        let actor_id = ActorId(id_str);
        let now = Timestamp::now();
        
        let info = ActorInfo {
            id: actor_id.clone(),
            actor_type: ActorType::Committee,
            state: ActorState::Pending,
            name: name_str.clone(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        Committee {
            id: actor_id,
            actor_type: ActorType::Committee,
            state: RwLock::new(ActorState::Pending),
            info: RwLock::new(info),
            name: name_str,
            description,
            members: RwLock::new(HashSet::new()),
            decision_rule: RwLock::new(decision_rule),
            decisions: RwLock::new(HashMap::new()),
            finalized_decisions: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a member to the committee
    pub fn add_member(&self, member_id: ActorId) -> Result<()> {
        let mut members = self.members.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on members".to_string())
        })?;
        
        members.insert(member_id);
        
        Ok(())
    }
    
    /// Remove a member from the committee
    pub fn remove_member(&self, member_id: &ActorId) -> Result<()> {
        let mut members = self.members.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on members".to_string())
        })?;
        
        members.remove(member_id);
        
        Ok(())
    }
    
    /// Check if an actor is a member of this committee
    pub fn is_member(&self, member_id: &ActorId) -> Result<bool> {
        let members = self.members.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on members".to_string())
        })?;
        
        Ok(members.contains(member_id))
    }
    
    /// Get all committee members
    pub fn get_members(&self) -> Result<Vec<ActorId>> {
        let members = self.members.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on members".to_string())
        })?;
        
        Ok(members.iter().cloned().collect())
    }
    
    /// Create a new decision
    pub fn create_decision(
        &self,
        id: impl Into<String>,
        description: impl Into<String>,
        proposal: impl Into<String>,
        closes_at: Option<Timestamp>,
    ) -> Result<()> {
        let decision_rule = self.decision_rule.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on decision_rule".to_string())
        })?;
        
        let decision = Decision::new(
            id.into(),
            description.into(),
            proposal.into(),
            decision_rule.clone(),
            closes_at,
        );
        
        let mut decisions = self.decisions.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on decisions".to_string())
        })?;
        
        decisions.insert(decision.id.clone(), decision);
        
        Ok(())
    }
    
    /// Add a vote to a decision
    pub fn add_vote(
        &self,
        decision_id: &str,
        member_id: ActorId,
        vote: bool,
        weight: Option<u64>,
        comments: Option<String>,
    ) -> Result<()> {
        // First check if the actor is a member
        if !self.is_member(&member_id)? {
            return Err(Error::Unauthorized(format!(
                "Actor {} is not a member of this committee",
                member_id
            )));
        }
        
        let mut decisions = self.decisions.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on decisions".to_string())
        })?;
        
        if let Some(decision) = decisions.get_mut(decision_id) {
            let vote = Vote {
                member_id,
                vote,
                weight,
                timestamp: Timestamp::now(),
                comments,
            };
            
            decision.add_vote(vote)?;
            
            // Check if the decision is ready for finalization
            let members = self.members.read().map_err(|_| {
                Error::LockError("Failed to acquire read lock on members".to_string())
            })?;
            
            if decision.is_ready_for_finalization(members.len()) {
                // Finalize the decision
                decision.finalize(members.len())?;
                
                // Move to finalized decisions
                let mut finalized = self.finalized_decisions.write().map_err(|_| {
                    Error::LockError("Failed to acquire write lock on finalized_decisions".to_string())
                })?;
                
                let finalized_decision = decisions.remove(decision_id).unwrap();
                finalized.insert(decision_id.to_string(), finalized_decision);
            }
            
            Ok(())
        } else {
            Err(Error::NotFound(format!("Decision not found: {}", decision_id)))
        }
    }
    
    /// Get a decision by ID
    pub fn get_decision(&self, decision_id: &str) -> Result<Option<Decision>> {
        // Check active decisions first
        let decisions = self.decisions.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on decisions".to_string())
        })?;
        
        if let Some(decision) = decisions.get(decision_id) {
            return Ok(Some(decision.clone()));
        }
        
        // Check finalized decisions
        let finalized = self.finalized_decisions.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on finalized_decisions".to_string())
        })?;
        
        if let Some(decision) = finalized.get(decision_id) {
            return Ok(Some(decision.clone()));
        }
        
        Ok(None)
    }
    
    /// Get all active decisions
    pub fn get_active_decisions(&self) -> Result<Vec<Decision>> {
        let decisions = self.decisions.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on decisions".to_string())
        })?;
        
        Ok(decisions.values().cloned().collect())
    }
    
    /// Get all finalized decisions
    pub fn get_finalized_decisions(&self) -> Result<Vec<Decision>> {
        let finalized = self.finalized_decisions.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on finalized_decisions".to_string())
        })?;
        
        Ok(finalized.values().cloned().collect())
    }
    
    /// Update the decision rule
    pub fn update_decision_rule(&self, rule: DecisionRule) -> Result<()> {
        let mut decision_rule = self.decision_rule.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on decision_rule".to_string())
        })?;
        
        *decision_rule = rule;
        
        Ok(())
    }
}

#[async_trait]
impl Actor for Committee {
    fn id(&self) -> &ActorId {
        &self.id
    }
    
    fn actor_type(&self) -> ActorType {
        self.actor_type.clone()
    }
    
    fn state(&self) -> ActorState {
        self.state.read().unwrap_or(ActorState::Pending)
    }
    
    fn info(&self) -> ActorInfo {
        let mut info = self.info.read().unwrap_or_else(|_| {
            panic!("Failed to acquire read lock on info")
        }).clone();
        
        // Update with current state
        info.state = self.state();
        info.updated_at = Timestamp::now();
        
        info
    }
    
    async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on state".to_string())
        })?;
        
        if *state != ActorState::Pending {
            return Err(Error::InvalidState(format!(
                "Cannot initialize actor in state: {:?}",
                *state
            )));
        }
        
        *state = ActorState::Active;
        
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on state".to_string())
        })?;
        
        if *state != ActorState::Pending && *state != ActorState::Suspended {
            return Err(Error::InvalidState(format!(
                "Cannot start actor in state: {:?}",
                *state
            )));
        }
        
        *state = ActorState::Active;
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on state".to_string())
        })?;
        
        if *state != ActorState::Active {
            return Err(Error::InvalidState(format!(
                "Cannot stop actor in state: {:?}",
                *state
            )));
        }
        
        *state = ActorState::Inactive;
        
        Ok(())
    }
    
    async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
        match message.category {
            // Membership management messages
            MessageCategory::MembershipManagement => {
                match message.payload {
                    MessagePayload::AddMember { member_id } => {
                        self.add_member(member_id)?;
                        Ok(None)
                    },
                    MessagePayload::RemoveMember { member_id } => {
                        self.remove_member(&member_id)?;
                        Ok(None)
                    },
                    _ => Err(Error::UnsupportedMessage(
                        "Unsupported membership management message".to_string()
                    )),
                }
            },
            
            // Decision management messages
            MessageCategory::GovernanceManagement => {
                match message.payload {
                    MessagePayload::CreateDecision { id, description, proposal, closes_at } => {
                        self.create_decision(id, description, proposal, closes_at)?;
                        Ok(None)
                    },
                    MessagePayload::CastVote { decision_id, vote, weight, comments } => {
                        self.add_vote(
                            &decision_id, 
                            message.sender.clone(), 
                            vote, 
                            weight, 
                            comments
                        )?;
                        Ok(None)
                    },
                    MessagePayload::UpdateDecisionRule { rule } => {
                        self.update_decision_rule(rule)?;
                        Ok(None)
                    },
                    _ => Err(Error::UnsupportedMessage(
                        "Unsupported governance management message".to_string()
                    )),
                }
            },
            
            // Other message categories
            _ => Err(Error::UnsupportedMessage(format!(
                "Unsupported message category: {:?}",
                message.category
            ))),
        }
    }
    
    async fn has_permission(&self, permission: &str) -> Result<bool> {
        // Committees have permissions based on their role
        match permission {
            "verify_facts" | "manage_governance" | "audit" => Ok(true),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_committee_actor() -> Result<()> {
        // Create a committee
        let committee = Committee::new(
            "test-committee",
            "Test Committee",
            Some("A test committee for unit tests".to_string()),
            DecisionRule::SimpleMajority,
        );
        
        // Check initial state
        assert_eq!(committee.id().0, "test-committee");
        assert_eq!(committee.actor_type(), ActorType::Committee);
        assert_eq!(committee.state(), ActorState::Pending);
        assert_eq!(committee.info().name, "Test Committee");
        
        // Add members
        let member1 = ActorId("member1".to_string());
        let member2 = ActorId("member2".to_string());
        let member3 = ActorId("member3".to_string());
        
        committee.add_member(member1.clone())?;
        committee.add_member(member2.clone())?;
        committee.add_member(member3.clone())?;
        
        assert!(committee.is_member(&member1)?);
        assert_eq!(committee.get_members()?.len(), 3);
        
        // Initialize the committee
        committee.initialize().await?;
        assert_eq!(committee.state(), ActorState::Active);
        
        // Create a decision
        committee.create_decision(
            "decision1",
            "Test Decision",
            "Approve the test proposal",
            None,
        )?;
        
        // Get the decision
        let decision_opt = committee.get_decision("decision1")?;
        assert!(decision_opt.is_some());
        
        // Add votes
        committee.add_vote(
            "decision1", 
            member1.clone(), 
            true, 
            None, 
            Some("Approved".to_string())
        )?;
        
        committee.add_vote(
            "decision1", 
            member2.clone(), 
            true, 
            None, 
            None
        )?;
        
        // This should finalize the decision (2/3 votes with simple majority)
        
        // Check that decision is now finalized
        let decision_opt = committee.get_decision("decision1")?;
        assert!(decision_opt.is_some());
        
        let decision = decision_opt.unwrap();
        assert!(decision.result.is_some());
        assert!(decision.result.unwrap()); // Should be approved
        
        // Check active and finalized decisions
        assert_eq!(committee.get_active_decisions()?.len(), 0);
        assert_eq!(committee.get_finalized_decisions()?.len(), 1);
        
        // Test permission
        assert!(committee.has_permission("verify_facts").await?);
        assert!(!committee.has_permission("create_program").await?);
        
        // Test membership removal
        committee.remove_member(&member1)?;
        assert!(!committee.is_member(&member1)?);
        assert_eq!(committee.get_members()?.len(), 2);
        
        // Test lifecycle
        committee.stop().await?;
        assert_eq!(committee.state(), ActorState::Inactive);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_committee_message_handling() -> Result<()> {
        // Create a committee
        let committee = Committee::new(
            "message-committee",
            "Message Committee",
            None,
            DecisionRule::SimpleMajority,
        );
        
        // Initialize the committee
        committee.initialize().await?;
        
        // Test handling add member message
        let add_member_msg = Message {
            id: "msg1".to_string(),
            sender: ActorId("system".to_string()),
            recipients: vec![committee.id().clone()],
            category: MessageCategory::MembershipManagement,
            payload: MessagePayload::AddMember { 
                member_id: ActorId("member1".to_string()) 
            },
            timestamp: Timestamp::now(),
            trace_id: None,
        };
        
        committee.handle_message(add_member_msg).await?;
        
        // Check if the member was added
        assert!(committee.is_member(&ActorId("member1".to_string()))?);
        
        // Test handling create decision message
        let create_decision_msg = Message {
            id: "msg2".to_string(),
            sender: ActorId("system".to_string()),
            recipients: vec![committee.id().clone()],
            category: MessageCategory::GovernanceManagement,
            payload: MessagePayload::CreateDecision { 
                id: "decision1".to_string(),
                description: "Test Decision".to_string(),
                proposal: "Approve the test".to_string(),
                closes_at: None,
            },
            timestamp: Timestamp::now(),
            trace_id: None,
        };
        
        committee.handle_message(create_decision_msg).await?;
        
        // Check if the decision was created
        let decision = committee.get_decision("decision1")?;
        assert!(decision.is_some());
        
        // Test handling cast vote message
        let cast_vote_msg = Message {
            id: "msg3".to_string(),
            sender: ActorId("member1".to_string()),
            recipients: vec![committee.id().clone()],
            category: MessageCategory::GovernanceManagement,
            payload: MessagePayload::CastVote { 
                decision_id: "decision1".to_string(),
                vote: true,
                weight: None,
                comments: Some("Approved".to_string()),
            },
            timestamp: Timestamp::now(),
            trace_id: None,
        };
        
        committee.handle_message(cast_vote_msg).await?;
        
        // Check if the vote was recorded
        let decision = committee.get_decision("decision1")?.unwrap();
        assert_eq!(decision.votes.len(), 1);
        assert_eq!(decision.votes[0].member_id, ActorId("member1".to_string()));
        assert!(decision.votes[0].vote);
        
        Ok(())
    }
} 