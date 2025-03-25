// Resource management system for handling system resources
// Original file: src/resource/mod.rs

// Resource system for Causality
//
// This module provides resource management capabilities, including resource
// registration, access control, and lifecycle management.

use serde::{Serialize, Deserialize};

// Module declarations
pub mod api;
pub mod resource_register;
pub mod capability;
pub mod authorization;
pub mod static_alloc;
pub mod epoch;
pub mod archival;
pub mod memory_api;
pub mod storage_adapter;
pub mod summarization;
pub mod tel;
pub mod lifecycle;
pub mod versioning;
pub mod storage;
pub mod capability_system;
pub mod nullifier;
pub mod capability_api;
pub mod allocator;
pub mod zk_integration;
pub mod usage;
pub mod resource_temporal_consistency;
pub mod manager;
pub mod fact_observer;
pub mod garbage_collection;
pub mod boundary_manager;
pub mod facade;
pub mod relationship_tracker;
pub mod capability_chain;
pub mod request;
pub mod lifecycle_manager;
pub mod content_addressed_register;
pub mod content_addressed_resource;
pub mod migrate_helpers;
pub mod unified_registry;
pub mod migrate_adapter;

// Re-export content-addressed modules
pub use resource_register::{
    ResourceRegister,
    ResourceLogic,
    FungibilityDomain,
    Quantity,
    RegisterState,
    StorageStrategy,
    StateVisibility
};
pub use content_addressed_register::{
    ContentAddressedRegister,
    ContentAddressedRegisterOperation,
    RegisterOperationType as ContentAddressedRegisterOperationType,
    ContentAddressedRegisterRegistry,
};
pub use content_addressed_resource::{Resource, ResourceRegistry};
pub use unified_registry::UnifiedRegistry;
pub use migrate_adapter::{ResourceToRegisterAdapter, RegisterSystemAdapter, MigrationAdapter};

// Re-export crypto modules
pub use causality_crypto::ContentId;

/// Adapter trait for resource registry operations
pub trait ResourceRegistryAdapter {
    /// Get a register by ID
    fn get_register(&self, id: &ContentId) -> causality_types::Result<ResourceRegister>;
    
    /// Create a new register
    fn create_register(&self, register: ResourceRegister) -> causality_types::Result<ContentId>;
    
    /// Update register state
    fn update_state(&self, id: &ContentId, new_state: resource_register::RegisterState) -> causality_types::Result<()>;
    
    /// Delete a register (mark as consumed)
    fn delete_register(&self, id: &ContentId) -> causality_types::Result<()>;
} 