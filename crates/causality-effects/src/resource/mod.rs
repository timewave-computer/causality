// Resource management integration module
//
// This module provides integration between capabilities, effects, and domains
// for resource management, including access control, lifecycle management,
// locking mechanisms, and dependency tracking.

// Sub-modules
pub mod access;    // Resource access patterns
pub mod capability; // Resource capability integration
pub mod lifecycle; // Resource lifecycle management
pub mod locking;   // Cross-domain resource locking
pub mod dependency; // Resource dependency tracking
pub mod examples;  // Usage examples
pub mod effects;   // Cross-domain resource effects
pub mod implementation; // Resource implementation for effects

// Test module
#[cfg(test)]
mod tests;

// Re-exports
pub use access::{
    ResourceAccessType, ResourceAccess, ResourceAccessTracker,
    ResourceAccessTracking, ResourceAccessManager
};

pub use lifecycle::{
    ResourceLifecycleEvent, LifecycleEvent, EffectResourceLifecycle,
    ResourceLifecycleEffect
};

pub use locking::{
    LockStatus, CrossDomainLockType, ResourceLock,
    CrossDomainLockManager, AcquireLockEffect, ReleaseLockEffect
};

pub use dependency::{
    DependencyType, ResourceDependency, ResourceDependencyManager
};

pub use capability::{
    ResourceCapability, ResourceLifecycleCapability, ResourceCapabilityManager
};

pub use examples::{
    resource_access_example, resource_lifecycle_example,
    resource_locking_example, resource_dependency_example,
    integrated_resource_management_example, run_all_examples
};

pub use effects::{
    CrossDomainResourceManagers, CrossDomainResourceTransferEffect,
    CrossDomainResourceLockEffect, CrossDomainResourceDependencyEffect,
    transfer_resource, lock_resource_across_domains, add_cross_domain_dependency,
    cross_domain_resource_example
};

pub use implementation::*; 