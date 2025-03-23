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
pub mod capability;
pub mod capability_system;
pub mod lifecycle_manager;
pub mod relationship_tracker;
pub mod storage;
pub mod storage_adapter;
pub mod relationship;
pub mod boundary_manager;
pub mod authorization;
pub mod time_map_integration;
pub mod tests;
pub mod error;

// Re-exports
pub use api::ResourceState;
pub use resource_register::RegisterState;

// Import from capabilities module if it exists
#[cfg(feature = "capabilities")]
use crate::capabilities::{Right, Capability, CapabilityType};

// Resource IDs
pub type ResourceId = String;
pub type RegisterId = String;
pub type CapabilityId = String;

// Re-export from resource_register
pub use resource_register::ResourceRegister;

// Define a ResourceRegisterTrait to wrap the ResourceRegister struct
// This allows using ResourceRegister as a trait object
pub trait ResourceRegisterTrait: Send + Sync + 'static {
    fn get_by_id(&self, id: &str) -> error::ResourceResult<ResourceRegister>;
    fn create(&self, register: ResourceRegister) -> error::ResourceResult<()>;
    fn update(&self, register: ResourceRegister) -> error::ResourceResult<()>;
    fn delete(&self, id: &str) -> error::ResourceResult<()>;
}

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

// Storage capabilities
pub use storage::StorageStrategy;

// Types that need to be defined since they're used in lib.rs but not available in the modules
pub struct StorageAdapter;

// Re-export from capability_system
pub use capability_system::AuthorizationService;
// Define locally since the original is not available
pub struct CapabilityValidator;

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

/// Storage strategy variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageStrategy {
    /// Store the resource fully on chain
    FullyOnChain {
        /// Visibility of the resource state
        visibility: StateVisibility
    },
    
    /// Store only a commitment on chain
    CommitmentBased,
    
    /// Store the resource using a hybrid approach
    Hybrid,
}

/// State visibility options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateVisibility {
    /// State is publicly visible
    Public,
    
    /// State is private
    Private,
    
    /// State is visible only to authorized parties
    Authorized,
}

// Error types for resource operations
pub mod error {
    use thiserror::Error;
    
    /// Error type for resource operations
    #[derive(Debug, Error)]
    pub enum ResourceError {
        #[error("Resource not found: {0}")]
        NotFound(String),
        
        #[error("Invalid operation: {0}")]
        InvalidOperation(String),
        
        #[error("Resource in invalid state: {0}")]
        InvalidState(String),
        
        #[error("Resource already exists: {0}")]
        AlreadyExists(String),
        
        #[error("Resource locked: {0}")]
        Locked(String),
        
        #[error("Permission denied: {0}")]
        PermissionDenied(String),
        
        #[error("Internal error: {0}")]
        InternalError(String),
    }
    
    /// Result type for resource operations
    pub type ResourceResult<T> = std::result::Result<T, ResourceError>;
}

// Re-export from authorization
pub use authorization::ResourceAuthorizationService;

// Re-export the time map integration
pub use time_map_integration::{ResourceTimeMapIntegration, ResourceTimeSnapshot}; 