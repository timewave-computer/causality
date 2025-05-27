// Replay state management
// Original file: src/log/replay/state.rs

// Replay state implementation for Causality Unified Log System
//
// This module provides the state structures used during log replay.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use causality_types::*;
use causality_types::ContentId;
use crate::log::EffectEntry;
use crate::log::FactEntry;

/// The state reconstructed during replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayState {
    /// The resources in the system
    pub resources: HashMap<ContentId, ResourceState>,
    /// The domains in the system
    pub domains: HashMap<DomainId, DomainState>,
    /// The facts that have been observed
    pub facts: Vec<FactEntry>,
    /// The effects that have been applied
    pub effects: Vec<EffectEntry>,
}

impl ReplayState {
    /// Create a new empty replay state
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            domains: HashMap::new(),
            facts: Vec::new(),
            effects: Vec::new(),
        }
    }
    
    /// Get the state of a specific resource
    pub fn get_resource(&self, id: &ContentId) -> Option<&ResourceState> {
        self.resources.get(id)
    }
    
    /// Get the state of a specific domain
    pub fn get_domain(&self, id: &DomainId) -> Option<&DomainState> {
        self.domains.get(id)
    }
    
    /// Get facts of a specific type
    pub fn get_facts_by_type(&self, fact_type: &str) -> Vec<&FactEntry> {
        self.facts.iter()
            // TODO: Revisit filtering - FactEntry no longer has fact_type.
            // .filter(|fact| fact.fact_type == fact_type)
            .collect()
    }
    
    /// Get facts for a specific resource
    pub fn get_facts_for_resource(&self, resource_id: &ContentId) -> Vec<&FactEntry> {
        self.facts.iter()
            // TODO: Revisit filtering - FactEntry no longer has resources.
            /*
            .filter(|fact| match &fact.resources {
                Some(resources) => resources.contains(resource_id),
                None => false,
            })
            */
            .collect()
    }
    
    /// Get effects of a specific type
    pub fn get_effects_by_type(&self, effect_type: &causality_core::EffectType) -> Vec<&EffectEntry> {
        self.effects
            .iter()
            // TODO: Revisit filtering - EffectEntry no longer has effect_type.
            // .filter(|effect| effect.effect_type.to_string() == effect_type.to_string())
            .collect()
    }
    
    /// Get effects for a specific resource
    pub fn get_effects_for_resource(&self, resource_id: &ContentId) -> Vec<&EffectEntry> {
        self.effects.iter()
             // TODO: Revisit filtering - EffectEntry no longer has resources.
            /*
            .filter(|effect| match &effect.resources {
                Some(resources) => resources.contains(resource_id),
                None => false,
            })
            */
            .collect()
    }
    
    /// Check if a specific resource is locked
    pub fn is_resource_locked(&self, resource_id: &ContentId) -> bool {
        self.resources.get(resource_id)
            .map_or(false, |state| state.locked)
    }

    pub fn facts_containing_resource(&self, resource_id: &ContentId) -> Vec<&FactEntry> {
        self.facts
            .iter()
            // TODO: Revisit filtering - FactEntry no longer has resources.
            /*
            .filter(|fact| match &fact.resources {
                Some(resources) => resources.contains(resource_id),
                None => false, // No resources in this fact
            })
            */
            .collect()
    }

    pub fn effects_containing_resource(&self, resource_id: &ContentId) -> Vec<&EffectEntry> {
        self.effects
            .iter()
             // TODO: Revisit filtering - EffectEntry no longer has resources.
            /*
            .filter(|effect| match &effect.resources {
                Some(resources) => resources.contains(resource_id),
                None => false, // No resources in this effect
            })
            */
            .collect()
    }

    /// Find facts matching a specific type and optional resource filter.
    pub fn find_facts(
        &self, 
        fact_type: &str, 
        resource_filter: Option<&HashSet<ContentId>>
    ) -> Vec<&FactEntry> {
        self.facts.iter()
            // TODO: Revisit filtering - FactEntry no longer has fact_type.
            // .filter(|fact| fact.fact_type == fact_type)
            .filter(|fact| { // Temporary: allow all fact types
                if let Some(filter) = resource_filter {
                    // TODO: Revisit resource filtering. FactEntry no longer has resources.
                    false // Need to implement resource check based on new structure or remove
                    /*
                    match &fact.resources {
                        Some(resources) => resources.iter().any(|r| filter.contains(r)),
                        None => false, // Fact has no resources, doesn't match filter
                    }
                    */
                } else {
                    true // No resource filter, include fact
                }
            })
            .collect()
    }

    /// Find effects matching a specific type and optional resource filter.
     pub fn find_effects(
        &self, 
        effect_type: &str, 
        resource_filter: Option<&HashSet<ContentId>>
    ) -> Vec<&EffectEntry> {
        self.effects.iter()
             // TODO: Revisit filtering - EffectEntry no longer has effect_type.
             // Maybe filter based on effect_id or details field?
            // .filter(|effect| effect.effect_type.to_string() == effect_type.to_string())
             .filter(|effect| { // Temporary: allow all effect types
                 if let Some(filter) = resource_filter {
                     // TODO: Revisit resource filtering. EffectEntry no longer has resources.
                    false // Need to implement resource check based on new structure or remove
                    /*
                     match &effect.resources {
                         Some(resources) => resources.iter().any(|r| filter.contains(r)),
                         None => false, // Effect has no resources, doesn't match filter
                     }
                     */
                 } else {
                     true // No resource filter, include effect
                 }
             })
            .collect()
    }
    
    /// Get all facts related to a set of resources.
    pub fn get_facts_by_resources(&self, resources: &HashSet<ContentId>) -> Vec<&FactEntry> {
        self.facts.iter()
            .filter(|fact| { 
                 // TODO: Revisit resource filtering. FactEntry no longer has resources.
                 false // Need to implement resource check based on new structure or remove
                /*
                match &fact.resources {
                    Some(fact_resources) => fact_resources.iter().any(|r| resources.contains(r)),
                    None => false,
                }
                */
            })
            .collect()
    }

    /// Get all effects related to a set of resources.
    pub fn get_effects_by_resources(&self, resources: &HashSet<ContentId>) -> Vec<&EffectEntry> {
        self.effects.iter()
            .filter(|effect| { 
                 // TODO: Revisit resource filtering. EffectEntry no longer has resources.
                 false // Need to implement resource check based on new structure or remove
                /*
                match &effect.resources {
                    Some(effect_resources) => effect_resources.iter().any(|r| resources.contains(r)),
                    None => false,
                }
                */
            })
            .collect()
    }
}

impl Default for ReplayState {
    fn default() -> Self {
        Self::new()
    }
}

/// The state of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    /// The resource ID
    pub id: ContentId,
    /// The current owner of the resource
    pub owner: Option<String>,
    /// Whether the resource is locked
    pub locked: bool,
    /// The waiters for this resource
    pub waiters: HashSet<String>,
    /// The last time this resource was modified
    pub last_modified: DateTime<Utc>,
    /// The log entry ID that last modified this resource
    pub last_entry_id: String,
}

impl ResourceState {
    /// Create a new resource state
    pub fn new(id: ContentId, entry_id: String) -> Self {
        Self {
            id,
            owner: None,
            locked: false,
            waiters: HashSet::new(),
            last_modified: Utc::now(),
            last_entry_id: entry_id,
        }
    }
    
    /// Set the resource owner
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }
    
    /// Lock the resource
    pub fn lock(mut self) -> Self {
        self.locked = true;
        self
    }
    
    /// Unlock the resource
    pub fn unlock(mut self) -> Self {
        self.locked = false;
        self
    }
    
    /// Add a waiter for this resource
    pub fn add_waiter(&mut self, waiter: impl Into<String>) {
        self.waiters.insert(waiter.into());
    }
    
    /// Remove a waiter from this resource
    pub fn remove_waiter(&mut self, waiter: &str) {
        self.waiters.remove(waiter);
    }
    
    /// Update the modification information
    pub fn update_modification(&mut self, entry_id: String) {
        self.last_modified = Utc::now();
        self.last_entry_id = entry_id;
    }
}

/// The state of a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainState {
    /// The domain ID
    pub id: DomainId,
    /// The current block height
    pub height: BlockHeight,
    /// The current block hash
    pub hash: Option<BlockHash>,
    /// The current timestamp
    pub timestamp: Timestamp,
    /// The log entry ID that last updated this domain
    pub last_entry_id: String,
}

impl DomainState {
    /// Create a new domain state
    pub fn new(id: DomainId, entry_id: String) -> Self {
        Self {
            id,
            height: BlockHeight::new(0),
            hash: None,
            timestamp: Timestamp::new(0),
            last_entry_id: entry_id,
        }
    }
    
    /// Update the domain state with a new block height and hash
    pub fn update(&mut self, height: BlockHeight, hash: Option<BlockHash>, timestamp: Timestamp, entry_id: String) {
        self.height = height;
        self.hash = hash;
        self.timestamp = timestamp;
        self.last_entry_id = entry_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr; // Import FromStr if needed for ContentId/DomainId
    
    #[test]
    fn test_resource_state() {
        // Use from_str or new depending on ContentId implementation
        let resource_id = ContentId::from_str("resource_1").expect("Invalid ContentId");
        let entry_id = "entry_1".to_string();
        
        let mut state = ResourceState::new(resource_id.clone(), entry_id.clone());
        assert_eq!(state.id, resource_id);
        assert_eq!(state.last_entry_id, entry_id);
        assert!(state.owner.is_none());
        assert!(!state.locked);
        assert!(state.waiters.is_empty());
        
        // Chain the calls to avoid partial move
        let mut state = ResourceState::new(resource_id.clone(), entry_id.clone())
            .with_owner("alice")
            .lock();
            
        assert_eq!(state.owner.unwrap(), "alice"); // owner is consumed here
        assert!(state.locked);
        
        // Reassign state after unlock
        state = state.unlock();
        assert!(!state.locked);
        
        state.add_waiter("bob");
        state.add_waiter("charlie");
        assert_eq!(state.waiters.len(), 2);
        assert!(state.waiters.contains("bob"));
        
        state.remove_waiter("bob");
        assert_eq!(state.waiters.len(), 1);
        assert!(!state.waiters.contains("bob"));
        
        state.update_modification("entry_2".to_string());
        assert_eq!(state.last_entry_id, "entry_2");
    }
    
    #[test]
    fn test_domain_state() {
        // Use DomainId::new
        let domain_id = DomainId::new("domain_1");
        let entry_id = "entry_1".to_string();
        
        let mut state = DomainState::new(domain_id.clone(), entry_id.clone());
        assert_eq!(state.id, domain_id);
        assert_eq!(state.last_entry_id, entry_id);
        assert_eq!(state.height, BlockHeight::new(0));
        assert!(state.hash.is_none());
        
        let new_height = BlockHeight::new(100);
        let new_hash = Some(BlockHash::new("abc123"));
        let new_timestamp = Timestamp::new(1000);
        state.update(new_height.clone(), new_hash.clone(), new_timestamp.clone(), "entry_2".to_string());
        
        assert_eq!(state.height, new_height);
        assert_eq!(state.hash, new_hash);
        assert_eq!(state.timestamp, new_timestamp);
        assert_eq!(state.last_entry_id, "entry_2");
    }
    
    #[test]
    fn test_replay_state() {
        let state = ReplayState::new();
        assert!(state.resources.is_empty());
        assert!(state.domains.is_empty());
        assert!(state.facts.is_empty());
        assert!(state.effects.is_empty());
        
        // Test with more data would require creating mock entries
    }
} 
