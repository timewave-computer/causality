// Resource module for content-addressed resources
//
// This module defines the Resource type and related functionality
// for handling content-addressed resources.
//
// MIGRATION NOTE: This file is being migrated to use ResourceRegister instead of
// the separate Resource struct, as part of the unification process.

use borsh::{BorshSerialize, BorshDeserialize};
use std::collections::HashMap;
use crate::crypto::{
    ContentAddressed, ContentId, HashOutput, HashFactory, HashError
};
use crate::resource::{
    ResourceRegister,
    migrate_helpers::{
        create_resource_register,
        create_register_with_metadata,
        update_register_data,
    }
};

/// A resource in the system
///
/// MIGRATION NOTE: This struct is being phased out in favor of ResourceRegister.
/// New code should use ResourceRegister directly. This struct is maintained for
/// backward compatibility during the migration process.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Resource {
    /// Name of the resource
    name: String,
    /// Type of resource
    resource_type: String,
    /// Resource data
    data: Vec<u8>,
    /// Resource metadata
    metadata: HashMap<String, String>,
    /// Version of the resource
    version: u64,
    /// Inner representation as ResourceRegister (for migration)
    #[borsh(skip)]
    inner: Option<ResourceRegister>,
}

impl Resource {
    /// Create a new resource
    ///
    /// MIGRATION NOTE: Consider using ResourceRegister::new directly
    /// or the migration helper create_resource_register() instead.
    pub fn new(name: impl Into<String>, resource_type: impl Into<String>, data: Vec<u8>) -> Self {
        let name_str = name.into();
        let resource_type_str = resource_type.into();
        
        // Create the inner ResourceRegister
        let register = create_resource_register(
            name_str.clone(),
            resource_type_str.clone(),
            data.clone()
        );
        
        Self {
            name: name_str,
            resource_type: resource_type_str,
            data,
            metadata: HashMap::new(),
            version: 1,
            inner: Some(register),
        }
    }
    
    /// Get the name of the resource
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the type of the resource
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    /// Get the resource data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Set the resource data
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data.clone();
        self.version += 1;
        
        // Update inner ResourceRegister if present
        if let Some(register) = &mut self.inner {
            let _ = update_register_data(register, data);
        }
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key_str = key.into();
        let value_str = value.into();
        
        self.metadata.insert(key_str.clone(), value_str.clone());
        
        // Update inner ResourceRegister if present
        if let Some(register) = &mut self.inner {
            register.metadata.insert(key_str, value_str);
        }
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
    
    /// Get the version of the resource
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Create an updated version of this resource
    pub fn update(&self, new_data: Vec<u8>) -> Self {
        let mut updated = self.clone();
        updated.data = new_data.clone();
        updated.version = self.version + 1;
        
        // Update inner ResourceRegister if present
        if let Some(register) = &mut updated.inner {
            let _ = update_register_data(register, new_data);
        }
        
        updated
    }
    
    /// Convert this Resource to a ResourceRegister
    ///
    /// This helper method facilitates migration to the unified ResourceRegister model
    pub fn to_resource_register(&self) -> ResourceRegister {
        // If we already have an inner ResourceRegister, return it
        if let Some(register) = &self.inner {
            return register.clone();
        }
        
        // Otherwise create a new one
        let mut metadata = self.metadata.clone();
        metadata.insert("version".to_string(), self.version.to_string());
        
        create_register_with_metadata(
            self.name.clone(),
            self.resource_type.clone(),
            self.data.clone(),
            metadata
        )
    }
    
    /// Create a Resource from a ResourceRegister
    ///
    /// This helper method facilitates backward compatibility during migration
    pub fn from_resource_register(register: &ResourceRegister) -> Self {
        // Extract the type from ResourceLogic
        let resource_type = match &register.resource_logic {
            crate::resource::resource_register::ResourceLogic::Custom(t) => t.clone(),
            crate::resource::resource_register::ResourceLogic::Fungible => "fungible".to_string(),
            crate::resource::resource_register::ResourceLogic::NonFungible => "non-fungible".to_string(),
            crate::resource::resource_register::ResourceLogic::Capability => "capability".to_string(),
            crate::resource::resource_register::ResourceLogic::Data => "data".to_string(),
        };
        
        // Extract name/domain from metadata
        let name = register.metadata.get("domain")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
            
        // Extract version from metadata
        let version = register.metadata.get("version")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1);
            
        Self {
            name,
            resource_type,
            data: register.contents.clone(),
            metadata: register.metadata.clone(),
            version,
            inner: Some(register.clone()),
        }
    }
}

impl ContentAddressed for Resource {
    fn content_hash(&self) -> HashOutput {
        // Get the configured hasher from the registry
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization of the resource
        let data = self.try_to_vec().unwrap();
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// A registry for content-addressed resources
///
/// MIGRATION NOTE: This struct is being phased out in favor of methods 
/// provided directly by the ResourceRegister functionality.
pub struct ResourceRegistry {
    /// Resources indexed by their content ID
    resources: HashMap<ContentId, Resource>,
    /// ResourceRegisters indexed by their content ID (migration path)
    registers: HashMap<ContentId, ResourceRegister>,
}

impl ResourceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            registers: HashMap::new(),
        }
    }
    
    /// Register a resource
    ///
    /// MIGRATION NOTE: This method now automatically converts the Resource
    /// to a ResourceRegister internally. Consider using register_register directly.
    pub fn register(&mut self, resource: Resource) -> ContentId {
        let content_id = resource.content_id();
        
        // Store in both collections during migration
        let register = resource.to_resource_register();
        self.resources.insert(content_id.clone(), resource);
        self.registers.insert(content_id.clone(), register);
        
        content_id
    }
    
    /// Register a ResourceRegister directly
    ///
    /// This is the preferred method to use during and after migration
    pub fn register_register(&mut self, register: ResourceRegister) -> ContentId {
        let content_id = register.content_id();
        
        // During migration, we maintain both collections
        let resource = Resource::from_resource_register(&register);
        self.resources.insert(content_id.clone(), resource);
        self.registers.insert(content_id.clone(), register);
        
        content_id
    }
    
    /// Get a resource by its content ID
    ///
    /// MIGRATION NOTE: Consider using get_register instead where possible
    pub fn get(&self, content_id: &ContentId) -> Option<&Resource> {
        self.resources.get(content_id)
    }
    
    /// Get a ResourceRegister by its content ID
    ///
    /// This is the preferred method to use during and after migration
    pub fn get_register(&self, content_id: &ContentId) -> Option<&ResourceRegister> {
        self.registers.get(content_id)
    }
    
    /// Remove a resource by its content ID
    pub fn remove(&mut self, content_id: &ContentId) -> Option<Resource> {
        // Remove from both collections
        self.registers.remove(content_id);
        self.resources.remove(content_id)
    }
    
    /// Check if the registry contains a resource
    pub fn contains(&self, content_id: &ContentId) -> bool {
        self.registers.contains_key(content_id)
    }
    
    /// Get the number of resources in the registry
    pub fn len(&self) -> usize {
        self.registers.len()
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.registers.is_empty()
    }
    
    /// Get an iterator over all resources
    pub fn iter(&self) -> impl Iterator<Item = (&ContentId, &Resource)> {
        self.resources.iter()
    }
    
    /// Get an iterator over all registers
    ///
    /// This is the preferred method to use during and after migration
    pub fn iter_registers(&self) -> impl Iterator<Item = (&ContentId, &ResourceRegister)> {
        self.registers.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_content_addressing() {
        // Create a resource
        let mut resource = Resource::new("test", "document", b"resource data".to_vec());
        resource.set_metadata("author", "Test User");
        
        // Get the content hash
        let hash = resource.content_hash();
        
        // Create an identical resource
        let mut resource2 = Resource::new("test", "document", b"resource data".to_vec());
        resource2.set_metadata("author", "Test User");
        
        // The content hashes should be identical
        assert_eq!(hash, resource2.content_hash());
        
        // Create a different resource
        let resource3 = Resource::new("different", "document", b"different data".to_vec());
        
        // The content hash should be different
        assert_ne!(hash, resource3.content_hash());
        
        // Test serialization and deserialization
        let bytes = resource.to_bytes();
        let deserialized = Resource::from_bytes(&bytes).unwrap();
        assert_eq!(resource, deserialized);
        
        // Test verification
        assert!(resource.verify());
        assert!(resource2.verify());
        assert!(resource3.verify());
    }
    
    #[test]
    fn test_resource_content_id() {
        // Create a resource
        let resource = Resource::new("test", "document", b"resource data".to_vec());
        
        // Get the content ID
        let content_id = resource.content_id();
        
        // The content ID should be derived from the content hash
        assert_eq!(content_id.hash(), &resource.content_hash());
        
        // Test string representation
        let id_str = content_id.to_string();
        assert!(id_str.starts_with("cid:"));
        
        // Test parsing
        let parsed_id = ContentId::parse(&id_str).unwrap();
        assert_eq!(content_id, parsed_id);
    }
    
    #[test]
    fn test_resource_registry() {
        // Create a registry
        let mut registry = ResourceRegistry::new();
        
        // Create some resources
        let resource1 = Resource::new("res1", "document", b"data1".to_vec());
        let resource2 = Resource::new("res2", "document", b"data2".to_vec());
        
        // Register the resources
        let id1 = registry.register(resource1.clone());
        let id2 = registry.register(resource2.clone());
        
        // Check that the registry contains the resources
        assert!(registry.contains(&id1));
        assert!(registry.contains(&id2));
        
        // Get the resources
        let retrieved1 = registry.get(&id1).unwrap();
        let retrieved2 = registry.get(&id2).unwrap();
        
        // Check that the retrieved resources match the originals
        assert_eq!(retrieved1, &resource1);
        assert_eq!(retrieved2, &resource2);
        
        // Check that the registry has the correct number of resources
        assert_eq!(registry.len(), 2);
        
        // Remove a resource
        let removed = registry.remove(&id1).unwrap();
        assert_eq!(removed, resource1);
        
        // Check that the resource is no longer in the registry
        assert!(!registry.contains(&id1));
        assert_eq!(registry.len(), 1);
    }
    
    #[test]
    fn test_resource_versioning() {
        // Create a resource
        let mut resource = Resource::new("test", "document", b"initial data".to_vec());
        assert_eq!(resource.version(), 1);
        
        // Update the resource data
        resource.set_data(b"updated data".to_vec());
        assert_eq!(resource.version(), 2);
        
        // Create an updated version
        let updated = resource.update(b"new data".to_vec());
        assert_eq!(updated.version(), 3);
        assert_eq!(resource.version(), 2); // Original unchanged
        
        // Check data
        assert_eq!(updated.data(), b"new data");
        assert_eq!(resource.data(), b"updated data");
    }
    
    #[test]
    fn test_migration_compatibility() {
        // Create a resource using the old way
        let mut resource = Resource::new("test", "document", b"resource data".to_vec());
        resource.set_metadata("author", "Test User");
        
        // Convert to ResourceRegister
        let register = resource.to_resource_register();
        
        // Verify the conversion preserved key properties
        assert_eq!(register.domain(), "test");
        assert_eq!(register.resource_type(), "document");
        assert_eq!(register.data(), b"resource data");
        assert_eq!(register.metadata().get("author"), Some(&"Test User".to_string()));
        
        // Convert back to Resource
        let round_trip = Resource::from_resource_register(&register);
        
        // Verify essential properties are preserved
        assert_eq!(round_trip.name(), resource.name());
        assert_eq!(round_trip.resource_type(), resource.resource_type());
        assert_eq!(round_trip.data(), resource.data());
        assert_eq!(round_trip.get_metadata("author"), resource.get_metadata("author"));
    }
    
    #[test]
    fn test_registry_with_registers() {
        // Create a registry
        let mut registry = ResourceRegistry::new();
        
        // Create register directly
        let register = create_resource_register("domain1", "document", b"data1".to_vec());
        
        // Register using the new way
        let id = registry.register_register(register.clone());
        
        // Verify it can be retrieved both ways during migration
        let retrieved_register = registry.get_register(&id).unwrap();
        let retrieved_resource = registry.get(&id).unwrap();
        
        assert_eq!(retrieved_register.domain(), "domain1");
        assert_eq!(retrieved_resource.name(), "domain1");
        assert_eq!(retrieved_register.data(), retrieved_resource.data());
    }
} 