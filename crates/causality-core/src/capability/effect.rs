// Effect capabilities
//
// This module provides effect-specific capability types and functionality
// that integrate with the core capability system.

use std::any::Any;
use std::fmt;
use std::sync::Arc;
use std::collections::HashSet;

use thiserror::Error;

// Fix imports to use the correct types
use crate::capability::{ResourceId, IdentityId, ContentAddressingError, ContentRef};
use crate::capability::utils;
use causality_types::{ContentHash, ContentId};
use std::marker::PhantomData;

// Temporary imported types until we restructure - matched with resource.rs
#[derive(Debug, Clone, PartialEq)]
struct ResourceGuard<T>(T);

impl<T> ResourceGuard<T> {
    // Add placeholder method
    pub fn read(&self) -> Result<&T, String> {
        Ok(&self.0)
    }
}

struct ResourceRegistry(Arc<()>);

impl ResourceRegistry {
    pub fn new() -> Self {
        Self(Arc::new(()))
    }
    
    // Add minimal implementations to fix errors
    fn register<T>(&self, resource: T, _owner: IdentityId) -> Result<Capability<T>, String> {
        Ok(Capability {
            id: ResourceId::new(utils::hash_string("placeholder")),
            grants: CapabilityGrants(255), // All permissions
            origin: None,
            _phantom: PhantomData,
        })
    }
    
    fn access<T>(&self, _capability: &Capability<T>) -> Result<ResourceGuard<T>, String> {
        unimplemented!("Not implemented in temporary structure")
    }
    
    fn access_by_content<T>(&self, _content_ref: &ContentRef<T>) -> Result<ResourceGuard<T>, String> {
        unimplemented!("Not implemented in temporary structure")
    }
    
    fn has_capability(&self, _identity: &IdentityId, _resource_id: &ResourceId) -> Result<bool, String> {
        unimplemented!("Not implemented in temporary structure")
    }
    
    fn transfer_capability<T>(&self, _capability: &Capability<T>, _from: &IdentityId, _to: &IdentityId) -> Result<(), String> {
        unimplemented!("Not implemented in temporary structure")
    }
}

// Define proper capability types with implementation
#[derive(Debug, Clone, PartialEq)]
struct Capability<T> {
    pub id: ResourceId,
    pub grants: CapabilityGrants,
    pub origin: Option<IdentityId>,
    pub _phantom: PhantomData<T>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CapabilityGrants(u8);

impl CapabilityGrants {
    pub fn new(can_read: bool, can_write: bool, can_delegate: bool) -> Self {
        let mut value = 0;
        if can_read { value |= 1 }
        if can_write { value |= 2 }
        if can_delegate { value |= 4 }
        Self(value)
    }
    
    pub fn read_only() -> Self {
        Self::new(true, false, false)
    }
    
    pub fn write_only() -> Self {
        Self::new(false, true, false)
    }
    
    pub fn full() -> Self {
        Self::new(true, true, true)
    }
    
    pub fn can_read(&self) -> bool {
        (self.0 & 1) != 0
    }
    
    pub fn can_write(&self) -> bool {
        (self.0 & 2) != 0
    }
    
    pub fn can_delegate(&self) -> bool {
        (self.0 & 4) != 0
    }
}

type CapabilityError = String;

/// Effect capability types for various operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectCapabilityType {
    /// Execute any effect
    ExecuteEffects,
    
    /// Query capabilities
    QueryEffects,
    
    /// Resource management
    CreateResource,
    ReadResource,
    UpdateResource,
    DeleteResource,
    
    /// Transaction capabilities
    InitiateTransaction,
    SignTransaction,
    CommitTransaction,
    
    /// Cross-domain capabilities
    CrossDomainTransfer,
    CrossDomainQuery,
    
    /// Administrative capabilities
    ManageEffects,
    
    /// Custom capability with name
    Custom(String)
}

impl EffectCapabilityType {
    /// Convert effect capability to string representation
    pub fn to_string(&self) -> String {
        match self {
            EffectCapabilityType::ExecuteEffects => "execute_effects".to_string(),
            EffectCapabilityType::QueryEffects => "query_effects".to_string(),
            EffectCapabilityType::CreateResource => "create_resource".to_string(),
            EffectCapabilityType::ReadResource => "read_resource".to_string(),
            EffectCapabilityType::UpdateResource => "update_resource".to_string(),
            EffectCapabilityType::DeleteResource => "delete_resource".to_string(),
            EffectCapabilityType::InitiateTransaction => "initiate_transaction".to_string(),
            EffectCapabilityType::SignTransaction => "sign_transaction".to_string(),
            EffectCapabilityType::CommitTransaction => "commit_transaction".to_string(),
            EffectCapabilityType::CrossDomainTransfer => "cross_domain_transfer".to_string(),
            EffectCapabilityType::CrossDomainQuery => "cross_domain_query".to_string(),
            EffectCapabilityType::ManageEffects => "manage_effects".to_string(),
            EffectCapabilityType::Custom(name) => format!("custom_{}", name),
        }
    }
    
    /// Create a capability from an effect capability type
    pub fn create_capability(&self, grants: CapabilityGrants, owner: IdentityId) -> EffectCapability {
        let id = self.create_resource_id();
        
        EffectCapability {
            capability_type: self.clone(),
            grants,
            id,
            origin: Some(owner),
            content_hash: None,
        }
    }
    
    /// Create a resource ID for an effect capability
    fn create_resource_id(&self) -> ResourceId {
        let capability_str = self.to_string();
        let id_str = format!("effect_{}", capability_str);
        ResourceId::new(utils::hash_string(&id_str))
    }
}

/// An effect-specific capability
#[derive(Debug, Clone)]
pub struct EffectCapability {
    /// The effect capability type
    pub capability_type: EffectCapabilityType,
    
    /// The capability grants
    pub grants: CapabilityGrants,
    
    /// The identifier for the capability
    pub id: ResourceId,
    
    /// The origin identity that created the capability
    pub origin: Option<IdentityId>,
    
    /// The content hash if content-addressed
    pub content_hash: Option<ContentHash>,
}

impl EffectCapability {
    /// Convert to a standard capability
    pub fn to_capability<T: Send + Sync + 'static>(&self) -> Capability<T> {
        Capability {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Create a new effect capability
    pub fn new(
        capability_type: EffectCapabilityType,
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

/// Error type for effect capability operations
#[derive(Error, Debug)]
pub enum EffectCapabilityError {
    #[error("Invalid capability type: {0}")]
    InvalidCapabilityType(String),
    
    #[error("Missing required grants")]
    MissingGrants,
    
    #[error("Underlying capability error")]
    CapabilityError(Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Content addressing error: {0}")]
    ContentAddressingError(String),
}

/// Effect registry with enhanced capability-based effect management
pub struct EffectCapabilityRegistry {
    /// The underlying resource registry
    registry: ResourceRegistry,
    
    /// Required capabilities for effect types
    required_capabilities: std::collections::HashMap<String, HashSet<EffectCapabilityType>>,
}

impl EffectCapabilityRegistry {
    /// Create a new effect capability registry
    pub fn new() -> Self {
        Self {
            registry: ResourceRegistry::new(),
            required_capabilities: std::collections::HashMap::new(),
        }
    }
    
    /// Register required capabilities for an effect type
    pub fn register_effect_requirements(
        &mut self,
        effect_type: &str,
        required_capabilities: HashSet<EffectCapabilityType>,
    ) {
        self.required_capabilities.insert(effect_type.to_string(), required_capabilities);
    }
    
    /// Get required capabilities for an effect type
    pub fn get_required_capabilities(&self, effect_type: &str) -> Option<&HashSet<EffectCapabilityType>> {
        self.required_capabilities.get(effect_type)
    }
    
    /// Register a resource and get an effect capability
    pub fn register<T: Send + Sync + 'static>(
        &self,
        resource: T,
        owner: IdentityId,
        capability_type: EffectCapabilityType,
    ) -> Result<EffectCapability, CapabilityError> {
        // Register in the core registry with full rights
        let capability = self.registry.register(resource, owner.clone())?;
        
        // Create an effect capability with the specified type
        let effect_capability = EffectCapability {
            capability_type,
            grants: capability.grants,
            id: capability.id,
            origin: capability.origin,
            content_hash: None,
        };
        
        Ok(effect_capability)
    }
    
    /// Access a resource using an effect capability
    pub fn access<T: Send + Sync + 'static>(
        &self,
        capability: &EffectCapability,
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
    
    /// Check if an identity has the required capabilities for an effect
    pub fn has_required_capabilities(
        &self,
        identity: &IdentityId,
        effect_type: &str,
    ) -> Result<bool, CapabilityError> {
        // Get the required capabilities for this effect
        if let Some(required) = self.get_required_capabilities(effect_type) {
            // Check each required capability
            for cap_type in required {
                let id = cap_type.create_resource_id();
                if !self.registry.has_capability(identity, &id)? {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            // No requirements defined, assume allowed
            Ok(true)
        }
    }
    
    /// Transfer a capability to another identity
    pub fn transfer_capability(
        &self,
        capability: &EffectCapability,
        from: &IdentityId,
        to: &IdentityId,
    ) -> Result<(), CapabilityError> {
        let std_capability = Capability {
            id: capability.id.clone(),
            grants: capability.grants.clone(),
            origin: capability.origin.clone(),
            _phantom: std::marker::PhantomData::<Box<dyn Any + Send + Sync>>,
        };
        
        self.registry.transfer_capability(&std_capability, from, to)
    }
}

/// Helper functions for working with effect capabilities
pub mod helpers {
    use super::*;
    
    /// Create a new effect capability registry
    pub fn create_effect_registry() -> EffectCapabilityRegistry {
        EffectCapabilityRegistry::new()
    }
    
    /// Create an effect registry with common effect types
    pub fn create_effect_registry_with_defaults() -> EffectCapabilityRegistry {
        let mut registry = EffectCapabilityRegistry::new();
        
        // Register common effect requirements
        
        // Query effect requirements
        let mut query_reqs = HashSet::new();
        query_reqs.insert(EffectCapabilityType::QueryEffects);
        query_reqs.insert(EffectCapabilityType::ReadResource);
        registry.register_effect_requirements("query", query_reqs);
        
        // Transaction effect requirements
        let mut tx_reqs = HashSet::new();
        tx_reqs.insert(EffectCapabilityType::InitiateTransaction);
        tx_reqs.insert(EffectCapabilityType::ReadResource);
        tx_reqs.insert(EffectCapabilityType::UpdateResource);
        registry.register_effect_requirements("transaction", tx_reqs);
        
        // Resource creation effect requirements
        let mut create_reqs = HashSet::new();
        create_reqs.insert(EffectCapabilityType::CreateResource);
        registry.register_effect_requirements("create_resource", create_reqs);
        
        registry
    }
    
    /// Create common effect capabilities
    pub fn create_execute_capability(owner: IdentityId) -> EffectCapability {
        EffectCapability::new(
            EffectCapabilityType::ExecuteEffects,
            CapabilityGrants::full(),
            owner,
        )
    }
    
    /// Create query capability
    pub fn create_query_capability(owner: IdentityId) -> EffectCapability {
        EffectCapability::new(
            EffectCapabilityType::QueryEffects,
            CapabilityGrants::read_only(),
            owner,
        )
    }
    
    /// Create resource management capability
    pub fn create_resource_capability(owner: IdentityId) -> EffectCapability {
        EffectCapability::new(
            EffectCapabilityType::CreateResource,
            CapabilityGrants::new(true, true, false),  // Read and write, but not delegate
            owner,
        )
    }
    
    /// Create admin capability
    pub fn create_admin_capability(owner: IdentityId) -> EffectCapability {
        EffectCapability::new(
            EffectCapabilityType::ManageEffects,
            CapabilityGrants::full(),
            owner,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_capability_types() {
        let execute = EffectCapabilityType::ExecuteEffects;
        let query = EffectCapabilityType::QueryEffects;
        let custom = EffectCapabilityType::Custom("test".to_string());
        
        // Test to_string
        assert_eq!(execute.to_string(), "execute_effects");
        assert_eq!(query.to_string(), "query_effects");
        assert_eq!(custom.to_string(), "custom_test");
    }
    
    #[test]
    fn test_effect_capability_registry() {
        // Create a registry
        let registry = EffectCapabilityRegistry::new();
        
        // Create an identity
        let alice = IdentityId::new();
        
        // Create a test resource
        let test_data = "Effect test data".to_string();
        
        // Register the resource
        let capability = registry.register(
            test_data,
            alice.clone(),
            EffectCapabilityType::ExecuteEffects,
        ).unwrap();
        
        // Verify capability type
        assert_eq!(
            capability.capability_type,
            EffectCapabilityType::ExecuteEffects
        );
        
        // Access the resource
        let guard = registry.access::<String>(&capability).unwrap();
        let data = guard.read().unwrap();
        assert_eq!(*data, "Effect test data".to_string());
    }
    
    #[test]
    fn test_effect_capability_helpers() {
        // Create an identity
        let alice = IdentityId::new();
        
        // Test execute capability
        let exec_cap = helpers::create_execute_capability(alice.clone());
        assert_eq!(exec_cap.capability_type, EffectCapabilityType::ExecuteEffects);
        assert_eq!(exec_cap.grants, CapabilityGrants::full());
        
        // Test query capability
        let query_cap = helpers::create_query_capability(alice.clone());
        assert_eq!(query_cap.capability_type, EffectCapabilityType::QueryEffects);
        assert_eq!(query_cap.grants, CapabilityGrants::read_only());
        
        // Test resource capability
        let resource_cap = helpers::create_resource_capability(alice.clone());
        assert_eq!(resource_cap.capability_type, EffectCapabilityType::CreateResource);
        assert!(resource_cap.grants.can_read());
        assert!(resource_cap.grants.can_write());
        assert!(!resource_cap.grants.can_delegate());
        
        // Test registry with defaults
        let registry = helpers::create_effect_registry_with_defaults();
        let query_reqs = registry.get_required_capabilities("query").unwrap();
        assert!(query_reqs.contains(&EffectCapabilityType::QueryEffects));
        assert!(query_reqs.contains(&EffectCapabilityType::ReadResource));
        
        let tx_reqs = registry.get_required_capabilities("transaction").unwrap();
        assert!(tx_reqs.contains(&EffectCapabilityType::InitiateTransaction));
        assert!(tx_reqs.contains(&EffectCapabilityType::UpdateResource));
    }
    
    #[test]
    fn test_required_capabilities() {
        // Create a registry with defaults
        let mut registry = helpers::create_effect_registry_with_defaults();
        
        // Create an identity
        let alice = IdentityId::new();
        
        // Create capabilities
        let query_cap = helpers::create_query_capability(alice.clone());
        let read_cap = EffectCapability::new(
            EffectCapabilityType::ReadResource,
            CapabilityGrants::read_only(),
            alice.clone(),
        );
        
        // Register the capabilities in the registry
        registry.register("Query capability", alice.clone(), query_cap.capability_type.clone()).unwrap();
        registry.register("Read capability", alice.clone(), read_cap.capability_type.clone()).unwrap();
        
        // Check if the identity has the required capabilities for query
        let has_query_caps = registry.has_required_capabilities(&alice, "query").unwrap();
        assert!(has_query_caps, "Should have required capabilities for query");
        
        // Check for a capability that doesn't have all requirements
        let has_tx_caps = registry.has_required_capabilities(&alice, "transaction").unwrap();
        assert!(!has_tx_caps, "Should not have required capabilities for transaction");
    }
} 