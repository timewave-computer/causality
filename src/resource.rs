// Resource management module for Causality Content-Addressed Code System
//
// This module provides functionality for resource allocation, tracking, and
// hierarchical subdivision, enabling safe and controlled resource usage.
// It also includes legacy resource management functionality for backward compatibility.

pub mod allocator;
pub mod request;
pub mod static_alloc;
pub mod usage;
pub mod manager;
pub mod register;
pub mod register_service;
pub mod register_tests;

// Re-export core types
pub use allocator::{ResourceAllocator, AllocationError};
pub use request::{ResourceRequest, ResourceGrant, GrantId};
pub use static_alloc::StaticAllocator;
pub use usage::ResourceUsage;
pub use manager::{ResourceManager, ResourceGuard, SharedResourceManager};
pub use register::{Register, RegisterId, RegisterContents, RegisterState, RegisterOperation, RegisterService};
pub use register_service::InMemoryRegisterService;

// Re-export AST correlation types for resource attribution
pub use crate::ast::{AstContext, CorrelationTracker};

// Resource module
//
// This module provides the capability-based resource API and related types.

// Module declarations
pub mod capability;
pub mod api;
pub mod memory_api;

// Re-exports
pub use capability::{
    ResourceCapability, CapabilityId, CapabilityRef, CapabilityRepository,
    Right, Restrictions, CapabilityError, CapabilityResult,
};

pub use api::{
    ResourceAPI, ResourceReader, ResourceWriter, ResourceMetadata, ResourceState,
    ResourceQuery, ResourceUpdateOptions, ResourceApiError, ResourceApiResult,
    MemoryResourceWriter,
};

// Re-exports from memory_api
pub use memory_api::MemoryResourceAPI;

// Re-export RegisterId as ResourceId from model::register
pub use crate::model::register::RegisterId as ResourceId; 