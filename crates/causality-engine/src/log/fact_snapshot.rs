// Fact snapshot functionality
// Original file: src/log/fact_snapshot.rs

// Fact Snapshot for Causality
//
// This module implements the FactSnapshot struct that represents
// a point-in-time collection of facts that effects depend on.

use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

use causality_types::{DomainId, Timestamp, ContentId};
// Import FactType from our own module instead of causality_types
use crate::log::fact_types::FactType;

// ContentId is already imported from causality_types

/// A unique identifier for a fact
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub String);

/// A struct representing a point-in-time snapshot of facts
/// that an effect depends on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSnapshot {
    /// Facts observed before the effect
    pub observed_facts: Vec<FactId>,
    
    /// The observer (committee) that observed the facts
    pub observer: String,
    
    /// The timestamp when the snapshot was created
    pub created_at: Timestamp,
    
    /// Register observations included in this snapshot
    pub register_observations: HashMap<ContentId, RegisterObservation>,
    
    /// Domains that contributed facts to this snapshot
    pub domains: HashSet<DomainId>,
    
    /// Additional metadata for the snapshot
    pub metadata: HashMap<String, String>,
}

/// Represents an observation of a register's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterObservation {
    /// The observed register ID
    pub register_id: ContentId,
    
    /// The fact ID of the register observation
    pub fact_id: FactId,
    
    /// The domain the register was observed in
    pub domain_id: DomainId,
    
    /// The timestamp of the observation
    pub observed_at: Timestamp,
    
    /// The hash of the register data
    pub data_hash: String,
}

impl FactSnapshot {
    /// Create a new empty fact snapshot
    pub fn new(observer: &str) -> Self {
        FactSnapshot {
            observed_facts: Vec::new(),
            observer: observer.to_string(),
            created_at: Timestamp::now(),
            register_observations: HashMap::new(),
            domains: HashSet::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a fact to the snapshot
    pub fn add_fact(&mut self, fact_id: FactId, domain_id: DomainId) {
        self.observed_facts.push(fact_id);
        self.domains.insert(domain_id);
    }
    
    /// Add a register observation to the snapshot
    pub fn add_register_observation(
        &mut self,
        register_id: ContentId,
        fact_id: FactId,
        domain_id: DomainId,
        data_hash: &str,
    ) {
        let observation = RegisterObservation {
            register_id: register_id.clone(),
            fact_id,
            domain_id: domain_id.clone(),
            observed_at: Timestamp::now(),
            data_hash: data_hash.to_string(),
        };
        
        self.register_observations.insert(register_id, observation);
        self.domains.insert(domain_id);
    }
    
    /// Add metadata to the snapshot
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Check if the snapshot contains a specific fact
    pub fn contains_fact(&self, fact_id: &FactId) -> bool {
        self.observed_facts.contains(fact_id)
    }
    
    /// Check if the snapshot contains an observation for a register
    pub fn contains_register(&self, register_id: &ContentId) -> bool {
        self.register_observations.contains_key(register_id)
    }
    
    /// Get the number of facts in the snapshot
    pub fn fact_count(&self) -> usize {
        self.observed_facts.len() + self.register_observations.len()
    }
    
    /// Get all domain IDs in this snapshot
    pub fn get_domains(&self) -> Vec<DomainId> {
        self.domains.iter().cloned().collect()
    }
}

/// Fact dependency type, used to indicate why a fact is needed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactDependencyType {
    /// Fact is required for the effect to be valid
    Required,
    
    /// Fact is used by the effect but not strictly required
    Optional,
    
    /// Fact provides additional context for the effect
    Context,
}

/// A struct representing a dependency on a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactDependency {
    /// The ID of the fact
    pub fact_id: FactId,
    
    /// The type of dependency
    pub dependency_type: FactDependencyType,
    
    /// The domain the fact comes from
    pub domain_id: DomainId,
    
    /// The type of the fact (optional)
    pub fact_type: Option<FactType>,
}

impl FactDependency {
    /// Create a new fact dependency
    pub fn new(
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) -> Self {
        FactDependency {
            fact_id,
            domain_id,
            dependency_type,
            fact_type: None,
        }
    }
    
    /// Add the fact type to this dependency
    pub fn with_fact_type(mut self, fact_type: FactType) -> Self {
        self.fact_type = Some(fact_type);
        self
    }
    
    /// Check if this is a required dependency
    pub fn is_required(&self) -> bool {
        matches!(self.dependency_type, FactDependencyType::Required)
    }
} 
