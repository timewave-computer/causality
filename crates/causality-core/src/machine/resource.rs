//! Resource heap management
//!
//! This module defines the resource heap for storing linear resources
//! that must be consumed exactly once.

use super::value::MachineValue;
use crate::lambda::TypeInner;
use crate::system::error::MachineError;
use crate::{Hash, Blake3Hasher, Hasher};

/// Resource identifier (content-addressed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub Hash);

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

impl ResourceId {
    /// Create a new resource ID from a hash
    pub fn new(hash: Hash) -> Self {
        ResourceId(hash)
    }
    
    /// Get the inner hash
    pub fn hash(&self) -> &Hash {
        &self.0
    }
}

/// Resource heap operations
#[derive(Debug, Clone)]
pub struct ResourceHeap {
    resources: std::collections::HashMap<ResourceId, Resource>,
}

impl ResourceHeap {
    /// Create a new empty resource heap
    pub fn new() -> Self {
        Self {
            resources: std::collections::HashMap::new(),
        }
    }
    
    /// Allocate a resource on the heap
    pub fn alloc_resource(&mut self, value: MachineValue, resource_type: TypeInner) -> ResourceId {
        // Generate content-addressed ID
        let mut data = Vec::new();
        data.extend_from_slice(format!("{:?}", value).as_bytes());
        data.extend_from_slice(format!("{:?}", resource_type).as_bytes());
        
        let hash = Blake3Hasher::hash(&data);
        let id = ResourceId::new(hash);
        
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