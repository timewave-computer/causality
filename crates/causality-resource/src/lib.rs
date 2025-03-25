// Resource management system for handling system resources
// Original file: src/resource/mod.rs

// Resource system for Causality
//
// This module provides resource management capabilities, including resource
// registration, access control, and lifecycle management.

use serde::{Serialize, Deserialize};

// Core resource management interfaces - always included
pub mod interface;
pub use interface::*;

// Resource adapters - included with the adapter feature
#[cfg(feature = "adapter")]
pub mod adapter;
#[cfg(feature = "adapter")]
pub use adapter::*;

// Module declarations - most are only included with legacy feature
pub mod api;

#[cfg(feature = "legacy")]
pub mod register;

#[cfg(feature = "legacy")]
pub mod capability;

#[cfg(feature = "legacy")]
pub mod authorization;

#[cfg(feature = "legacy")]
pub mod static_allocator;

#[cfg(feature = "legacy")]
pub mod epoch;

#[cfg(feature = "legacy")]
pub mod archival;

#[cfg(feature = "legacy")]
pub mod memory_api;

#[cfg(feature = "legacy")]
pub mod storage_adapter;

#[cfg(feature = "legacy")]
pub mod summarization;

#[cfg(feature = "legacy")]
pub mod tel_adapter;

#[cfg(feature = "legacy")]
pub mod lifecycle;

#[cfg(feature = "legacy")]
pub mod versioning;

#[cfg(feature = "legacy")]
pub mod storage;

#[cfg(feature = "legacy")]
pub mod nullifier;

#[cfg(feature = "legacy")]
pub mod allocator;

#[cfg(feature = "legacy")]
pub mod zk_integration;

#[cfg(feature = "legacy")]
pub mod usage;

#[cfg(feature = "legacy")]
pub mod temporal_consistency;

// These modules are deprecated - use interfaces and implementations
// in causality-effects and causality-domain instead
#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use ResourceInterface implementations in causality-effects or causality-domain instead"
)]
pub mod manager;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod fact_observer;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod gc;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod boundary;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod facade;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod relationship;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod request;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod migration_utils;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod migration;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod registry;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod resource;

// Directory-based modules
#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod account;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod relationship;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod capability;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub mod lifecycle;

// Re-export content-addressed modules
#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub use register::{
    ResourceRegister,
    ResourceLogic,
    FungibilityDomain,
    Quantity,
    RegisterState,
    StorageStrategy,
    StateVisibility
};

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub use resource::{Resource, ResourceRegistry};

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub use registry::UnifiedRegistry;

#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use implementations in causality-effects or causality-domain instead"
)]
pub use migration::{ResourceToRegisterAdapter, RegisterSystemAdapter, MigrationAdapter};

// Re-export crypto modules
pub use causality_crypto::ContentId;

/// Adapter trait for resource registry operations
#[cfg(feature = "legacy")]
#[deprecated(
    since = "0.2.0", 
    note = "Use ResourceInterface implementations in causality-effects or causality-domain instead"
)]
pub trait ResourceRegistryAdapter {
    /// Get a register by ID
    fn get_register(&self, id: &ContentId) -> causality_types::Result<ResourceRegister>;
    
    /// Create a new register
    fn create_register(&self, register: ResourceRegister) -> causality_types::Result<ContentId>;
    
    /// Update register state
    fn update_state(&self, id: &ContentId, new_state: register::RegisterState) -> causality_types::Result<()>;
    
    /// Delete a register (mark as consumed)
    fn delete_register(&self, id: &ContentId) -> causality_types::Result<()>;
} 