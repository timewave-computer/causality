// Resource Storage Module
//
// This module provides interfaces and implementations for storing resources
// using content addressing principles. It includes support for versioning,
// indexing, and efficient retrieval.

mod storage;

pub use storage::{
    ResourceStorageError, ResourceStorageResult,
    ResourceVersion, ResourceIndexEntry, 
    ResourceStorage, ContentAddressedResourceStorage,
    InMemoryResourceStorage, ResourceStorageConfig, 
    create_resource_storage,
}; 