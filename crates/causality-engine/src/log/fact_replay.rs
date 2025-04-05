// Fact replay functionality
// Original file: src/log/fact_replay.rs

// Fact Replay Engine for Causality
//
// This module provides mechanisms for replaying facts in chronological
// order to reconstruct state and verify operation correctness.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use causality_types::DomainId;
use causality_error::{EngineError, Result};
use crate::log::{FactEntry, LogStorage};
use crate::log::fact_types::{FactType, RegisterFact};
use crate::log::fact::{FactId, FactSnapshot};
use causality_types::ContentId;

#[cfg(feature = "md5")]
use causality_crypto::md5::Md5ChecksumFunction;
#[cfg(not(feature = "md5"))]
use causality_crypto::hash::HashFactory;

/// Callback type for fact replay events
pub type FactReplayCallback = Box<dyn Fn(&FactEntry) -> Result<()> + Send>;

/// Status of a fact replay operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactReplayStatus {
    /// The replay is running
    Running,
    /// The replay is paused
    Paused,
    /// The replay is completed
    Completed,
    /// The replay has failed
    Failed(String),
}

/// Configuration for fact replay
#[derive(Debug, Clone)]
pub struct FactReplayConfig {
    /// Whether to verify facts during replay
    pub verify_facts: bool,
    /// Whether to apply register state updates
    pub apply_register_updates: bool,
    /// Whether to stop on errors
    pub stop_on_error: bool,
    /// Maximum number of facts to replay
    pub max_facts: Option<usize>,
    /// Filter by domain IDs (empty means all domains)
    pub domain_filter: HashSet<DomainId>,
    /// Filter by resource IDs (empty means all resources)
    pub resource_filter: HashSet<ContentId>,
}

impl Default for FactReplayConfig {
    fn default() -> Self {
        FactReplayConfig {
            verify_facts: true,
            apply_register_updates: true,
            stop_on_error: true,
            max_facts: None,
            domain_filter: HashSet::new(),
            resource_filter: HashSet::new(),
        }
    }
}

/// Engine for replaying facts to reconstruct state
pub struct FactReplayEngine {
    /// The storage to read facts from
    storage: Arc<Mutex<dyn LogStorage + Send>>,
    /// Configuration for replay
    config: FactReplayConfig,
    /// Current status of replay
    status: FactReplayStatus,
    /// Map of register IDs to their current state
    register_states: HashMap<ContentId, Vec<u8>>,
    /// Map of fact IDs to their entries
    fact_cache: HashMap<FactId, FactEntry>,
    /// Callbacks for fact replay events
    callbacks: Vec<FactReplayCallback>,
    /// Domain ID
    domain_id: DomainId,
}

impl std::fmt::Debug for FactReplayEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FactReplayEngine")
            .field("config", &self.config)
            .field("status", &self.status)
            .field("register_states_count", &self.register_states.len())
            .field("fact_cache_count", &self.fact_cache.len())
            .field("callbacks_count", &self.callbacks.len())
            .field("domain_id", &self.domain_id)
            .finish()
    }
}

impl FactReplayEngine {
    /// Create a new fact replay engine
    pub fn new(
        storage: Arc<Mutex<dyn LogStorage + Send>>,
        config: FactReplayConfig,
        domain_id: DomainId,
    ) -> Self {
        FactReplayEngine {
            storage,
            config,
            status: FactReplayStatus::Paused,
            register_states: HashMap::new(),
            fact_cache: HashMap::new(),
            callbacks: Vec::new(),
            domain_id,
        }
    }
    
    /// Add a callback for fact replay events
    pub fn add_callback(&mut self, callback: FactReplayCallback) {
        self.callbacks.push(callback);
    }
    
    /// Start or resume the replay
    pub fn start(&mut self) -> Result<()> {
        self.status = FactReplayStatus::Running;
        self.replay_facts()
    }
    
    /// Pause the replay
    pub fn pause(&mut self) {
        if self.status == FactReplayStatus::Running {
            self.status = FactReplayStatus::Paused;
        }
    }
    
    /// Get the current status
    pub fn status(&self) -> &FactReplayStatus {
        &self.status
    }
    
    /// Replay facts from storage
    fn replay_facts(&mut self) -> Result<()> {
        // First, collect all the facts we need to process
        let facts_to_process = {
            let storage = self.storage.lock()
                .map_err(|_| EngineError::LogError("Failed to lock storage".to_string()))?;
            
            let entry_count = storage.entry_count()?;
            let entries = storage.read(0, entry_count)?;
            
            // Filter and collect facts
            let mut result = Vec::new();
            for entry in entries {
                if let crate::log::EntryData::Fact(fact) = entry.data {
                    // Apply domain filter if configured
                    if !self.config.domain_filter.is_empty() 
                        && !self.config.domain_filter.contains(&fact.domain_id) {
                        continue;
                    }
                    
                    // Apply resource filter if configured
                    if self.config.resource_filter.is_empty() 
                        || (fact.resources.is_some() && fact.resources.as_ref().unwrap().iter().any(|r| self.config.resource_filter.contains(r))) {
                        result.push(fact);
                    }
                }
            }
            
            // Sort facts by timestamp
            result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            result
        }; // storage lock is released here
        
        // Now process the facts
        let mut facts_processed = 0;
        for fact in facts_to_process {
            // Check if we should stop
            if let Some(max_facts) = self.config.max_facts {
                if facts_processed >= max_facts {
                    self.status = FactReplayStatus::Completed;
                    return Ok(());
                }
            }
            
            // Process the fact
            if let Err(e) = self.process_fact(&fact) {
                if self.config.stop_on_error {
                    self.status = FactReplayStatus::Failed(e.to_string());
                    return Err(e);
                }
            }
            
            // Cache the fact
            let fact_id = self.build_fact_id(&fact);
            self.fact_cache.insert(fact_id, fact.clone());
            
            // Invoke callbacks
            for callback in &self.callbacks {
                if let Err(e) = callback(&fact) {
                    if self.config.stop_on_error {
                        self.status = FactReplayStatus::Failed(e.to_string());
                        return Err(e);
                    }
                }
            }
            
            facts_processed += 1;
            
            // Check if we've been paused
            if self.status == FactReplayStatus::Paused {
                return Ok(());
            }
        }
        
        self.status = FactReplayStatus::Completed;
        Ok(())
    }
    
    /// Process a single fact
    fn process_fact(&mut self, fact: &FactEntry) -> Result<()> {
        // Placeholder for fact verification logic
        if self.config.verify_facts {
            // In a real implementation, this would verify the fact's proof
        }
        
        // Handle register-related facts if configured
        if self.config.apply_register_updates {
            match self.extract_fact_type(fact) {
                Some(FactType::RegisterState) => {
                    if let Some(register_fact) = self.extract_register_fact(fact) {
                        self.apply_register_fact(register_fact)?;
                    }
                }
                _ => {},
            }
        }
        
        Ok(())
    }
    
    /// Extract the fact type from a fact entry
    fn extract_fact_type(&self, fact: &FactEntry) -> Option<FactType> {
        // In a real implementation, this would deserialize the fact data
        // and extract the fact type
        Some(FactType::RegisterState) // For testing purposes
    }
    
    /// Extract a RegisterFact from a FactEntry
    fn extract_register_fact(&self, fact: &FactEntry) -> Option<RegisterFact> {
        // In a real implementation, this would deserialize the fact data
        // For now, we create a dummy RegisterFact
        Some(RegisterFact {
            register_id: ContentId::new("dummy_register"),
            state: Vec::new(),
            version: 1,
            previous_hash: None,
        })
    }
    
    /// Apply a register fact to update register state
    fn apply_register_fact(&mut self, register_fact: RegisterFact) -> Result<()> {
        // Update the register state with the provided fact data
        self.register_states.insert(register_fact.register_id, register_fact.state);
        Ok(())
    }
    
    /// Get the current state of a register
    pub fn get_register_state(&self, register_id: &ContentId) -> Option<&Vec<u8>> {
        self.register_states.get(register_id)
    }
    
    /// Check if a fact has been observed in the replay
    pub fn has_fact(&self, fact_id: &FactId) -> bool {
        self.fact_cache.contains_key(fact_id)
    }
    
    /// Get a fact from the cache
    pub fn get_fact(&self, fact_id: &FactId) -> Option<&FactEntry> {
        self.fact_cache.get(fact_id)
    }
    
    /// Create a fact snapshot from the current state
    pub fn create_snapshot(&self, observer: &str) -> FactSnapshot {
        let mut snapshot = FactSnapshot::new(observer);
        
        // Add all facts in the cache
        for (fact_id, fact_entry) in &self.fact_cache {
            snapshot.add_fact(fact_id.clone(), fact_entry.domain_id.clone());
        }
        
        // Add all register states
        for (register_id, state) in &self.register_states {
            let fact_id = FactId(format!("register:{}", register_id));
            
            // Create a fake domain ID for simplicity
            let domain_id = DomainId::new("fact-replay");
            
            // Create a data hash from the state
            #[cfg(feature = "md5")]
            let data_hash = Md5ChecksumFunction::compute(state).to_hex();
            
            #[cfg(not(feature = "md5"))]
            let data_hash = {
                let hash_factory = HashFactory::default();
                let mut hasher = hash_factory.default_content_hasher().unwrap();
                hasher.update(state);
                hasher.finalize().to_hex()
            };
            
            snapshot.add_register_observation(
                register_id.to_string().as_str(),
                FactId(format!("register:{}", register_id)),
                self.domain_id.clone(),
                0, // Value as a placeholder
            );
        }
        
        snapshot
    }
    
    /// Build a unique ID for this fact
    fn build_fact_id(&self, fact: &FactEntry) -> FactId {
        // Format fact ID as "type:resource_id" if resources are available
        if let Some(resources) = &fact.resources {
            if let Some(first_resource) = resources.first() {
                return FactId(format!("{}:{}", fact.fact_type, first_resource));
            }
        }
        
        // Fallback to just using the fact type with a timestamp
        FactId(format!("{}:{}", fact.fact_type, fact.timestamp))
    }
    
    /// Calculate a hash for a register using the given hash factory
    fn hash_register(&self, hash_factory: &HashFactory, register_id: &ContentId, data: &[u8]) -> String {
        // Use the hash factory to create a content hasher with the default algorithm
        let hasher = hash_factory.default_content_hasher().unwrap();
        
        // Update the hasher with register ID and data
        let mut hasher_box = hasher;
        hasher_box.update(register_id.to_string().as_bytes());
        hasher_box.update(data);
        
        // Finalize and return the hash
        hasher_box.finalize().to_string()
    }
} 
