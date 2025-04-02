// Resource Types
//
// This module re-exports essential resource types for compatibility with
// existing code that uses direct imports from crate::resource_types.

// Import the standard library Result
use std::result::Result;

// Re-export key types that are publicly available
// ResourceId and ResourceTypeId are available from the types module
pub use crate::resource::types::{ResourceId, ResourceTypeId};

// Import ResourceType and ResourceState from resource module
pub use crate::resource::{ResourceType, ResourceState, ResourceError};

// Define placeholder for types not publicly available
// Use standard Result with ResourceError
pub type ResourceResult<T> = Result<T, ResourceError>;

// Define placeholder structs for types not available publicly
#[derive(Debug, Clone, Default)]
pub struct ResourceSchema {
    pub name: String,
    pub version: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceMap {
    pub resource_ids: Vec<ResourceId>,
}

#[derive(Debug, Clone)]
pub struct ResourcePermission {
    pub resource_id: ResourceId,
    pub permissions: Vec<String>,
}

// Import ResourceGuard from lib root export
pub use crate::ResourceGuard;

// Re-export content addressing related types
pub use causality_types::{
    ContentId,
    ContentHash,
    ContentAddressed,
};

// Re-export storage types
// The original imports don't exist, so let's use the correct paths
pub use crate::resource::storage::{
    ResourceStorageError as StorageError,
    ResourceStorageResult as StorageResult,
    ResourceStorage as ContentAddressedStorage, 
    // ContentAddressedResourceStorage is used instead of ContentAddressedStorageExt
    ResourceStorage as ContentAddressedStorageExt,
};

// GetResourceOptions doesn't exist, so let's define a placeholder struct
#[derive(Default, Debug, Clone)]
pub struct GetResourceOptions {
    pub include_history: bool,
    pub include_metadata: bool,
}

// For backward compatibility
pub type ContentAddressedStorageError = crate::resource::ResourceError; 