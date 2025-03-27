// Capability-based resource safety
//
// This module provides a capability-based approach to
// resource safety, allowing direct method calls while
// ensuring capability-based access control through content addressing.

mod examples;

// Direct implementation of domain, resource, and effect capabilities
pub mod domain;
pub mod resource;
pub mod effect;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::serialization::Serializer;

pub use examples::{basic_capability_example, content_addressed_example, capability_delegation_example, complex_resource_example};

/// A content hash that uniquely identifies a value based on its content
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Create a new content hash from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    /// Get the bytes of this hash
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    /// Generate a hash for specific content
    pub fn for_content(content: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(content);
        Self(hasher.finalize().into_bytes())
    }
    
    /// Generate a hash for a serializable object
    pub fn for_object<T: Serialize>(object: &T) -> Result<Self, ContentAddressingError> {
        let bytes = Serializer::to_bytes(object)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))?;
        Ok(Self::for_content(&bytes))
    }
    
    /// Generate a hash with a namespace and name (deterministic)
    pub fn for_name(namespace: &str, name: &str) -> Self {
        let namespace_hash = blake3::hash(namespace.as_bytes());
        let mut hasher = blake3::Hasher::new_keyed(&namespace_hash.into_bytes());
        hasher.update(name.as_bytes());
        Self(hasher.finalize().into_bytes())
    }
    
    /// Convert to a causality-types ContentHash
    pub fn to_types_content_hash(&self) -> causality_types::ContentHash {
        causality_types::ContentHash::new(
            "blake3", 
            self.0.to_vec()
        )
    }
    
    /// Convert to a causality-types HashOutput
    pub fn to_hash_output(&self) -> causality_types::HashOutput {
        causality_types::HashOutput::new(
            self.0,
            causality_types::HashAlgorithm::Blake3
        )
    }
    
    /// Create from a causality-types ContentHash
    pub fn from_types_content_hash(hash: &causality_types::ContentHash) -> Result<Self, ContentAddressingError> {
        if hash.algorithm.to_lowercase() != "blake3" {
            return Err(ContentAddressingError::Other(
                format!("Unsupported algorithm: {}", hash.algorithm)
            ));
        }
        
        if hash.bytes.len() != 32 {
            return Err(ContentAddressingError::Other(
                format!("Invalid hash length: expected 32, got {}", hash.bytes.len())
            ));
        }
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash.bytes);
        
        Ok(Self(bytes))
    }
    
    /// Create from a causality-types HashOutput
    pub fn from_hash_output(hash: &causality_types::HashOutput) -> Result<Self, ContentAddressingError> {
        if hash.algorithm() != causality_types::HashAlgorithm::Blake3 {
            return Err(ContentAddressingError::Other(
                format!("Unsupported algorithm: {:?}", hash.algorithm())
            ));
        }
        
        let bytes = hash.as_bytes();
        if bytes.len() != 32 {
            return Err(ContentAddressingError::Other(
                format!("Invalid hash length: expected 32, got {}", bytes.len())
            ));
        }
        
        let mut data = [0u8; 32];
        data.copy_from_slice(bytes);
        
        Ok(Self(data))
    }
    
    /// Convert to a causality-types ContentId
    pub fn to_content_id(&self) -> Result<causality_types::ContentId, ContentAddressingError> {
        let hash_output = self.to_hash_output();
        Ok(causality_types::ContentId::from(hash_output))
    }
    
    /// Get a normalized string representation (algorithm:hex)
    pub fn normalized_string(&self) -> String {
        let hex_str = self.0
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        
        format!("blake3:{}", hex_str)
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_str = self.0
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        
        write!(f, "ContentHash({})", hex_str)
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_str = self.0
            .iter()
            .take(6) // Show first 6 bytes (12 hex chars)
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        
        write!(f, "{}", hex_str)
    }
}

/// A reference to a content-addressed object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentRef<T> {
    /// The content hash of the referenced object
    pub hash: ContentHash,
    
    /// Phantom data to indicate what type this references
    pub _phantom: PhantomData<T>,
}

impl<T> ContentRef<T> {
    /// Create a new content reference
    pub fn new(hash: ContentHash) -> Self {
        Self {
            hash,
            _phantom: PhantomData,
        }
    }
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

/// A unique identifier for a resource, based on content addressing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Content hash that identifies this resource
    pub hash: ContentHash,
    
    /// Optional name for improved readability
    pub name: Option<String>,
}

impl ResourceId {
    /// Create a new random resource identifier
    pub fn new() -> Self {
        let random_bytes = rand::random::<[u8; 32]>();
        Self {
            hash: ContentHash::from_bytes(random_bytes),
            name: None,
        }
    }
    
    /// Create a resource identifier with a specific name
    /// This is deterministic based on the name
    pub fn new_with_name(name: &str) -> Self {
        Self {
            hash: ContentHash::for_name("resource", name),
            name: Some(name.to_string()),
        }
    }
    
    /// Get the underlying content hash
    pub fn content_hash(&self) -> &ContentHash {
        &self.hash
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        if let Some(name) = &self.name {
            format!("{}:{}", name, self.hash.normalized_string())
        } else {
            self.hash.normalized_string()
        }
    }
    
    /// Convert to a causality-types ContentId
    pub fn to_content_id(&self) -> Result<causality_types::ContentId, ContentAddressingError> {
        self.hash.to_content_id()
    }
    
    /// Create from a causality-types ContentId
    pub fn from_content_id(content_id: &causality_types::ContentId, name: Option<String>) -> Result<Self, ContentAddressingError> {
        let hash = ContentHash::from_hash_output(content_id.hash())?;
        Ok(Self {
            hash,
            name,
        })
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// An identity that can hold capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentityId {
    /// Content hash that identifies this identity
    pub hash: ContentHash,
    
    /// Optional name for improved readability
    pub name: Option<String>,
}

impl IdentityId {
    /// Create a new random identity identifier
    pub fn new() -> Self {
        let random_bytes = rand::random::<[u8; 32]>();
        Self {
            hash: ContentHash::from_bytes(random_bytes),
            name: None,
        }
    }
    
    /// Create an identity with a specific name
    pub fn new_with_name(name: &str) -> Self {
        Self {
            hash: ContentHash::for_name("identity", name),
            name: Some(name.to_string()),
        }
    }
    
    /// Get the underlying content hash
    pub fn content_hash(&self) -> &ContentHash {
        &self.hash
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        if let Some(name) = &self.name {
            format!("{}:{}", name, self.hash)
        } else {
            self.hash.to_string()
        }
    }
}

impl Default for IdentityId {
    fn default() -> Self {
        Self::new()
    }
}

/// The capabilities granted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrants {
    /// Grants read access
    pub read: bool,
    
    /// Grants write access
    pub write: bool,
    
    /// Grants the ability to delegate this capability
    pub delegate: bool,
}

impl CapabilityGrants {
    /// Create a capability with full access
    pub fn full() -> Self {
        Self {
            read: true,
            write: true,
            delegate: true,
        }
    }
    
    /// Create a read-only capability
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            delegate: false,
        }
    }
    
    /// Create a write-only capability
    pub fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            delegate: false,
        }
    }
    
    /// Create a capability with specific permissions
    pub fn new(read: bool, write: bool, delegate: bool) -> Self {
        Self {
            read,
            write,
            delegate,
        }
    }
    
    /// Check if this grant includes all permissions in another grant
    pub fn includes(&self, other: &Self) -> bool {
        (other.read && !self.read) || 
        (other.write && !self.write) || 
        (other.delegate && !self.delegate)
    }
    
    /// Can read with this capability
    pub fn can_read(&self) -> bool {
        self.read
    }
    
    /// Can write with this capability
    pub fn can_write(&self) -> bool {
        self.write
    }
    
    /// Can delegate this capability
    pub fn can_delegate(&self) -> bool {
        self.delegate
    }
}

/// A capability that grants access to a resource, content-addressed by default
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability<T: ?Sized> {
    /// The resource identifier
    pub id: ResourceId,
    
    /// The capabilities this capability grants
    pub grants: CapabilityGrants,
    
    /// Origin identity that created this capability
    pub origin: Option<IdentityId>,
    
    /// The content hash of this capability, computed automatically
    pub content_hash: Option<ContentHash>,
    
    /// Phantom data to associate this capability with a type
    pub _phantom: PhantomData<T>,
}

impl<T: ?Sized> Capability<T> {
    /// Create a new capability
    pub fn new(
        id: ResourceId,
        grants: CapabilityGrants,
        origin: Option<IdentityId>,
    ) -> Result<Self, ContentAddressingError> {
        let mut capability = Self {
            id,
            grants,
            origin,
            content_hash: None,
            _phantom: PhantomData,
        };
        
        // Calculate the content hash
        let hash = capability.calculate_hash()?;
        capability.content_hash = Some(hash);
        
        Ok(capability)
    }
    
    /// Calculate the content hash of this capability
    fn calculate_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        // Create a version without the hash for hashing
        let for_hashing = Capability {
            id: self.id.clone(),
            grants: self.grants,
            origin: self.origin.clone(),
            content_hash: None,
            _phantom: PhantomData,
        };
        
        ContentHash::for_object(&for_hashing)
    }
    
    /// Create a content reference to this capability
    pub fn to_content_ref(&self) -> Result<ContentRef<Self>, ContentAddressingError> {
        let hash = match self.content_hash {
            Some(hash) => hash,
            None => self.calculate_hash()?,
        };
        
        Ok(ContentRef::new(hash))
    }
    
    /// Check if this capability is valid
    pub fn verify(&self) -> Result<bool, ContentAddressingError> {
        if let Some(stored_hash) = self.content_hash {
            let calculated_hash = self.calculate_hash()?;
            
            if stored_hash != calculated_hash {
                return Err(ContentAddressingError::HashMismatch {
                    expected: stored_hash,
                    actual: calculated_hash,
                });
            }
        }
        
        Ok(true)
    }
}

impl<T: ?Sized> ContentAddressed for Capability<T> {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        match self.content_hash {
            Some(hash) => Ok(hash),
            None => self.calculate_hash(),
        }
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        let capability: Self = Serializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))?;
        
        // Verify hash integrity
        capability.verify()?;
        
        Ok(capability)
    }
}

/// A resource guard that provides safe access to a resource
pub struct ResourceGuard<T: ?Sized> {
    /// The resource being guarded
    resource: Arc<RwLock<Box<T>>>,
    
    /// The capability used to access this resource
    capability: Capability<T>,
    
    /// The registry that created this guard
    registry: Arc<ResourceRegistry>,
}

impl<T: ?Sized> ResourceGuard<T> {
    /// Get read access to the resource
    pub fn read(&self) -> Result<impl std::ops::Deref<Target = T> + '_, CapabilityError> {
        if !self.capability.grants.read {
            return Err(CapabilityError::AccessDenied("read".into()));
        }
        
        let guard = self.resource.read().map_err(|_| CapabilityError::LockError)?;
        Ok(ResourceReadGuard { guard })
    }
    
    /// Get write access to the resource
    pub fn write(&self) -> Result<impl std::ops::DerefMut<Target = T> + '_, CapabilityError> {
        if !self.capability.grants.write {
            return Err(CapabilityError::AccessDenied("write".into()));
        }
        
        let guard = self.resource.write().map_err(|_| CapabilityError::LockError)?;
        Ok(ResourceWriteGuard { guard })
    }
    
    /// Get the capability that was used to create this guard
    pub fn capability(&self) -> &Capability<T> {
        &self.capability
    }
    
    /// Create a new capability with reduced permissions
    pub fn create_restricted_capability(
        &self,
        grants: CapabilityGrants,
    ) -> Result<Capability<T>, CapabilityError> {
        if !self.capability.grants.delegate {
            return Err(CapabilityError::AccessDenied("delegate".into()));
        }
        
        // Check that we're not trying to grant more permissions than we have
        if !self.capability.grants.includes(&grants) {
            return Err(CapabilityError::CannotEscalatePrivilege);
        }
        
        Capability::new(
            self.capability.id.clone(), 
            grants, 
            self.capability.origin.clone()
        ).map_err(|e| CapabilityError::ContentAddressing(e))
    }
    
    /// Get the content hash of this guard's capability
    pub fn content_hash(&self) -> Result<ContentHash, CapabilityError> {
        self.capability.content_hash()
            .map_err(|e| CapabilityError::ContentAddressing(e))
    }
}

// Helper struct for providing read access
struct ResourceReadGuard<'a, T: ?Sized> {
    guard: std::sync::RwLockReadGuard<'a, Box<T>>,
}

impl<'a, T: ?Sized> std::ops::Deref for ResourceReadGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &**self.guard
    }
}

// Helper struct for providing write access
struct ResourceWriteGuard<'a, T: ?Sized> {
    guard: std::sync::RwLockWriteGuard<'a, Box<T>>,
}

impl<'a, T: ?Sized> std::ops::Deref for ResourceWriteGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &**self.guard
    }
}

impl<'a, T: ?Sized> std::ops::DerefMut for ResourceWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut **self.guard
    }
}

/// Errors that can occur during capability operations
#[derive(Error, Debug)]
pub enum CapabilityError {
    /// Access to a resource was denied
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(ResourceId),
    
    /// Cannot escalate privilege
    #[error("Cannot escalate privilege")]
    CannotEscalatePrivilege,
    
    /// Lock error
    #[error("Lock error")]
    LockError,
    
    /// Identity not found
    #[error("Identity not found: {0}")]
    IdentityNotFound(IdentityId),
    
    /// Content addressing error
    #[error("Content addressing error: {0}")]
    ContentAddressing(#[from] ContentAddressingError),
    
    /// Other error
    #[error("Capability error: {0}")]
    Other(String),
}

/// A trait for storing and retrieving content-addressed objects
pub trait ContentAddressedStorage: Send + Sync + 'static {
    /// Store an object by its content hash
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentHash, ContentAddressingError>;
    
    /// Retrieve an object by its content hash
    fn get<T: ContentAddressed>(&self, hash: &ContentHash) -> Result<T, ContentAddressingError>;
    
    /// Check if an object exists
    fn exists(&self, hash: &ContentHash) -> Result<bool, ContentAddressingError>;
}

/// In-memory storage for content-addressed objects
pub struct InMemoryContentAddressedStorage {
    /// Storage for objects keyed by content hash
    storage: std::sync::RwLock<std::collections::HashMap<ContentHash, Vec<u8>>>,
}

impl InMemoryContentAddressedStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            storage: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl ContentAddressedStorage for InMemoryContentAddressedStorage {
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentHash, ContentAddressingError> {
        let hash = object.content_hash()?;
        let bytes = object.to_bytes()?;
        
        let mut storage = self.storage.write().map_err(|_| 
            ContentAddressingError::StorageError("Failed to acquire lock".into())
        )?;
        
        storage.insert(hash, bytes);
        Ok(hash)
    }
    
    fn get<T: ContentAddressed>(&self, hash: &ContentHash) -> Result<T, ContentAddressingError> {
        let storage = self.storage.read().map_err(|_| 
            ContentAddressingError::StorageError("Failed to acquire lock".into())
        )?;
        
        let bytes = storage.get(hash).ok_or_else(|| 
            ContentAddressingError::StorageError(format!("Object not found for hash: {:?}", hash))
        )?;
        
        T::from_bytes(bytes)
    }
    
    fn exists(&self, hash: &ContentHash) -> Result<bool, ContentAddressingError> {
        let storage = self.storage.read().map_err(|_| 
            ContentAddressingError::StorageError("Failed to acquire lock".into())
        )?;
        
        Ok(storage.contains_key(hash))
    }
}

/// A registry for managing resources using capabilities
pub struct ResourceRegistry {
    /// Resources managed by this registry
    resources: Arc<RwLock<HashMap<ResourceId, Arc<dyn Any + Send + Sync>>>>,
    
    /// Type registry for resources
    type_registry: Arc<RwLock<HashMap<ResourceId, &'static str>>>,
    
    /// Capability graph tracking who has what capabilities
    capability_graph: Arc<RwLock<HashMap<IdentityId, Vec<ResourceId>>>>,
    
    /// Content-addressed storage
    content_storage: Box<dyn ContentAddressedStorage>,
}

impl ResourceRegistry {
    /// Create a new resource registry
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            type_registry: Arc::new(RwLock::new(HashMap::new())),
            capability_graph: Arc::new(RwLock::new(HashMap::new())),
            content_storage: Box::new(InMemoryContentAddressedStorage::new()),
        }
    }
    
    /// Create a new registry with custom content storage
    pub fn new_with_storage(storage: Box<dyn ContentAddressedStorage>) -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            type_registry: Arc::new(RwLock::new(HashMap::new())),
            capability_graph: Arc::new(RwLock::new(HashMap::new())),
            content_storage: storage,
        }
    }
    
    /// Register a resource and get a capability for it
    pub fn register<T: Send + Sync + 'static>(
        &self,
        resource: T,
        owner: IdentityId,
    ) -> Result<Capability<T>, CapabilityError> {
        // Create a unique resource ID
        let resource_id = ResourceId::new();
        
        // Store the resource
        let boxed_resource = Box::new(resource);
        let resource_arc = Arc::new(RwLock::new(boxed_resource as Box<T>));
        
        {
            let mut resources = self.resources.write().map_err(|_| CapabilityError::LockError)?;
            resources.insert(resource_id.clone(), resource_arc.clone() as Arc<dyn Any + Send + Sync>);
            
            let mut type_registry = self.type_registry.write().map_err(|_| CapabilityError::LockError)?;
            type_registry.insert(resource_id.clone(), std::any::type_name::<T>());
        }
        
        // Create a capability with full access
        let capability = Capability::new(
            resource_id,
            CapabilityGrants::full(),
            Some(owner.clone()),
        ).map_err(|e| CapabilityError::ContentAddressing(e))?;
        
        // Store the capability in the content-addressed storage
        self.content_storage.store(&capability)
            .map_err(|e| CapabilityError::ContentAddressing(e))?;
        
        // Record the capability in the graph
        {
            let mut graph = self.capability_graph.write().map_err(|_| CapabilityError::LockError)?;
            let entry = graph.entry(owner).or_insert_with(Vec::new);
            entry.push(capability.id.clone());
        }
        
        Ok(capability)
    }
    
    /// Access a resource using a capability
    pub fn access<T: Send + Sync + 'static>(
        &self,
        capability: &Capability<T>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        // Verify the capability's content hash
        capability.verify()
            .map_err(|e| CapabilityError::ContentAddressing(e))?;
        
        // Look up the resource
        let resources = self.resources.read().map_err(|_| CapabilityError::LockError)?;
        let resource = resources.get(&capability.id)
            .ok_or_else(|| CapabilityError::ResourceNotFound(capability.id.clone()))?;
        
        // Downcast to the correct type
        let typed_resource = resource.clone()
            .downcast::<RwLock<Box<T>>>()
            .map_err(|_| CapabilityError::Other("Type mismatch".into()))?;
        
        // Create a guard
        let guard = ResourceGuard {
            resource: typed_resource,
            capability: capability.clone(),
            registry: Arc::new(self.clone()),
        };
        
        Ok(guard)
    }
    
    /// Access a resource by content reference
    pub fn access_by_content<T: Send + Sync + 'static>(
        &self,
        content_ref: &ContentRef<Capability<T>>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        // Get the capability from content storage
        let capability = self.content_storage.get::<Capability<T>>(&content_ref.hash)
            .map_err(|e| CapabilityError::ContentAddressing(e))?;
        
        self.access(&capability)
    }
    
    /// Transfer a capability to another identity
    pub fn transfer_capability<T: ?Sized>(
        &self,
        capability: &Capability<T>,
        from: &IdentityId,
        to: &IdentityId,
    ) -> Result<(), CapabilityError> {
        // Verify the capability
        capability.verify()
            .map_err(|e| CapabilityError::ContentAddressing(e))?;
        
        // Check if the source identity has this capability
        {
            let graph = self.capability_graph.read().map_err(|_| CapabilityError::LockError)?;
            let capabilities = graph.get(from)
                .ok_or_else(|| CapabilityError::IdentityNotFound(from.clone()))?;
                
            if !capabilities.contains(&capability.id) {
                return Err(CapabilityError::AccessDenied("transfer".into()));
            }
        }
        
        // Update the capability graph
        {
            let mut graph = self.capability_graph.write().map_err(|_| CapabilityError::LockError)?;
            
            // Remove from source
            if let Some(capabilities) = graph.get_mut(from) {
                capabilities.retain(|id| id != &capability.id);
            }
            
            // Add to destination
            let entry = graph.entry(to.clone()).or_insert_with(Vec::new);
            entry.push(capability.id.clone());
        }
        
        Ok(())
    }
    
    /// Revoke a capability from an identity
    pub fn revoke_capability<T: ?Sized>(
        &self,
        capability: &Capability<T>,
        from: &IdentityId,
    ) -> Result<(), CapabilityError> {
        // Verify the capability
        capability.verify()
            .map_err(|e| CapabilityError::ContentAddressing(e))?;
        
        // Update the capability graph
        {
            let mut graph = self.capability_graph.write().map_err(|_| CapabilityError::LockError)?;
            
            // Remove from identity
            if let Some(capabilities) = graph.get_mut(from) {
                capabilities.retain(|id| id != &capability.id);
            } else {
                return Err(CapabilityError::IdentityNotFound(from.clone()));
            }
        }
        
        Ok(())
    }
    
    /// Check if an identity has a capability
    pub fn has_capability<T: ?Sized>(
        &self,
        identity: &IdentityId,
        resource_id: &ResourceId,
    ) -> Result<bool, CapabilityError> {
        let graph = self.capability_graph.read().map_err(|_| CapabilityError::LockError)?;
        
        match graph.get(identity) {
            Some(capabilities) => Ok(capabilities.contains(resource_id)),
            None => Err(CapabilityError::IdentityNotFound(identity.clone())),
        }
    }
    
    /// List all capabilities for an identity
    pub fn list_capabilities(
        &self,
        identity: &IdentityId,
    ) -> Result<Vec<ResourceId>, CapabilityError> {
        let graph = self.capability_graph.read().map_err(|_| CapabilityError::LockError)?;
        
        match graph.get(identity) {
            Some(capabilities) => Ok(capabilities.clone()),
            None => Err(CapabilityError::IdentityNotFound(identity.clone())),
        }
    }
    
    /// Get the content storage
    pub fn content_storage(&self) -> &dyn ContentAddressedStorage {
        self.content_storage.as_ref()
    }
}

impl Clone for ResourceRegistry {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone(),
            type_registry: self.type_registry.clone(),
            capability_graph: self.capability_graph.clone(),
            content_storage: Box::new(InMemoryContentAddressedStorage::new()),
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for working with capabilities
pub mod helpers {
    use super::*;
    
    /// Create a new identity
    pub fn create_identity() -> IdentityId {
        IdentityId::new()
    }
    
    /// Create a new registry
    pub fn create_registry() -> ResourceRegistry {
        ResourceRegistry::new()
    }
    
    /// Create a new content-addressed registry
    pub fn create_content_addressed_registry() -> ResourceRegistry {
        ResourceRegistry::new_with_storage(Box::new(InMemoryContentAddressedStorage::new()))
    }
    
    /// Register a resource in the registry
    pub fn register_resource<T: Send + Sync + 'static>(
        registry: &ResourceRegistry,
        resource: T,
        owner: &IdentityId,
    ) -> Result<Capability<T>, CapabilityError> {
        registry.register(resource, owner.clone())
    }
    
    /// Access a resource using a capability
    pub fn access_resource<T: Send + Sync + 'static>(
        registry: &ResourceRegistry,
        capability: &Capability<T>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        registry.access(capability)
    }
    
    /// Access a resource by content reference
    pub fn access_resource_by_content<T: Send + Sync + 'static>(
        registry: &ResourceRegistry,
        content_ref: &ContentRef<Capability<T>>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        registry.access_by_content(content_ref)
    }
    
    /// Create a read-only capability from a guard
    pub fn create_read_only<T: ?Sized>(
        guard: &ResourceGuard<T>,
    ) -> Result<Capability<T>, CapabilityError> {
        guard.create_restricted_capability(CapabilityGrants::read_only())
    }
    
    /// Create a write-only capability from a guard
    pub fn create_write_only<T: ?Sized>(
        guard: &ResourceGuard<T>,
    ) -> Result<Capability<T>, CapabilityError> {
        guard.create_restricted_capability(CapabilityGrants::write_only())
    }
    
    /// Create a content reference for a capability
    pub fn create_content_ref<T: ?Sized>(
        capability: &Capability<T>,
    ) -> Result<ContentRef<Capability<T>>, ContentAddressingError> {
        capability.to_content_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_basic() {
        let registry = ResourceRegistry::new();
        let alice = IdentityId::new();
        
        // Register a resource
        let data = "Hello, World!".to_string();
        let capability = registry.register(data, alice.clone()).unwrap();
        
        // Access the resource
        let guard = registry.access(&capability).unwrap();
        let resource = guard.read().unwrap();
        assert_eq!(*resource, "Hello, World!".to_string());
        
        // Verify content addressing works
        let content_ref = capability.to_content_ref().unwrap();
        let guard2 = registry.access_by_content(&content_ref).unwrap();
        let resource2 = guard2.read().unwrap();
        assert_eq!(*resource2, "Hello, World!".to_string());
    }
    
    #[test]
    fn test_capability_grants() {
        let registry = ResourceRegistry::new();
        let alice = IdentityId::new();
        let bob = IdentityId::new();
        
        // Register a resource
        let data = "Hello, World!".to_string();
        let capability = registry.register(data, alice.clone()).unwrap();
        
        // Access the resource
        let guard = registry.access(&capability).unwrap();
        
        // Create a read-only capability
        let read_only = guard.create_restricted_capability(CapabilityGrants::read_only()).unwrap();
        
        // Transfer it to Bob
        registry.transfer_capability(&read_only, &alice, &bob).unwrap();
        
        // Bob accesses the resource
        let bob_guard = registry.access(&read_only).unwrap();
        
        // Bob can read
        let bob_read = bob_guard.read().unwrap();
        assert_eq!(*bob_read, "Hello, World!".to_string());
        
        // But Bob can't write
        let bob_write = bob_guard.write();
        assert!(bob_write.is_err());
        
        // And Bob can't delegate
        let bob_delegate = bob_guard.create_restricted_capability(CapabilityGrants::read_only());
        assert!(bob_delegate.is_err());
    }
    
    #[test]
    fn test_content_addressing() {
        let registry = ResourceRegistry::new();
        let alice = IdentityId::new();
        
        // Register a resource
        let data = "Hello, World!".to_string();
        let capability = registry.register(data, alice.clone()).unwrap();
        
        // Get the content hash and verify it's consistent
        let hash1 = capability.content_hash().unwrap();
        let hash2 = capability.content_hash().unwrap();
        assert_eq!(hash1, hash2);
        
        // Create a content reference
        let content_ref = capability.to_content_ref().unwrap();
        assert_eq!(content_ref.hash, hash1);
        
        // Access by content reference
        let guard = registry.access_by_content(&content_ref).unwrap();
        let resource = guard.read().unwrap();
        assert_eq!(*resource, "Hello, World!".to_string());
    }
    
    #[test]
    fn test_resource_identity() {
        // Test deterministic resource IDs
        let res1 = ResourceId::new_with_name("test_resource");
        let res2 = ResourceId::new_with_name("test_resource");
        assert_eq!(res1, res2);
        
        // Test different names create different IDs
        let res3 = ResourceId::new_with_name("another_resource");
        assert_ne!(res1, res3);
        
        // Test identity IDs
        let id1 = IdentityId::new_with_name("alice");
        let id2 = IdentityId::new_with_name("alice");
        let id3 = IdentityId::new_with_name("bob");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
} 