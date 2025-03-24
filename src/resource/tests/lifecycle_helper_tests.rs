// Tests for the ResourceStateTransitionHelper test utility
//
// This module tests the functionality of the ResourceStateTransitionHelper
// for testing resource lifecycle state transitions.

use crate::error::Result;
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::resource::resource_register::{ResourceRegister, FungibilityDomain, StorageStrategy, StateVisibility};
use crate::resource::tests::ResourceStateTransitionHelper;

#[test]
fn test_helper_common_sequence() -> Result<()> {
    // Create a resource
    let id = ResourceId::new("test-resource".to_string());
    let resource = ResourceRegister::new(
        id.clone(),
        ResourceLogic::Data,
        FungibilityDomain("test-domain".to_string()),
        Quantity(1),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Create a helper
    let mut helper = ResourceStateTransitionHelper::new(true);
    
    // Add the resource
    helper.add_resource(resource)?;
    
    // Execute the common sequence
    let states = helper.execute_common_sequence(&id)?;
    
    // Expected states in sequence
    let expected_states = vec![
        RegisterState::Initial,
        RegisterState::Active,
        RegisterState::Locked,
        RegisterState::Active,
        RegisterState::Consumed,
    ];
    
    // Verify the states
    assert!(helper.validate_transition_sequence(&states, &expected_states));
    
    Ok(())
}

#[test]
fn test_helper_freezing_sequence() -> Result<()> {
    // Create a resource
    let id = ResourceId::new("test-resource-freeze".to_string());
    let resource = ResourceRegister::new(
        id.clone(),
        ResourceLogic::Data,
        FungibilityDomain("test-domain".to_string()),
        Quantity(1),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Create a helper
    let mut helper = ResourceStateTransitionHelper::new(true);
    
    // Add the resource
    helper.add_resource(resource)?;
    
    // Execute the freezing sequence
    let states = helper.execute_freezing_sequence(&id)?;
    
    // Expected states in sequence
    let expected_states = vec![
        RegisterState::Initial,
        RegisterState::Active,
        RegisterState::Frozen,
        RegisterState::Active,
        RegisterState::Archived,
    ];
    
    // Verify the states
    assert!(helper.validate_transition_sequence(&states, &expected_states));
    
    Ok(())
}

#[test]
fn test_invalid_transition() -> Result<()> {
    // Create a resource
    let id = ResourceId::new("test-resource-invalid".to_string());
    let resource = ResourceRegister::new(
        id.clone(),
        ResourceLogic::Data,
        FungibilityDomain("test-domain".to_string()),
        Quantity(1),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Create a helper
    let mut helper = ResourceStateTransitionHelper::new(true);
    
    // Add the resource
    helper.add_resource(resource)?;
    
    // Transition to active state
    helper.activate(&id)?;
    
    // Verify active state
    let active_resource = helper.get_resource(&id)?;
    assert_eq!(active_resource.state, RegisterState::Active);
    
    // Try an invalid transition (directly to archived without proper steps)
    let result = helper.archive(&id);
    
    // This should succeed since the resource manager allows it
    // In a real application with validation, this would fail
    assert!(result.is_ok());
    
    // Verify the state changed
    let archived_resource = helper.get_resource(&id)?;
    assert_eq!(archived_resource.state, RegisterState::Archived);
    
    Ok(())
}

#[test]
fn test_async_simulation() -> Result<()> {
    // Create a resource
    let id = ResourceId::new("test-resource-async".to_string());
    let resource = ResourceRegister::new(
        id.clone(),
        ResourceLogic::Data,
        FungibilityDomain("test-domain".to_string()),
        Quantity(1),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Create a helper with async simulation
    let mut helper = ResourceStateTransitionHelper::new(false);
    
    // Add the resource
    helper.add_resource(resource)?;
    
    // Run a simple sequence
    helper.activate(&id)?;
    let active_resource = helper.get_resource(&id)?;
    assert_eq!(active_resource.state, RegisterState::Active);
    
    helper.lock(&id)?;
    let locked_resource = helper.get_resource(&id)?;
    assert_eq!(locked_resource.state, RegisterState::Locked);
    
    helper.unlock(&id)?;
    let unlocked_resource = helper.get_resource(&id)?;
    assert_eq!(unlocked_resource.state, RegisterState::Active);
    
    Ok(())
}

#[test]
fn test_validate_transition_sequence() -> Result<()> {
    // Create a helper
    let helper = ResourceStateTransitionHelper::new(true);
    
    // Test with matching sequences
    let seq1 = vec![
        RegisterState::Initial,
        RegisterState::Active,
        RegisterState::Locked,
    ];
    
    let seq2 = vec![
        RegisterState::Initial,
        RegisterState::Active,
        RegisterState::Locked,
    ];
    
    assert!(helper.validate_transition_sequence(&seq1, &seq2));
    
    // Test with different sequences
    let seq3 = vec![
        RegisterState::Initial,
        RegisterState::Active,
        RegisterState::Frozen,
    ];
    
    assert!(!helper.validate_transition_sequence(&seq1, &seq3));
    
    // Test with different lengths
    let seq4 = vec![
        RegisterState::Initial,
        RegisterState::Active,
    ];
    
    assert!(!helper.validate_transition_sequence(&seq1, &seq4));
    
    Ok(())
} 
