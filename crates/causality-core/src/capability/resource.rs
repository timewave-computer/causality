// Resource capabilities
//
// This module provides resource-specific capability types and functionality
// that integrate with the core capability system.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

use super::{
    ResourceId, IdentityId, Capability, CapabilityGrants, 
    ResourceGuard, ResourceRegistry, CapabilityError,
    ContentHash, ContentRef, ContentAddressed
};

/// Resource access types for capability permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceAccessType {
    /// Read-only access
    Read,
    
    /// Write access
    Write,
    
    /// Execute access (for code resources)
    Execute,
    
    /// Lock access (for exclusive access)
    Lock,
    
    /// Transfer access (for ownership changes)
    Transfer
}

/// Resource lifecycle capability types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceLifecycleType {
    /// Create new resources
    Create,
    
    /// Activate resources
    Activate,
    
    /// Update existing resources
    Update,
    
    /// Lock resources
    Lock,
    
    /// Unlock resources
    Unlock,
    
    /// Freeze resources (prevent modification)
    Freeze,
    
    /// Unfreeze resources
    Unfreeze,
    
    /// Consume resources (one-time use)
    Consume,
    
    /// Archive resources (preserve but make inactive)
    Archive,
    
    /// All lifecycle capabilities
    All
}

/// Cross-domain lock types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CrossDomainLockType {
    /// Shared read lock
    SharedRead,
    
    /// Exclusive write lock
    ExclusiveWrite,
    
    /// Intent lock (for two-phase locking)
    Intent,
    
    /// Upgrade lock (can be promoted)
    Upgrade
}

/// Dependency types between resources
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Strong dependency (owns lifecycle)
    Strong,
    
    /// Weak dependency (references only)
    Weak,
    
    /// Triggers on change
    Trigger,
    
    /// Derives values from
    Derives
}

/// Resource-specific capability types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceCapabilityType {
    /// Access capabilities
    Access(ResourceAccessType),
    
    /// Lifecycle capabilities
    Lifecycle(ResourceLifecycleType),
    
    /// Locking capabilities
    Lock(CrossDomainLockType),
    
    /// Dependency capabilities
    Dependency(DependencyType),
    
    /// Combined capability for full control
    FullControl,
}

impl ResourceCapabilityType {
    /// Convert resource capability to string representation
    pub fn to_string(&self) -> String {
        match self {
            ResourceCapabilityType::Access(access) => {
                format!("access_{:?}", access).to_lowercase()
            },
            ResourceCapabilityType::Lifecycle(lifecycle) => {
                format!("lifecycle_{:?}", lifecycle).to_lowercase()
            },
            ResourceCapabilityType::Lock(lock) => {
                format!("lock_{:?}", lock).to_lowercase()
            },
            ResourceCapabilityType::Dependency(dep) => {
                format!("dep_{:?}", dep).to_lowercase()
            },
            ResourceCapabilityType::FullControl => "full_control".to_string(),
        }
    }
    
    /// Create a capability from a resource capability type
    pub fn create_capability(&self, grants: CapabilityGrants, owner: IdentityId) -> ResourceCapability {
        let id = self.create_resource_id();
        
        ResourceCapability {
            capability_type: self.clone(),
            grants,
            id,
            origin: Some(owner),
            content_hash: None,
        }
    }
    
    /// Create a resource ID for a resource capability
    fn create_resource_id(&self) -> ResourceId {
        let capability_str = self.to_string();
        let id_str = format!("resource_{}", capability_str);
        ResourceId::new(super::utils::hash_string(&id_str))
    }
}

/// A resource-specific capability
#[derive(Debug, Clone)]
pub struct ResourceCapability {
    /// The resource capability type
    pub capability_type: ResourceCapabilityType,
    
    /// The capability grants
    pub grants: CapabilityGrants,
    
    /// The identifier for the capability
    pub id: ResourceId,
    
    /// The origin identity that created the capability
    pub origin: Option<IdentityId>,
    
    /// The content hash if content-addressed
    pub content_hash: Option<ContentHash>,
}

impl ResourceCapability {
    /// Convert to a standard capability
    pub fn to_capability<T: Send + Sync + 'static>(&self) -> Capability<T> {
        Capability {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Create a new resource capability
    pub fn new(
        capability_type: ResourceCapabilityType,
        grants: CapabilityGrants,
        owner: IdentityId
    ) -> Self {
        capability_type.create_capability(grants, owner)
    }
    
    /// Create a content-addressed version of this capability
    pub fn to_content_addressed(&self, content_hash: ContentHash) -> Self {
        let mut result = self.clone();
        result.content_hash = Some(content_hash);
        result
    }
    
    /// Get the content hash if this is content-addressed
    pub fn content_hash(&self) -> Option<&ContentHash> {
        self.content_hash.as_ref()
    }
    
    /// Check if this capability is content-addressed
    pub fn is_content_addressed(&self) -> bool {
        self.content_hash.is_some()
    }
}

/// Error type for resource capability operations
#[derive(Error, Debug)]
pub enum ResourceCapabilityError {
    #[error("Invalid capability type: {0}")]
    InvalidCapabilityType(String),
    
    #[error("Missing required grants")]
    MissingGrants,
    
    #[error("Underlying capability error: {0}")]
    CapabilityError(#[from] CapabilityError),
    
    #[error("Content addressing error: {0}")]
    ContentAddressingError(String),
}

/// Resource registry with enhanced capability-based resource management
pub struct ResourceCapabilityRegistry {
    /// The underlying resource registry
    registry: ResourceRegistry,
}

impl ResourceCapabilityRegistry {
    /// Create a new resource capability registry
    pub fn new() -> Self {
        Self {
            registry: ResourceRegistry::new(),
        }
    }
    
    /// Register a resource and get a resource capability
    pub fn register<T: Send + Sync + 'static>(
        &self,
        resource: T,
        owner: IdentityId,
        capability_type: ResourceCapabilityType,
    ) -> Result<ResourceCapability, CapabilityError> {
        // Register in the core registry with full rights
        let capability = self.registry.register(resource, owner.clone())?;
        
        // Create a resource capability with the specified type
        let resource_capability = ResourceCapability {
            capability_type,
            grants: capability.grants,
            id: capability.id,
            origin: capability.origin,
            content_hash: None,
        };
        
        Ok(resource_capability)
    }
    
    /// Access a resource using a resource capability
    pub fn access<T: Send + Sync + 'static>(
        &self,
        capability: &ResourceCapability,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        // Create a standard capability
        let std_capability = capability.to_capability::<T>();
        
        // Access with the standard capability
        self.registry.access(&std_capability)
    }
    
    /// Access a resource by content reference
    pub fn access_by_content<T: Send + Sync + 'static>(
        &self,
        content_ref: &ContentRef<T>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        self.registry.access_by_content(content_ref)
    }
    
    /// Check if an identity has a capability
    pub fn has_capability(
        &self,
        identity: &IdentityId,
        resource_id: &ResourceId,
    ) -> Result<bool, CapabilityError> {
        self.registry.has_capability(identity, resource_id)
    }
    
    /// Transfer a capability to another identity
    pub fn transfer_capability(
        &self,
        capability: &ResourceCapability,
        from: &IdentityId,
        to: &IdentityId,
    ) -> Result<(), CapabilityError> {
        let std_capability = Capability {
            id: capability.id.clone(),
            grants: capability.grants.clone(),
            origin: capability.origin.clone(),
            _phantom: std::marker::PhantomData::<dyn Any + Send + Sync>,
        };
        
        self.registry.transfer_capability(&std_capability, from, to)
    }
}

/// Helper functions for working with resource capabilities
pub mod helpers {
    use super::*;
    
    /// Create a new resource capability registry
    pub fn create_resource_registry() -> ResourceCapabilityRegistry {
        ResourceCapabilityRegistry::new()
    }
    
    /// Create a content-addressed resource capability registry
    pub fn create_content_addressed_resource_registry() -> ResourceCapabilityRegistry {
        let registry = ResourceCapabilityRegistry::new();
        // Set up the registry to use content addressing
        // This can be enhanced later
        registry
    }
    
    /// Create a read capability
    pub fn create_read_capability(owner: IdentityId) -> ResourceCapability {
        ResourceCapability::new(
            ResourceCapabilityType::Access(ResourceAccessType::Read),
            CapabilityGrants::read_only(),
            owner,
        )
    }
    
    /// Create a write capability
    pub fn create_write_capability(owner: IdentityId) -> ResourceCapability {
        ResourceCapability::new(
            ResourceCapabilityType::Access(ResourceAccessType::Write),
            CapabilityGrants::write_only(),
            owner,
        )
    }
    
    /// Create a full access capability
    pub fn create_full_capability(owner: IdentityId) -> ResourceCapability {
        ResourceCapability::new(
            ResourceCapabilityType::FullControl,
            CapabilityGrants::full(),
            owner,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_capability_types() {
        let read_access = ResourceCapabilityType::Access(ResourceAccessType::Read);
        let create_lifecycle = ResourceCapabilityType::Lifecycle(ResourceLifecycleType::Create);
        let full_control = ResourceCapabilityType::FullControl;
        
        // Test to_string
        assert_eq!(read_access.to_string(), "access_read");
        assert_eq!(create_lifecycle.to_string(), "lifecycle_create");
        assert_eq!(full_control.to_string(), "full_control");
    }
    
    #[test]
    fn test_resource_capability_registry() {
        // Create a registry
        let registry = ResourceCapabilityRegistry::new();
        
        // Create an identity
        let alice = IdentityId::new();
        
        // Create a test resource
        let test_data = "Resource test data".to_string();
        
        // Register the resource
        let capability = registry.register(
            test_data,
            alice.clone(),
            ResourceCapabilityType::Access(ResourceAccessType::Read),
        ).unwrap();
        
        // Verify capability type
        assert_eq!(
            capability.capability_type,
            ResourceCapabilityType::Access(ResourceAccessType::Read)
        );
        
        // Access the resource
        let guard = registry.access::<String>(&capability).unwrap();
        let data = guard.read().unwrap();
        assert_eq!(*data, "Resource test data".to_string());
    }
    
    #[test]
    fn test_resource_capability_helpers() {
        // Create an identity
        let alice = IdentityId::new();
        
        // Test read capability
        let read_cap = helpers::create_read_capability(alice.clone());
        match &read_cap.capability_type {
            ResourceCapabilityType::Access(access_type) => {
                assert_eq!(*access_type, ResourceAccessType::Read);
            },
            _ => panic!("Wrong capability type"),
        }
        assert_eq!(read_cap.grants, CapabilityGrants::read_only());
        
        // Test write capability
        let write_cap = helpers::create_write_capability(alice.clone());
        match &write_cap.capability_type {
            ResourceCapabilityType::Access(access_type) => {
                assert_eq!(*access_type, ResourceAccessType::Write);
            },
            _ => panic!("Wrong capability type"),
        }
        assert_eq!(write_cap.grants, CapabilityGrants::write_only());
        
        // Test full capability
        let full_cap = helpers::create_full_capability(alice.clone());
        match &full_cap.capability_type {
            ResourceCapabilityType::FullControl => {},
            _ => panic!("Wrong capability type"),
        }
        assert_eq!(full_cap.grants, CapabilityGrants::full());
    }
} 