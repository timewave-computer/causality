// Resource storage types
//
// Types for resource storage in the Causality system.

use crate::resource::{ResourceId, ResourceState};
use causality_error::{Error, Result};
use std::fmt::Debug;

/// Error types for resource storage operations
#[derive(Debug, thiserror::Error)]
pub enum ResourceStorageError {
    #[error("Resource not found: {0}")]
    NotFound(ResourceId),
    
    #[error("Resource already exists: {0}")]
    AlreadyExists(ResourceId),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Interface for storing resource states
pub trait ResourceStateStorage: Debug + Send + Sync {
    /// Save a resource state
    fn save_state(&self, id: &ResourceId, state: &ResourceState) -> Result<()>;
    
    /// Load a resource state
    fn load_state(&self, id: &ResourceId) -> Result<Option<ResourceState>>;
    
    /// Check if a resource state exists
    fn has_state(&self, id: &ResourceId) -> Result<bool>;
    
    /// Delete a resource state
    fn delete_state(&self, id: &ResourceId) -> Result<()>;
    
    /// List all resource IDs
    fn list_resources(&self) -> Result<Vec<ResourceId>>;
}

/// A resource state storage that uses memory
#[derive(Debug, Default)]
pub struct InMemoryResourceStateStorage {
    states: std::collections::HashMap<ResourceId, ResourceState>,
}

impl InMemoryResourceStateStorage {
    /// Create a new in-memory resource state storage
    pub fn new() -> Self {
        Self {
            states: std::collections::HashMap::new(),
        }
    }
}

impl ResourceStateStorage for InMemoryResourceStateStorage {
    fn save_state(&self, id: &ResourceId, state: &ResourceState) -> Result<()> {
        let mut states = self.states.clone();
        states.insert(id.clone(), state.clone());
        Ok(())
    }
    
    fn load_state(&self, id: &ResourceId) -> Result<Option<ResourceState>> {
        Ok(self.states.get(id).cloned())
    }
    
    fn has_state(&self, id: &ResourceId) -> Result<bool> {
        Ok(self.states.contains_key(id))
    }
    
    fn delete_state(&self, id: &ResourceId) -> Result<()> {
        let mut states = self.states.clone();
        states.remove(id);
        Ok(())
    }
    
    fn list_resources(&self) -> Result<Vec<ResourceId>> {
        Ok(self.states.keys().cloned().collect())
    }
} 