// obligation.rs - Obligation manager for tracking capability obligations
//
// This file implements the obligation manager which tracks and enforces
// capability obligations for agents in the system.

use crate::resource_types::{ResourceId, ResourceType};
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::effect::Effect;

use super::types::{AgentId, AgentError};
use super::agent::Agent;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Obligation error types
#[derive(Error, Debug)]
pub enum ObligationError {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Capability error
    #[error("Capability error: {0}")]
    CapabilityError(String),
    
    /// Obligation validation error
    #[error("Obligation validation error: {0}")]
    ValidationError(String),
    
    /// Obligation tracking error
    #[error("Obligation tracking error: {0}")]
    TrackingError(String),
    
    /// Obligation reporting error
    #[error("Obligation reporting error: {0}")]
    ReportingError(String),
    
    /// Enforcement error
    #[error("Enforcement error: {0}")]
    EnforcementError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for obligation operations
pub type ObligationResult<T> = Result<T, ObligationError>;

/// Status of an obligation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObligationStatus {
    /// Obligation is active
    Active,
    
    /// Obligation has been fulfilled
    Fulfilled {
        /// When the obligation was fulfilled
        when: DateTime<Utc>,
        
        /// How the obligation was fulfilled
        how: String,
    },
    
    /// Obligation has been violated
    Violated {
        /// When the obligation was violated
        when: DateTime<Utc>,
        
        /// How the obligation was violated
        how: String,
    },
    
    /// Obligation has been waived
    Waived {
        /// When the obligation was waived
        when: DateTime<Utc>,
        
        /// Why the obligation was waived
        reason: String,
        
        /// Agent who waived the obligation
        by_agent: AgentId,
    },
}

/// Type of obligation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObligationType {
    /// Use a capability within a certain time frame
    UseWithinTime(Duration),
    
    /// Use a capability a maximum number of times
    UseMaximumTimes(u32),
    
    /// Use a capability a minimum number of times
    UseMinimumTimes(u32),
    
    /// Report usage of a capability
    ReportUsage {
        /// To whom the report should be sent
        to_agent: AgentId,
        
        /// When the report is due
        due: DateTime<Utc>,
    },
    
    /// Delegate a capability only to specific agents
    DelegateOnlyTo(Vec<AgentId>),
    
    /// Do not delegate a capability
    DoNotDelegate,
    
    /// Revoke a capability at a specific time
    RevokeAt(DateTime<Utc>),
    
    /// Pay for capability usage
    PayForUsage {
        /// Amount to pay
        amount: String,
        
        /// Currency
        currency: String,
        
        /// Payment deadline
        deadline: DateTime<Utc>,
    },
    
    /// Custom obligation
    Custom {
        /// Obligation type
        obligation_type: String,
        
        /// Obligation parameters
        parameters: HashMap<String, String>,
    },
}

/// Unique identifier for an obligation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ObligationId(String);

impl ObligationId {
    /// Create a new obligation ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObligationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ObligationId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ObligationId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// An obligation associated with a capability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obligation {
    /// Unique ID for the obligation
    id: ObligationId,
    
    /// Capability this obligation is associated with
    capability_id: String,
    
    /// Agent who has the obligation
    agent_id: AgentId,
    
    /// Type of obligation
    obligation_type: ObligationType,
    
    /// Current status of the obligation
    status: ObligationStatus,
    
    /// When the obligation was created
    created_at: DateTime<Utc>,
    
    /// When the obligation was last updated
    updated_at: DateTime<Utc>,
    
    /// Agent who created the obligation
    created_by: AgentId,
    
    /// Metadata
    metadata: HashMap<String, String>,
}

impl Obligation {
    /// Create a new obligation
    pub fn new(
        capability_id: impl Into<String>,
        agent_id: AgentId,
        obligation_type: ObligationType,
        created_by: AgentId,
    ) -> Self {
        let now = Utc::now();
        let capability_id = capability_id.into();
        let id = ObligationId::new(format!(
            "obligation-{}-{}-{}",
            capability_id,
            agent_id,
            now.timestamp()
        ));
        
        Self {
            id,
            capability_id,
            agent_id,
            obligation_type,
            status: ObligationStatus::Active,
            created_at: now,
            updated_at: now,
            created_by,
            metadata: HashMap::new(),
        }
    }
    
    /// Get the obligation ID
    pub fn id(&self) -> &ObligationId {
        &self.id
    }
    
    /// Get the capability ID
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
    
    /// Get the agent ID
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    
    /// Get the obligation type
    pub fn obligation_type(&self) -> &ObligationType {
        &self.obligation_type
    }
    
    /// Get the obligation status
    pub fn status(&self) -> &ObligationStatus {
        &self.status
    }
    
    /// Check if the obligation is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ObligationStatus::Active)
    }
    
    /// Check if the obligation is fulfilled
    pub fn is_fulfilled(&self) -> bool {
        matches!(self.status, ObligationStatus::Fulfilled { .. })
    }
    
    /// Check if the obligation is violated
    pub fn is_violated(&self) -> bool {
        matches!(self.status, ObligationStatus::Violated { .. })
    }
    
    /// Get the creation time
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    
    /// Get the last update time
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
    
    /// Get the creator
    pub fn created_by(&self) -> &AgentId {
        &self.created_by
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }
    
    /// Mark the obligation as fulfilled
    pub fn fulfill(&mut self, description: impl Into<String>) {
        self.status = ObligationStatus::Fulfilled {
            when: Utc::now(),
            how: description.into(),
        };
        self.updated_at = Utc::now();
    }
    
    /// Mark the obligation as violated
    pub fn violate(&mut self, description: impl Into<String>) {
        self.status = ObligationStatus::Violated {
            when: Utc::now(),
            how: description.into(),
        };
        self.updated_at = Utc::now();
    }
    
    /// Waive the obligation
    pub fn waive(&mut self, reason: impl Into<String>, by_agent: AgentId) {
        self.status = ObligationStatus::Waived {
            when: Utc::now(),
            reason: reason.into(),
            by_agent,
        };
        self.updated_at = Utc::now();
    }
    
    /// Check if the obligation has a deadline
    pub fn has_deadline(&self) -> bool {
        match &self.obligation_type {
            ObligationType::UseWithinTime(_) => true,
            ObligationType::ReportUsage { due, .. } => true,
            ObligationType::RevokeAt(time) => true,
            ObligationType::PayForUsage { deadline, .. } => true,
            ObligationType::Custom { parameters, .. } => parameters.contains_key("deadline"),
            _ => false,
        }
    }
    
    /// Get the deadline, if any
    pub fn deadline(&self) -> Option<DateTime<Utc>> {
        match &self.obligation_type {
            ObligationType::UseWithinTime(duration) => {
                let deadline = self.created_at + chrono::Duration::from_std(*duration).unwrap_or_default();
                Some(deadline)
            },
            ObligationType::ReportUsage { due, .. } => Some(*due),
            ObligationType::RevokeAt(time) => Some(*time),
            ObligationType::PayForUsage { deadline, .. } => Some(*deadline),
            ObligationType::Custom { parameters, .. } => {
                parameters.get("deadline").and_then(|s| {
                    s.parse::<i64>().ok().map(|ts| {
                        DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_default()
                    })
                })
            },
            _ => None,
        }
    }
    
    /// Check if the obligation is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = self.deadline() {
            Utc::now() > deadline
        } else {
            false
        }
    }
}

/// Summary of obligation status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObligationSummary {
    /// Total number of obligations
    pub total: usize,
    
    /// Number of active obligations
    pub active: usize,
    
    /// Number of fulfilled obligations
    pub fulfilled: usize,
    
    /// Number of violated obligations
    pub violated: usize,
    
    /// Number of waived obligations
    pub waived: usize,
    
    /// Number of overdue obligations
    pub overdue: usize,
}

/// ObligationManager for tracking and enforcing capability obligations
#[derive(Clone)]
pub struct ObligationManager {
    /// All obligations
    obligations: Arc<RwLock<HashMap<ObligationId, Obligation>>>,
    
    /// Obligations by agent
    agent_obligations: Arc<RwLock<HashMap<AgentId, HashSet<ObligationId>>>>,
    
    /// Obligations by capability
    capability_obligations: Arc<RwLock<HashMap<String, HashSet<ObligationId>>>>,
    
    /// Usage counts for capabilities
    capability_usage: Arc<RwLock<HashMap<String, u32>>>,
}

impl ObligationManager {
    /// Create a new obligation manager
    pub fn new() -> Self {
        Self {
            obligations: Arc::new(RwLock::new(HashMap::new())),
            agent_obligations: Arc::new(RwLock::new(HashMap::new())),
            capability_obligations: Arc::new(RwLock::new(HashMap::new())),
            capability_usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add an obligation
    pub async fn add_obligation(&self, obligation: Obligation) -> ObligationResult<ObligationId> {
        let obligation_id = obligation.id().clone();
        
        // Update the indices
        {
            let mut agent_obligations = self.agent_obligations.write().await;
            let agent_set = agent_obligations.entry(obligation.agent_id.clone()).or_insert_with(HashSet::new);
            agent_set.insert(obligation_id.clone());
        }
        
        {
            let mut capability_obligations = self.capability_obligations.write().await;
            let capability_set = capability_obligations.entry(obligation.capability_id.clone()).or_insert_with(HashSet::new);
            capability_set.insert(obligation_id.clone());
        }
        
        // Add the obligation
        {
            let mut obligations = self.obligations.write().await;
            obligations.insert(obligation_id.clone(), obligation);
        }
        
        Ok(obligation_id)
    }
    
    /// Get an obligation by ID
    pub async fn get_obligation(&self, obligation_id: &ObligationId) -> ObligationResult<Obligation> {
        let obligations = self.obligations.read().await;
        
        obligations.get(obligation_id)
            .cloned()
            .ok_or_else(|| ObligationError::TrackingError(format!("Obligation not found: {}", obligation_id)))
    }
    
    /// Get all obligations for an agent
    pub async fn get_agent_obligations(&self, agent_id: &AgentId) -> ObligationResult<Vec<Obligation>> {
        let agent_obligations = self.agent_obligations.read().await;
        let obligations = self.obligations.read().await;
        
        let obligation_ids = agent_obligations.get(agent_id)
            .map(|ids| ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        
        let result = obligation_ids.iter()
            .filter_map(|id| obligations.get(id).cloned())
            .collect();
        
        Ok(result)
    }
    
    /// Get all obligations for a capability
    pub async fn get_capability_obligations(&self, capability_id: &str) -> ObligationResult<Vec<Obligation>> {
        let capability_obligations = self.capability_obligations.read().await;
        let obligations = self.obligations.read().await;
        
        let obligation_ids = capability_obligations.get(capability_id)
            .map(|ids| ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        
        let result = obligation_ids.iter()
            .filter_map(|id| obligations.get(id).cloned())
            .collect();
        
        Ok(result)
    }
    
    /// Update an obligation's status
    pub async fn update_obligation_status(
        &self,
        obligation_id: &ObligationId,
        status: ObligationStatus,
    ) -> ObligationResult<()> {
        let mut obligations = self.obligations.write().await;
        
        let obligation = obligations.get_mut(obligation_id)
            .ok_or_else(|| ObligationError::TrackingError(format!("Obligation not found: {}", obligation_id)))?;
        
        match &status {
            ObligationStatus::Fulfilled { how, .. } => {
                obligation.fulfill(how.clone());
            },
            ObligationStatus::Violated { how, .. } => {
                obligation.violate(how.clone());
            },
            ObligationStatus::Waived { reason, by_agent, .. } => {
                obligation.waive(reason.clone(), by_agent.clone());
            },
            ObligationStatus::Active => {
                // Simply update the status
                obligation.status = status;
                obligation.updated_at = Utc::now();
            },
        }
        
        Ok(())
    }
    
    /// Record capability usage for tracking obligations
    pub async fn record_capability_usage(
        &self,
        capability_id: &str,
        agent_id: &AgentId,
    ) -> ObligationResult<()> {
        // Update usage count
        {
            let mut usage = self.capability_usage.write().await;
            let count = usage.entry(capability_id.to_string()).or_insert(0);
            *count += 1;
        }
        
        // Check for obligations that might be fulfilled by this usage
        let agent_obligations = self.get_agent_obligations(agent_id).await?;
        
        for obligation in agent_obligations {
            if obligation.capability_id() == capability_id && obligation.is_active() {
                match obligation.obligation_type() {
                    ObligationType::UseWithinTime(_) => {
                        // Usage within time obligation is fulfilled when used
                        self.update_obligation_status(
                            &obligation.id,
                            ObligationStatus::Fulfilled {
                                when: Utc::now(),
                                how: "Capability was used within required time".to_string(),
                            },
                        ).await?;
                    },
                    ObligationType::UseMinimumTimes(min_times) => {
                        // Check if the minimum times have been reached
                        let usage = self.capability_usage.read().await;
                        if let Some(count) = usage.get(capability_id) {
                            if *count >= *min_times {
                                self.update_obligation_status(
                                    &obligation.id,
                                    ObligationStatus::Fulfilled {
                                        when: Utc::now(),
                                        how: format!("Capability was used at least {} times", min_times),
                                    },
                                ).await?;
                            }
                        }
                    },
                    ObligationType::UseMaximumTimes(max_times) => {
                        // Check if the maximum times has been exceeded
                        let usage = self.capability_usage.read().await;
                        if let Some(count) = usage.get(capability_id) {
                            if *count > *max_times {
                                self.update_obligation_status(
                                    &obligation.id,
                                    ObligationStatus::Violated {
                                        when: Utc::now(),
                                        how: format!("Capability was used more than {} times", max_times),
                                    },
                                ).await?;
                            }
                        }
                    },
                    _ => {
                        // Other obligation types aren't directly fulfilled by usage
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check for overdue obligations and update their status
    pub async fn check_overdue_obligations(&self) -> ObligationResult<Vec<Obligation>> {
        let mut overdue = Vec::new();
        let mut updates = Vec::new();
        
        // Find active obligations that are overdue
        {
            let obligations = self.obligations.read().await;
            
            for obligation in obligations.values() {
                if obligation.is_active() && obligation.is_overdue() {
                    overdue.push(obligation.clone());
                }
            }
        }
        
        // Update overdue obligations
        for obligation in &overdue {
            match obligation.obligation_type() {
                ObligationType::UseWithinTime(duration) => {
                    // If not used within time, it's violated
                    updates.push((
                        obligation.id().clone(),
                        ObligationStatus::Violated {
                            when: Utc::now(),
                            how: format!("Capability was not used within {} seconds", duration.as_secs()),
                        },
                    ));
                },
                ObligationType::ReportUsage { due, .. } => {
                    // If report not submitted by due date, it's violated
                    updates.push((
                        obligation.id().clone(),
                        ObligationStatus::Violated {
                            when: Utc::now(),
                            how: format!("Usage report was not submitted by {}", due),
                        },
                    ));
                },
                ObligationType::RevokeAt(time) => {
                    // This is a special case - it's not violated, it requires action
                    // The actual revocation would be handled elsewhere
                    continue;
                },
                ObligationType::PayForUsage { deadline, amount, currency, .. } => {
                    // If payment not made by deadline, it's violated
                    updates.push((
                        obligation.id().clone(),
                        ObligationStatus::Violated {
                            when: Utc::now(),
                            how: format!("Payment of {} {} was not made by {}", amount, currency, deadline),
                        },
                    ));
                },
                _ => {
                    // Other obligation types don't have deadlines or are handled differently
                }
            }
        }
        
        // Apply the updates
        for (id, status) in updates {
            self.update_obligation_status(&id, status).await?;
        }
        
        Ok(overdue)
    }
    
    /// Generate a summary of obligations
    pub async fn generate_summary(&self) -> ObligationResult<ObligationSummary> {
        let obligations = self.obligations.read().await;
        
        let mut total = 0;
        let mut active = 0;
        let mut fulfilled = 0;
        let mut violated = 0;
        let mut waived = 0;
        let mut overdue = 0;
        
        for obligation in obligations.values() {
            total += 1;
            
            match obligation.status() {
                ObligationStatus::Active => {
                    active += 1;
                    if obligation.is_overdue() {
                        overdue += 1;
                    }
                },
                ObligationStatus::Fulfilled { .. } => fulfilled += 1,
                ObligationStatus::Violated { .. } => violated += 1,
                ObligationStatus::Waived { .. } => waived += 1,
            }
        }
        
        Ok(ObligationSummary {
            total,
            active,
            fulfilled,
            violated,
            waived,
            overdue,
        })
    }
    
    /// Get all obligations with a specific status
    pub async fn get_obligations_by_status(&self, status: &ObligationStatus) -> ObligationResult<Vec<Obligation>> {
        let obligations = self.obligations.read().await;
        
        let matching = obligations.values()
            .filter(|o| {
                match (o.status(), status) {
                    (ObligationStatus::Active, ObligationStatus::Active) => true,
                    (ObligationStatus::Fulfilled { .. }, ObligationStatus::Fulfilled { .. }) => true,
                    (ObligationStatus::Violated { .. }, ObligationStatus::Violated { .. }) => true,
                    (ObligationStatus::Waived { .. }, ObligationStatus::Waived { .. }) => true,
                    _ => false,
                }
            })
            .cloned()
            .collect();
        
        Ok(matching)
    }
    
    /// Get all overdue obligations
    pub async fn get_overdue_obligations(&self) -> ObligationResult<Vec<Obligation>> {
        let obligations = self.obligations.read().await;
        
        let overdue = obligations.values()
            .filter(|o| o.is_active() && o.is_overdue())
            .cloned()
            .collect();
        
        Ok(overdue)
    }
    
    /// Remove an obligation
    pub async fn remove_obligation(&self, obligation_id: &ObligationId) -> ObligationResult<()> {
        // Get the obligation first to retrieve agent and capability IDs
        let obligation = self.get_obligation(obligation_id).await?;
        let agent_id = obligation.agent_id().clone();
        let capability_id = obligation.capability_id().to_string();
        
        // Update the indices
        {
            let mut agent_obligations = self.agent_obligations.write().await;
            if let Some(agent_set) = agent_obligations.get_mut(&agent_id) {
                agent_set.remove(obligation_id);
                if agent_set.is_empty() {
                    agent_obligations.remove(&agent_id);
                }
            }
        }
        
        {
            let mut capability_obligations = self.capability_obligations.write().await;
            if let Some(capability_set) = capability_obligations.get_mut(&capability_id) {
                capability_set.remove(obligation_id);
                if capability_set.is_empty() {
                    capability_obligations.remove(&capability_id);
                }
            }
        }
        
        // Remove the obligation
        {
            let mut obligations = self.obligations.write().await;
            obligations.remove(obligation_id);
        }
        
        Ok(())
    }
    
    /// Create a report of obligations for an agent
    pub async fn generate_agent_report(&self, agent_id: &AgentId) -> ObligationResult<String> {
        let obligations = self.get_agent_obligations(agent_id).await?;
        
        let mut active = 0;
        let mut fulfilled = 0;
        let mut violated = 0;
        let mut waived = 0;
        let mut overdue = 0;
        
        for obligation in &obligations {
            match obligation.status() {
                ObligationStatus::Active => {
                    active += 1;
                    if obligation.is_overdue() {
                        overdue += 1;
                    }
                },
                ObligationStatus::Fulfilled { .. } => fulfilled += 1,
                ObligationStatus::Violated { .. } => violated += 1,
                ObligationStatus::Waived { .. } => waived += 1,
            }
        }
        
        let report = format!(
            "Obligation Report for Agent {}\n\
             Total Obligations: {}\n\
             Active: {}\n\
             Fulfilled: {}\n\
             Violated: {}\n\
             Waived: {}\n\
             Overdue: {}\n",
            agent_id,
            obligations.len(),
            active,
            fulfilled,
            violated,
            waived,
            overdue
        );
        
        Ok(report)
    }
    
    /// Enforce obligations by checking and updating their status
    pub async fn enforce_obligations(&self) -> ObligationResult<ObligationEnforcementResult> {
        // Check for overdue obligations
        let overdue = self.check_overdue_obligations().await?;
        
        // Look for obligations that require special enforcement
        let mut revocations = Vec::new();
        
        let obligations = self.obligations.read().await;
        for obligation in obligations.values() {
            if obligation.is_active() {
                match obligation.obligation_type() {
                    ObligationType::RevokeAt(time) => {
                        let now = Utc::now();
                        if now >= *time {
                            // Add to list of capabilities that need to be revoked
                            revocations.push((
                                obligation.agent_id().clone(),
                                obligation.capability_id().to_string(),
                                obligation.id().clone(),
                            ));
                        }
                    },
                    _ => {
                        // Other obligation types are handled differently
                    }
                }
            }
        }
        
        // Return the enforcement result
        Ok(ObligationEnforcementResult {
            overdue_count: overdue.len(),
            revocations,
        })
    }
}

impl Default for ObligationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of obligation enforcement
#[derive(Clone, Debug)]
pub struct ObligationEnforcementResult {
    /// Number of overdue obligations
    pub overdue_count: usize,
    
    /// List of capabilities that need to be revoked
    pub revocations: Vec<(AgentId, String, ObligationId)>,
}

/// Trait for adding obligations to capabilities
#[async_trait]
pub trait CapabilityObligation {
    /// Add an obligation to a capability
    async fn add_obligation_to_capability(
        &self,
        capability_id: &str,
        agent_id: &AgentId,
        obligation_type: ObligationType,
    ) -> ObligationResult<ObligationId>;
    
    /// Check if a capability has any active obligations
    async fn has_active_obligations(&self, capability_id: &str) -> ObligationResult<bool>;
    
    /// Get all obligations for a capability
    async fn get_capability_obligations(&self, capability_id: &str) -> ObligationResult<Vec<Obligation>>;
    
    /// Record usage of a capability
    async fn record_capability_usage(&self, capability_id: &str, agent_id: &AgentId) -> ObligationResult<()>;
    
    /// Check if using a capability would violate any obligations
    async fn check_capability_usage(
        &self,
        capability_id: &str,
        agent_id: &AgentId,
    ) -> ObligationResult<bool>;
}

#[async_trait]
impl CapabilityObligation for ObligationManager {
    async fn add_obligation_to_capability(
        &self,
        capability_id: &str,
        agent_id: &AgentId,
        obligation_type: ObligationType,
    ) -> ObligationResult<ObligationId> {
        let obligation = Obligation::new(
            capability_id,
            agent_id.clone(),
            obligation_type,
            agent_id.clone(), // For simplicity, assume the agent is adding their own obligation
        );
        
        self.add_obligation(obligation).await
    }
    
    async fn has_active_obligations(&self, capability_id: &str) -> ObligationResult<bool> {
        let obligations = self.get_capability_obligations(capability_id).await?;
        
        let has_active = obligations.iter().any(|o| o.is_active());
        
        Ok(has_active)
    }
    
    async fn get_capability_obligations(&self, capability_id: &str) -> ObligationResult<Vec<Obligation>> {
        self.get_capability_obligations(capability_id).await
    }
    
    async fn record_capability_usage(&self, capability_id: &str, agent_id: &AgentId) -> ObligationResult<()> {
        self.record_capability_usage(capability_id, agent_id).await
    }
    
    async fn check_capability_usage(
        &self,
        capability_id: &str,
        agent_id: &AgentId,
    ) -> ObligationResult<bool> {
        let obligations = self.get_capability_obligations(capability_id).await?;
        
        // Check if there are any obligation violations that would prevent usage
        for obligation in &obligations {
            if obligation.agent_id() == agent_id && obligation.is_active() {
                match obligation.obligation_type() {
                    ObligationType::UseMaximumTimes(max_times) => {
                        // Check if the maximum times would be exceeded
                        let usage = self.capability_usage.read().await;
                        if let Some(count) = usage.get(capability_id) {
                            if *count >= *max_times {
                                return Ok(false); // Would violate the maximum times obligation
                            }
                        }
                    },
                    ObligationType::DoNotDelegate => {
                        // This obligation doesn't affect usage, only delegation
                    },
                    ObligationType::DelegateOnlyTo(_) => {
                        // This obligation doesn't affect usage, only delegation
                    },
                    ObligationType::RevokeAt(time) => {
                        // Check if the revocation time has passed
                        let now = Utc::now();
                        if now >= *time {
                            return Ok(false); // Should be revoked already
                        }
                    },
                    _ => {
                        // Other obligation types don't prevent usage
                    }
                }
            }
        }
        
        // No violations found
        Ok(true)
    }
}

// Effect for obligation management
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObligationEffect {
    /// Agent ID
    pub agent_id: AgentId,
    
    /// Effect type
    pub effect_type: ObligationEffectType,
    
    /// Capability ID
    pub capability_id: String,
    
    /// Obligation ID (for update and remove)
    pub obligation_id: Option<ObligationId>,
    
    /// Obligation type (for create)
    pub obligation_type: Option<ObligationType>,
}

/// Types of obligation effects
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObligationEffectType {
    /// Create a new obligation
    Create,
    
    /// Update an existing obligation
    Update(ObligationStatus),
    
    /// Remove an obligation
    Remove,
    
    /// Record capability usage
    RecordUsage,
}

impl ObligationEffect {
    /// Create a new obligation creation effect
    pub fn create(
        agent_id: AgentId,
        capability_id: impl Into<String>,
        obligation_type: ObligationType,
    ) -> Self {
        Self {
            agent_id,
            effect_type: ObligationEffectType::Create,
            capability_id: capability_id.into(),
            obligation_id: None,
            obligation_type: Some(obligation_type),
        }
    }
    
    /// Create a new obligation update effect
    pub fn update(
        agent_id: AgentId,
        capability_id: impl Into<String>,
        obligation_id: ObligationId,
        status: ObligationStatus,
    ) -> Self {
        Self {
            agent_id,
            effect_type: ObligationEffectType::Update(status),
            capability_id: capability_id.into(),
            obligation_id: Some(obligation_id),
            obligation_type: None,
        }
    }
    
    /// Create a new obligation removal effect
    pub fn remove(
        agent_id: AgentId,
        capability_id: impl Into<String>,
        obligation_id: ObligationId,
    ) -> Self {
        Self {
            agent_id,
            effect_type: ObligationEffectType::Remove,
            capability_id: capability_id.into(),
            obligation_id: Some(obligation_id),
            obligation_type: None,
        }
    }
    
    /// Create a new usage recording effect
    pub fn record_usage(
        agent_id: AgentId,
        capability_id: impl Into<String>,
    ) -> Self {
        Self {
            agent_id,
            effect_type: ObligationEffectType::RecordUsage,
            capability_id: capability_id.into(),
            obligation_id: None,
            obligation_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use crate::resource::agent::AgentType;
    
    #[tokio::test]
    async fn test_obligation_creation() {
        let agent_id = AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User);
        let created_by = agent_id.clone();
        
        // Create an obligation
        let obligation = Obligation::new(
            "test-capability",
            agent_id,
            ObligationType::UseWithinTime(Duration::from_secs(3600)),
            created_by,
        );
        
        // Check properties
        assert_eq!(obligation.capability_id(), "test-capability");
        assert!(matches!(obligation.status(), ObligationStatus::Active));
        assert!(obligation.has_deadline());
        assert!(!obligation.is_overdue()); // Should not be overdue yet
        
        // Test metadata
        let mut obligation = obligation;
        obligation.set_metadata("importance", "high");
        assert_eq!(obligation.get_metadata("importance"), Some(&"high".to_string()));
    }
    
    #[tokio::test]
    async fn test_obligation_status_updates() {
        let agent_id = AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User);
        let created_by = agent_id.clone();
        
        // Create an obligation
        let mut obligation = Obligation::new(
            "test-capability",
            agent_id.clone(),
            ObligationType::UseMinimumTimes(5),
            created_by,
        );
        
        // Check initial status
        assert!(obligation.is_active());
        assert!(!obligation.is_fulfilled());
        
        // Fulfill the obligation
        obligation.fulfill("Used capability 5 times");
        
        // Check updated status
        assert!(!obligation.is_active());
        assert!(obligation.is_fulfilled());
        
        // Create another obligation
        let mut obligation2 = Obligation::new(
            "test-capability-2",
            agent_id.clone(),
            ObligationType::UseMaximumTimes(10),
            created_by.clone(),
        );
        
        // Violate the obligation
        obligation2.violate("Used capability more than 10 times");
        
        // Check status
        assert!(!obligation2.is_active());
        assert!(obligation2.is_violated());
        
        // Create a third obligation
        let mut obligation3 = Obligation::new(
            "test-capability-3",
            agent_id.clone(),
            ObligationType::DoNotDelegate,
            created_by.clone(),
        );
        
        // Waive the obligation
        let waiver_agent = AgentId::from_content_hash(ContentHash::calculate(b"waiver-agent").as_bytes(), AgentType::Operator);
        obligation3.waive("No longer required", waiver_agent.clone());
        
        // Check status
        assert!(!obligation3.is_active());
        if let ObligationStatus::Waived { by_agent, .. } = obligation3.status() {
            assert_eq!(by_agent, &waiver_agent);
        } else {
            panic!("Expected Waived status");
        }
    }
    
    #[tokio::test]
    async fn test_obligation_manager_basic() {
        let manager = ObligationManager::new();
        let agent_id = AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User);
        let created_by = agent_id.clone();
        
        // Create and add an obligation
        let obligation = Obligation::new(
            "test-capability",
            agent_id.clone(),
            ObligationType::UseWithinTime(Duration::from_secs(3600)),
            created_by.clone(),
        );
        
        let obligation_id = manager.add_obligation(obligation).await.unwrap();
        
        // Get the obligation
        let retrieved = manager.get_obligation(&obligation_id).await.unwrap();
        assert_eq!(retrieved.capability_id(), "test-capability");
        
        // Get agent obligations
        let agent_obligations = manager.get_agent_obligations(&agent_id).await.unwrap();
        assert_eq!(agent_obligations.len(), 1);
        
        // Get capability obligations
        let capability_obligations = manager.get_capability_obligations("test-capability").await.unwrap();
        assert_eq!(capability_obligations.len(), 1);
    }
    
    #[tokio::test]
    async fn test_obligation_usage_tracking() {
        let manager = ObligationManager::new();
        let agent_id = AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User);
        let created_by = agent_id.clone();
        
        // Create a minimum usage obligation
        let min_obligation = Obligation::new(
            "min-capability",
            agent_id.clone(),
            ObligationType::UseMinimumTimes(3),
            created_by.clone(),
        );
        
        let min_id = manager.add_obligation(min_obligation).await.unwrap();
        
        // Create a maximum usage obligation
        let max_obligation = Obligation::new(
            "max-capability",
            agent_id.clone(),
            ObligationType::UseMaximumTimes(2),
            created_by.clone(),
        );
        
        let max_id = manager.add_obligation(max_obligation).await.unwrap();
        
        // Record usage for min-capability
        manager.record_capability_usage("min-capability", &agent_id).await.unwrap();
        manager.record_capability_usage("min-capability", &agent_id).await.unwrap();
        
        // Not fulfilled yet
        let min_ob = manager.get_obligation(&min_id).await.unwrap();
        assert!(min_ob.is_active());
        
        // Record one more usage
        manager.record_capability_usage("min-capability", &agent_id).await.unwrap();
        
        // Should be fulfilled now
        let min_ob = manager.get_obligation(&min_id).await.unwrap();
        assert!(min_ob.is_fulfilled());
        
        // Record usage for max-capability
        manager.record_capability_usage("max-capability", &agent_id).await.unwrap();
        manager.record_capability_usage("max-capability", &agent_id).await.unwrap();
        
        // Still valid
        let max_ob = manager.get_obligation(&max_id).await.unwrap();
        assert!(max_ob.is_active());
        
        // Exceed the maximum
        manager.record_capability_usage("max-capability", &agent_id).await.unwrap();
        
        // Should be violated now
        let max_ob = manager.get_obligation(&max_id).await.unwrap();
        assert!(max_ob.is_violated());
    }
    
    #[tokio::test]
    async fn test_obligation_summary() {
        let manager = ObligationManager::new();
        let agent_id = AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User);
        let created_by = agent_id.clone();
        
        // Create different obligations with different statuses
        let active_ob = Obligation::new(
            "active-cap",
            agent_id.clone(),
            ObligationType::UseMinimumTimes(5),
            created_by.clone(),
        );
        
        let mut fulfilled_ob = Obligation::new(
            "fulfilled-cap",
            agent_id.clone(),
            ObligationType::UseWithinTime(Duration::from_secs(3600)),
            created_by.clone(),
        );
        fulfilled_ob.fulfill("Used within time");
        
        let mut violated_ob = Obligation::new(
            "violated-cap",
            agent_id.clone(),
            ObligationType::UseMaximumTimes(3),
            created_by.clone(),
        );
        violated_ob.violate("Exceeded max usage");
        
        let mut waived_ob = Obligation::new(
            "waived-cap",
            agent_id.clone(),
            ObligationType::DoNotDelegate,
            created_by.clone(),
        );
        waived_ob.waive("No longer applicable", created_by.clone());
        
        // Add all obligations
        manager.add_obligation(active_ob).await.unwrap();
        manager.add_obligation(fulfilled_ob).await.unwrap();
        manager.add_obligation(violated_ob).await.unwrap();
        manager.add_obligation(waived_ob).await.unwrap();
        
        // Generate summary
        let summary = manager.generate_summary().await.unwrap();
        
        // Check counts
        assert_eq!(summary.total, 4);
        assert_eq!(summary.active, 1);
        assert_eq!(summary.fulfilled, 1);
        assert_eq!(summary.violated, 1);
        assert_eq!(summary.waived, 1);
    }
    
    #[tokio::test]
    async fn test_capability_obligation_trait() {
        let manager = ObligationManager::new();
        let agent_id = AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User);
        
        // Add an obligation using the trait
        let obligation_id = manager.add_obligation_to_capability(
            "test-capability",
            &agent_id,
            ObligationType::UseWithinTime(Duration::from_secs(3600)),
        ).await.unwrap();
        
        // Check if the capability has active obligations
        let has_active = manager.has_active_obligations("test-capability").await.unwrap();
        assert!(has_active);
        
        // Record a usage
        manager.record_capability_usage("test-capability", &agent_id).await.unwrap();
        
        // Check if the obligation is now fulfilled
        let obligation = manager.get_obligation(&obligation_id).await.unwrap();
        assert!(obligation.is_fulfilled());
        
        // Add a maximum usage obligation
        manager.add_obligation_to_capability(
            "max-capability",
            &agent_id,
            ObligationType::UseMaximumTimes(2),
        ).await.unwrap();
        
        // Record usages up to the limit
        manager.record_capability_usage("max-capability", &agent_id).await.unwrap();
        manager.record_capability_usage("max-capability", &agent_id).await.unwrap();
        
        // Check if additional usage would be allowed
        let can_use = manager.check_capability_usage("max-capability", &agent_id).await.unwrap();
        assert!(!can_use, "Should not allow usage beyond the maximum");
    }
} 