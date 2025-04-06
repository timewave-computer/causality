// Resource type definitions
//
// This file contains the core type definitions for the resource system,
// including resource identifiers, types, and tags.

use std::fmt::{self, Display, Debug};
use std::str::FromStr;
use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};

use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use thiserror::Error;
use async_trait::async_trait;
use blake3;

// Import specifically from correct modules
use causality_types::ContentId;
pub use causality_types::crypto_primitives::ContentHash;
use causality_types::content_addressing::storage::{
    ContentAddressedStorage, ContentAddressedStorageExt, StorageError
};
use causality_types::crypto_primitives::{ContentAddressed, HashError, HashOutput, HashAlgorithm};
use causality_crypto::hash::ContentHasher;
use crate::resource::*;
use crate::serialization::{to_bytes, from_bytes};
use causality_types::ContentId as TypesContentId;
use crate::id_utils::convert_from_types_content_id;

/// Resource identifier type
///
/// A unique identifier for a resource in the system, based on content addressing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Content hash of the resource
    pub hash: causality_types::crypto_primitives::ContentHash,
    
    /// Optional human-readable name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ResourceId {
    /// Create a new resource ID with the specified hash
    pub fn new(hash: causality_types::crypto_primitives::ContentHash) -> Self {
        Self {
            hash,
            name: None,
        }
    }
    
    /// Create a new resource ID with a name
    pub fn with_name(hash: causality_types::crypto_primitives::ContentHash, name: impl Into<String>) -> Self {
        Self {
            hash,
            name: Some(name.into()),
        }
    }
    
    /// Create from a crypto ContentHash
    pub fn from_crypto_hash(hash: &causality_crypto::ContentHash) -> Self {
        let types_hash = crate::utils::content_addressing::convert_legacy_to_types_hash(hash);
        Self::new(types_hash)
    }
    
    /// Get the content hash
    pub fn content_hash(&self) -> &causality_types::crypto_primitives::ContentHash {
        &self.hash
    }
    
    /// Get the name of this resource ID
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    
    /// Create a resource ID from a string
    pub fn from_string(s: &str) -> Result<Self, String> {
        // Try to see if it's a named resource first
        if let Some((name, hash_str)) = s.split_once(':') {
            // Try to decode the hash part as hex
            if let Ok(bytes) = hex::decode(hash_str) {
                if bytes.len() == 32 {
                    return Ok(Self::with_name(ContentHash::new("blake3", bytes), name));
                }
            }
            // If not a hex string, hash the hash_str itself
            let bytes = hash_str.as_bytes().to_vec();
            return Ok(Self::with_name(ContentHash::new("blake3", bytes), name));
        }
        
        // Otherwise, try to decode as a plain hash
        if let Ok(bytes) = hex::decode(s) {
            if bytes.len() == 32 {
                return Ok(Self::new(ContentHash::new("blake3", bytes)));
            }
        }
        
        // If not a hex string, hash the input string itself
        let bytes = s.as_bytes().to_vec();
        Ok(Self::new(ContentHash::new("blake3", bytes)))
    }
    
    /// Create a resource ID from a legacy ContentId
    pub fn from_legacy_content_id(content_id: &ContentId) -> Self {
        // Since ContentId and TypesContentId are the same type (aliases),
        // we can directly call from_content_id
        Self::from_content_id(content_id)
            .unwrap_or_else(|_| {
                // Fallback: create a new ContentHash from the ContentId's bytes
                let hash_value = content_id.as_bytes().to_vec();
                let hash = ContentHash::new("blake3", hash_value);
                Self::new(hash)
            })
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}:{}", name, self.hash)
        } else {
            write!(f, "{}", self.hash)
        }
    }
}

impl FromStr for ResourceId {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

/// Resource type
///
/// Describes the type of a resource, which determines its behavior and capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceType {
    /// Type name
    pub name: String,
    
    /// Type version
    pub version: String,
    
    /// Type namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

impl ResourceType {
    /// Create a new resource type
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            namespace: None,
        }
    }
    
    /// Create a new resource type with namespace
    pub fn new_with_namespace(
        name: impl Into<String>,
        version: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            namespace: Some(namespace.into()),
        }
    }
    
    /// Get the fully qualified type name
    pub fn qualified_name(&self) -> String {
        if let Some(namespace) = &self.namespace {
            format!("{}.{}", namespace, self.name)
        } else {
            self.name.clone()
        }
    }
    
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &ResourceType) -> bool {
        self.name == other.name
            && (self.version == other.version
                || self.version == "*" 
                || other.version == "*")
            && self.namespace == other.namespace
    }
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}:{}:{}", namespace, self.name, self.version)
        } else {
            write!(f, "{}:{}", self.name, self.version)
        }
    }
}

impl FromStr for ResourceType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        
        match parts.len() {
            // name:version
            2 => Ok(ResourceType::new(parts[0], parts[1])),
            
            // namespace:name:version
            3 => Ok(ResourceType::new_with_namespace(parts[1], parts[2], parts[0])),
            
            _ => Err(format!("Invalid resource type format: {}", s)),
        }
    }
}

/// Resource tag
///
/// A tag that can be attached to a resource for filtering and organization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceTag {
    /// Tag key
    pub key: String,
    
    /// Tag value
    pub value: String,
}

impl ResourceTag {
    /// Create a new resource tag
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl Display for ResourceTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl FromStr for ResourceTag {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();
        
        if parts.len() == 2 {
            Ok(ResourceTag::new(parts[0], parts[1]))
        } else {
            Err(format!("Invalid resource tag format: {}", s))
        }
    }
}

// Resource Type Registry
//
// This module provides a content-addressed registry of resource types
// with versioning and schema validation support for resources in the system.

/// Unique identifier for a resource type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceTypeId {
    /// Base name of the resource type
    name: String,
    
    /// Optional namespace
    namespace: Option<String>,
    
    /// Version of the resource type
    version: Option<String>,
    
    /// Content hash of the resource type definition
    content_hash: Option<ContentId>,
}

impl ResourceTypeId {
    /// Create a new resource type ID
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: None,
            version: None,
            content_hash: None,
        }
    }
    
    /// Create a new resource type ID with namespace
    pub fn with_namespace(namespace: &str, name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: Some(namespace.to_string()),
            version: None,
            content_hash: None,
        }
    }
    
    /// Create a new resource type ID with namespace and version
    pub fn with_version(namespace: &str, name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: Some(namespace.to_string()),
            version: Some(version.to_string()),
            content_hash: None,
        }
    }
    
    /// Set the content hash for this resource type
    pub fn with_content_hash(mut self, hash: ContentId) -> Self {
        self.content_hash = Some(hash);
        self
    }
    
    /// Get the name of this resource type
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the namespace of this resource type, if any
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
    
    /// Get the version of this resource type, if any
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    
    /// Get the content hash of this resource type, if set
    pub fn content_hash(&self) -> Option<&ContentId> {
        self.content_hash.as_ref()
    }
    
    /// Get the fully qualified name of this resource type
    pub fn qualified_name(&self) -> String {
        match (&self.namespace, &self.version) {
            (Some(ns), Some(ver)) => format!("{}:{}:{}", ns, self.name, ver),
            (Some(ns), None) => format!("{}:{}", ns, self.name),
            (None, Some(ver)) => format!("{}:{}", self.name, ver),
            (None, None) => self.name.clone(),
        }
    }
    
    /// Check if this resource type is compatible with another
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // If the names don't match, they're not compatible
        if self.name != other.name {
            return false;
        }
        
        // If the namespaces don't match, they're not compatible
        if self.namespace != other.namespace {
            return false;
        }
        
        // If both have content hashes and they match, they're definitely compatible
        if let (Some(h1), Some(h2)) = (self.content_hash(), other.content_hash()) {
            if h1 == h2 {
                return true;
            }
        }
        
        // If one has no version, consider compatible (for backward compatibility)
        if self.version.is_none() || other.version.is_none() {
            return true;
        }
        
        // Both have versions, they must match exactly
        self.version == other.version
    }
    
    /// Get major version number if available
    pub fn major_version(&self) -> Option<u32> {
        self.version.as_ref().and_then(|v| {
            v.split('.')
                .next()
                .and_then(|major| major.parse::<u32>().ok())
        })
    }
    
    /// Get minor version number if available
    pub fn minor_version(&self) -> Option<u32> {
        self.version.as_ref().and_then(|v| {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() > 1 {
                parts[1].parse::<u32>().ok()
            } else {
                None
            }
        })
    }
    
    /// Create a new version of this resource type
    pub fn with_new_version(&self, version: &str) -> Self {
        let mut new_id = self.clone();
        new_id.version = Some(version.to_string());
        new_id.content_hash = None; // Reset content hash since this is a new version
        new_id
    }
}

impl std::fmt::Display for ResourceTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.qualified_name())
    }
}

/// Helper function to convert ContentHash to HashOutput
fn content_hash_to_hash_output(hash: &causality_types::crypto_primitives::ContentHash) -> Result<HashOutput, HashError> {
    // Use the to_hash_output method of ContentHash
    hash.to_hash_output()
}

/// Helper function to convert ContentId to HashOutput
fn content_id_to_hash_output(id: &causality_types::crypto_primitives::ContentId) -> Result<HashOutput, HashError> {
    // Use the hash method of ContentId
    Ok(id.hash().clone())
}

impl ContentAddressed for ResourceTypeId {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        if let Some(hash) = &self.content_hash {
            return content_id_to_hash_output(hash);
        }
        
        // Otherwise, compute it
        let bytes = self.to_bytes()?;
        let hash_result = blake3::hash(&bytes);
        let mut data = [0u8; 32];
        data.copy_from_slice(hash_result.as_bytes());
        
        Ok(HashOutput::new(data, HashAlgorithm::Blake3))
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        serde_json::to_vec(self)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        serde_json::from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Resource schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSchema {
    /// Schema format (e.g., "json-schema", "protobuf", etc.)
    pub format: String,
    
    /// Schema definition
    pub definition: String,
    
    /// Schema version
    pub version: String,
    
    /// Content hash of this schema
    pub content_hash: Option<causality_types::crypto_primitives::HashOutput>,
}

impl ContentAddressed for ResourceSchema {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        if let Some(hash) = &self.content_hash {
            return Ok(hash.clone());
        }
        
        // Otherwise, compute it
        let bytes = self.to_bytes()?;
        let hash_result = blake3::hash(&bytes);
        let mut data = [0u8; 32];
        data.copy_from_slice(hash_result.as_bytes());
        
        Ok(HashOutput::new(data, HashAlgorithm::Blake3))
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        serde_json::to_vec(self)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        serde_json::from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Resource type compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTypeCompatibility {
    /// Base resource type ID
    pub base_type: ResourceTypeId,
    
    /// Compatible resource type IDs
    pub compatible_types: Vec<ResourceTypeId>,
    
    /// Conversion rules (if any)
    pub conversion_rules: Option<String>,
}

/// Complete resource type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTypeDefinition {
    /// Resource type ID
    pub id: ResourceTypeId,
    
    /// Resource schema
    pub schema: ResourceSchema,
    
    /// Resource description
    pub description: Option<String>,
    
    /// Resource documentation
    pub documentation: Option<String>,
    
    /// Whether this resource type is deprecated
    pub deprecated: bool,
    
    /// Compatible with other resource types
    pub compatible_with: Vec<ResourceTypeCompatibility>,
    
    /// Required capabilities for various operations
    pub required_capabilities: HashMap<String, Vec<String>>,
    
    /// Creation date
    pub created_at: u64,
    
    /// Last updated date
    pub updated_at: u64,
}

impl ContentAddressed for ResourceTypeDefinition {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        if let Some(content_hash) = &self.id.content_hash() {
            return content_id_to_hash_output(content_hash);
        } else {
            // Compute a hash for the definition
            let bytes = self.to_bytes()?;
            let hash_result = blake3::hash(&bytes);
            let mut data = [0u8; 32];
            data.copy_from_slice(hash_result.as_bytes());
            
            Ok(HashOutput::new(data, HashAlgorithm::Blake3))
        }
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        serde_json::to_vec(self)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        serde_json::from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Resource type registry errors
#[derive(Error, Debug)]
pub enum ResourceTypeRegistryError {
    #[error("Resource type not found: {0}")]
    NotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Resource type already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Compatibility error: {0}")]
    CompatibilityError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for resource type registry operations
pub type ResourceTypeRegistryResult<T> = Result<T, ResourceTypeRegistryError>;

/// Trait for resource type registry
#[async_trait]
pub trait ResourceTypeRegistry: Send + Sync + Debug {
    /// Register a new resource type
    async fn register_resource_type(
        &self, 
        definition: ResourceTypeDefinition
    ) -> ResourceTypeRegistryResult<ResourceTypeId>;
    
    /// Get a resource type by ID
    async fn get_resource_type(
        &self, 
        id: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<ResourceTypeDefinition>;
    
    /// Check if a resource type exists
    async fn has_resource_type(&self, id: &ResourceTypeId) -> ResourceTypeRegistryResult<bool>;
    
    /// Find compatible resource types
    async fn find_compatible_types(
        &self, 
        _id: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>>;
    
    /// Get all versions of a resource type
    async fn get_all_versions(
        &self, 
        name: &str, 
        namespace: Option<&str>
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>>;
    
    /// Get the latest version of a resource type
    async fn get_latest_version(
        &self, 
        name: &str, 
        namespace: Option<&str>
    ) -> ResourceTypeRegistryResult<ResourceTypeId>;
    
    /// Check if two resource types are compatible
    async fn are_compatible(
        &self, 
        id1: &ResourceTypeId, 
        id2: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<bool>;
    
    /// Validate a resource against its type schema
    async fn validate_resource(
        &self, 
        resource_type: &ResourceTypeId, 
        resource_data: &[u8]
    ) -> ResourceTypeRegistryResult<bool>;
    
    /// Find all resource types with a specific capability requirement
    async fn find_types_with_capability(
        &self,
        capability: &str
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>>;
}

/// Content-addressed implementation of resource type registry
pub struct ContentAddressedResourceTypeRegistry {
    /// Underlying content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Cache of resource type definitions
    type_cache: HashMap<ResourceTypeId, ResourceTypeDefinition>,
    
    /// Index of resource types by name and namespace
    name_index: HashMap<(String, Option<String>), Vec<ResourceTypeId>>,
    
    /// Index of compatibility relationships
    compatibility_index: HashMap<ResourceTypeId, Vec<ResourceTypeId>>,
    
    /// Index of resource types by capability
    capability_index: HashMap<String, Vec<ResourceTypeId>>,
}

impl Debug for ContentAddressedResourceTypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContentAddressedResourceTypeRegistry")
            .field("type_cache", &self.type_cache)
            .field("name_index", &self.name_index)
            .field("compatibility_index", &self.compatibility_index)
            .field("capability_index", &self.capability_index)
            .finish()
    }
}

impl ContentAddressedResourceTypeRegistry {
    /// Create a new content-addressed resource type registry
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            storage,
            type_cache: HashMap::new(),
            name_index: HashMap::new(),
            compatibility_index: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }
    
    /// Add to name index
    fn index_by_name(&mut self, id: &ResourceTypeId) {
        let key = (id.name.clone(), id.namespace.clone());
        let ids = self.name_index
            .entry(key)
            .or_insert_with(Vec::new);
        
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    
    /// Add to compatibility index
    fn index_compatibility(&mut self, definition: &ResourceTypeDefinition) {
        // Index this type's compatibility with others
        for compat in &definition.compatible_with {
            let compatible_types = self.compatibility_index
                .entry(compat.base_type.clone())
                .or_insert_with(Vec::new);
            
            if !compatible_types.contains(&definition.id) {
                compatible_types.push(definition.id.clone());
            }
            
            // Also add the compatibility in the other direction
            for compat_type in &compat.compatible_types {
                let base_types = self.compatibility_index
                    .entry(compat_type.clone())
                    .or_insert_with(Vec::new);
                
                if !base_types.contains(&definition.id) {
                    base_types.push(definition.id.clone());
                }
            }
        }
    }
    
    /// Add to capability index
    fn index_capabilities(&mut self, definition: &ResourceTypeDefinition) {
        for (_, caps) in &definition.required_capabilities {
            for cap in caps {
                let types = self.capability_index
                    .entry(cap.clone())
                    .or_insert_with(Vec::new);
                
                if !types.contains(&definition.id) {
                    types.push(definition.id.clone());
                }
            }
        }
    }
    
    /// Find the latest version from a list of resource type IDs
    fn find_latest_version(&self, ids: &[ResourceTypeId]) -> Option<ResourceTypeId> {
        ids.iter()
            .filter_map(|id| {
                // Extract version components
                let major = id.major_version();
                let minor = id.minor_version();
                
                // Return with version info for sorting
                Some((id.clone(), major?, minor.unwrap_or(0)))
            })
            .max_by(|a, b| {
                // Sort by major version first, then minor
                let (_, a_major, a_minor) = a;
                let (_, b_major, b_minor) = b;
                
                a_major.cmp(&b_major).then(a_minor.cmp(&b_minor))
            })
            .map(|(id, _, _)| id)
    }
}

#[async_trait]
impl ResourceTypeRegistry for ContentAddressedResourceTypeRegistry {
    async fn register_resource_type(
        &self, 
        mut definition: ResourceTypeDefinition
    ) -> ResourceTypeRegistryResult<ResourceTypeId> {
        // Compute content hash for the schema
        let schema_hash = definition.schema.content_hash()
            .map_err(|e| ResourceTypeRegistryError::SerializationError(e.to_string()))?;
        
        // Update schema with content hash
        definition.schema.content_hash = Some(schema_hash);
        
        // Compute content hash for the resource type
        let type_hash = definition.content_hash()
            .map_err(|e| ResourceTypeRegistryError::SerializationError(e.to_string()))?;
        
        // Update resource type ID with content hash
        let type_hash_str = type_hash.to_string();
        let parts: Vec<&str> = type_hash_str.split(':').collect();
        let algorithm = if parts.len() > 1 { parts[0] } else { "blake3" };
        let value = if parts.len() > 1 { parts[1] } else { &type_hash_str };
        
        let content_id = ContentId::new(algorithm.to_string() + ":" + value);
        definition.id = definition.id.with_content_hash(content_id.clone());
        
        // Check if already exists
        if self.has_resource_type(&definition.id).await? {
            return Err(ResourceTypeRegistryError::AlreadyExists(
                format!("Resource type already exists: {}", definition.id)
            ));
        }
        
        // Serialize the definition
        let definition_bytes = to_bytes(&definition)
            .map_err(|e| ResourceTypeRegistryError::SerializationError(e.to_string()))?;
        
        // Store in content-addressed storage
        self.storage.store_bytes(&definition_bytes)
            .map_err(|e| ResourceTypeRegistryError::StorageError(e.to_string()))?;
        
        // In a real implementation, update indexes
        // Here we would need proper locking or database transactions
        
        Ok(definition.id.clone())
    }
    
    async fn get_resource_type(
        &self, 
        id: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<ResourceTypeDefinition> {
        // Check cache first (in a real implementation)
        
        // Retrieve from content-addressed storage by content hash
        if let Some(content_hash) = &id.content_hash {
            // Convert ContentId to the type expected by the storage
            let crypto_content_id = convert_from_types_content_id(content_hash);
            
            let bytes = self.storage.get_bytes(&crypto_content_id)
                .map_err(|e| match e {
                    StorageError::NotFound(_) => 
                        ResourceTypeRegistryError::NotFound(format!("Resource type not found: {}", id)),
                    _ => ResourceTypeRegistryError::StorageError(e.to_string()),
                })?;
            
            // Deserialize the definition
            let definition: ResourceTypeDefinition = from_bytes(&bytes)
                .map_err(|e| ResourceTypeRegistryError::SerializationError(e.to_string()))?;
            
            Ok(definition)
        } else {
            Err(ResourceTypeRegistryError::NotFound(
                format!("Resource type has no content hash: {}", id)
            ))
        }
    }
    
    async fn has_resource_type(&self, id: &ResourceTypeId) -> ResourceTypeRegistryResult<bool> {
        // Check if exists in storage based on content hash
        if let Some(content_hash) = &id.content_hash {
            // Convert ContentId to the type expected by the storage
            let crypto_content_id = convert_from_types_content_id(content_hash);
            
            Ok(self.storage.contains(&crypto_content_id))
        } else {
            Ok(false)
        }
    }
    
    async fn find_compatible_types(
        &self, 
        _id: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>> {
        // In a real implementation, query compatibility index
        // For now, just return empty list
        Ok(Vec::new())
    }
    
    async fn get_all_versions(
        &self, 
        name: &str, 
        namespace: Option<&str>
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>> {
        // In a real implementation, query name index
        // For now, just return empty list
        Ok(Vec::new())
    }
    
    async fn get_latest_version(
        &self, 
        name: &str, 
        namespace: Option<&str>
    ) -> ResourceTypeRegistryResult<ResourceTypeId> {
        // Get all versions
        let versions = self.get_all_versions(name, namespace).await?;
        
        if versions.is_empty() {
            return Err(ResourceTypeRegistryError::NotFound(
                format!("No versions found for resource type: {}", name)
            ));
        }
        
        // Find latest version
        if let Some(latest) = self.find_latest_version(&versions) {
            Ok(latest)
        } else {
            Err(ResourceTypeRegistryError::NotFound(
                format!("No valid versions found for resource type: {}", name)
            ))
        }
    }
    
    async fn are_compatible(
        &self, 
        id1: &ResourceTypeId, 
        id2: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<bool> {
        // Direct compatibility check
        if id1.is_compatible_with(id2) {
            return Ok(true);
        }
        
        // Check in compatibility index
        // In a real implementation, this would query the index
        
        // For now, just return false for anything not directly compatible
        Ok(false)
    }
    
    async fn validate_resource(
        &self, 
        resource_type: &ResourceTypeId, 
        resource_data: &[u8]
    ) -> ResourceTypeRegistryResult<bool> {
        // Get the resource type definition
        let definition = self.get_resource_type(resource_type).await?;
        
        // In a real implementation, use the schema to validate the resource data
        // For now, just return true
        Ok(true)
    }
    
    async fn find_types_with_capability(
        &self,
        capability: &str
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>> {
        // In a real implementation, query capability index
        // For now, just return empty list
        Ok(Vec::new())
    }
}

/// In-memory implementation of resource type registry for testing
#[derive(Debug)]
pub struct InMemoryResourceTypeRegistry {
    types: HashMap<ResourceTypeId, ResourceTypeDefinition>,
    name_index: HashMap<(String, Option<String>), Vec<ResourceTypeId>>,
    compatibility_index: HashMap<ResourceTypeId, Vec<ResourceTypeId>>,
    capability_index: HashMap<String, Vec<ResourceTypeId>>,
}

impl InMemoryResourceTypeRegistry {
    /// Create a new in-memory resource type registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            name_index: HashMap::new(),
            compatibility_index: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }
    
    /// Add to name index
    fn index_by_name(&mut self, id: &ResourceTypeId) {
        let key = (id.name.clone(), id.namespace.clone());
        let ids = self.name_index
            .entry(key)
            .or_insert_with(Vec::new);
        
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    
    /// Add to compatibility index
    fn index_compatibility(&mut self, definition: &ResourceTypeDefinition) {
        // Index this type's compatibility with others
        for compat in &definition.compatible_with {
            let compatible_types = self.compatibility_index
                .entry(compat.base_type.clone())
                .or_insert_with(Vec::new);
            
            if !compatible_types.contains(&definition.id) {
                compatible_types.push(definition.id.clone());
            }
            
            // Also add the compatibility in the other direction
            for compat_type in &compat.compatible_types {
                let base_types = self.compatibility_index
                    .entry(compat_type.clone())
                    .or_insert_with(Vec::new);
                
                if !base_types.contains(&definition.id) {
                    base_types.push(definition.id.clone());
                }
            }
        }
    }
    
    /// Add to capability index
    fn index_capabilities(&mut self, definition: &ResourceTypeDefinition) {
        for (_, caps) in &definition.required_capabilities {
            for cap in caps {
                let types = self.capability_index
                    .entry(cap.clone())
                    .or_insert_with(Vec::new);
                
                if !types.contains(&definition.id) {
                    types.push(definition.id.clone());
                }
            }
        }
    }
    
    /// Find the latest version from a list of resource type IDs
    fn find_latest_version(&self, ids: &[ResourceTypeId]) -> Option<ResourceTypeId> {
        ids.iter()
            .filter_map(|id| {
                // Extract version components
                let major = id.major_version();
                let minor = id.minor_version();
                
                // Return with version info for sorting
                Some((id.clone(), major?, minor.unwrap_or(0)))
            })
            .max_by(|a, b| {
                // Sort by major version first, then minor
                let (_, a_major, a_minor) = a;
                let (_, b_major, b_minor) = b;
                
                a_major.cmp(&b_major).then(a_minor.cmp(&b_minor))
            })
            .map(|(id, _, _)| id)
    }
}

#[async_trait]
impl ResourceTypeRegistry for InMemoryResourceTypeRegistry {
    async fn register_resource_type(
        &self, 
        mut definition: ResourceTypeDefinition
    ) -> ResourceTypeRegistryResult<ResourceTypeId> {
        // Compute content hash for the schema
        let schema_hash = definition.schema.content_hash()
            .map_err(|e| ResourceTypeRegistryError::SerializationError(e.to_string()))?;
        
        // Update schema with content hash
        definition.schema.content_hash = Some(schema_hash);
        
        // Compute content hash for the resource type
        let type_hash = definition.content_hash()
            .map_err(|e| ResourceTypeRegistryError::SerializationError(e.to_string()))?;
        
        // Update resource type ID with content hash
        let type_hash_str = type_hash.to_string();
        let parts: Vec<&str> = type_hash_str.split(':').collect();
        let algorithm = if parts.len() > 1 { parts[0] } else { "blake3" };
        let value = if parts.len() > 1 { parts[1] } else { &type_hash_str };
        
        let content_id = ContentId::new(algorithm.to_string() + ":" + value);
        definition.id = definition.id.with_content_hash(content_id);
        
        // Check if already exists
        let mut types = self.types.clone();
        if types.contains_key(&definition.id) {
            return Err(ResourceTypeRegistryError::AlreadyExists(
                format!("Resource type already exists: {}", definition.id)
            ));
        }
        
        // Store in memory
        let type_id = definition.id.clone();
        types.insert(type_id.clone(), definition.clone());
        
        // Update indexes
        let mut name_index = self.name_index.clone();
        let key = (type_id.name.clone(), type_id.namespace.clone());
        let ids = name_index
            .entry(key)
            .or_insert_with(Vec::new);
        
        if !ids.contains(&type_id) {
            ids.push(type_id.clone());
        }
        
        // Update compatibility index
        let mut compatibility_index = self.compatibility_index.clone();
        for compat in &definition.compatible_with {
            let compatible_types = compatibility_index
                .entry(compat.base_type.clone())
                .or_insert_with(Vec::new);
            
            if !compatible_types.contains(&definition.id) {
                compatible_types.push(definition.id.clone());
            }
            
            // Also add the compatibility in the other direction
            for compat_type in &compat.compatible_types {
                let base_types = compatibility_index
                    .entry(compat_type.clone())
                    .or_insert_with(Vec::new);
                
                if !base_types.contains(&definition.id) {
                    base_types.push(definition.id.clone());
                }
            }
        }
        
        // Update capability index
        let mut capability_index = self.capability_index.clone();
        for (_, caps) in &definition.required_capabilities {
            for cap in caps {
                let types = capability_index
                    .entry(cap.clone())
                    .or_insert_with(Vec::new);
                
                if !types.contains(&definition.id) {
                    types.push(definition.id.clone());
                }
            }
        }
        
        Ok(type_id)
    }
    
    async fn get_resource_type(
        &self, 
        id: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<ResourceTypeDefinition> {
        if let Some(definition) = self.types.get(id) {
            Ok(definition.clone())
        } else {
            Err(ResourceTypeRegistryError::NotFound(
                format!("Resource type not found: {}", id)
            ))
        }
    }
    
    async fn has_resource_type(&self, id: &ResourceTypeId) -> ResourceTypeRegistryResult<bool> {
        Ok(self.types.contains_key(id))
    }
    
    async fn find_compatible_types(
        &self, 
        _id: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>> {
        // In a real implementation, query compatibility index
        // For now, just return empty list
        Ok(Vec::new())
    }
    
    async fn get_all_versions(
        &self, 
        name: &str, 
        namespace: Option<&str>
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>> {
        let key = (name.to_string(), namespace.map(|s| s.to_string()));
        if let Some(versions) = self.name_index.get(&key) {
            Ok(versions.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn get_latest_version(
        &self, 
        name: &str, 
        namespace: Option<&str>
    ) -> ResourceTypeRegistryResult<ResourceTypeId> {
        // Get all versions
        let versions = self.get_all_versions(name, namespace).await?;
        
        if versions.is_empty() {
            return Err(ResourceTypeRegistryError::NotFound(
                format!("No versions found for resource type: {}", name)
            ));
        }
        
        // Find latest version
        if let Some(latest) = self.find_latest_version(&versions) {
            Ok(latest)
        } else {
            Err(ResourceTypeRegistryError::NotFound(
                format!("No valid versions found for resource type: {}", name)
            ))
        }
    }
    
    async fn are_compatible(
        &self, 
        id1: &ResourceTypeId, 
        id2: &ResourceTypeId
    ) -> ResourceTypeRegistryResult<bool> {
        // Direct compatibility check
        if id1.is_compatible_with(id2) {
            return Ok(true);
        }
        
        // Check in compatibility index
        if let Some(compatible_types) = self.compatibility_index.get(id1) {
            if compatible_types.contains(id2) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn validate_resource(
        &self, 
        resource_type: &ResourceTypeId, 
        _resource_data: &[u8]
    ) -> ResourceTypeRegistryResult<bool> {
        // Check if the resource type exists
        if !self.types.contains_key(resource_type) {
            return Err(ResourceTypeRegistryError::NotFound(
                format!("Resource type not found: {}", resource_type)
            ));
        }
        
        // In a real implementation, use the schema to validate the resource data
        // For now, just return true
        Ok(true)
    }
    
    async fn find_types_with_capability(
        &self,
        capability: &str
    ) -> ResourceTypeRegistryResult<Vec<ResourceTypeId>> {
        if let Some(types) = self.capability_index.get(capability) {
            Ok(types.clone())
        } else {
            Ok(Vec::new())
        }
    }
}

/// Create a configured resource type registry
pub fn create_resource_type_registry(
    storage: Arc<dyn ContentAddressedStorage>,
) -> Arc<dyn ResourceTypeRegistry> {
    Arc::new(ContentAddressedResourceTypeRegistry::new(storage))
}

pub fn create_resource_type_definition(
    registry: &InMemoryResourceTypeRegistry,
    name: &str,
    version: &str,
    schema: &str,
) -> ResourceTypeDefinition {
    let mut definition = ResourceTypeDefinition {
        id: ResourceTypeId::with_version("test", name, version),
        schema: ResourceSchema {
            format: "json-schema".to_string(),
            definition: schema.to_string(),
            version: "1.0".to_string(),
            content_hash: None,
        },
        description: Some(format!("Resource type for {}", name)),
        documentation: None,
        deprecated: false,
        compatible_with: Vec::new(),
        required_capabilities: HashMap::new(),
        created_at: 12345,
        updated_at: 12345,
    };

    // Compute schema hash
    let schema_hash = definition.schema.content_hash().unwrap();
    definition.schema.content_hash = Some(schema_hash);

    // Compute type hash
    let type_hash = definition.content_hash().unwrap();
    let type_hash_str = type_hash.to_string();
    let parts: Vec<&str> = type_hash_str.split(':').collect();
    let algorithm = if parts.len() > 1 { parts[0] } else { "blake3" };
    let value = if parts.len() > 1 { parts[1] } else { &type_hash_str };
    
    let content_id = ContentId::new(algorithm.to_string() + ":" + value);
    definition.id = definition.id.with_content_hash(content_id);

    definition
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::content_addressing::storage::InMemoryStorage;
    
    fn create_test_schema() -> ResourceSchema {
        ResourceSchema {
            format: "json-schema".to_string(),
            definition: r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#.to_string(),
            version: "1.0".to_string(),
            content_hash: None,
        }
    }
    
    fn create_test_resource_type(name: &str, version: &str) -> ResourceTypeDefinition {
        ResourceTypeDefinition {
            id: ResourceTypeId::with_version("test", name, version),
            schema: create_test_schema(),
            description: Some(format!("Test resource type {}", name)),
            documentation: None,
            deprecated: false,
            compatible_with: Vec::new(),
            required_capabilities: HashMap::new(),
            created_at: 12345,
            updated_at: 12345,
        }
    }
    
    #[tokio::test]
    async fn test_resource_type_compatibility() {
        let type1 = ResourceTypeId::with_version("test", "user", "1.0");
        let type2 = ResourceTypeId::with_version("test", "user", "1.1");
        let type3 = ResourceTypeId::with_version("test", "user", "2.0");
        let type4 = ResourceTypeId::with_version("test", "profile", "1.0");
        
        // Same base name and namespace, minor version difference should be compatible
        assert!(type1.is_compatible_with(&type2));
        
        // Same base name and namespace, major version difference should not be compatible
        assert!(!type1.is_compatible_with(&type3));
        
        // Different base name should not be compatible
        assert!(!type1.is_compatible_with(&type4));
    }
    
    #[tokio::test]
    async fn test_in_memory_registry() {
        let registry = InMemoryResourceTypeRegistry::new();
        
        // Create test resource types
        let type1 = create_test_resource_type("user", "1.0");
        let type2 = create_test_resource_type("user", "1.1");
        let type3 = create_test_resource_type("profile", "1.0");
        
        // Register resource types
        let id1 = registry.register_resource_type(type1.clone()).await.unwrap();
        let id2 = registry.register_resource_type(type2.clone()).await.unwrap();
        let id3 = registry.register_resource_type(type3.clone()).await.unwrap();
        
        // Verify resource types exist
        assert!(registry.has_resource_type(&id1).await.unwrap());
        assert!(registry.has_resource_type(&id2).await.unwrap());
        assert!(registry.has_resource_type(&id3).await.unwrap());
        
        // Get resource types
        let retrieved1 = registry.get_resource_type(&id1).await.unwrap();
        let retrieved2 = registry.get_resource_type(&id2).await.unwrap();
        
        // Verify properties
        assert_eq!(retrieved1.id, id1);
        assert_eq!(retrieved2.id, id2);
        assert_eq!(retrieved1.description, Some("Test resource type user".to_string()));
        
        // Get latest version
        let latest = registry.get_latest_version("user", Some("test")).await.unwrap();
        assert_eq!(latest, id2); // 1.1 is newer than 1.0
        
        // Get all versions
        let versions = registry.get_all_versions("user", Some("test")).await.unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&id1));
        assert!(versions.contains(&id2));
    }
}

// Convert ResourceId to ContentId
impl From<ResourceId> for TypesContentId {
    fn from(resource_id: ResourceId) -> Self {
        // Create a ContentId from the hash using HashOutput
        let hash_output = resource_id.hash.to_hash_output()
            .unwrap_or_else(|_| {
                // Create a default HashOutput if conversion fails
                let mut data = [0u8; 32];
                data.copy_from_slice(&resource_id.hash.bytes[..32]);
                HashOutput::new(data, HashAlgorithm::Blake3)
            });
        
        ContentId::from(hash_output)
    }
}

// Convert ContentId to ResourceId
impl TryFrom<TypesContentId> for ResourceId {
    type Error = crate::resource::ResourceError;
    
    fn try_from(content_id: TypesContentId) -> Result<Self, Self::Error> {
        // Get the HashOutput from ContentId and create a ContentHash
        let hash_output = content_id.hash();
        let content_hash = ContentHash::from_hash_output(hash_output);
        
        Ok(ResourceId::new(content_hash))
    }
}

// Make it easier to convert references with these helper methods
impl ResourceId {
    pub fn to_content_id(&self) -> TypesContentId {
        // Create a ContentId from the hash using HashOutput
        let hash_output = self.hash.to_hash_output()
            .unwrap_or_else(|_| {
                // Create a default HashOutput if conversion fails
                let mut data = [0u8; 32];
                data.copy_from_slice(&self.hash.bytes[..32]);
                HashOutput::new(data, HashAlgorithm::Blake3)
            });
        
        ContentId::from(hash_output)
    }
    
    pub fn from_content_id(content_id: &TypesContentId) -> Result<Self, crate::resource::ResourceError> {
        // Get the HashOutput from ContentId and create a ContentHash
        let hash_output = content_id.hash();
        let content_hash = ContentHash::from_hash_output(hash_output);
        
        Ok(ResourceId::new(content_hash))
    }
} 
