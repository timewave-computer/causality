// Resource Storage Implementation
//
// This module provides interfaces and implementations for storing resources
// using content addressing principles. It includes support for versioning,
// indexing, and efficient retrieval.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::content::{ContentId, ContentHash, ContentAddressed, ContentAddressingError};
use crate::resource::{ResourceId, ResourceTypeId, ResourceSchema};
use crate::storage::ContentAddressedStorage;
use crate::serialization::{SerializationError, to_bytes, from_bytes};

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
    pub content_hash: ContentHash,
    
    /// When this version was created (timestamp)
    pub created_at: u64,
    
    /// Reference to previous version (if any)
    pub previous_version: Option<ContentHash>,
    
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
    pub current_hash: ContentHash,
    
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
    async fn store_resource<T: ContentAddressed + Send + Sync>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId>;
    
    /// Get a resource by ID (latest version)
    async fn get_resource<T: ContentAddressed + Send + Sync>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T>;
    
    /// Get a specific version of a resource
    async fn get_resource_version<T: ContentAddressed + Send + Sync>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T>;
    
    /// Check if a resource exists
    async fn has_resource(&self, resource_id: &ResourceId) -> ResourceStorageResult<bool>;
    
    /// Update a resource
    async fn update_resource<T: ContentAddressed + Send + Sync>(
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
    /// Create a new content-addressed resource storage
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
}

#[async_trait]
impl ResourceStorage for ContentAddressedResourceStorage {
    async fn store_resource<T: ContentAddressed + Send + Sync>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId> {
        // Generate content hash for the resource
        let content_hash = resource.content_hash().map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Create resource ID from content hash
        let resource_id = ResourceId::from(content_hash.clone());
        
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
        self.storage.store(&resource_bytes, &content_hash).await.map_err(|e| 
            ResourceStorageError::StorageError(e.to_string())
        )?;
        
        // Create metadata for this version
        let timestamp = self.current_timestamp();
        let version = ResourceVersion {
            resource_id: resource_id.clone(),
            version: 1, // First version
            content_hash: content_hash.clone(),
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
            current_hash: content_hash,
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
    
    async fn get_resource<T: ContentAddressed + Send + Sync>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T> {
        // Get the current version from the index
        let content_hash = {
            let resource_index = self.resource_index.read().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            resource_index.get(resource_id)
                .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?
                .current_hash
                .clone()
        };
        
        // Retrieve from content-addressed storage
        let resource_bytes = self.storage.get(&content_hash).await.map_err(|e| 
            match e {
                crate::content::ContentAddressingError::NotFound(_) => 
                    ResourceStorageError::NotFound(format!("Resource data not found: {}", resource_id)),
                _ => ResourceStorageError::StorageError(e.to_string()),
            }
        )?;
        
        // Deserialize the resource
        from_bytes::<T>(&resource_bytes).map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )
    }
    
    async fn get_resource_version<T: ContentAddressed + Send + Sync>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T> {
        // Get the version history
        let content_hash = {
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
        
        // Retrieve from content-addressed storage
        let resource_bytes = self.storage.get(&content_hash).await.map_err(|e| 
            match e {
                crate::content::ContentAddressingError::NotFound(_) => 
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
    
    async fn update_resource<T: ContentAddressed + Send + Sync>(
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
        let (current_version, resource_type, previous_hash) = {
            let resource_index = self.resource_index.read().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            let entry = resource_index.get(resource_id)
                .ok_or_else(|| ResourceStorageError::NotFound(format!("Resource not found: {}", resource_id)))?;
                
            (entry.current_version, entry.resource_type.clone(), entry.current_hash.clone())
        };
        
        // Generate content hash for the new version
        let content_hash = resource.content_hash().map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Skip update if content hasn't changed
        if content_hash == previous_hash {
            return Ok(current_version);
        }
        
        // Serialize the resource
        let resource_bytes = to_bytes(&resource).map_err(|e| 
            ResourceStorageError::SerializationError(e.to_string())
        )?;
        
        // Store in content-addressed storage
        self.storage.store(&resource_bytes, &content_hash).await.map_err(|e| 
            ResourceStorageError::StorageError(e.to_string())
        )?;
        
        // Increment version
        let new_version = current_version + 1;
        let timestamp = self.current_timestamp();
        
        // Create metadata for this version
        let version = ResourceVersion {
            resource_id: resource_id.clone(),
            version: new_version,
            content_hash: content_hash.clone(),
            created_at: timestamp,
            previous_version: Some(previous_hash),
            resource_type: resource_type.clone(),
            metadata: metadata.clone().unwrap_or_default(),
        };
        
        // Update index entry
        {
            let mut resource_index = self.resource_index.write().map_err(|e| 
                ResourceStorageError::InternalError(format!("Failed to acquire resource index lock: {}", e))
            )?;
            
            if let Some(entry) = resource_index.get_mut(resource_id) {
                entry.current_version = new_version;
                entry.current_hash = content_hash;
                entry.updated_at = timestamp;
            }
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
    async fn store_resource<T: ContentAddressed + Send + Sync>(
        &self, 
        resource: T,
        resource_type: ResourceTypeId,
        metadata: Option<HashMap<String, String>>
    ) -> ResourceStorageResult<ResourceId> {
        self.storage.store_resource(resource, resource_type, metadata).await
    }
    
    async fn get_resource<T: ContentAddressed + Send + Sync>(
        &self, 
        resource_id: &ResourceId
    ) -> ResourceStorageResult<T> {
        self.storage.get_resource(resource_id).await
    }
    
    async fn get_resource_version<T: ContentAddressed + Send + Sync>(
        &self, 
        resource_id: &ResourceId, 
        version: u64
    ) -> ResourceStorageResult<T> {
        self.storage.get_resource_version(resource_id, version).await
    }
    
    async fn has_resource(&self, resource_id: &ResourceId) -> ResourceStorageResult<bool> {
        self.storage.has_resource(resource_id).await
    }
    
    async fn update_resource<T: ContentAddressed + Send + Sync>(
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
) -> Arc<dyn ResourceStorage> {
    // For now, just create the basic implementation
    // In the future, this could be extended to create different types
    // of storage based on the configuration
    Arc::new(ContentAddressedResourceStorage::new(storage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryContentAddressedStorage;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestResource {
        pub name: String,
        pub value: i32,
    }
    
    impl ContentAddressed for TestResource {
        fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
            // Create a simple content hash for testing
            let data = format!("{}:{}", self.name, self.value);
            let hash = blake3::hash(data.as_bytes());
            Ok(ContentHash::new(hash.as_bytes().to_vec()))
        }
    }
    
    #[tokio::test]
    async fn test_resource_storage_basic_operations() {
        // Create in-memory storage
        let cas_storage = Arc::new(InMemoryContentAddressedStorage::new());
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