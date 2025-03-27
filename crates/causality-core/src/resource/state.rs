// Resource state management
//
// This file defines the state management interfaces and types for resources.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use super::types::{ResourceId, ResourceType, ResourceTag};
use super::interface::{ResourceState, ResourceOperation, ResourceResult};

/// Resource state data
///
/// Contains all state information for a resource, including its
/// attributes, metadata, and current lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceStateData {
    /// Resource identifier
    pub id: ResourceId,
    
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Current lifecycle state
    pub state: ResourceState,
    
    /// Resource attributes
    #[serde(default)]
    pub attributes: HashMap<String, serde_json::Value>,
    
    /// Resource tags
    #[serde(default)]
    pub tags: Vec<ResourceTag>,
    
    /// Resource metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Version of the resource state
    pub version: u64,
    
    /// Timestamp of the last update
    pub updated_at: u64,
    
    /// Timestamp of creation
    pub created_at: u64,
}

impl ResourceStateData {
    /// Create a new resource state
    pub fn new(
        id: ResourceId,
        resource_type: ResourceType,
        created_at: u64,
    ) -> Self {
        Self {
            id,
            resource_type,
            state: ResourceState::Created,
            attributes: HashMap::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            version: 1,
            updated_at: created_at,
            created_at,
        }
    }
    
    /// Get a resource attribute
    pub fn get_attribute(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }
    
    /// Set a resource attribute
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.attributes.insert(key.into(), value.into());
    }
    
    /// Add a tag to the resource
    pub fn add_tag(&mut self, tag: ResourceTag) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Remove a tag from the resource
    pub fn remove_tag(&mut self, tag: &ResourceTag) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Check if the resource has a tag
    pub fn has_tag(&self, tag: &ResourceTag) -> bool {
        self.tags.contains(tag)
    }
    
    /// Get all tags with a specific key
    pub fn get_tags_by_key(&self, key: &str) -> Vec<&ResourceTag> {
        self.tags.iter().filter(|t| t.key == key).collect()
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    /// Update the resource state
    pub fn update_state(&mut self, new_state: ResourceState, timestamp: u64) -> ResourceResult<()> {
        let old_state = self.state.clone();
        
        // Validate state transition
        if !is_valid_state_transition(&old_state, &new_state) {
            return ResourceResult::error(
                ResourceOperation::Custom("update_state".into()),
                old_state,
                format!("Invalid state transition from {:?} to {:?}", old_state, new_state),
            );
        }
        
        self.state = new_state.clone();
        self.version += 1;
        self.updated_at = timestamp;
        
        ResourceResult::ok(
            ResourceOperation::Custom("update_state".into()),
            new_state,
            (),
        )
    }
    
    /// Get the current version of the resource
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Increment the version of the resource
    pub fn increment_version(&mut self, timestamp: u64) {
        self.version += 1;
        self.updated_at = timestamp;
    }
}

/// Resource state store
///
/// An interface for storing and retrieving resource state.
pub trait ResourceStateStore: Send + Sync {
    /// Get the state of a resource
    fn get(&self, id: &ResourceId) -> Option<ResourceStateData>;
    
    /// Save the state of a resource
    fn save(&self, state: ResourceStateData) -> Result<(), String>;
    
    /// Delete a resource from the store
    fn delete(&self, id: &ResourceId) -> Result<(), String>;
    
    /// Check if a resource exists
    fn exists(&self, id: &ResourceId) -> bool {
        self.get(id).is_some()
    }
    
    /// Find resources matching a filter
    fn find(
        &self,
        resource_type: Option<&ResourceType>,
        state: Option<&ResourceState>,
        tags: Option<&[ResourceTag]>,
    ) -> Vec<ResourceStateData>;
}

/// In-memory resource state store implementation
pub struct InMemoryStateStore {
    states: RwLock<HashMap<ResourceId, ResourceStateData>>,
}

impl InMemoryStateStore {
    /// Create a new in-memory state store
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceStateStore for InMemoryStateStore {
    fn get(&self, id: &ResourceId) -> Option<ResourceStateData> {
        let states = self.states.read().unwrap();
        states.get(id).cloned()
    }
    
    fn save(&self, state: ResourceStateData) -> Result<(), String> {
        let mut states = self.states.write().unwrap();
        states.insert(state.id.clone(), state);
        Ok(())
    }
    
    fn delete(&self, id: &ResourceId) -> Result<(), String> {
        let mut states = self.states.write().unwrap();
        states.remove(id);
        Ok(())
    }
    
    fn find(
        &self,
        resource_type: Option<&ResourceType>,
        state: Option<&ResourceState>,
        tags: Option<&[ResourceTag]>,
    ) -> Vec<ResourceStateData> {
        let states = self.states.read().unwrap();
        
        states
            .values()
            .filter(|s| {
                // Filter by type if specified
                if let Some(rt) = resource_type {
                    if !s.resource_type.is_compatible_with(rt) {
                        return false;
                    }
                }
                
                // Filter by state if specified
                if let Some(st) = state {
                    if &s.state != st {
                        return false;
                    }
                }
                
                // Filter by tags if specified
                if let Some(tags) = tags {
                    if !tags.iter().all(|tag| s.has_tag(tag)) {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }
}

/// State store provider
///
/// A factory for creating state stores.
pub trait StateStoreProvider: Send + Sync {
    /// Create a new state store
    fn create_store(&self) -> Arc<dyn ResourceStateStore>;
}

/// In-memory state store provider
pub struct InMemoryStateStoreProvider;

impl StateStoreProvider for InMemoryStateStoreProvider {
    fn create_store(&self) -> Arc<dyn ResourceStateStore> {
        Arc::new(InMemoryStateStore::new())
    }
}

/// Default state store provider
///
/// Returns an in-memory state store by default.
pub struct DefaultStateStoreProvider;

impl StateStoreProvider for DefaultStateStoreProvider {
    fn create_store(&self) -> Arc<dyn ResourceStateStore> {
        Arc::new(InMemoryStateStore::new())
    }
}

/// Check if a state transition is valid
fn is_valid_state_transition(from: &ResourceState, to: &ResourceState) -> bool {
    use ResourceState::*;
    
    match (from, to) {
        // Any state can transition to itself (no change)
        (a, b) if a == b => true,
        
        // Created state can transition to Active or Consumed
        (Created, Active) => true,
        (Created, Consumed) => true,
        
        // Active state can transition to any other state except Created
        (Active, Frozen) => true,
        (Active, Locked) => true,
        (Active, Archived) => true,
        (Active, Consumed) => true,
        
        // Frozen state can transition to Active
        (Frozen, Active) => true,
        
        // Locked state can transition to Active
        (Locked, Active) => true,
        
        // Archived state can transition to Active
        (Archived, Active) => true,
        
        // Consumed state cannot transition to any other state
        (Consumed, _) => false,
        
        // Any other transition is invalid
        _ => false,
    }
} 