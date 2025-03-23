// Resource Module
//
// Provides resource management functionality through a unified resource register model
// with lifecycle management, relationship tracking, and capability-based authorization.

// Module declarations
pub mod api;
pub mod allocator;
pub mod request;
pub mod static_alloc;
pub mod usage;
pub mod manager;
pub mod resource_register;
pub mod capability_system;
pub mod capability; // file-based module replacing capability/mod.rs
pub mod lifecycle_manager;
pub mod relationship_tracker;
pub mod storage;
pub mod storage_adapter;
pub mod relationship;
pub mod boundary_manager;
pub mod authorization;
pub mod resource_temporal_consistency;
pub mod tests;

// Using crate's error module
use crate::error::{self, Result as CrateResult};

// Re-exports
pub use api::ResourceState;
pub use resource_register::RegisterState;

// Import from capabilities module if it exists
#[cfg(feature = "capabilities")]
use crate::capabilities::{Right, Capability, CapabilityType};

// Resource IDs - using the ResourceId from types.rs
pub use crate::types::ResourceId;
pub type RegisterId = String;
pub type CapabilityId = String;

// Resource-specific result type
pub type ResourceResult<T> = std::result::Result<T, error::Error>;

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

// Re-export key types from relationship module
pub use relationship::{
    CrossDomainRelationship,
    CrossDomainRelationshipType,
    CrossDomainMetadata,
    CrossDomainRelationshipManager,
    ValidationLevel,
    ValidationResult,
    SyncStrategy,
    SyncStatus,
    SyncResult,
    SchedulerConfig,
    SchedulerStatus,
};

// Re-export storage capabilities
pub use storage::StorageStrategy;
pub use storage::StateVisibility;

// Re-export from capability_system
pub use capability_system::AuthorizationService;

// Types that need to be defined
pub struct ResourceLogic {
    // Fungibility type (Fungible, NonFungible, etc.)
    pub logic_type: ResourceLogicType,
}

pub enum ResourceLogicType {
    Fungible,
    NonFungible,
    SemiFungible,
    Custom(String),
}

pub struct Quantity(pub u64);

impl Quantity {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

// Fungibility domain for resources
pub struct FungibilityDomain {
    pub domain_type: String,
}

impl FungibilityDomain {
    pub fn new(domain_type: &str) -> Self {
        Self { domain_type: domain_type.to_string() }
    }
}

// Common resource logic implementation
impl ResourceLogic {
    pub fn new() -> Self {
        ResourceLogic { logic_type: ResourceLogicType::Fungible }
    }
    
    pub fn validate_transition(&self, _from: ResourceState, _to: ResourceState) -> bool {
        // Default implementation that allows any transition
        true
    }
}

// Re-export from authorization
pub use authorization::ResourceAuthorizationService;

// Re-export the temporal consistency management
pub use resource_temporal_consistency::{ResourceTemporalConsistency, ResourceTimeSnapshot}; 