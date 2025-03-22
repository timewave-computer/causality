// Fact Dependency Validator for Causality
//
// This module provides validation for fact dependencies in effects.

use std::collections::{HashMap, HashSet};
use crate::types::{DomainId, ResourceId, Timestamp};
use crate::error::{Error, Result};
use crate::log::fact_types::{FactType, RegisterFact, ZKProofFact};
use crate::log::fact_snapshot::{FactSnapshot, FactId, FactDependency, FactDependencyType, RegisterObservation};
use crate::effect::Effect;
use crate::resource::register::RegisterId;

/// Validates fact dependencies for effects
#[derive(Debug, Clone)]
pub struct FactDependencyValidator {
    /// Observed facts
    observed_facts: HashSet<FactId>,
    /// Register observations
    register_observations: HashMap<RegisterId, RegisterObservation>,
    /// Domains containing facts
    domains: HashSet<DomainId>,
    /// Map of fact IDs to verified status
    fact_cache: HashMap<FactId, bool>,
    /// Map of domain IDs to their allowed maximum age (in seconds)
    domain_freshness: HashMap<DomainId, u64>,
}

impl FactDependencyValidator {
    /// Create a new fact dependency validator
    pub fn new() -> Self {
        FactDependencyValidator {
            observed_facts: HashSet::new(),
            register_observations: HashMap::new(),
            domains: HashSet::new(),
            fact_cache: HashMap::new(),
            domain_freshness: HashMap::new(),
        }
    }
    
    /// Add an observed fact
    pub fn add_fact(&mut self, fact_id: FactId, domain_id: &DomainId) {
        self.observed_facts.insert(fact_id);
        self.domains.insert(domain_id.clone());
    }
    
    /// Add a register observation
    pub fn add_register_observation(&mut self, observation: RegisterObservation) {
        self.register_observations.insert(observation.register_id.clone(), observation);
        self.domains.insert(observation.domain_id.clone());
    }
    
    /// Set the freshness requirement for a domain
    pub fn set_domain_freshness(&mut self, domain_id: DomainId, max_age_seconds: u64) {
        self.domain_freshness.insert(domain_id, max_age_seconds);
    }
    
    /// Check if a fact exists and is verified
    pub fn is_fact_verified(&self, fact_id: &FactId) -> bool {
        self.fact_cache.get(fact_id).copied().unwrap_or(false)
    }
    
    /// Validate a set of fact dependencies
    pub fn validate(&self, dependencies: &[FactDependency]) -> bool {
        for dependency in dependencies {
            if dependency.is_required() && !self.observed_facts.contains(&dependency.fact_id) {
                return false;
            }
        }
        true
    }
    
    /// Validate register dependencies
    pub fn validate_register_dependencies(&self, register_dependencies: &[RegisterId]) -> bool {
        for register_id in register_dependencies {
            if !self.register_observations.contains_key(register_id) {
                return false;
            }
        }
        true
    }
    
    /// Get observed facts
    pub fn observed_facts(&self) -> &HashSet<FactId> {
        &self.observed_facts
    }
    
    /// Get register observations
    pub fn register_observations(&self) -> &HashMap<RegisterId, RegisterObservation> {
        &self.register_observations
    }
    
    /// Get domains
    pub fn domains(&self) -> &HashSet<DomainId> {
        &self.domains
    }
    
    /// Build from a list of fact dependencies
    pub fn from_dependencies(dependencies: &[FactDependency]) -> Self {
        let mut validator = FactDependencyValidator::new();
        
        for dependency in dependencies {
            validator.add_fact(dependency.fact_id.clone(), &dependency.domain_id);
        }
        
        validator
    }
    
    /// Check if a domain is contained in the validator
    pub fn contains_domain(&self, domain_id: &DomainId) -> bool {
        self.domains.contains(domain_id)
    }
    
    /// Check if a fact is contained in the validator
    pub fn contains_fact(&self, fact_id: &FactId) -> bool {
        self.observed_facts.contains(fact_id)
    }
    
    /// Check if a register is contained in the validator
    pub fn contains_register(&self, register_id: &RegisterId) -> bool {
        self.register_observations.contains_key(register_id)
    }
    
    /// Merge a list of fact dependencies into this validator
    pub fn merge_dependencies(&mut self, dependencies: &[FactDependency]) {
        for dependency in dependencies {
            self.add_fact(dependency.fact_id.clone(), &dependency.domain_id);
        }
    }
    
    /// Merge a register observation into this validator
    pub fn merge_register_observation(&mut self, observation: &RegisterObservation) {
        self.add_register_observation(observation.clone());
    }
    
    /// Merge another validator into this one
    pub fn merge(&mut self, other: &FactDependencyValidator) {
        for fact_id in other.observed_facts() {
            self.observed_facts.insert(fact_id.clone());
        }
        
        for (register_id, observation) in other.register_observations() {
            self.register_observations.insert(register_id.clone(), observation.clone());
        }
        
        for domain_id in other.domains() {
            self.domains.insert(domain_id.clone());
        }
    }
    
    /// Validate a single fact dependency
    pub fn validate_dependency(&self, dependency: &FactDependency) -> Result<()> {
        // Check if the fact exists in the cache
        if !self.fact_cache.contains_key(&dependency.fact_id) {
            return Err(Error::ValidationError(format!(
                "Fact dependency not found: {}",
                dependency.fact_id.0
            )));
        }
        
        // If it's a required dependency, check if it's verified
        if dependency.is_required() && !self.is_fact_verified(&dependency.fact_id) {
            return Err(Error::ValidationError(format!(
                "Required fact dependency not verified: {}",
                dependency.fact_id.0
            )));
        }
        
        Ok(())
    }
    
    /// Validate all dependencies in an effect
    pub fn validate_effect_dependencies(&self, effect: &dyn Effect) -> Result<()> {
        let dependencies = effect.fact_dependencies();
        
        for dependency in dependencies {
            self.validate_dependency(&dependency)?;
        }
        
        // If the effect has a fact snapshot, validate it too
        if let Some(snapshot) = effect.fact_snapshot() {
            self.validate_snapshot(&snapshot)?;
        }
        
        Ok(())
    }
    
    /// Validate an entire fact snapshot
    pub fn validate_snapshot(&self, snapshot: &FactSnapshot) -> Result<()> {
        // Check all facts in the snapshot
        for fact_id in &snapshot.observed_facts {
            if !self.fact_cache.contains_key(fact_id) {
                return Err(Error::ValidationError(format!(
                    "Fact in snapshot not found: {}",
                    fact_id.0
                )));
            }
        }
        
        // Check all register observations
        for (register_id, observation) in &snapshot.register_observations {
            // Make sure the fact for the observation exists
            if !self.fact_cache.contains_key(&observation.fact_id) {
                return Err(Error::ValidationError(format!(
                    "Register observation fact not found: {} for register {}",
                    observation.fact_id.0, register_id
                )));
            }
            
            // Check if we have a more recent observation
            if let Some(latest) = self.register_observations.get(register_id) {
                if latest.observed_at > observation.observed_at {
                    return Err(Error::ValidationError(format!(
                        "Register observation is outdated for register {}",
                        register_id
                    )));
                }
            }
        }
        
        // Check domain freshness if configured
        for domain_id in &snapshot.domains {
            if let Some(max_age) = self.domain_freshness.get(domain_id) {
                // TODO: Implement actual time-based freshness check
                // For now, we'll just assume it's fresh enough
            }
        }
        
        Ok(())
    }
    
    /// Load facts from a snapshot into the validator
    pub fn load_from_snapshot(&mut self, snapshot: &FactSnapshot) {
        // Add all facts
        for fact_id in &snapshot.observed_facts {
            self.add_fact(fact_id.clone(), &snapshot.domains[0]);
        }
        
        // Add all register observations
        for (_, observation) in &snapshot.register_observations {
            self.add_register_observation(observation.clone());
        }
    }
} 