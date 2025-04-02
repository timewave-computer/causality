// Fact tracking module
// This file defines types for tracking facts

use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Serialize, Deserialize};
use causality_types::DomainId;

/// A fact ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub String);

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of dependency between facts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactDependencyType {
    /// The fact is required for the effect
    Required,
    /// The fact is optional for the effect
    Optional,
    /// The effect creates this fact
    Creates,
    /// The effect invalidates this fact
    Invalidates,
}

/// Dependency on a fact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactDependency {
    /// The fact ID
    pub fact_id: FactId,
    /// The domain ID
    pub domain_id: DomainId,
    /// The type of dependency
    pub dependency_type: FactDependencyType,
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
        }
    }
}

/// Register observation in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterObservation {
    /// The fact ID
    pub fact_id: FactId,
    /// The domain ID
    pub domain_id: DomainId,
    /// The register value
    pub value: u64,
}

/// A snapshot of facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSnapshot {
    /// The observer name
    pub observer: String,
    /// The observed facts
    pub observed_facts: HashSet<FactId>,
    /// The domains involved
    pub domains: HashSet<DomainId>,
    /// Register observations
    pub register_observations: HashMap<String, RegisterObservation>,
}

impl FactSnapshot {
    /// Create a new fact snapshot
    pub fn new(observer: &str) -> Self {
        FactSnapshot {
            observer: observer.to_string(),
            observed_facts: HashSet::new(),
            domains: HashSet::new(),
            register_observations: HashMap::new(),
        }
    }
    
    /// Add a fact to the snapshot
    pub fn add_fact(&mut self, fact_id: FactId, domain_id: DomainId) {
        self.observed_facts.insert(fact_id);
        self.domains.insert(domain_id);
    }
    
    /// Add a register observation
    pub fn add_register_observation(
        &mut self,
        name: &str,
        fact_id: FactId,
        domain_id: DomainId,
        value: u64,
    ) {
        self.register_observations.insert(
            name.to_string(),
            RegisterObservation {
                fact_id,
                domain_id,
                value,
            },
        );
    }
    
    /// Check if the snapshot contains a fact
    pub fn contains_fact(&self, fact_id: &FactId) -> bool {
        self.observed_facts.contains(fact_id)
    }
} 
