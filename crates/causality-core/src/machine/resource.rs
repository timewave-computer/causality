//! Resource heap management
//!
//! This module defines the resource heap for storing linear resources
//! that must be consumed exactly once.

use super::value::MachineValue;
use crate::lambda::TypeInner;
use crate::system::error::MachineError;
use crate::{Blake3Hasher, Hasher};
use std::collections::BTreeMap;

// Re-export ResourceId for easier access
pub use crate::system::content_addressing::ResourceId;

/// Linear resource stored in the resource heap
#[derive(Debug, Clone)]
pub struct Resource {
    /// The resource value
    pub value: MachineValue,
    
    /// Resource type
    pub resource_type: TypeInner,
    
    /// Whether this resource has been consumed
    pub consumed: bool,
}

/// Resource heap operations
#[derive(Debug, Clone)]
pub struct ResourceHeap {
    /// Map from resource IDs to resources
    resources: BTreeMap<ResourceId, Resource>,
}

impl ResourceHeap {
    /// Create a new empty resource heap
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }
    
    /// Allocate a resource on the heap
    pub fn alloc_resource(&mut self, value: MachineValue, resource_type: TypeInner) -> ResourceId {
        // Generate content-addressed ID
        let mut data = Vec::new();
        data.extend_from_slice(format!("{:?}", value).as_bytes());
        data.extend_from_slice(format!("{:?}", resource_type).as_bytes());
        
        let hash = Blake3Hasher::hash(&data);
        let id = ResourceId::from_bytes(hash.into());
        
        self.resources.insert(id, Resource {
            value,
            resource_type,
            consumed: false,
        });
        
        id
    }
    
    /// Consume a resource from the heap
    pub fn consume_resource(&mut self, id: ResourceId) -> Result<MachineValue, MachineError> {
        let resource = self.resources.get_mut(&id)
            .ok_or(MachineError::InvalidResource(id))?;
        
        if resource.consumed {
            return Err(MachineError::ResourceAlreadyConsumed(id));
        }
        
        resource.consumed = true;
        Ok(resource.value.clone())
    }
    
    /// Check if a resource exists and hasn't been consumed
    pub fn is_available(&self, id: ResourceId) -> bool {
        self.resources.get(&id)
            .map(|r| !r.consumed)
            .unwrap_or(false)
    }
    
    /// Get a reference to a resource without consuming it
    pub fn peek_resource(&self, id: ResourceId) -> Result<&Resource, MachineError> {
        self.resources.get(&id)
            .ok_or(MachineError::InvalidResource(id))
    }
}

impl Default for ResourceHeap {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for machine state to manage resources
pub trait ResourceManager {
    /// Allocate a resource on the heap
    fn alloc_resource(&mut self, value: MachineValue, resource_type: TypeInner) -> ResourceId;
    
    /// Consume a resource from the heap
    fn consume_resource(&mut self, id: ResourceId) -> Result<MachineValue, MachineError>;
} 