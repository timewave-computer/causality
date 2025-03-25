// Unified registry for resource management
// Original file: src/resource/unified_registry.rs

// Unified Registry for ResourceRegister
//
// This module provides a registry for ResourceRegister instances
// that consolidates functionality from both ResourceRegistry and
// RegisterRegistry into a single, simplified implementation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use causality_crypto::{ContentId, ContentAddressed};
use causality_types::{Error, Result};
use causality_resource::{ResourceRegister, RegisterState};
use causality_resource::RelationshipTracker;
use causality_resource_manager::ResourceRegisterLifecycleManager;

/// A unified registry for ResourceRegister instances
///
/// This registry replaces both ResourceRegistry and RegisterRegistry
/// with a single implementation that exclusively uses ResourceRegister.
pub struct UnifiedRegistry {
    /// ResourceRegisters indexed by their content ID
    registers: HashMap<ContentId, ResourceRegister>,
    /// Lifecycle manager for ResourceRegisters
    lifecycle_manager: Option<Arc<ResourceRegisterLifecycleManager>>,
    /// Relationship tracker for ResourceRegisters
    relationship_tracker: Option<Arc<RelationshipTracker>>,
}

impl UnifiedRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            lifecycle_manager: None,
            relationship_tracker: None,
        }
    }
    
    /// Create a new registry with a lifecycle manager
    pub fn with_lifecycle_manager(lifecycle_manager: Arc<ResourceRegisterLifecycleManager>) -> Self {
        Self {
            registers: HashMap::new(),
            lifecycle_manager: Some(lifecycle_manager),
            relationship_tracker: None,
        }
    }
    
    /// Create a new registry with a relationship tracker
    pub fn with_relationship_tracker(relationship_tracker: Arc<RelationshipTracker>) -> Self {
        Self {
            registers: HashMap::new(),
            lifecycle_manager: None,
            relationship_tracker: Some(relationship_tracker),
        }
    }
    
    /// Create a new registry with both lifecycle manager and relationship tracker
    pub fn with_lifecycle_and_relationships(
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
    ) -> Self {
        Self {
            registers: HashMap::new(),
            lifecycle_manager: Some(lifecycle_manager),
            relationship_tracker: Some(relationship_tracker),
        }
    }
    
    /// Get a shared instance of the registry
    pub fn shared() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new()))
    }
    
    /// Register a ResourceRegister
    pub fn register(&mut self, register: ResourceRegister) -> Result<ContentId> {
        let content_id = register.content_id();
        
        // If we have a lifecycle manager, register with it
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            lifecycle_manager.register_resource(content_id.clone())
                .map_err(|e| Error::ResourceError(format!("Failed to register resource: {}", e)))?;
        }
        
        // Store in our registry
        self.registers.insert(content_id.clone(), register);
        
        Ok(content_id)
    }
    
    /// Get a ResourceRegister by its content ID
    pub fn get(&self, content_id: &ContentId) -> Result<Option<ResourceRegister>> {
        match self.registers.get(content_id) {
            Some(register) => Ok(Some(register.clone())),
            None => {
                // If not in our registry, try to get from lifecycle manager
                if let Some(lifecycle_manager) = &self.lifecycle_manager {
                    match lifecycle_manager.get_register(content_id) {
                        Ok(register) => {
                            // Got the register from lifecycle manager, return it
                            Ok(Some(register))
                        },
                        Err(e) => Err(Error::ResourceError(format!("Failed to get register: {}", e))),
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    /// Get all ResourceRegisters
    pub fn get_all(&self) -> Result<HashMap<ContentId, ResourceRegister>> {
        Ok(self.registers.clone())
    }
    
    /// Update a ResourceRegister
    pub fn update(&mut self, content_id: &ContentId, update_fn: impl FnOnce(&mut ResourceRegister) -> Result<()>) -> Result<()> {
        // Get the register
        let register = match self.get(content_id)? {
            Some(register) => register,
            None => return Err(Error::ResourceNotFound(content_id.clone())),
        };
        
        // Apply the update
        let mut register_clone = register.clone();
        update_fn(&mut register_clone)?;
        
        // If we have a lifecycle manager, update the state if it changed
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            if register.state != register_clone.state {
                lifecycle_manager.update_state(content_id, register_clone.state.clone())
                    .map_err(|e| Error::ResourceError(format!("Failed to update state: {}", e)))?;
            }
        }
        
        // Store the updated register
        self.registers.insert(content_id.clone(), register_clone);
        
        Ok(())
    }
    
    /// Check if a ResourceRegister exists
    pub fn contains(&self, content_id: &ContentId) -> Result<bool> {
        if self.registers.contains_key(content_id) {
            return Ok(true);
        }
        
        // If not in our registry, check lifecycle manager
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            match lifecycle_manager.get_state(content_id) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
    
    /// Consume a ResourceRegister
    pub fn consume(&mut self, content_id: &ContentId) -> Result<()> {
        self.update(content_id, |register| {
            register.state = RegisterState::Consumed;
            Ok(())
        })
    }
    
    /// Remove a ResourceRegister
    pub fn remove(&mut self, content_id: &ContentId) -> Result<Option<ResourceRegister>> {
        // First check if it exists
        if !self.contains(content_id)? {
            return Ok(None);
        }
        
        // Remove from registry
        let register = self.registers.remove(content_id);
        
        // If we have a lifecycle manager, remove from it too
        if let Some(lifecycle_manager) = &self.lifecycle_manager {
            // We don't propagate lifecycle manager errors here, as the register
            // has already been removed from our registry
            let _ = lifecycle_manager.remove_resource(content_id);
        }
        
        // If we have a relationship tracker, clean up relationships
        if let Some(relationship_tracker) = &self.relationship_tracker {
            // We don't propagate relationship tracker errors here, as the register
            // has already been removed from our registry
            let _ = relationship_tracker.clear_relationships_for(content_id);
        }
        
        Ok(register)
    }
    
    /// Get the number of registers in the registry
    pub fn len(&self) -> usize {
        self.registers.len()
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.registers.is_empty()
    }
    
    /// Clear the registry
    pub fn clear(&mut self) {
        self.registers.clear();
    }
}

// Implement the Clone trait for UnifiedRegistry
impl Clone for UnifiedRegistry {
    fn clone(&self) -> Self {
        Self {
            registers: self.registers.clone(),
            lifecycle_manager: self.lifecycle_manager.clone(),
            relationship_tracker: self.relationship_tracker.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_resource::{ResourceLogic, FungibilityDomain, Quantity, StorageStrategy, StateVisibility};
    use causality_tel::Metadata;
    
    #[test]
    fn test_unified_registry_basic() {
        // Create a registry
        let mut registry = UnifiedRegistry::new();
        
        // Create some ResourceRegisters
        let register1 = ResourceRegister::new(
            ContentId::new("test-register-1"),
            ResourceLogic::Fungible,
            FungibilityDomain("ETH".to_string()),
            Quantity(100),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        let register2 = ResourceRegister::new(
            ContentId::new("test-register-2"),
            ResourceLogic::NonFungible,
            FungibilityDomain("NFT".to_string()),
            Quantity(1),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        // Register the ResourceRegisters
        let id1 = registry.register(register1.clone()).unwrap();
        let id2 = registry.register(register2.clone()).unwrap();
        
        // Check that the registry contains the ResourceRegisters
        assert!(registry.contains(&id1).unwrap());
        assert!(registry.contains(&id2).unwrap());
        
        // Get the ResourceRegisters
        let retrieved1 = registry.get(&id1).unwrap().unwrap();
        let retrieved2 = registry.get(&id2).unwrap().unwrap();
        
        // Check that the retrieved ResourceRegisters match the originals
        assert_eq!(retrieved1.id, register1.id);
        assert_eq!(retrieved2.id, register2.id);
        
        // Update a ResourceRegister
        registry.update(&id1, |register| {
            register.quantity = Quantity(200);
            Ok(())
        }).unwrap();
        
        // Check that the update was applied
        let updated = registry.get(&id1).unwrap().unwrap();
        assert_eq!(updated.quantity, Quantity(200));
        
        // Check length
        assert_eq!(registry.len(), 2);
        
        // Remove a ResourceRegister
        let removed = registry.remove(&id1).unwrap().unwrap();
        assert_eq!(removed.id, register1.id);
        
        // Check that it was removed
        assert!(!registry.contains(&id1).unwrap());
        assert_eq!(registry.len(), 1);
        
        // Consume a ResourceRegister
        registry.consume(&id2).unwrap();
        
        // Check that it was consumed
        let consumed = registry.get(&id2).unwrap().unwrap();
        assert_eq!(consumed.state, RegisterState::Consumed);
        
        // Clear the registry
        registry.clear();
        
        // Check that it's empty
        assert!(registry.is_empty());
    }
    
    #[test]
    fn test_get_all_registers() {
        // Create a registry
        let mut registry = UnifiedRegistry::new();
        
        // Create some ResourceRegisters
        let register1 = ResourceRegister::new(
            ContentId::new("test-register-1"),
            ResourceLogic::Fungible,
            FungibilityDomain("ETH".to_string()),
            Quantity(100),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        let register2 = ResourceRegister::new(
            ContentId::new("test-register-2"),
            ResourceLogic::NonFungible,
            FungibilityDomain("NFT".to_string()),
            Quantity(1),
            Metadata::default(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        );
        
        // Register the ResourceRegisters
        let id1 = registry.register(register1.clone()).unwrap();
        let id2 = registry.register(register2.clone()).unwrap();
        
        // Get all registers
        let all_registers = registry.get_all().unwrap();
        
        // Check that we got all registers
        assert_eq!(all_registers.len(), 2);
        assert!(all_registers.contains_key(&id1));
        assert!(all_registers.contains_key(&id2));
    }
} 