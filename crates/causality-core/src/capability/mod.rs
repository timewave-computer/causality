//! Capability module
//!
//! Provides capability-based security primitives for the Causality system.

// Direct implementation of domain, resource, and effect capabilities
pub mod domain;
pub mod resource;
pub mod effect;
pub mod examples;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use causality_types::{ContentHash, ContentId};
use blake3;

use crate::serialization::Serializer;

pub use examples::{basic_capability_example, content_addressed_example, capability_delegation_example, complex_resource_example};

/// Errors that can occur during capability operations
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Invalid capability grants: {0}")]
    InvalidGrants(String),
    
    #[error("Capability not transferable: {0}")]
    NotTransferable(String),
    
    #[error("Content addressing error: {0}")]
    ContentAddressingError(#[from] ContentAddressingError),
    
    #[error("{0}")]
    Other(String),
}

/// Errors that can occur during content addressing operations
#[derive(Error, Debug)]
pub enum ContentAddressingError {
    /// An error occurred during serialization
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// An error occurred during hashing
    #[error("Hashing error: {0}")]
    HashingError(String),
    
    /// An error occurred during storage
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// The content hash did not match
    #[error("Content hash mismatch: expected {expected}, found {actual}")]
    HashMismatch {
        expected: ContentHash,
        actual: ContentHash,
    },
    
    /// Other error
    #[error("Content addressing error: {0}")]
    Other(String),
}

/// A reference to a content-addressed object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentRef<T: ?Sized> {
    /// The content hash of the referenced object
    pub hash: ContentHash,
    
    /// Phantom data to indicate what type this references
    pub _phantom: PhantomData<T>,
}

impl<T: ?Sized> ContentRef<T> {
    /// Create a new content reference
    pub fn new(hash: ContentHash) -> Self {
        Self {
            hash,
            _phantom: PhantomData,
        }
    }
}

/// Represents the grants associated with a capability
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityGrants {
    /// Whether the capability allows reading
    pub can_read: bool,
    
    /// Whether the capability allows writing
    pub can_write: bool,
    
    /// Whether the capability can be delegated
    pub can_delegate: bool,
}

impl CapabilityGrants {
    /// Create a new capability grants object
    pub fn new(can_read: bool, can_write: bool, can_delegate: bool) -> Self {
        Self {
            can_read,
            can_write,
            can_delegate,
        }
    }
    
    /// Create read-only grants
    pub fn read_only() -> Self {
        Self::new(true, false, false)
    }
    
    /// Create write-only grants
    pub fn write_only() -> Self {
        Self::new(false, true, false)
    }
    
    /// Create full grants (read, write, delegate)
    pub fn full() -> Self {
        Self::new(true, true, true)
    }
    
    /// Check if this grant allows reading
    pub fn allows_read(&self) -> bool {
        self.can_read
    }
    
    /// Check if this grant allows writing
    pub fn allows_write(&self) -> bool {
        self.can_write
    }
    
    /// Check if this grant allows delegation
    pub fn allows_delegation(&self) -> bool {
        self.can_delegate
    }
}

/// A capability for accessing a resource
#[derive(Debug)]
pub struct Capability<T: ?Sized> {
    /// The resource ID this capability grants access to
    pub id: ResourceId,
    
    /// The grants associated with this capability
    pub grants: CapabilityGrants,
    
    /// The identity that originally created this capability
    pub origin: Option<IdentityId>,
    
    /// Phantom data to indicate what type this capability is for
    pub _phantom: PhantomData<T>,
}

/// A reference to a capability, type-erased
pub type CapabilityRef = Capability<dyn std::any::Any + Send + Sync>;

impl<T: ?Sized> Clone for Capability<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Capability<T> {
    /// Create a new capability
    pub fn new(id: ResourceId, grants: CapabilityGrants, origin: Option<IdentityId>) -> Self {
        Self {
            id,
            grants,
            origin,
            _phantom: PhantomData,
        }
    }
}

/// A trait for objects that are content-addressed
pub trait ContentAddressed: Sized {
    /// Get the content hash of this object
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError>;
    
    /// Verify that this object matches its expected hash
    fn verify(&self, expected_hash: &ContentHash) -> Result<bool, ContentAddressingError> {
        let actual_hash = self.content_hash()?;
        Ok(&actual_hash == expected_hash)
    }
    
    /// Convert to a serialized form
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError>;
    
    /// Create from serialized form
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError>;
    
    /// Create a content reference to this object
    fn to_content_ref(&self) -> Result<ContentRef<Self>, ContentAddressingError> {
        let hash = self.content_hash()?;
        Ok(ContentRef::new(hash))
    }
}

/// A guard for accessing a resource, ensuring it's properly managed
#[derive(Debug)]
pub struct ResourceGuard<T: ?Sized> {
    /// The resource being guarded
    resource: Box<T>,
    
    /// Capability that granted access to this resource
    capability: Capability<T>,
}

impl<T: ?Sized> ResourceGuard<T> {
    /// Create a new resource guard
    pub fn new(resource: T, capability: Capability<T>) -> Self 
    where
        T: Sized,
    {
        Self {
            resource: Box::new(resource),
            capability,
        }
    }
    
    /// Get a reference to the underlying resource
    pub fn resource(&self) -> &T {
        &self.resource
    }
    
    /// Get the capability reference
    pub fn capability(&self) -> &Capability<T> {
        &self.capability
    }
    
    /// Read the resource (requires read permission)
    pub fn read(&self) -> Result<&T, CapabilityError> {
        if !self.capability.grants.allows_read() {
            return Err(CapabilityError::PermissionDenied("Read permission required".to_string()));
        }
        Ok(&self.resource)
    }
    
    /// Write to the resource (requires write permission)
    pub fn write(&mut self) -> Result<&mut T, CapabilityError> {
        if !self.capability.grants.allows_write() {
            return Err(CapabilityError::PermissionDenied("Write permission required".to_string()));
        }
        Ok(&mut self.resource)
    }
    
    /// Create a restricted capability from this guard
    pub fn create_restricted_capability(&self, grants: CapabilityGrants) -> Result<Capability<T>, CapabilityError> {
        if !self.capability.grants.allows_delegation() {
            return Err(CapabilityError::PermissionDenied("Delegation permission required".to_string()));
        }
        
        // Ensure we're not escalating privileges
        let new_grants = CapabilityGrants {
            can_read: grants.can_read && self.capability.grants.can_read,
            can_write: grants.can_write && self.capability.grants.can_write,
            can_delegate: grants.can_delegate && self.capability.grants.can_delegate,
        };
        
        Ok(Capability {
            id: self.capability.id.clone(),
            grants: new_grants,
            origin: self.capability.origin.clone(),
            _phantom: PhantomData,
        })
    }
    
    /// Convert to a content-addressed capability
    pub fn to_content_addressed<U>(&self) -> Result<Capability<U>, CapabilityError> 
    where 
        T: Serialize,
    {
        let content_hash = utils::hash_object(&self.resource)
            .map_err(|e| CapabilityError::ContentAddressingError(e))?;
        
        Ok(Capability {
            id: ResourceId::with_name(content_hash.clone(), format!("content_{}", content_hash)),
            grants: self.capability.grants.clone(),
            origin: self.capability.origin.clone(),
            _phantom: PhantomData,
        })
    }
}

impl<T: ?Sized + Clone> Clone for ResourceGuard<T> 
where
    Box<T>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            capability: self.capability.clone(),
        }
    }
}

/// A registry for resources and capabilities
#[derive(Debug)]
pub struct ResourceRegistry {
    /// Resources stored by ID
    resources: HashMap<ResourceId, Box<dyn Any + Send + Sync>>,
    
    /// Capabilities by identity
    capabilities: HashMap<IdentityId, Vec<ResourceId>>,
}

impl ResourceRegistry {
    /// Create a new resource registry
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }
    
    /// Register a resource and create a capability for it
    pub fn register<T: 'static + Send + Sync + Serialize>(&mut self, resource: T, owner: IdentityId) -> Result<Capability<T>, CapabilityError> {
        // Create a unique ID for this resource
        let id = ResourceId::new(utils::hash_object(&resource).map_err(CapabilityError::ContentAddressingError)?);
        
        // Store the resource
        self.resources.insert(id.clone(), Box::new(resource));
        
        // Create a capability with full access
        let capability = Capability {
            id: id.clone(),
            grants: CapabilityGrants::full(),
            origin: Some(owner.clone()),
            _phantom: PhantomData,
        };
        
        // Register the capability for this identity
        self.capabilities.entry(owner).or_insert_with(Vec::new).push(id);
        
        Ok(capability)
    }
    
    /// Access a resource using a capability
    pub fn access<T: 'static + Send + Sync + Clone>(&self, capability: &Capability<T>) -> Result<ResourceGuard<T>, CapabilityError> {
        // Get the resource
        let resource = self.resources.get(&capability.id)
            .ok_or_else(|| CapabilityError::ResourceNotFound(format!("Resource not found: {}", capability.id)))?;
        
        // Downcast to the correct type
        let resource = resource.downcast_ref::<T>()
            .ok_or_else(|| CapabilityError::Other("Type mismatch".to_string()))?;
        
        // Create a resource guard with cloned resource and capability
        let guard = ResourceGuard {
            resource: Box::new(resource.clone()),
            capability: capability.clone(),
        };
        
        Ok(guard)
    }
    
    /// Check if an identity has a capability for a resource
    pub fn has_capability(&self, identity: &IdentityId, resource_id: &ResourceId) -> Result<bool, CapabilityError> {
        if let Some(capabilities) = self.capabilities.get(identity) {
            Ok(capabilities.contains(resource_id))
        } else {
            Ok(false)
        }
    }
    
    /// Transfer a capability from one identity to another
    pub fn transfer_capability<T: 'static>(&mut self, capability: &Capability<T>, from: &IdentityId, to: &IdentityId) -> Result<(), CapabilityError> {
        // Check if the sender has the capability
        if !self.has_capability(from, &capability.id)? {
            return Err(CapabilityError::PermissionDenied("Sender does not have this capability".to_string()));
        }
        
        // Check if the capability can be delegated
        if !capability.grants.allows_delegation() {
            return Err(CapabilityError::NotTransferable("This capability cannot be delegated".to_string()));
        }
        
        // Add the capability to the recipient
        self.capabilities.entry(to.clone()).or_insert_with(Vec::new).push(capability.id.clone());
        
        Ok(())
    }
    
    /// Access a resource by content reference
    pub fn access_by_content<T: 'static + Send + Sync + Clone>(&self, content_ref: &ContentRef<T>) -> Result<ResourceGuard<T>, CapabilityError> {
        // Find a resource with the matching content hash
        for (id, resource) in &self.resources {
            if id.hash == content_ref.hash {
                // Downcast to the correct type
                if let Some(typed_resource) = resource.downcast_ref::<T>() {
                    // Create a temporary capability for this access
                    let capability = Capability {
                        id: id.clone(),
                        grants: CapabilityGrants::read_only(),
                        origin: None,
                        _phantom: PhantomData,
                    };
                    
                    // Return a resource guard
                    let guard = ResourceGuard {
                        resource: Box::new(typed_resource.clone()),
                        capability,
                    };
                    
                    return Ok(guard);
                }
            }
        }
        
        Err(CapabilityError::ResourceNotFound(format!("Resource with content hash {} not found", content_ref.hash)))
    }
}

/// An identity identifier
pub type IdentityId = String;

/// A resource identifier, content-addressed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// The content hash of this resource
    pub hash: ContentHash,
    
    /// Optional name for this resource
    pub name: Option<String>,
}

impl ResourceId {
    /// Create a new resource ID
    pub fn new(hash: ContentHash) -> Self {
        Self {
            hash,
            name: None,
        }
    }
    
    /// Create a new resource ID with a name
    pub fn with_name(hash: ContentHash, name: impl Into<String>) -> Self {
        Self {
            hash,
            name: Some(name.into()),
        }
    }
    
    /// Get the content hash of this resource ID
    pub fn content_hash(&self) -> &ContentHash {
        &self.hash
    }
}
    
impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}:{}", name, self.hash)
        } else {
            write!(f, "{}", self.hash)
        }
    }
}

/// Helper functions module for creating capabilities
pub mod helpers {
    use super::*;
    
    /// Create a content-addressed resource registry
    pub fn create_content_addressed_registry() -> ResourceRegistry {
        ResourceRegistry::new()
    }
}

// Re-export helper functions for creating ContentHash
pub mod utils {
    use super::*;
    
    /// Create a ContentHash from serializing an object
    pub fn hash_object<T: Serialize>(object: &T) -> Result<ContentHash, ContentAddressingError> {
        let bytes = serde_json::to_vec(object).map_err(|e| {
            ContentAddressingError::SerializationError(e.to_string())
        })?;
        
        let hash = blake3::hash(&bytes);
        Ok(ContentHash::new("blake3", hash.as_bytes().to_vec()))
    }
    
    /// Generate a ContentHash from bytes
    pub fn hash_bytes(bytes: &[u8]) -> ContentHash {
        // Hash the bytes using blake3
        let hash_result = blake3::hash(bytes);
        let hash_bytes = hash_result.as_bytes().to_vec();
        
        // Create a properly formatted ContentHash
        ContentHash::new("blake3", hash_bytes)
    }
    
    /// Generate a ContentHash from a string
    pub fn hash_string(s: &str) -> ContentHash {
        hash_bytes(s.as_bytes())
    }
}
