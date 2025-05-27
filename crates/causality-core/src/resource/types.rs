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
use causality_types::ContentId as TypesContentId;

// Define ResourceState enum here
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ResourceState {
    Created,
    Active,
    Inactive,
    Locked,
    Frozen,     // Immutable but accessible
    Consumed,   // Used up
    Archived,   // Kept for history but inactive
    Deleted,    // Marked for deletion
    Custom(String), // For domain-specific states
}

impl Default for ResourceState {
    fn default() -> Self {
        ResourceState::Created
    }
}

/// Resource identifier type
///
/// A unique identifier for a resource in the system, based on content addressing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
    
    /// Create a new resource ID with a random content hash
    pub fn new_random() -> Self {
        use causality_crypto::hash::random_hash;
        let random_bytes = random_hash().as_bytes().to_vec();
        Self::new(ContentHash::new("blake3", random_bytes))
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
    type Error = String;
    
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
    
    pub fn from_content_id(content_id: &TypesContentId) -> Result<Self, String> {
        // Get the HashOutput from ContentId and create a ContentHash
        let hash_output = content_id.hash();
        let content_hash = ContentHash::from_hash_output(hash_output);
        
        Ok(ResourceId::new(content_hash))
    }
} 
