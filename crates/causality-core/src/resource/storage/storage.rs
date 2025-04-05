// Resource Storage Implementation
//
// This module provides interfaces and implementations for storing resources
// using content addressing principles. It includes support for versioning,
// indexing, and efficient retrieval.

use crate::resource::Resource;

use std::fmt::{Debug};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use futures::stream::StreamExt;

use crate::resource::{ResourceId, ResourceTypeId};
use crate::serialization::{to_bytes, from_bytes};
use crate::utils::content_addressing::{content_hash_to_id};

// Make consistent imports from causality_types
use causality_types::ContentId;
use causality_types::ContentAddressed;
use causality_types::content_addressing::storage::{
    ContentAddressedStorage, ContentAddressedStorageExt, StorageError
};
use causality_types::ContentHash;
use causality_types::crypto_primitives::HashOutput;
use causality_crypto::HashError;
use crate::id_utils::{convert_to_types_content_id, convert_from_types_content_id};
// Alias TypesContentId for clarity

/// Errors that can occur during resource storage operations
#[derive(Error, Debug)]
pub enum ResourceStorageError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Resource validation error: {0}")]
    ValidationError(String),
    
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Version conflict: {0}")]
    VersionConflict(String),
    
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for resource storage operations
pub type ResourceStorageResult<T> = Result<T, ResourceStorageError>;

/// Versioned resource record for tracking resource history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceVersion {
    /// Resource identifier
    pub resource_id: ResourceId,
    
    /// Version number (monotonically increasing)
    pub version: u64,
    
    /// Content hash of this version
    pub content_hash: ContentId,
    
    /// When this version was created (timestamp)
    pub created_at: u64,
    
    /// Reference to previous version (if any)
    pub previous_version: Option<u64>,
    
    /// Resource type ID
    pub resource_type: ResourceTypeId,
    
    /// Metadata for this version
    pub metadata: HashMap<String, String>,
}

/// Index entry for resource lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIndexEntry {
    /// Resource ID
    pub resource_id: ResourceId,
    
    /// Resource type
    pub resource_type: ResourceTypeId,
    
    /// Current version number
    pub current_version: u64,
    
    /// Content hash of the current version
    pub current_hash: ContentId,
    
    /// Tags for this resource
    pub tags: HashSet<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last update timestamp
    pub updated_at: u64,
}

/// Interface for storing and retrieving resources
#[async_trait]
pub trait ResourceStorage: Send + Sync + Debug {
    /// Store a resource
    async fn store_resource<T: ContentAddressed + Send + Sync + serde::Serialize>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId>;
    
    /// Get a resource by ID (latest version)
    async fn get_resource<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T>;
    
    /// Get a specific version of a resource
    async fn get_resource_version<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T>;
    
    /// Check if a resource exists
    async fn has_resource(&self, resource_id: &ResourceId) -> ResourceStorageResult<bool>;
    
    /// Update a resource
    async fn update_resource<T: ContentAddressed + Send + Sync + Serialize>(
        &self, 
        resource_id: &ResourceId, 
        resource: T,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<u64>;
    
    /// Add a tag to a resource
    async fn add_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()>;
    
    /// Remove a tag from a resource
    async fn remove_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()>;
    
    /// Find resources by type
    async fn find_resources_by_type(
        &self, 
        resource_type: &ResourceTypeId
    ) -> ResourceStorageResult<Vec<ResourceId>>;
    
    /// Find resources by tag
    async fn find_resources_by_tag(
        &self, 
        tag: &str
    ) -> ResourceStorageResult<Vec<ResourceId>>;
    
    /// Get resource version history
    async fn get_version_history(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<Vec<ResourceVersion>>;
    
    /// Get resource metadata
    async fn get_resource_metadata(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<HashMap<String, String>>;
}

/// Convert a HashOutput into ContentId
fn hash_output_to_content_id(hash: &HashOutput) -> ContentId {
    ContentId::from(hash.clone())
}

/// Content-addressed resource storage implementation
#[derive(Debug)]
pub struct ContentAddressedResourceStorage {
    /// Underlying content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Resource index by ID
    resource_index: RwLock<HashMap<ResourceId, ResourceIndexEntry>>,
    
    /// Resource index by type
    type_index: RwLock<HashMap<ResourceTypeId, HashSet<ResourceId>>>,
    
    /// Resource index by tag
    tag_index: RwLock<HashMap<String, HashSet<ResourceId>>>,
    
    /// Resource version history
    version_history: RwLock<HashMap<ResourceId, Vec<ResourceVersion>>>,
}

impl ContentAddressedResourceStorage {
    /// Create a new content-addressed resource storage with the given underlying storage
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            storage,
            resource_index: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
            tag_index: RwLock::new(HashMap::new()),
            version_history: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add to type index
    fn index_by_type(&self, resource_id: &ResourceId, resource_type: &ResourceTypeId) -> ResourceStorageResult<()> {
        let mut type_index = self.type_index.write().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire type index lock: {}", e))
        )?;
        
        let resources = type_index
            .entry(resource_type.clone())
            .or_insert_with(HashSet::new);
            
        resources.insert(resource_id.clone());
        
        Ok(())
    }
    
    /// Add to tag index
    fn index_by_tag(&self, resource_id: &ResourceId, tag: &str) -> ResourceStorageResult<()> {
        let mut tag_index = self.tag_index.write().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire tag index lock: {}", e))
        )?;
        
        let resources = tag_index
            .entry(tag.to_string())
            .or_insert_with(HashSet::new);
            
        resources.insert(resource_id.clone());
        
        Ok(())
    }
    
    /// Remove from tag index
    fn remove_from_tag_index(&self, resource_id: &ResourceId, tag: &str) -> ResourceStorageResult<()> {
        let mut tag_index = self.tag_index.write().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire tag index lock: {}", e))
        )?;
        
        if let Some(resources) = tag_index.get_mut(tag) {
            resources.remove(resource_id);
            
            // Remove the tag entry if it's empty
            if resources.is_empty() {
                tag_index.remove(tag);
            }
        }
        
        Ok(())
    }
    
    /// Add version to history
    fn add_to_version_history(&self, version: ResourceVersion) -> ResourceStorageResult<()> {
        let mut version_history = self.version_history.write().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire version history lock: {}", e))
        )?;
        
        let history = version_history
            .entry(version.resource_id.clone())
            .or_insert_with(Vec::new);
            
        history.push(version);
        
        Ok(())
    }
    
    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    // Replace with internal helper methods if needed
    async fn get_bytes_internal(&self, content_id: &ContentId) -> ResourceStorageResult<Vec<u8>> {
        // Convert ContentId for storage access
        let crypto_content_id = convert_from_types_content_id(content_id);
        self.storage.get_bytes(&crypto_content_id)
            .map_err(|e| ResourceStorageError::StorageError(e.to_string()))
    }

    async fn contains_internal(&self, content_id: &ContentId) -> ResourceStorageResult<bool> {
        // Convert ContentId for storage check
        let crypto_content_id = convert_from_types_content_id(content_id);
        Ok(self.storage.contains(&crypto_content_id))
    }
}

/// Remove function that uses private ContentHash
// fn content_hash_to_content_id(hash: &causality_types::content::ContentHash) -> ContentId {
//     ContentId::from_content_hash(hash)
// }

#[async_trait]
impl ResourceStorage for ContentAddressedResourceStorage {
    async fn store_resource<T: ContentAddressed + Send + Sync + serde::Serialize>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId> {
        // Generate content hash for the resource
        let hash_output = resource.content_hash().map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Create resource ID directly from the HashOutput
        let content_id = ContentId::from(hash_output.clone());
        let resource_id = ResourceId::from_content_id(&content_id)
            .expect("Failed to create ResourceId from ContentId");
        
        // Check if resource already exists
        if self.has_resource(&resource_id).await? {
            return Err(ResourceStorageError::AlreadyExists(
                format!("Resource already exists: {}", resource_id)
            ));
        }
        
        // Serialize the resource
        let resource_bytes = to_bytes(&resource).map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Store in content-addressed storage
        let stored_content_id = self.storage.store_bytes(&resource_bytes).map_err(|e| 
            ResourceStorageError::StorageError(e.to_string())
        )?;
        
        // Create metadata for this version
        let timestamp = self.current_timestamp();
        let version = ResourceVersion {
            resource_id: resource_id.clone(),
            version: 1, // First version
            content_hash: convert_to_types_content_id(&stored_content_id),
            created_at: timestamp,
            previous_version: None,
            resource_type: resource_type.clone(),
            metadata: metadata.clone().unwrap_or_default(),
        };
        
        // Create index entry
        let index_entry = ResourceIndexEntry {
            resource_id: resource_id.clone(),
            resource_type: resource_type.clone(),
            current_version: 1,
            current_hash: convert_to_types_content_id(&stored_content_id),
            tags: HashSet::new(),
            created_at: timestamp,
            updated_at: timestamp,
        };
        
        // Update indexes
        {
            let mut resource_index = self.resource_index.write().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            resource_index.insert(resource_id.clone(), index_entry);
        }
        
        // Add to type index
        self.index_by_type(&resource_id, &resource_type)?;
        
        // Add to version history
        self.add_to_version_history(version)?;
        
        Ok(resource_id)
    }
    
    async fn get_resource<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T> {
        // Get the current version from the index
        let content_id = {
            let resource_index = self.resource_index.read().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            resource_index.get(resource_id)
                .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?
                .current_hash
                .clone()
        };
        
        // Retrieve from content-addressed storage using the correct conversion function
        let crypto_content_id = convert_from_types_content_id(&content_id);
        let resource_bytes = self.storage.get_bytes(&crypto_content_id)
            .map_err(|e| 
                match e {
                    StorageError::NotFound(_) => 
                        ResourceStorageError::NotFound(format!("Resource data not found: {}", resource_id)),
                    _ => ResourceStorageError::StorageError(e.to_string()),
                }
            )?;
        
        // Deserialize the resource
        from_bytes::<T>(&resource_bytes).map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )
    }
    
    async fn get_resource_version<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T> {
        // Get the version history
        let content_id = {
            let version_history = self.version_history.read().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire version history lock: {}", e))
            )?;
            
            let history = version_history.get(resource_id)
                .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?;
                
            let version_entry = history.iter()
                .find(|v| v.version == version)
                .ok_or_else(|| ResourceStorageError::NotFound(
                    format!("Version {} not found for resource: {}", version, resource_id)
                ))?;
                
            version_entry.content_hash.clone()
        };
        
        // Retrieve from content-addressed storage using the correct conversion function
        let crypto_content_id = convert_from_types_content_id(&content_id);
        let resource_bytes = self.storage.get_bytes(&crypto_content_id)
            .map_err(|e| 
                match e {
                    StorageError::NotFound(_) => 
                        ResourceStorageError::NotFound(format!("Resource data not found: {}", resource_id)),
                    _ => ResourceStorageError::StorageError(e.to_string()),
                }
            )?;
        
        // Deserialize the resource
        from_bytes::<T>(&resource_bytes).map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )
    }
    
    async fn has_resource(&self, resource_id: &ResourceId) -> ResourceStorageResult<bool> {
        let resource_index = self.resource_index.read().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
        )?;
        
        Ok(resource_index.contains_key(resource_id))
    }
    
    async fn update_resource<T: ContentAddressed + Send + Sync + Serialize>(
        &self, 
        resource_id: &ResourceId, 
        resource: T,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<u64> {
        // Check if resource exists
        if !self.has_resource(resource_id).await? {
            return Err(ResourceStorageError::NotFound(
                format!("Resource not found: {}", resource_id)
            ));
        }
        
        // Get current version info
        let (current_version, resource_type, previous_id) = {
            let resource_index = self.resource_index.read().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            let entry = resource_index.get(resource_id)
                .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?;
                
            (entry.current_version, entry.resource_type.clone(), entry.current_hash.clone())
        };
        
        // Generate content hash for the new version
        let hash_output = resource.content_hash().map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Create ContentId from HashOutput
        let content_id = hash_output_to_content_id(&hash_output);
        
        // Skip update if content hasn't changed
        if content_id == previous_id {
            return Ok(current_version); // No change, return current version
        }
        
        // Serialize the resource
        let resource_bytes = to_bytes(&resource).map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Store in content-addressed storage
        let stored_id = self.storage.store_bytes(&resource_bytes).map_err(|e| 
            ResourceStorageError::StorageError(e.to_string())
        )?;
        
        // Create metadata for this version
        let timestamp = self.current_timestamp();
        let new_version = current_version + 1;
        let version = ResourceVersion {
            resource_id: resource_id.clone(),
            version: new_version,
            content_hash: convert_to_types_content_id(&stored_id),
            created_at: timestamp,
            previous_version: Some(current_version),
            resource_type: resource_type.clone(),
            metadata: metadata.clone().unwrap_or_default(),
        };
        
        // Update index entry
        {
            let mut resource_index = self.resource_index.write().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            let entry = resource_index.get_mut(resource_id)
                .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?;
                
            entry.current_version = new_version;
            entry.current_hash = convert_to_types_content_id(&stored_id);
            entry.updated_at = timestamp;
        }
        
        // Add to version history
        self.add_to_version_history(version)?;
        
        Ok(new_version)
    }
    
    async fn add_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()> {
        // Check if resource exists
        if !self.has_resource(resource_id).await? {
            return Err(ResourceStorageError::NotFound(
                format!("Resource not found: {}", resource_id)
            ));
        }
        
        // Add tag to resource
        {
            let mut resource_index = self.resource_index.write().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            if let Some(entry) = resource_index.get_mut(resource_id) {
                entry.tags.insert(tag.to_string());
            }
        }
        
        // Add to tag index
        self.index_by_tag(resource_id, tag)?;
        
        Ok(())
    }
    
    async fn remove_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()> {
        // Check if resource exists
        if !self.has_resource(resource_id).await? {
            return Err(ResourceStorageError::NotFound(
                format!("Resource not found: {}", resource_id)
            ));
        }
        
        // Remove tag from resource
        {
            let mut resource_index = self.resource_index.write().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            if let Some(entry) = resource_index.get_mut(resource_id) {
                entry.tags.remove(tag);
            }
        }
        
        // Remove from tag index
        self.remove_from_tag_index(resource_id, tag)?;
        
        Ok(())
    }
    
    async fn find_resources_by_type(
        &self, 
        resource_type: &ResourceTypeId
    ) -> ResourceStorageResult<Vec<ResourceId>> {
        let type_index = self.type_index.read().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire type index lock: {}", e))
        )?;
        
        Ok(type_index.get(resource_type)
            .map(|resources| resources.iter().cloned().collect())
            .unwrap_or_default())
    }
    
    async fn find_resources_by_tag(
        &self, 
        tag: &str
    ) -> ResourceStorageResult<Vec<ResourceId>> {
        let tag_index = self.tag_index.read().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire tag index lock: {}", e))
        )?;
        
        Ok(tag_index.get(tag)
            .map(|resources| resources.iter().cloned().collect())
            .unwrap_or_default())
    }
    
    async fn get_version_history(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<Vec<ResourceVersion>> {
        let version_history = self.version_history.read().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire version history lock: {}", e))
        )?;
        
        Ok(version_history.get(resource_id)
            .map(|history| history.clone())
            .unwrap_or_default())
    }
    
    async fn get_resource_metadata(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<HashMap<String, String>> {
        // Get the latest version from history
        let version_history = self.version_history.read().map_err(|e| 
            ResourceStorageError::InternalError(format!("Failed to acquire version history lock: {}", e))
        )?;
        
        let history = version_history.get(resource_id)
            .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?;
            
        let latest = history.iter()
            .max_by_key(|v| v.version)
            .ok_or_else(|| ResourceStorageError::InternalError(
                format!("No versions found for resource: {}", resource_id)
            ))?;
            
        Ok(latest.metadata.clone())
    }
}

/// In-memory implementation of resource storage for testing purposes
#[derive(Debug)]
pub struct InMemoryResourceStorage {
    /// Underlying storage implementation
    storage: ContentAddressedResourceStorage,
}

impl InMemoryResourceStorage {
    /// Create a new in-memory resource storage
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            storage: ContentAddressedResourceStorage::new(storage),
        }
    }
}

#[async_trait]
impl ResourceStorage for InMemoryResourceStorage {
    async fn store_resource<T: ContentAddressed + Send + Sync + serde::Serialize>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId> {
        self.storage.store_resource(resource, resource_type, metadata).await
    }
    
    async fn get_resource<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T> {
        self.storage.get_resource(resource_id).await
    }
    
    async fn get_resource_version<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T> {
        self.storage.get_resource_version(resource_id, version).await
    }
    
    async fn has_resource(&self, resource_id: &ResourceId) -> ResourceStorageResult<bool> {
        self.storage.has_resource(resource_id).await
    }
    
    async fn update_resource<T: ContentAddressed + Send + Sync + Serialize>(
        &self, 
        resource_id: &ResourceId, 
        resource: T,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<u64> {
        self.storage.update_resource(resource_id, resource, metadata).await
    }
    
    async fn add_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()> {
        self.storage.add_tag(resource_id, tag).await
    }
    
    async fn remove_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()> {
        self.storage.remove_tag(resource_id, tag).await
    }
    
    async fn find_resources_by_type(
        &self, 
        resource_type: &ResourceTypeId
    ) -> ResourceStorageResult<Vec<ResourceId>> {
        self.storage.find_resources_by_type(resource_type).await
    }
    
    async fn find_resources_by_tag(
        &self, 
        tag: &str
    ) -> ResourceStorageResult<Vec<ResourceId>> {
        self.storage.find_resources_by_tag(tag).await
    }
    
    async fn get_version_history(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<Vec<ResourceVersion>> {
        self.storage.get_version_history(resource_id).await
    }
    
    async fn get_resource_metadata(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<HashMap<String, String>> {
        self.storage.get_resource_metadata(resource_id).await
    }
}

/// Configuration options for resource storage
#[derive(Debug, Clone)]
pub struct ResourceStorageConfig {
    /// Enable versioning for resources
    pub enable_versioning: bool,
    
    /// Enable caching for resources
    pub enable_caching: bool,
    
    /// Maximum versions to keep per resource
    pub max_versions_per_resource: Option<usize>,
    
    /// Cache size (in number of resources)
    pub cache_size: Option<usize>,
}

impl Default for ResourceStorageConfig {
    fn default() -> Self {
        Self {
            enable_versioning: true,
            enable_caching: true,
            max_versions_per_resource: Some(10),
            cache_size: Some(1000),
        }
    }
}

/// Create a resource storage implementation based on configuration
pub fn create_resource_storage(
    storage: Arc<dyn ContentAddressedStorage>,
    config: ResourceStorageConfig,
) -> ResourceStorageEnum {
    // For now, just create the basic implementation
    // In the future, this could be extended to create different types
    // of storage based on the configuration
    ResourceStorageEnum::ContentAddressed(ContentAddressedResourceStorage::new(storage))
}

/// Enum to hold different storage implementations
#[derive(Debug)]
pub enum ResourceStorageEnum {
    /// Content addressed storage implementation
    ContentAddressed(ContentAddressedResourceStorage),
    /// In-memory storage implementation
    InMemory(InMemoryResourceStorage),
}

#[async_trait]
impl ResourceStorage for ResourceStorageEnum {
    async fn store_resource<T: ContentAddressed + Send + Sync + serde::Serialize>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId> {
        match self {
            Self::ContentAddressed(storage) => storage.store_resource(resource, resource_type, metadata).await,
            Self::InMemory(storage) => storage.store_resource(resource, resource_type, metadata).await,
        }
    }
    
    async fn get_resource<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T> {
        match self {
            Self::ContentAddressed(storage) => storage.get_resource(resource_id).await,
            Self::InMemory(storage) => storage.get_resource(resource_id).await,
        }
    }
    
    async fn get_resource_version<T: ContentAddressed + Send + Sync + DeserializeOwned>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T> {
        match self {
            Self::ContentAddressed(storage) => storage.get_resource_version(resource_id, version).await,
            Self::InMemory(storage) => storage.get_resource_version(resource_id, version).await,
        }
    }
    
    async fn has_resource(&self, resource_id: &ResourceId) -> ResourceStorageResult<bool> {
        match self {
            Self::ContentAddressed(storage) => storage.has_resource(resource_id).await,
            Self::InMemory(storage) => storage.has_resource(resource_id).await,
        }
    }
    
    async fn update_resource<T: ContentAddressed + Send + Sync + Serialize>(
        &self, 
        resource_id: &ResourceId, 
        resource: T,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<u64> {
        match self {
            Self::ContentAddressed(storage) => storage.update_resource(resource_id, resource, metadata).await,
            Self::InMemory(storage) => storage.update_resource(resource_id, resource, metadata).await,
        }
    }
    
    async fn add_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()> {
        match self {
            Self::ContentAddressed(storage) => storage.add_tag(resource_id, tag).await,
            Self::InMemory(storage) => storage.add_tag(resource_id, tag).await,
        }
    }
    
    async fn remove_tag(
        &self, 
        resource_id: &ResourceId, 
        tag: &str
    ) -> ResourceStorageResult<()> {
        match self {
            Self::ContentAddressed(storage) => storage.remove_tag(resource_id, tag).await,
            Self::InMemory(storage) => storage.remove_tag(resource_id, tag).await,
        }
    }
    
    async fn find_resources_by_type(
        &self, 
        resource_type: &ResourceTypeId
    ) -> ResourceStorageResult<Vec<ResourceId>> {
        match self {
            Self::ContentAddressed(storage) => storage.find_resources_by_type(resource_type).await,
            Self::InMemory(storage) => storage.find_resources_by_type(resource_type).await,
        }
    }
    
    async fn find_resources_by_tag(
        &self, 
        tag: &str
    ) -> ResourceStorageResult<Vec<ResourceId>> {
        match self {
            Self::ContentAddressed(storage) => storage.find_resources_by_tag(tag).await,
            Self::InMemory(storage) => storage.find_resources_by_tag(tag).await,
        }
    }
    
    async fn get_version_history(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<Vec<ResourceVersion>> {
        match self {
            Self::ContentAddressed(storage) => storage.get_version_history(resource_id).await,
            Self::InMemory(storage) => storage.get_version_history(resource_id).await,
        }
    }
    
    async fn get_resource_metadata(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<HashMap<String, String>> {
        match self {
            Self::ContentAddressed(storage) => storage.get_resource_metadata(resource_id).await,
            Self::InMemory(storage) => storage.get_resource_metadata(resource_id).await,
        }
    }
}

/// In-memory implementation of content-addressed storage
#[derive(Debug)]
pub struct InMemoryContentAddressedStorage {
    objects: RwLock<HashMap<ContentId, Vec<u8>>>,
}

impl InMemoryContentAddressedStorage {
    /// Create a new empty in-memory storage
    pub fn new() -> Self {
        Self {
            objects: RwLock::new(HashMap::new()),
        }
    }
    
    /// Store binary data and return content ID
    pub fn store_bytes(&self, bytes: &[u8]) -> ResourceStorageResult<ContentId> {
        // Create a content hash from the bytes
        let hash = hash_bytes(bytes);
        let content_id = content_hash_to_id(&hash);
        
        // Store the bytes with the content ID as the key
        let mut objects = self.objects.write().unwrap();
        
        // Skip if already exists
        if objects.contains_key(&content_id) {
            return Ok(content_id);
        }
        
        // Store the bytes
        objects.insert(content_id.clone(), bytes.to_vec());
        
        Ok(content_id)
    }
    
    /// Store an object in the content-addressed storage
    pub fn store_object<T: ContentAddressed + Send + Sync + serde::Serialize>(&self, object: &T) -> ResourceStorageResult<ContentId> {
        // Serialize the object
        let bytes = match object.to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => return Err(ResourceStorageError::SerializationError(err.to_string())),
        };
        
        // Store the bytes
        self.store_bytes(&bytes)
    }
    
    /// Check if an object exists in storage
    pub fn contains(&self, id: &ContentId) -> bool {
        let objects = self.objects.read().unwrap();
        objects.contains_key(id)
    }
    
    /// Retrieve binary data for an object
    pub fn get_bytes(&self, id: &ContentId) -> ResourceStorageResult<Vec<u8>> {
        let objects = self.objects.read().unwrap();
        
        objects.get(id)
            .cloned()
            .ok_or_else(|| ResourceStorageError::NotFound(
                format!("Object not found: {}", id)
            ))
    }
    
    /// Retrieve an object from storage by its content ID
    pub fn get_object<T: ContentAddressed + Send + Sync + DeserializeOwned>(&self, id: &ContentId) -> ResourceStorageResult<T> {
        let bytes = self.get_bytes(id)?;
        match T::from_bytes(&bytes) {
            Ok(obj) => Ok(obj),
            Err(err) => Err(ResourceStorageError::SerializationError(err.to_string())),
        }
    }
    
    /// Remove an object from storage
    pub fn remove(&self, id: &ContentId) -> ResourceStorageResult<()> {
        let mut objects = self.objects.write().unwrap();
        
        if objects.remove(id).is_none() {
            return Err(ResourceStorageError::NotFound(
                format!("Object not found: {}", id)
            ));
        }
        
        Ok(())
    }
    
    /// Clear all objects from storage
    pub fn clear(&self) {
        let mut objects = self.objects.write().unwrap();
        objects.clear();
    }
    
    /// Get the number of objects in storage
    pub fn len(&self) -> usize {
        let objects = self.objects.read().unwrap();
        objects.len()
    }
    
    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Implement ContentAddressedStorage from causality-types
impl causality_types::content_addressing::storage::ContentAddressedStorage for InMemoryContentAddressedStorage {
    fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, causality_types::content_addressing::storage::StorageError> {
        match self.store_bytes(bytes) {
            Ok(id) => Ok(id),
            Err(err) => Err(causality_types::content_addressing::storage::StorageError::IoError(err.to_string())),
        }
    }
    
    fn contains(&self, id: &ContentId) -> bool {
        self.contains(id)
    }
    
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, causality_types::content_addressing::storage::StorageError> {
        match self.get_bytes(id) {
            Ok(bytes) => Ok(bytes),
            Err(err) => match err {
                ResourceStorageError::NotFound(msg) => 
                    Err(causality_types::content_addressing::storage::StorageError::NotFound(msg)),
                _ => Err(causality_types::content_addressing::storage::StorageError::IoError(err.to_string())),
            }
        }
    }
    
    fn remove(&self, id: &ContentId) -> Result<(), causality_types::content_addressing::storage::StorageError> {
        match self.remove(id) {
            Ok(()) => Ok(()),
            Err(err) => match err {
                ResourceStorageError::NotFound(msg) => 
                    Err(causality_types::content_addressing::storage::StorageError::NotFound(msg)),
                _ => Err(causality_types::content_addressing::storage::StorageError::IoError(err.to_string())),
            }
        }
    }
    
    fn clear(&self) {
        self.clear()
    }
    
    fn len(&self) -> usize {
        self.len()
    }
}

/// Create a ContentHash from bytes
fn hash_bytes(bytes: &[u8]) -> ContentHash {
    // Hash the bytes using blake3
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes().to_vec();
    
    // Create a properly formatted ContentHash using the string value of the algorithm
    ContentHash::new("blake3", hash_bytes)
}

// Re-add the impl block and implement missing methods with errors
impl ContentAddressed for Box<dyn Resource> {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        // Ideally, Box<dyn Resource> should not need its own content_hash.
        // The underlying resource should provide it.
        // Returning an error or a default hash might be options.
        // For now, let's assume there's a method to get the underlying hash
        // This might need refinement based on how Box<dyn Resource> is used.
        Err(HashError::SerializationError("Cannot calculate content hash for Box<dyn Resource> directly".to_string()))
        // Or perhaps delegate if the Resource trait had content_hash?
        // (**self).content_hash() 
    }

    fn content_id(&self) -> Result<ContentId, HashError> {
        let hash = self.content_hash()?;
        Ok(ContentId::from(hash.clone()))
    }

    // Implement missing methods
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        Err(HashError::SerializationError("Cannot serialize Box<dyn Resource>".to_string()))
    }

    fn from_bytes(_bytes: &[u8]) -> Result<Self, HashError> where Self: Sized {
        Err(HashError::SerializationError("Cannot deserialize Box<dyn Resource>".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::content_addressing::storage::InMemoryStorage;
    use causality_types::crypto_primitives::{HashOutput, HashAlgorithm, HashError};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestResource {
        pub name: String,
        pub value: i32,
    }
    
    impl ContentAddressed for TestResource {
        fn content_hash(&self) -> Result<HashOutput, HashError> {
            // Create a simple content hash for testing
            let data = format!("{}:{}", self.name, self.value);
            let hash = blake3::hash(data.as_bytes());
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(hash.as_bytes());
            Ok(HashOutput::new(hash_bytes, HashAlgorithm::Blake3))
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
    
    #[tokio::test]
    async fn test_resource_storage_basic_operations() {
        // Create in-memory storage
        let cas_storage = Arc::new(InMemoryStorage::new());
        let storage = InMemoryResourceStorage::new(cas_storage);
        
        // Create a test resource
        let resource = TestResource {
            name: "test".to_string(),
            value: 42,
        };
        
        let resource_type = ResourceTypeId::new("test_type");
        
        // Store the resource
        let resource_id = storage.store_resource(
            resource.clone(),
            resource_type.clone(),
            None
        ).await.unwrap();
        
        // Check if resource exists
        assert!(storage.has_resource(&resource_id).await.unwrap());
        
        // Get the resource
        let retrieved: TestResource = storage.get_resource(&resource_id).await.unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.value, 42);
        
        // Update the resource
        let updated = TestResource {
            name: "test".to_string(),
            value: 84,
        };
        
        let new_version = storage.update_resource(
            &resource_id,
            updated.clone(),
            None
        ).await.unwrap();
        
        assert_eq!(new_version, 2);
        
        // Get the updated resource
        let retrieved: TestResource = storage.get_resource(&resource_id).await.unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.value, 84);
        
        // Get a specific version
        let original: TestResource = storage.get_resource_version(&resource_id, 1).await.unwrap();
        assert_eq!(original.name, "test");
        assert_eq!(original.value, 42);
        
        // Add tags
        storage.add_tag(&resource_id, "important").await.unwrap();
        storage.add_tag(&resource_id, "test_tag").await.unwrap();
        
        // Find by tag
        let resources = storage.find_resources_by_tag("important").await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0], resource_id);
        
        // Find by type
        let resources = storage.find_resources_by_type(&resource_type).await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0], resource_id);
        
        // Remove tag
        storage.remove_tag(&resource_id, "important").await.unwrap();
        
        // Should no longer be found by removed tag
        let resources = storage.find_resources_by_tag("important").await.unwrap();
        assert_eq!(resources.len(), 0);
        
        // Should still be found by other tag
        let resources = storage.find_resources_by_tag("test_tag").await.unwrap();
        assert_eq!(resources.len(), 1);
        
        // Get version history
        let history = storage.get_version_history(&resource_id).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[1].version, 2);
    }
} 