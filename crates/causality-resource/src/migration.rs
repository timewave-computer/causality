// Resource migration system
// Original file: src/resource/migrate_adapter.rs

// Migration adapters for the ResourceRegister unification
//
// This module provides adapter classes to help migrate code from the
// old Resource + Register model to the new unified ResourceRegister model.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use causality_crypto::ContentId;
use causality_types::{Error, Result};
use causality_resource::{
    ResourceRegister, RegisterState, ResourceLogic,
    FungibilityDomain, Quantity, StateVisibility, StorageStrategy
};
use causality_resource::Resource;
use causality_resource::UnifiedRegistry;
use causality_tel::Metadata;

/// A migration adapter for ResourceRegistry
///
/// This adapter wraps a UnifiedRegistry instance and implements the
/// functionality of the old ResourceRegistry using the new unified system.
/// Use this to help migrate code that depends on ResourceRegistry.
pub struct ResourceToRegisterAdapter {
    /// The underlying unified registry
    registry: Arc<RwLock<UnifiedRegistry>>,
}

impl ResourceToRegisterAdapter {
    /// Create a new resource registry adapter around a UnifiedRegistry
    pub fn new(registry: Arc<RwLock<UnifiedRegistry>>) -> Self {
        Self {
            registry,
        }
    }
    
    /// Create a standalone adapter with its own registry
    pub fn standalone() -> Self {
        Self {
            registry: UnifiedRegistry::shared(),
        }
    }
    
    /// Register a resource
    pub fn register(&self, resource: Resource) -> Result<ContentId> {
        // Convert Resource to ResourceRegister
        let register = resource.to_resource_register();
        
        // Register with the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.register(register)
    }
    
    /// Get a resource by ID
    pub fn get(&self, id: &ContentId) -> Result<Option<Resource>> {
        // Get the register from the unified registry
        let registry = self.registry.read().map_err(|_| 
            Error::ResourceError("Failed to acquire read lock on registry".to_string()))?;
        
        match registry.get(id)? {
            Some(register) => {
                // Convert ResourceRegister to Resource
                let resource = Resource::from_resource_register(&register);
                Ok(Some(resource))
            },
            None => Ok(None),
        }
    }
    
    /// Update a resource
    pub fn update(&self, id: &ContentId, resource: Resource) -> Result<()> {
        // Convert Resource to ResourceRegister
        let register = resource.to_resource_register();
        
        // Update the register in the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.update(id, |r| {
            *r = register.clone();
            Ok(())
        })
    }
    
    /// Remove a resource
    pub fn remove(&self, id: &ContentId) -> Result<Option<Resource>> {
        // Remove the register from the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        match registry.remove(id)? {
            Some(register) => {
                // Convert ResourceRegister to Resource
                let resource = Resource::from_resource_register(&register);
                Ok(Some(resource))
            },
            None => Ok(None),
        }
    }
    
    /// Get the underlying UnifiedRegistry
    pub fn get_unified_registry(&self) -> Arc<RwLock<UnifiedRegistry>> {
        self.registry.clone()
    }
}

/// A migration adapter for RegisterRegistry
///
/// This adapter wraps a UnifiedRegistry instance and implements the
/// functionality of the old RegisterRegistry using the new unified system.
/// Use this to help migrate code that depends on RegisterRegistry.
pub struct RegisterSystemAdapter {
    /// The underlying unified registry
    registry: Arc<RwLock<UnifiedRegistry>>,
}

impl RegisterSystemAdapter {
    /// Create a new register registry adapter around a UnifiedRegistry
    pub fn new(registry: Arc<RwLock<UnifiedRegistry>>) -> Self {
        Self {
            registry,
        }
    }
    
    /// Create a standalone adapter with its own registry
    pub fn standalone() -> Self {
        Self {
            registry: UnifiedRegistry::shared(),
        }
    }
    
    /// Register a register
    pub fn register(&self, register: ResourceRegister) -> Result<ContentId> {
        // Register with the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.register(register)
    }
    
    /// Get a register by ID
    pub fn get(&self, id: &ContentId) -> Result<Option<ResourceRegister>> {
        // Get the register from the unified registry
        let registry = self.registry.read().map_err(|_| 
            Error::ResourceError("Failed to acquire read lock on registry".to_string()))?;
        
        registry.get(id)
    }
    
    /// Update a register
    pub fn update(&self, id: &ContentId, register: ResourceRegister) -> Result<()> {
        // Update the register in the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.update(id, |r| {
            *r = register.clone();
            Ok(())
        })
    }
    
    /// Update a register's state
    pub fn update_state(&self, id: &ContentId, new_state: RegisterState) -> Result<()> {
        // Update the register's state in the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.update(id, |register| {
            register.state = new_state;
            Ok(())
        })
    }
    
    /// Consume a register
    pub fn consume(&self, id: &ContentId) -> Result<()> {
        // Consume the register in the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.consume(id)
    }
    
    /// Remove a register
    pub fn remove(&self, id: &ContentId) -> Result<Option<ResourceRegister>> {
        // Remove the register from the unified registry
        let mut registry = self.registry.write().map_err(|_| 
            Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
        
        registry.remove(id)
    }
    
    /// Get the underlying UnifiedRegistry
    pub fn get_unified_registry(&self) -> Arc<RwLock<UnifiedRegistry>> {
        self.registry.clone()
    }
}

/// A unified adapter that implements both ResourceRegistry and RegisterRegistry interfaces
///
/// This adapter provides a single interface that can replace both the old
/// ResourceRegistry and RegisterRegistry in code that uses both.
pub struct MigrationAdapter {
    /// The resource registry adapter
    resource_adapter: ResourceToRegisterAdapter,
    /// The register registry adapter
    register_adapter: RegisterSystemAdapter,
}

impl MigrationAdapter {
    /// Create a new unified adapter around a UnifiedRegistry
    pub fn new(registry: Arc<RwLock<UnifiedRegistry>>) -> Self {
        Self {
            resource_adapter: ResourceToRegisterAdapter::new(registry.clone()),
            register_adapter: RegisterSystemAdapter::new(registry),
        }
    }
    
    /// Create a standalone adapter with its own registry
    pub fn standalone() -> Self {
        let registry = UnifiedRegistry::shared();
        Self {
            resource_adapter: ResourceToRegisterAdapter::new(registry.clone()),
            register_adapter: RegisterSystemAdapter::new(registry),
        }
    }
    
    /// Get the resource registry adapter
    pub fn as_resource_registry(&self) -> &ResourceToRegisterAdapter {
        &self.resource_adapter
    }
    
    /// Get the register registry adapter
    pub fn as_register_registry(&self) -> &RegisterSystemAdapter {
        &self.register_adapter
    }
    
    /// Get the underlying UnifiedRegistry
    pub fn get_unified_registry(&self) -> Arc<RwLock<UnifiedRegistry>> {
        self.resource_adapter.get_unified_registry()
    }
    
    /// Get all registers from the unified registry
    pub fn get_all_registers(&self) -> Result<HashMap<ContentId, ResourceRegister>> {
        let registry = self.get_unified_registry();
        let registry_guard = registry.read().map_err(|_| 
            Error::ResourceError("Failed to acquire read lock on registry".to_string()))?;
        
        registry_guard.get_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_resource::{ResourceLogic, FungibilityDomain, Quantity, StorageStrategy, StateVisibility};
    use causality_tel::Metadata;
    
    #[test]
    fn test_resource_registry_adapter() -> Result<()> {
        // Create a unified registry
        let registry = UnifiedRegistry::shared();
        
        // Create an adapter
        let adapter = ResourceToRegisterAdapter::new(registry);
        
        // Create a resource
        let resource = Resource::new(
            "test-resource",
            "test-type",
            Vec::new()
        );
        
        // Register the resource
        let id = adapter.register(resource.clone())?;
        
        // Get the resource
        let retrieved = adapter.get(&id)?.unwrap();
        
        // Verify it matches
        assert_eq!(retrieved.name(), resource.name());
        
        // Remove the resource
        let removed = adapter.remove(&id)?.unwrap();
        
        // Verify it matches
        assert_eq!(removed.name(), resource.name());
        
        Ok(())
    }
    
    #[test]
    fn test_register_registry_adapter() -> Result<()> {
        // Create a unified registry
        let registry = UnifiedRegistry::shared();
        
        // Create an adapter
        let adapter = RegisterSystemAdapter::new(registry);
        
        // Create a register
        let register = ResourceRegister::new(
            ContentId::new("test-register"),
            ResourceLogic::Fungible,
            FungibilityDomain("TEST".to_string()),
            Quantity(100),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        // Register the register
        let id = adapter.register(register.clone())?;
        
        // Get the register
        let retrieved = adapter.get(&id)?.unwrap();
        
        // Verify it matches
        assert_eq!(retrieved.id, register.id);
        
        // Update the register's state
        adapter.update_state(&id, RegisterState::Consumed)?;
        
        // Get the register again
        let updated = adapter.get(&id)?.unwrap();
        
        // Verify the state was updated
        assert_eq!(updated.state, RegisterState::Consumed);
        
        // Remove the register
        let removed = adapter.remove(&id)?.unwrap();
        
        // Verify it matches
        assert_eq!(removed.id, register.id);
        
        Ok(())
    }
    
    #[test]
    fn test_unified_registry_adapter() -> Result<()> {
        // Create a unified adapter
        let adapter = MigrationAdapter::standalone();
        
        // Create a resource
        let resource = Resource::new(
            "test-resource",
            "test-type",
            Vec::new()
        );
        
        // Register the resource using the resource adapter
        let resource_id = adapter.as_resource_registry().register(resource.clone())?;
        
        // Create a register
        let register = ResourceRegister::new(
            ContentId::new("test-register"),
            ResourceLogic::Fungible,
            FungibilityDomain("TEST".to_string()),
            Quantity(200),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        // Register the register using the register adapter
        let register_id = adapter.as_register_registry().register(register.clone())?;
        
        // Verify we can get both from the appropriate adapter
        let retrieved_resource = adapter.as_resource_registry().get(&resource_id)?.unwrap();
        let retrieved_register = adapter.as_register_registry().get(&register_id)?.unwrap();
        
        assert_eq!(retrieved_resource.name(), resource.name());
        assert_eq!(retrieved_register.id, register.id);
        
        // Verify they were both stored in the same underlying registry
        let registry = adapter.get_unified_registry();
        let registry_guard = registry.read().unwrap();
        
        assert!(registry_guard.contains(&resource_id).unwrap());
        assert!(registry_guard.contains(&register_id).unwrap());
        
        // Count total registers in the registry
        assert_eq!(registry_guard.len(), 2);
        
        Ok(())
    }
} 