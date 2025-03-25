// Resource lifecycle management
// Original file: src/resource/lifecycle.rs

// Register Lifecycle and State Management for ResourceRegister
//
// This module implements lifecycle management for the unified ResourceRegister model
// as defined in ADR-021. It leverages the core functionality from lifecycle_manager.rs
// and provides a simpler interface specifically for ResourceRegister operations.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use causality_types::{Error, Result};
use causality_types::Metadata;
use causality_crypto::ContentId;
use causality_resource_manager::{ResourceRegisterLifecycleManager, StateTransitionRecord, RegisterOperationType};
use causality_resource::{ResourceRegister, RegisterState};

/// Reason for a state transition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionReason {
    /// User requested the transition
    UserRequested,
    
    /// System automatically performed the transition
    SystemAutomatic,
    
    /// Part of a batch operation
    BatchOperation(String),
    
    /// Related to another resource
    RelatedResource(ContentId),
    
    /// Triggered by a timer or deadline
    TimerExpired,
    
    /// Security policy enforcement
    SecurityPolicy,
    
    /// Consensus decision
    ConsensusDecision,
    
    /// Custom reason
    Custom(String),
}

/// ResourceLifecycle provides lifecycle management for ResourceRegister instances
pub struct ResourceLifecycle {
    /// The lifecycle manager that handles state transitions
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
}

impl ResourceLifecycle {
    /// Create a new resource lifecycle manager
    pub fn new() -> Self {
        Self {
            lifecycle_manager: Arc::new(ResourceRegisterLifecycleManager::new()),
        }
    }
    
    /// Create a new resource lifecycle with an existing lifecycle manager
    pub fn with_manager(lifecycle_manager: Arc<ResourceRegisterLifecycleManager>) -> Self {
        Self {
            lifecycle_manager,
        }
    }
    
    /// Register a resource in the lifecycle manager
    pub fn register(&self, resource: &ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        Arc::clone(&self.lifecycle_manager)
            .register_resource(resource_id)
    }
    
    /// Activate a resource
    pub fn activate(&self, resource: &mut ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.activate(&resource_id)?;
        resource.state = RegisterState::Active;
        
        Ok(())
    }
    
    /// Lock a resource
    pub fn lock(&self, resource: &mut ResourceRegister, locker_id: Option<&ContentId>) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.lock(&resource_id, locker_id)?;
        resource.state = RegisterState::Locked;
        
        // Store the locker ID in the resource if provided
        let locker_resource_id = locker_id.cloned();
        if let Some(locker_id) = locker_resource_id {
            resource.controller = Some(locker_id);
        }
        
        Ok(())
    }
    
    /// Unlock a resource
    pub fn unlock(&self, resource: &mut ResourceRegister, unlocker_id: Option<&ContentId>) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.unlock(&resource_id, unlocker_id)?;
        resource.state = RegisterState::Active;
        
        // Remove the controller if it matches the unlocker
        let unlocker_resource_id = unlocker_id.cloned();
        if let (Some(unlocker_id), Some(controller)) = (unlocker_resource_id, &resource.controller) {
            if unlocker_id == *controller {
                resource.controller = None;
            }
        }
        
        Ok(())
    }
    
    /// Freeze a resource
    pub fn freeze(&self, resource: &mut ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.freeze(&resource_id)?;
        resource.state = RegisterState::Frozen;
        
        Ok(())
    }
    
    /// Unfreeze a resource
    pub fn unfreeze(&self, resource: &mut ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.unfreeze(&resource_id)?;
        resource.state = RegisterState::Active;
        
        Ok(())
    }
    
    /// Mark a resource as pending
    pub fn mark_pending(&self, resource: &mut ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.mark_pending(&resource_id)?;
        resource.state = RegisterState::Pending;
        
        Ok(())
    }
    
    /// Consume a resource
    pub fn consume(&self, resource: &mut ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.consume(&resource_id)?;
        resource.state = RegisterState::Consumed;
        
        Ok(())
    }
    
    /// Archive a resource
    pub fn archive(&self, resource: &mut ResourceRegister) -> Result<()> {
        let resource_id = resource.id.clone();
        
        // Update the resource state and lifecycle manager
        let mut lifecycle_manager = Arc::clone(&self.lifecycle_manager);
        lifecycle_manager.archive(&resource_id)?;
        resource.state = RegisterState::Archived;
        
        Ok(())
    }
    
    /// Check if a resource can perform a specific operation in its current state
    pub fn can_perform_operation(&self, resource: &ResourceRegister, operation: &ResourceOperation) -> Result<bool> {
        let resource_id = resource.id.clone();
        
        // Convert ResourceOperation to RegisterOperationType
        let operation_type = match operation {
            ResourceOperation::Create => Ok(causality_resource_manager::RegisterOperationType::Create),
            ResourceOperation::Update => Ok(causality_resource_manager::RegisterOperationType::Update),
            ResourceOperation::Lock => Ok(causality_resource_manager::RegisterOperationType::Lock),
            ResourceOperation::Unlock => Ok(causality_resource_manager::RegisterOperationType::Unlock),
            ResourceOperation::Freeze => Ok(causality_resource_manager::RegisterOperationType::Freeze),
            ResourceOperation::Unfreeze => Ok(causality_resource_manager::RegisterOperationType::Unfreeze),
            ResourceOperation::Consume => Ok(causality_resource_manager::RegisterOperationType::Consume),
            ResourceOperation::Archive => Ok(causality_resource_manager::RegisterOperationType::Archive),
        }?;
        
        // Check if the operation is valid
        self.lifecycle_manager.is_operation_valid(&resource_id, &operation_type)
    }
    
    /// Get the transition history for a resource
    pub fn get_transition_history(&self, resource: &ResourceRegister) -> Result<Vec<StateTransitionRecord>> {
        let resource_id = resource.id.clone();
        
        self.lifecycle_manager.get_transition_history(&resource_id)
    }
}

/// Operations that can be performed on a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOperation {
    /// Create a new resource
    Create,
    
    /// Update an existing resource
    Update,
    
    /// Lock a resource
    Lock,
    
    /// Unlock a resource
    Unlock,
    
    /// Freeze a resource
    Freeze,
    
    /// Unfreeze a resource
    Unfreeze,
    
    /// Consume a resource
    Consume,
    
    /// Archive a resource
    Archive,
}

/// Factory for creating ResourceLifecycle instances
pub struct ResourceLifecycleFactory {
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
}

impl ResourceLifecycleFactory {
    /// Create a new factory with a shared lifecycle manager
    pub fn new() -> Self {
        Self {
            lifecycle_manager: Arc::new(ResourceRegisterLifecycleManager::new()),
        }
    }
    
    /// Create a new ResourceLifecycle that shares the same underlying lifecycle manager
    pub fn create_lifecycle(&self) -> ResourceLifecycle {
        ResourceLifecycle::with_manager(Arc::clone(&self.lifecycle_manager))
    }
}

impl Default for ResourceLifecycleFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_resource::{ResourceRegister, ResourceLogic, FungibilityDomain, Quantity};
    use causality_resource::StorageStrategy;
    
    #[test]
    fn test_resource_lifecycle_basic() -> Result<()> {
        // Create a lifecycle manager
        let lifecycle = ResourceLifecycle::new();
        
        // Create a test resource
        let mut resource = ResourceRegister::new(
            ContentId::nil(),
            ResourceLogic::Fungible,
            FungibilityDomain::new("token"),
            Quantity::new(100),
            Metadata::default(),
            StorageStrategy::fully_on_chain(),
        );
        
        // Register and activate the resource
        lifecycle.register(&resource)?;
        lifecycle.activate(&mut resource)?;
        
        // Verify state is active
        assert_eq!(resource.state, RegisterState::Active);
        
        // Lock the resource
        lifecycle.lock(&mut resource, None)?;
        
        // Verify state is locked
        assert_eq!(resource.state, RegisterState::Locked);
        
        // Unlock the resource
        lifecycle.unlock(&mut resource, None)?;
        
        // Verify state is active again
        assert_eq!(resource.state, RegisterState::Active);
        
        // Check operation permissions
        assert!(lifecycle.can_perform_operation(&resource, &ResourceOperation::Freeze)?);
        assert!(!lifecycle.can_perform_operation(&resource, &ResourceOperation::Unlock)?);
        
        // Consume the resource
        lifecycle.consume(&mut resource)?;
        
        // Verify state is consumed
        assert_eq!(resource.state, RegisterState::Consumed);
        
        // Verify that activation is no longer possible
        assert!(!lifecycle.can_perform_operation(&resource, &ResourceOperation::Update)?);
        
        Ok(())
    }
    
    #[test]
    fn test_resource_lifecycle_factory() -> Result<()> {
        // Create a factory
        let factory = ResourceLifecycleFactory::new();
        
        // Create two lifecycle managers
        let lifecycle1 = factory.create_lifecycle();
        let lifecycle2 = factory.create_lifecycle();
        
        // Create test resources
        let mut resource1 = ResourceRegister::new(
            ContentId::nil(),
            ResourceLogic::Fungible,
            FungibilityDomain::new("token1"),
            Quantity::new(100),
            Metadata::default(),
            StorageStrategy::fully_on_chain(),
        );
        
        let mut resource2 = ResourceRegister::new(
            ContentId::nil(),
            ResourceLogic::Fungible,
            FungibilityDomain::new("token2"),
            Quantity::new(200),
            Metadata::default(),
            StorageStrategy::fully_on_chain(),
        );
        
        // Register and activate both resources
        lifecycle1.register(&resource1)?;
        lifecycle1.activate(&mut resource1)?;
        
        lifecycle2.register(&resource2)?;
        lifecycle2.activate(&mut resource2)?;
        
        // Lock resource2 with resource1
        lifecycle2.lock(&mut resource2, Some(&resource1.id))?;
        
        // Verify states
        assert_eq!(resource1.state, RegisterState::Active);
        assert_eq!(resource2.state, RegisterState::Locked);
        
        Ok(())
    }
}
