// Resource Capability Management
//
// Defines structures and traits for managing resource access permissions.

use std::sync::{Arc, Mutex, PoisonError};
use std::marker::PhantomData;
use std::fmt::Debug;
use thiserror::Error;
use std::any::Any;

use crate::resource::types::ResourceId;
use crate::identity::IdentityId; // Assuming IdentityId is defined here
use crate::effect::types::Right; // Assuming Right enum is defined here

// Import the capability struct with resource_id/right from context
use crate::effect::context::Capability as ContextCapability;
// Import the capability struct with just a name, aliased to avoid collision
// use crate::effect::capability::Capability as NamedCapability; // Keep if needed later

/// Error type for capability operations
#[derive(Error, Debug)]
pub enum CapabilityError {
    #[error("Missing required capability: {0:?}")]
    MissingCapability(ContextCapability), // Use the context::Capability

    #[error("Invalid capability format: {0}")]
    InvalidFormat(String),

    #[error("Capability validation failed: {0}")]
    ValidationFailed(String),

    #[error("Resource access denied: {0}")]
    AccessDenied(String),

    #[error("Resource not found for capability: {0}")]
    ResourceNotFound(ResourceId),

    #[error("Internal registry error: {0}")]
    RegistryError(String),
}

// Convert Mutex PoisonError to CapabilityError
impl<T> From<PoisonError<T>> for CapabilityError {
    fn from(err: PoisonError<T>) -> Self {
        CapabilityError::RegistryError(format!("Registry lock poisoned: {}", err))
    }
}

/// Represents a capability to access a specific resource with a specific right.
/// This is the key primitive for our capability-based security system.
#[derive(Debug)]
pub struct ResourceCapability<T: Send + Sync + 'static + ?Sized> {
    /// Identifier for the associated resource
    pub resource_id: ResourceId,
    /// Type of access granted by this capability
    pub right: Right,
    /// Marker to associate with concrete resource type
    _marker: PhantomData<fn() -> T>, // Zero-sized phantom data for type info
}

impl<T: Send + Sync + 'static + ?Sized> ResourceCapability<T> {
    /// Creates a new resource capability.
    pub fn new(resource_id: ResourceId, right: Right) -> Self {
        ResourceCapability {
            resource_id,
            right,
            _marker: PhantomData
        }
    }

    /// Converts this ResourceCapability to the standard context::Capability type.
    pub fn to_context_capability(&self) -> ContextCapability {
        ContextCapability { // Construct the context::Capability
            resource_id: self.resource_id.clone(),
            right: self.right.clone(),
        }
    }
}

/// A guard object providing temporary access to a resource.
/// Dropping the guard releases the access (e.g., releases a lock).
#[derive(Debug)]
pub struct ResourceGuard<'a, T: Send + Sync + 'static + ?Sized> {
    // Represents the held resource - could be a reference, lock guard, etc.
    resource: &'a T, // Example: Direct reference (adjust based on actual storage)
    // Store the capability used to obtain the guard for potential checks
    capability: ResourceCapability<T>,
}

impl<'a, T: Send + Sync + 'static + ?Sized> ResourceGuard<'a, T> {
    /// Provides read access to the guarded resource.
    pub fn read(&self) -> &T {
        self.resource
    }

    // Add write access method if needed, potentially requiring mutable access
    // pub fn write(&mut self) -> &mut T { ... }

    /// Get the capability used to acquire this guard.
    pub fn capability(&self) -> &ResourceCapability<T> {
        &self.capability
    }
}

/// Trait for a registry that manages resources and their capabilities.
pub trait ResourceRegistry: Send + Sync + Debug {
    /// Registers a resource, associates it with an owner, and returns an initial capability.
    fn register<T: Send + Sync + 'static>(
        &self,
        resource: T,
        owner: IdentityId,
        initial_right: Right,
    ) -> Result<ResourceCapability<T>, CapabilityError>;

    /// Accesses a resource through a capability.
    /// Returns a new box containing the resource if access is granted.
    fn access_boxed<T: Send + Sync + Clone + 'static>(
        &self,
        capability: &ResourceCapability<T>,
    ) -> Result<Box<T>, CapabilityError>;

    /// Grants access to a resource using a specific capability.
    /// Returns a guard object that provides access.
    fn access<'a, T: Send + Sync + 'static>(
        &'a self,
        _capability: &ResourceCapability<T>,
    ) -> Result<ResourceGuard<'a, T>, CapabilityError>;

    /// Checks if an identity holds a specific capability (or sufficient rights) for a resource.
    fn has_capability(
        &self,
        identity: &IdentityId,
        resource_id: &ResourceId,
        required_right: Right,
    ) -> Result<bool, CapabilityError>;

    /// Grants a new capability for a resource to another identity.
    /// Requires the granting identity to have sufficient rights (e.g., Admin or ownership).
    fn grant_capability<T: Send + Sync + 'static>(
        &self,
        resource_id: &ResourceId,
        granter: &IdentityId,
        grantee: &IdentityId,
        right_to_grant: Right,
    ) -> Result<ResourceCapability<T>, CapabilityError>;

    /// Revokes a capability from an identity.
    /// Requires appropriate permissions (e.g., Admin or ownership).
    fn revoke_capability<T: Send + Sync + 'static>(
        &self,
        capability_to_revoke: &ResourceCapability<T>, // Or identify by resource_id + grantee + right
        revoker: &IdentityId,
    ) -> Result<(), CapabilityError>;

    // Consider transfer ownership vs. grant/revoke?
    // fn transfer_ownership<T: Send + Sync + 'static>(
    //     &self,
    //     resource_id: &ResourceId,
    //     current_owner: &IdentityId,
    //     new_owner: &IdentityId,
    // ) -> Result<(), CapabilityError>;
}

// --- Placeholder In-Memory Registry Implementation --- 
use std::collections::HashMap;

#[derive(Debug)]
struct ResourceEntry {
    // Store the resource itself using Any to handle different types.
    // Requires downcasting during access.
    resource: Box<dyn Any + Send + Sync>,
    owner: IdentityId,
    // Store granted capabilities (mapping grantee IdentityId to their Right)
    grants: HashMap<IdentityId, Right>,
}

#[derive(Debug, Default)] // Added Default
pub struct InMemoryResourceRegistry {
    // Use ResourceId as the key.
    resources: Mutex<HashMap<ResourceId, ResourceEntry>>,
}

impl InMemoryResourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // Helper to check permissions
    fn check_permission(
        &self, 
        entry: &ResourceEntry, 
        identity: &IdentityId, 
        required_right: Right
    ) -> bool {
        if identity == &entry.owner {
            return true; // Owner has all rights
        }
        entry.grants.get(identity).map_or(false, |granted_right| granted_right >= &required_right)
    }
}

impl ResourceRegistry for InMemoryResourceRegistry {
    fn register<T: Send + Sync + 'static>(
        &self,
        resource: T,
        owner: IdentityId,
        initial_right: Right, // Grant owner initial right
    ) -> Result<ResourceCapability<T>, CapabilityError> {
        let resource_id = ResourceId::new_random(); // Generate a unique ID
        let mut grants = HashMap::new();
        grants.insert(owner.clone(), initial_right.clone()); // Grant initial right to owner

        let entry = ResourceEntry {
            resource: Box::new(resource),
            owner: owner.clone(),
            grants,
        };

        let mut resources = self.resources.lock()?;
        if resources.contains_key(&resource_id) {
            // This shouldn't happen with random IDs, but handle defensively
            return Err(CapabilityError::RegistryError("Duplicate resource ID generated".to_string()));
        }
        resources.insert(resource_id.clone(), entry);

        Ok(ResourceCapability::new(resource_id, initial_right))
    }

    fn access_boxed<T: Send + Sync + Clone + 'static>(
        &self,
        capability: &ResourceCapability<T>,
    ) -> Result<Box<T>, CapabilityError> {
        let resources = self.resources.lock()?;
        let entry = resources.get(&capability.resource_id)
            .ok_or_else(|| CapabilityError::ResourceNotFound(capability.resource_id.clone()))?;
        
        // Simplified permission check using the capability's right
        if capability.right < Right::Read { // Example: Must have at least Read
             return Err(CapabilityError::AccessDenied("Insufficient rights".to_string()));
        }

        let resource_any = &entry.resource;
        let resource_ref = resource_any.downcast_ref::<T>()
            .ok_or_else(|| CapabilityError::RegistryError("Resource type mismatch".to_string()))?;

        // Clone the resource to avoid lifetime issues
        Ok(Box::new(resource_ref.clone()))
    }

    fn access<'a, T: Send + Sync + 'static>(
        &'a self,
        _capability: &ResourceCapability<T>,
    ) -> Result<ResourceGuard<'a, T>, CapabilityError> {
        // Just return an error by default, since we can't implement this properly
        // without lifetime issues in the current design
        Err(CapabilityError::RegistryError("Direct access with ResourceGuard is not implemented. Use access_boxed instead.".to_string()))
    }

    fn has_capability(
        &self,
        identity: &IdentityId,
        resource_id: &ResourceId,
        required_right: Right,
    ) -> Result<bool, CapabilityError> {
        let resources = self.resources.lock()?;
        if let Some(entry) = resources.get(resource_id) {
            Ok(self.check_permission(entry, identity, required_right))
        } else {
            Ok(false) // Resource not found
        }
    }

    fn grant_capability<T: Send + Sync + 'static>(
        &self,
        resource_id: &ResourceId,
        granter: &IdentityId,
        grantee: &IdentityId,
        right_to_grant: Right,
    ) -> Result<ResourceCapability<T>, CapabilityError> {
        let mut resources = self.resources.lock()?;
        let entry = resources.get_mut(resource_id)
             .ok_or_else(|| CapabilityError::ResourceNotFound(resource_id.clone()))?;

        // Check if granter has permission (Owner or Delegate right required to grant)
        if !self.check_permission(entry, granter, Right::Delegate) { // Use Delegate right
             return Err(CapabilityError::AccessDenied("Granter lacks permission".to_string()));
        }

        // Add or update the grant
        entry.grants.insert(grantee.clone(), right_to_grant.clone());

        // Return a new capability representing the grant
        Ok(ResourceCapability::new(resource_id.clone(), right_to_grant))
    }

    fn revoke_capability<T: Send + Sync + 'static>(
        &self,
        capability_to_revoke: &ResourceCapability<T>, // Use capability details
        revoker: &IdentityId,
    ) -> Result<(), CapabilityError> {
        let mut resources = self.resources.lock()?;
        let entry = resources.get_mut(&capability_to_revoke.resource_id)
             .ok_or_else(|| CapabilityError::ResourceNotFound(capability_to_revoke.resource_id.clone()))?;

        // Check if revoker has permission (Owner or Delegate)
        if !self.check_permission(entry, revoker, Right::Delegate) { // Use Delegate right
             return Err(CapabilityError::AccessDenied("Revoker lacks permission".to_string()));
        }

        // Find the grantee from the capability (difficult without grantee ID)
        // Alternative: modify signature to take resource_id, grantee, right
        // For now, remove *all* grants for the specific right matching the capability
        entry.grants.retain(|_, granted_right| granted_right != &capability_to_revoke.right);

        Ok(())
    }
}


// --- Helper Functions (Obsolete?) ---
// These helpers seem tied to the old ResourceCapability structure and registry.
// Mark as potentially obsolete or refactor if still needed.
/*
pub mod helpers {
    use super::*;
    // ... (Keep create_resource_registry if useful) ...

    // TODO: Refactor or remove these capability creation helpers
    pub fn create_read_capability(owner: IdentityId) -> ResourceCapability<()> { ... }
    pub fn create_write_capability(owner: IdentityId) -> ResourceCapability<()> { ... }
    pub fn create_full_capability(owner: IdentityId) -> ResourceCapability<()> { ... }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::effect::types::Right; // Already imported above

    #[test]
    fn test_resource_capability_creation_and_conversion() {
        let resource_id = ResourceId::new_random();
        let capability = ResourceCapability::<String>::new(resource_id.clone(), Right::Write);

        // Convert to context::Capability and check fields
        let context_cap: ContextCapability = capability.to_context_capability();
        assert_eq!(context_cap.resource_id, resource_id);
        assert_eq!(context_cap.right, Right::Write);
    }

    #[test]
    fn test_in_memory_registry() {
        let registry = InMemoryResourceRegistry::new();
        let owner = IdentityId::new_random();
        let grantee = IdentityId::new_random();

        // Register a resource
        let initial_capability = registry.register(
            "hello".to_string(),
            owner.clone(),
            Right::Delegate // Owner gets Delegate rights
        ).unwrap();

        let resource_id = initial_capability.resource_id.clone();

        // Check owner has capability
        assert!(registry.has_capability(&owner, &resource_id, Right::Read).unwrap());
        assert!(registry.has_capability(&owner, &resource_id, Right::Delegate).unwrap());
        assert!(!registry.has_capability(&grantee, &resource_id, Right::Read).unwrap());

        // Owner grants Read capability to grantee
        let granted_cap = registry.grant_capability::<String>(
            &resource_id,
            &owner,
            &grantee,
            Right::Read
        ).unwrap();
        assert_eq!(granted_cap.right, Right::Read);

        // Check grantee now has Read capability
        assert!(registry.has_capability(&grantee, &resource_id, Right::Read).unwrap());
        assert!(!registry.has_capability(&grantee, &resource_id, Right::Write).unwrap());

        // Grantee tries to access with Read capability
        let access_cap = ResourceCapability::<String>::new(resource_id.clone(), Right::Read);
        let resource_box = registry.access_boxed(&access_cap).unwrap();
        assert_eq!(*resource_box, "hello");

        // Owner revokes Read capability from grantee (using the granted capability info)
        registry.revoke_capability(&granted_cap, &owner).unwrap();

        // Check grantee no longer has Read capability
        assert!(!registry.has_capability(&grantee, &resource_id, Right::Read).unwrap());
    }

    #[test]
    fn test_access_denied() {
        let registry = InMemoryResourceRegistry::new();
        let owner = IdentityId::new_random();
        let attacker = IdentityId::new_random();

        let resource_cap = registry.register(123i32, owner, Right::Read).unwrap();

        // Attacker tries to access with insufficient capability - using Custom variant with empty string
        let bad_cap = ResourceCapability::<i32>::new(resource_cap.resource_id.clone(), Right::Custom("".to_string())); // Changed from None
        let result = registry.access_boxed(&bad_cap);
        assert!(matches!(result, Err(CapabilityError::AccessDenied(_))));

        // Attacker tries to access non-existent resource
        let fake_resource_id = ResourceId::new_random();
        let fake_cap = ResourceCapability::<i32>::new(fake_resource_id, Right::Read);
        let result = registry.access_boxed(&fake_cap);
        assert!(matches!(result, Err(CapabilityError::ResourceNotFound(_))));

        // Type mismatch (trying to access integer as string)
        let wrong_type_cap = ResourceCapability::<String>::new(resource_cap.resource_id.clone(), Right::Read);
        let result = registry.access_boxed(&wrong_type_cap);
        assert!(matches!(result, Err(CapabilityError::RegistryError(_))));
    }
} 

// Manual impl of Clone for ResourceCapability that doesn't require T: Clone
impl<T: Send + Sync + 'static + ?Sized> Clone for ResourceCapability<T> {
    fn clone(&self) -> Self {
        ResourceCapability {
            resource_id: self.resource_id.clone(),
            right: self.right.clone(),
            _marker: PhantomData,
        }
    }
} 