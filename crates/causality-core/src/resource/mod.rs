// Resource Management Module
//
// This module provides a unified resource management system with content-addressed
// storage and a consistent resource model.

use std::sync::Arc;

// Internal module structure
pub mod types;
pub mod storage;
mod state;

// Re-export key types and traits from types module
pub use types::{
    ResourceTypeId, 
    ResourceId,
    ResourceType,
    ResourceTag,
    ResourceState,
    ResourceTypeRegistry,
    ResourceTypeRegistryError,
    ResourceTypeRegistryResult,
    ResourceTypeDefinition
};

// Re-export key types from state module
pub use state::{
    ResourceStateData, 
    ResourceStateStore, 
    StateStoreProvider
};

// Re-export storage types
pub use storage::types::{
    ResourceStateStorage,
    ResourceStorageError
};

// Import ContentAddressedStorage from causality_types
use causality_types::ContentAddressedStorage;