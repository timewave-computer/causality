// Tests for integration between effect templates and resource lifecycle manager
//
// These tests validate that the effect templates correctly interact with the
// resource lifecycle manager and that constraints are properly enforced.

use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use crate::address::Address;
use crate::error::Result;
use crate::types::{ResourceId, DomainId, Metadata};
use crate::resource::{
    ResourceRegister,
    RegisterState,
    RegisterOperationType,
    ResourceManager,
    ResourceRegisterLifecycleManager,
    RelationshipTracker,
    RelationshipType,
    FungibilityDomain,
    ResourceLogic,
    Quantity,
    StorageStrategy,
    StateVisibility,
};
use crate::effect::{Effect, ExecutionBoundary, EffectContext, EffectOutcome};
use crate::effect::templates::{
    create_resource_effect,
    update_resource_effect,
    lock_resource_effect,
    unlock_resource_effect,
    consume_resource_effect,
    transfer_resource_effect,
    freeze_resource_effect,
    unfreeze_resource_effect,
    archive_resource_effect,
    create_resource_with_boundary_effect,
    cross_domain_resource_effect,
    resource_operation_with_capability_effect,
    resource_operation_with_timemap_effect,
    resource_operation_with_commitment_effect,
};
use crate::time::TimeMapSnapshot;

// Helper to create a test resource
fn create_test_resource(id: &str) -> ResourceRegister {
    ResourceRegister::new(
        ResourceId::from(id.to_string()),
        ResourceLogic::Fungible,
        FungibilityDomain::new("test_token"),
        Quantity::new(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain {
            visibility: StateVisibility::Public,
        },
    )
}

// Helper to create a test domain ID
fn create_test_domain() -> DomainId {
    DomainId::from("test_domain".to_string())
}

// Helper to create a test address
fn create_test_address() -> Address {
    Address::from("test_account".to_string())
}

#[tokio::test]
async fn test_create_resource_effect_integration() -> Result<()> {
    // Create a resource and domain
    let resource = create_test_resource("resource1");
    let domain_id = create_test_domain();
    let invoker = create_test_address();
    
    // Create the effect
    let effect = create_resource_effect(&resource, domain_id.clone(), invoker.clone())?;
    
    // Create a lifecycle manager
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    
    // Register the resource
    lifecycle_manager.register_resource(resource.id.clone())?;
    
    // Create a resource manager that uses our lifecycle manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    
    // Add the resource to the manager
    resource_manager.add_resource(resource.clone())?;
    
    // Get state before
    let state_before = resource_manager.get_resource_state(&resource.id)?;
    assert_eq!(state_before, RegisterState::Initial);
    
    // Create a mock context
    let context = EffectContext {
        execution_id: Some(uuid::Uuid::new_v4()),
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager)),
        ..Default::default()
    };
    
    // Execute the effect
    let outcome = effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(outcome.success);
    
    // Verify resource state
    let resource_after = resource_manager.get_resource(&resource.id)?;
    assert_eq!(resource_after.state, RegisterState::Active);
    
    Ok(())
}

#[tokio::test]
async fn test_lifecycle_state_transition_chain() -> Result<()> {
    // Create a resource and domain
    let resource = create_test_resource("resource2");
    let domain_id = create_test_domain();
    let invoker = create_test_address();
    
    // Create a lifecycle manager
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    
    // Register the resource
    lifecycle_manager.register_resource(resource.id.clone())?;
    
    // Activate the resource
    lifecycle_manager.activate(&resource.id)?;
    
    // Create a resource manager that uses our lifecycle manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    
    // Add the resource to the manager with active state
    let mut active_resource = resource.clone();
    active_resource.state = RegisterState::Active;
    resource_manager.add_resource(active_resource.clone())?;
    
    // Create the lock effect
    let lock_effect = lock_resource_effect(&mut active_resource.clone(), domain_id.clone(), invoker.clone())?;
    
    // Create a mock context
    let context = EffectContext {
        execution_id: Some(uuid::Uuid::new_v4()),
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager.clone())),
        ..Default::default()
    };
    
    // Execute the lock effect
    let lock_outcome = lock_effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(lock_outcome.success);
    
    // Verify resource state
    let resource_after_lock = resource_manager.get_resource(&resource.id)?;
    assert_eq!(resource_after_lock.state, RegisterState::Locked);
    
    // Create the unlock effect
    let mut locked_resource = resource_after_lock.clone();
    let unlock_effect = unlock_resource_effect(&mut locked_resource, domain_id.clone(), invoker.clone())?;
    
    // Execute the unlock effect
    let unlock_outcome = unlock_effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(unlock_outcome.success);
    
    // Verify resource state
    let resource_after_unlock = resource_manager.get_resource(&resource.id)?;
    assert_eq!(resource_after_unlock.state, RegisterState::Active);
    
    Ok(())
}

#[tokio::test]
async fn test_invalid_state_transition() -> Result<()> {
    // Create a resource and domain
    let resource = create_test_resource("resource3");
    let domain_id = create_test_domain();
    let invoker = create_test_address();
    
    // Create a lifecycle manager
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    
    // Register the resource
    lifecycle_manager.register_resource(resource.id.clone())?;
    
    // Create a resource manager that uses our lifecycle manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    
    // Add the resource to the manager (still in initial state)
    resource_manager.add_resource(resource.clone())?;
    
    // Try to create a consume effect (should fail due to invalid state transition)
    let mut relationship_tracker = RelationshipTracker::new();
    let consume_effect = consume_resource_effect(
        &mut resource.clone(), 
        domain_id.clone(), 
        invoker.clone(),
        &mut relationship_tracker,
    )?;
    
    // Create a mock context
    let context = EffectContext {
        execution_id: Some(uuid::Uuid::new_v4()),
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager)),
        ..Default::default()
    };
    
    // Execute the consume effect (should fail)
    let consume_result = consume_effect.execute_async(&context).await;
    
    // Should fail because the resource is in Initial state, not Active
    assert!(consume_result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_boundary_aware_resource_creation() -> Result<()> {
    // Create a resource and domain
    let resource = create_test_resource("resource4");
    let domain_id = create_test_domain();
    let invoker = create_test_address();
    let boundary = ExecutionBoundary::new("test_boundary".to_string());
    
    // Create the boundary-aware effect
    let effect = create_resource_with_boundary_effect(
        &resource, 
        boundary.clone(), 
        domain_id.clone(), 
        invoker.clone()
    )?;
    
    // Create a lifecycle manager
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    
    // Register the resource
    lifecycle_manager.register_resource(resource.id.clone())?;
    
    // Create a resource manager that uses our lifecycle manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    
    // Add the resource to the manager
    resource_manager.add_resource(resource.clone())?;
    
    // Create a mock context
    let context = EffectContext {
        execution_id: Some(uuid::Uuid::new_v4()),
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager)),
        boundary_manager: Some(Arc::new(MockBoundaryManager::new(true))),
        ..Default::default()
    };
    
    // Execute the effect
    let outcome = effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(outcome.success);
    assert!(outcome.metadata.contains_key("boundary"));
    
    Ok(())
}

#[tokio::test]
async fn test_capability_validated_operation() -> Result<()> {
    // Create a resource and domain
    let resource = create_test_resource("resource5");
    let domain_id = create_test_domain();
    let invoker = create_test_address();
    
    // Create a lifecycle manager
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    
    // Register the resource
    lifecycle_manager.register_resource(resource.id.clone())?;
    
    // Activate the resource
    lifecycle_manager.activate(&resource.id)?;
    
    // Create a resource manager that uses our lifecycle manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    
    // Add the resource to the manager with active state
    let mut active_resource = resource.clone();
    active_resource.state = RegisterState::Active;
    resource_manager.add_resource(active_resource.clone())?;
    
    // Create the capability-validated freeze effect
    let capability_ids = vec![
        "freeze_capability".to_string(),
        "admin_capability".to_string(),
    ];
    
    let freeze_effect = resource_operation_with_capability_effect(
        &mut active_resource.clone(),
        domain_id.clone(),
        invoker.clone(),
        RegisterOperationType::Freeze,
        capability_ids.clone(),
    )?;
    
    // Create a mock context with authorization service
    let context = EffectContext {
        execution_id: Some(uuid::Uuid::new_v4()),
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: capability_ids.clone(),
        resource_manager: Some(Arc::new(resource_manager)),
        authorization_service: Some(Arc::new(MockAuthorizationService::new(true))),
        ..Default::default()
    };
    
    // Execute the freeze effect
    let freeze_outcome = freeze_effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(freeze_outcome.success);
    
    Ok(())
}

#[tokio::test]
async fn test_time_map_validated_operation() -> Result<()> {
    // Create a resource and domain
    let resource = create_test_resource("resource6");
    let domain_id = create_test_domain();
    let invoker = create_test_address();
    
    // Create a lifecycle manager
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    
    // Register the resource
    lifecycle_manager.register_resource(resource.id.clone())?;
    
    // Activate the resource
    lifecycle_manager.activate(&resource.id)?;
    
    // Create a resource manager that uses our lifecycle manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    
    // Add the resource to the manager with active state
    let mut active_resource = resource.clone();
    active_resource.state = RegisterState::Active;
    resource_manager.add_resource(active_resource.clone())?;
    
    // Create a time map snapshot
    let time_snapshot = TimeMapSnapshot::default();
    
    // Create the time-map validated lock effect
    let lock_effect = resource_operation_with_timemap_effect(
        &mut active_resource.clone(),
        domain_id.clone(),
        invoker.clone(),
        RegisterOperationType::Lock,
        time_snapshot.clone(),
    )?;
    
    // Create a mock context with time service
    let context = EffectContext {
        execution_id: Some(uuid::Uuid::new_v4()),
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager)),
        time_service: Some(Arc::new(MockTimeService::new(true))),
        ..Default::default()
    };
    
    // Execute the lock effect
    let lock_outcome = lock_effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(lock_outcome.success);
    
    Ok(())
}

// Mock implementations for testing

#[derive(Clone)]
struct MockBoundaryManager {
    allow_crossing: bool,
}

impl MockBoundaryManager {
    fn new(allow_crossing: bool) -> Self {
        Self { allow_crossing }
    }
    
    fn can_cross_boundary(
        &self,
        _boundary: &ExecutionBoundary,
        _invoker: Option<&Address>,
    ) -> Result<bool> {
        Ok(self.allow_crossing)
    }
}

#[derive(Clone)]
struct MockAuthorizationService {
    allow_operation: bool,
}

impl MockAuthorizationService {
    fn new(allow_operation: bool) -> Self {
        Self { allow_operation }
    }
    
    fn check_operation_allowed(
        &self,
        _resource_id: &ResourceId,
        _operation_type: RegisterOperationType,
        _capability_ids: &[impl AsRef<str>],
    ) -> Result<bool> {
        Ok(self.allow_operation)
    }
}

#[derive(Clone)]
struct MockTimeService {
    allow_snapshot: bool,
}

impl MockTimeService {
    fn new(allow_snapshot: bool) -> Self {
        Self { allow_snapshot }
    }
    
    fn validate_snapshot(&self, _snapshot: &TimeMapSnapshot) -> Result<bool> {
        Ok(self.allow_snapshot)
    }
} 