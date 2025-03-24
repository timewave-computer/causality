use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::address::Address;
use crate::resource::{CapabilityId, ContentId};
use crate::error::{Error, Result};

/// Type alias for capability references
pub type CapabilityRef = String;

/// Capability-related errors
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("Capability not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient rights: {0}")]
    InsufficientRights(String),
    
    #[error("Capability expired")]
    Expired,
    
    #[error("Capability revoked")]
    Revoked,
}

/// Capability rights
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Right {
    /// Read resource data
    Read,
    /// Write resource data
    Write,
    /// Delete a resource
    Delete,
    /// Control resource access
    Control,
    /// Delegate capabilities
    Delegate,
    /// Administrative operations
    Admin,
    /// Custom right
    Custom(String),
}

/// Errors that can occur during resource API operations
#[derive(Debug, Error)]
pub enum ResourceApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Capability error: {0}")]
    CapabilityError(#[from] CapabilityError),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Resource locked: {0}")]
    ResourceLocked(String),
    
    #[error("Resource conflict: {0}")]
    ResourceConflict(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<ResourceApiError> for Error {
    fn from(err: ResourceApiError) -> Self {
        match err {
            ResourceApiError::NotFound(s) => Error::ResourceNotFound(s.into()),
            ResourceApiError::AccessDenied(s) => Error::PermissionDenied(s),
            ResourceApiError::CapabilityError(e) => Error::PermissionDenied(e.to_string()),
            ResourceApiError::InvalidOperation(s) => Error::InvalidOperation(s),
            ResourceApiError::SerializationError(s) => Error::SerializationError(s),
            ResourceApiError::ValidationError(s) => Error::ValidationError(s),
            ResourceApiError::ResourceLocked(s) => Error::ResourceAlreadyLocked(s.into()),
            ResourceApiError::ResourceConflict(s) => Error::AlreadyExists(s),
            ResourceApiError::StorageError(s) => Error::StorageError(s),
            ResourceApiError::InternalError(s) => Error::InternalError(s),
        }
    }
}

impl From<Error> for ResourceApiError {
    fn from(err: Error) -> Self {
        match err {
            Error::ResourceNotFound(id) => ResourceApiError::NotFound(id.to_string()),
            Error::PermissionDenied(s) => ResourceApiError::AccessDenied(s),
            Error::InvalidOperation(s) => ResourceApiError::InvalidOperation(s),
            Error::SerializationError(s) => ResourceApiError::SerializationError(s),
            Error::ValidationError(s) => ResourceApiError::ValidationError(s),
            Error::ResourceAlreadyLocked(id) => ResourceApiError::ResourceLocked(id.to_string()),
            Error::AlreadyExists(s) => ResourceApiError::ResourceConflict(s),
            Error::StorageError(s) => ResourceApiError::StorageError(s),
            Error::InternalError(s) => ResourceApiError::InternalError(s),
            _ => ResourceApiError::InternalError(err.to_string()),
        }
    }
}

/// Result type for resource API operations
pub type ResourceApiResult<T> = std::result::Result<T, ResourceApiError>;

/// Resource metadata
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// Resource type
    pub resource_type: String,
    
    /// Resource owner
    pub owner: Address,
    
    /// Resource creation time
    pub created_at: u64,
    
    /// Last updated time
    pub updated_at: u64,
    
    /// Domain associated with the resource
    pub domain: Option<String>,
    
    /// Content type
    pub content_type: Option<String>,
    
    /// Size in bytes
    pub size: Option<u64>,
    
    /// Additional metadata fields
    pub custom: HashMap<String, String>,
}

/// Resource state
#[derive(Debug, Clone)]
pub enum ResourceState {
    /// Resource is active and available
    Active,
    
    /// Resource is locked for exclusive access
    Locked(CapabilityId),
    
    /// Resource has been archived
    Archived,
    
    /// Resource has been deleted but not purged
    Deleted,
    
    /// Resource is being transferred between domains
    InTransfer,
    
    /// Resource is not in a stable state (e.g., during creation)
    Provisional,
}

/// Trait for read-only operations on a resource
#[async_trait]
pub trait ResourceReader {
    /// Get the resource ID
    fn id(&self) -> &ContentId;
    
    /// Get the resource type
    fn resource_type(&self) -> &str;
    
    /// Get the metadata
    async fn metadata(&self, capability: &CapabilityRef) -> ResourceApiResult<ResourceMetadata>;
    
    /// Get the resource state
    async fn state(&self, capability: &CapabilityRef) -> ResourceApiResult<ResourceState>;
    
    /// Get the raw data of the resource
    async fn data(&self, capability: &CapabilityRef) -> ResourceApiResult<Vec<u8>>;
    
    /// Get a structured view of the resource
    async fn structured_data<T: serde::de::DeserializeOwned>(
        &self, 
        capability: &CapabilityRef
    ) -> ResourceApiResult<T> {
        let data = self.data(capability).await?;
        serde_json::from_slice(&data)
            .map_err(|e| ResourceApiError::SerializationError(e.to_string()))
    }
}

/// Trait for write operations on a resource
#[async_trait]
pub trait ResourceWriter: ResourceReader {
    /// Update the resource data
    async fn update_data(
        &mut self,
        capability: &CapabilityRef,
        data: Vec<u8>,
    ) -> ResourceApiResult<()>;
    
    /// Update with structured data
    async fn update_structured_data<T: serde::Serialize>(
        &mut self,
        capability: &CapabilityRef,
        data: &T,
    ) -> ResourceApiResult<()> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| ResourceApiError::SerializationError(e.to_string()))?;
        self.update_data(capability, serialized).await
    }
    
    /// Update resource metadata
    async fn update_metadata(
        &mut self,
        capability: &CapabilityRef,
        updates: HashMap<String, String>,
    ) -> ResourceApiResult<()>;
    
    /// Lock the resource for exclusive access
    async fn lock(
        &mut self,
        capability: &CapabilityRef,
    ) -> ResourceApiResult<CapabilityRef>;
    
    /// Unlock the resource
    async fn unlock(
        &mut self,
        capability: &CapabilityRef,
    ) -> ResourceApiResult<()>;
    
    /// Delete the resource
    async fn delete(
        &mut self,
        capability: &CapabilityRef,
    ) -> ResourceApiResult<()>;
}

/// Query options for resource lookups
#[derive(Debug, Default, Clone)]
pub struct ResourceQuery {
    /// Filter by resource type
    pub resource_type: Option<String>,
    
    /// Filter by owner
    pub owner: Option<Address>,
    
    /// Filter by domain
    pub domain: Option<String>,
    
    /// Filter by metadata keys and values
    pub metadata: HashMap<String, String>,
    
    /// Max results to return
    pub limit: Option<usize>,
    
    /// Pagination offset
    pub offset: Option<usize>,
    
    /// Sort field
    pub sort_by: Option<String>,
    
    /// Sort direction: true for ascending, false for descending
    pub ascending: bool,
}

/// Resource update options
#[derive(Debug, Clone)]
pub struct ResourceUpdateOptions {
    /// Update the resource type
    pub resource_type: Option<String>,
    
    /// Update the owner
    pub owner: Option<Address>,
    
    /// Update the domain
    pub domain: Option<String>,
    
    /// Metadata to update
    pub metadata: HashMap<String, String>,
    
    /// Whether to override existing metadata (true) or merge (false)
    pub override_metadata: bool,
}

/// A fully-featured resource API that provides access to resources through capabilities
#[async_trait]
pub trait ResourceAPI: Send + Sync {
    /// Create a new resource
    async fn create_resource(
        &self,
        capability: &CapabilityRef,
        resource_type: &str,
        owner: &Address,
        data: Vec<u8>,
        metadata: Option<HashMap<String, String>>,
    ) -> ResourceApiResult<(ContentId, CapabilityRef)>;
    
    /// Create a resource with structured data
    async fn create_structured_resource<T: serde::Serialize + Send + Sync>(
        &self,
        capability: &CapabilityRef,
        resource_type: &str,
        owner: &Address,
        data: &T,
        metadata: Option<HashMap<String, String>>,
    ) -> ResourceApiResult<(ContentId, CapabilityRef)> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| ResourceApiError::SerializationError(e.to_string()))?;
        self.create_resource(capability, resource_type, owner, serialized, metadata).await
    }
    
    /// Get a resource by ID
    async fn get_resource(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
    ) -> ResourceApiResult<Box<dyn ResourceReader + Send + Sync>>;
    
    /// Get a mutable resource by ID
    async fn get_resource_mut(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
    ) -> ResourceApiResult<Box<dyn ResourceWriter + Send + Sync>>;
    
    /// Find resources based on a query
    async fn find_resources(
        &self,
        capability: &CapabilityRef,
        query: ResourceQuery,
    ) -> ResourceApiResult<Vec<Box<dyn ResourceReader + Send + Sync>>>;
    
    /// Update a resource
    async fn update_resource(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
        data: Option<Vec<u8>>,
        options: Option<ResourceUpdateOptions>,
    ) -> ResourceApiResult<()>;
    
    /// Delete a resource
    async fn delete_resource(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
    ) -> ResourceApiResult<()>;
    
    /// Check if a resource exists
    async fn resource_exists(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
    ) -> ResourceApiResult<bool>;
    
    /// Create a capability for a resource
    async fn create_capability(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
        rights: Vec<Right>,
        holder: &Address,
    ) -> ResourceApiResult<CapabilityRef>;
    
    /// Get capabilities for a resource
    async fn get_capabilities(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
    ) -> ResourceApiResult<Vec<CapabilityRef>>;
    
    /// Revoke a capability
    async fn revoke_capability(
        &self,
        capability: &CapabilityRef,
        capability_to_revoke: &CapabilityId,
    ) -> ResourceApiResult<()>;
    
    /// Delegate a capability
    async fn delegate_capability(
        &self,
        capability: &CapabilityRef,
        resource_id: &ContentId,
        rights: Vec<Right>,
        new_holder: &Address,
    ) -> ResourceApiResult<CapabilityRef>;
    
    /// Compose capabilities
    async fn compose_capabilities(
        &self,
        capabilities: &[CapabilityRef],
        new_holder: &Address,
    ) -> ResourceApiResult<CapabilityRef>;
}

/// A memory-backed implementation of ResourceWriter
pub struct MemoryResourceWriter {
    id: ContentId,
    resource_type: String,
    owner: Address,
    data: Vec<u8>,
    metadata: ResourceMetadata,
    state: ResourceState,
}

#[async_trait]
impl ResourceReader for MemoryResourceWriter {
    fn id(&self) -> &ContentId {
        &self.id
    }
    
    fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    async fn metadata(&self, capability: &CapabilityRef) -> ResourceApiResult<ResourceMetadata> {
        // Check if the capability has read rights
        if !capability.capability().has_right(&Right::Read) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have read rights".into()
            ));
        }
        
        Ok(self.metadata.clone())
    }
    
    async fn state(&self, capability: &CapabilityRef) -> ResourceApiResult<ResourceState> {
        // Check if the capability has read rights
        if !capability.capability().has_right(&Right::Read) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have read rights".into()
            ));
        }
        
        Ok(self.state.clone())
    }
    
    async fn data(&self, capability: &CapabilityRef) -> ResourceApiResult<Vec<u8>> {
        // Check if the capability has read rights
        if !capability.capability().has_right(&Right::Read) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have read rights".into()
            ));
        }
        
        // Check if resource is accessible
        match &self.state {
            ResourceState::Active => {}
            ResourceState::Locked(locker_id) => {
                if capability.id() != locker_id {
                    return Err(ResourceApiError::ResourceLocked(
                        "Resource is locked by another capability".into()
                    ));
                }
            }
            _ => {
                return Err(ResourceApiError::InvalidOperation(
                    format!("Resource is not accessible in state: {:?}", self.state)
                ));
            }
        }
        
        Ok(self.data.clone())
    }
}

#[async_trait]
impl ResourceWriter for MemoryResourceWriter {
    async fn update_data(
        &mut self,
        capability: &CapabilityRef,
        data: Vec<u8>,
    ) -> ResourceApiResult<()> {
        // Check if the capability has write rights
        if !capability.capability().has_right(&Right::Write) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have write rights".into()
            ));
        }
        
        // Check if resource is accessible
        match &self.state {
            ResourceState::Active => {}
            ResourceState::Locked(locker_id) => {
                if capability.id() != locker_id {
                    return Err(ResourceApiError::ResourceLocked(
                        "Resource is locked by another capability".into()
                    ));
                }
            }
            _ => {
                return Err(ResourceApiError::InvalidOperation(
                    format!("Resource is not accessible in state: {:?}", self.state)
                ));
            }
        }
        
        // Update the data
        self.data = data;
        
        // Update the metadata
        self.metadata.updated_at = unix_timestamp_now();
        if let Some(size) = Some(self.data.len() as u64) {
            self.metadata.size = size;
        }
        
        Ok(())
    }
    
    async fn update_metadata(
        &mut self,
        capability: &CapabilityRef,
        updates: HashMap<String, String>,
    ) -> ResourceApiResult<()> {
        // Check if the capability has write rights
        let cap = capability.capability();
        if !cap.has_right(&Right::Write) && !cap.has_right(&Right::Custom("UpdateMetadata".into())) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have write or update metadata rights".into()
            ));
        }
        
        // Update the metadata
        for (key, value) in updates {
            self.metadata.custom.insert(key, value);
        }
        
        self.metadata.updated_at = unix_timestamp_now();
        
        Ok(())
    }
    
    async fn lock(
        &mut self,
        capability: &CapabilityRef,
    ) -> ResourceApiResult<CapabilityRef> {
        // Check if the capability has write rights
        if !capability.capability().has_right(&Right::Write) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have write rights".into()
            ));
        }
        
        // Check if resource is already locked
        match &self.state {
            ResourceState::Locked(_) => {
                return Err(ResourceApiError::ResourceLocked(
                    "Resource is already locked".into()
                ));
            }
            ResourceState::Active => {
                // Proceed with locking
            }
            _ => {
                return Err(ResourceApiError::InvalidOperation(
                    format!("Resource cannot be locked in state: {:?}", self.state)
                ));
            }
        }
        
        // Lock the resource with this capability
        self.state = ResourceState::Locked(capability.id().clone());
        
        // Return the same capability
        Ok(capability.clone())
    }
    
    async fn unlock(
        &mut self,
        capability: &CapabilityRef,
    ) -> ResourceApiResult<()> {
        // Check if resource is locked by this capability
        match &self.state {
            ResourceState::Locked(locker_id) => {
                if capability.id() != locker_id {
                    return Err(ResourceApiError::AccessDenied(
                        "Resource is locked by a different capability".into()
                    ));
                }
                
                // Unlock the resource
                self.state = ResourceState::Active;
                Ok(())
            }
            _ => {
                Err(ResourceApiError::InvalidOperation(
                    "Resource is not locked".into()
                ))
            }
        }
    }
    
    async fn delete(
        &mut self,
        capability: &CapabilityRef,
    ) -> ResourceApiResult<()> {
        // Check if the capability has delete rights
        if !capability.capability().has_right(&Right::Delete) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have delete rights".into()
            ));
        }
        
        // Mark the resource as deleted
        self.state = ResourceState::Deleted;
        
        Ok(())
    }
}

/// Helper function to get current Unix timestamp
fn unix_timestamp_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
} 
