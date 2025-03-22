// Resource Module
//
// Provides resource management functionality through a unified resource register model
// with lifecycle management, relationship tracking, and capability-based authorization.

pub mod api;
pub mod allocator;
pub mod request;
pub mod static_alloc;
pub mod usage;
pub mod manager;
pub mod resource_register;
pub mod capability;
pub mod capability_system;
pub mod lifecycle_manager;
pub mod relationship_tracker;
pub mod storage;
pub mod storage_adapter;

// Re-exports
pub use api::ResourceState;
pub use resource_register::RegisterState;
// Import Right from capabilities module
use crate::capabilities::{Right, Capability, CapabilityType};

// Resource IDs
pub type ResourceId = String;
pub type RegisterId = String;
pub type CapabilityId = String;

// Re-export from resource_register
pub use resource_register::ResourceRegister;

// Re-export from lifecycle_manager
pub use lifecycle_manager::ResourceRegisterLifecycleManager;
// TransitionReason enum is defined locally since the original is private
pub enum TransitionReason {
    UserInitiated,
    PolicyEnforced,
    SystemScheduled,
    ErrorRecovery,
    Migration
}
pub use lifecycle_manager::RegisterOperationType;

// Re-export from relationship_tracker
pub use relationship_tracker::RelationshipTracker;
pub use relationship_tracker::ResourceRelationship;
pub use relationship_tracker::RelationshipType;
pub use relationship_tracker::RelationshipDirection;

// Storage capabilities
pub use storage::StorageStrategy;

// Types that need to be defined since they're used in lib.rs but not available in the modules
pub struct StorageAdapter;
// Remove duplicate Right and Capability definitions
// pub struct Right;
// pub struct Capability;
// pub enum CapabilityType {
//     Read,
//     Write,
//     Execute,
//     Delegate
// }

// Re-export from capability_system
pub use capability_system::AuthorizationService;
// Define locally since the original is not available
pub struct CapabilityValidator;

// Types that need to be defined
pub struct ResourceLogic;
pub struct Quantity(pub u64);

// Common resource logic implementation
impl ResourceLogic {
    pub fn new() -> Self {
        ResourceLogic {}
    }
    
    pub fn validate_transition(&self, _from: ResourceState, _to: ResourceState) -> bool {
        // Default implementation that allows any transition
        true
    }
} 