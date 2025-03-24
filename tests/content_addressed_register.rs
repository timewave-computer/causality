use std::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};

use causality::crypto::{
    ContentAddressed, ContentId, HashOutput, 
    HashAlgorithm, HashFactory
};
use causality::resource::{
    StorageStrategy, StateVisibility
};
use causality::resource::resource_register::{
    ResourceLogic, FungibilityDomain, Quantity, 
    RegisterState
};
use causality::resource::content_addressed_register::{
    ContentAddressedRegister, ContentAddressedRegisterOperation,
    RegisterOperationType, ContentAddressedRegisterRegistry
};

#[test]
fn test_content_addressed_register_creation() {
    // Create a register
    let register = ContentAddressedRegister::new(
        "test-resource-1".to_string(),
        ResourceLogic::Fungible,
        FungibilityDomain("token".to_string()),
        Quantity(100),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    // Verify that content addressing works
    let content_hash = register.content_hash();
    let content_id = register.content_id();
    
    // Ensure the register verifies against its hash
    assert!(register.verify());
    
    // Convert to bytes and back
    let bytes = register.to_bytes();
    let restored = ContentAddressedRegister::from_bytes(&bytes).unwrap();
    
    // Verify that the restored register has the same hash
    assert_eq!(restored.content_hash(), content_hash);
    assert_eq!(restored.id, "test-resource-1");
    assert_eq!(restored.quantity, Quantity(100));
}

#[test]
fn test_register_operations() {
    // Create a registry
    let mut registry = ContentAddressedRegisterRegistry::new();
    
    // Create a register
    let register = ContentAddressedRegister::new(
        "test-resource-2".to_string(),
        ResourceLogic::Fungible,
        FungibilityDomain("token".to_string()),
        Quantity(100),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    // Register it
    let content_id = registry.register(register);
    
    // Create an update operation
    let operation = ContentAddressedRegisterOperation::new(
        RegisterOperationType::UpdateRegister,
        content_id.clone(),
    )
    .with_pre_state(RegisterState::Initial)
    .with_post_state(RegisterState::Active)
    .with_parameter("quantity", serde_json::json!(200));
    
    // Apply the operation
    let updated_id = registry.apply_operation(operation).unwrap();
    
    // Retrieve the updated register
    let updated = registry.get_register(&updated_id).unwrap();
    
    // Verify the update was applied
    assert_eq!(updated.quantity, Quantity(200));
    assert_eq!(updated.state, RegisterState::Active);
    
    // Create a freeze operation
    let freeze_op = ContentAddressedRegisterOperation::new(
        RegisterOperationType::FreezeRegister,
        updated_id.clone(),
    );
    
    // Apply the freeze operation
    let frozen_id = registry.apply_operation(freeze_op).unwrap();
    let frozen = registry.get_register(&frozen_id).unwrap();
    
    // Verify the register is frozen
    assert_eq!(frozen.state, RegisterState::Frozen);
}

#[test]
fn test_register_state_transitions() {
    // Create a registry
    let mut registry = ContentAddressedRegisterRegistry::new();
    
    // Create a register
    let register = ContentAddressedRegister::new_active(
        "test-resource-3".to_string(),
        ResourceLogic::NonFungible,
        FungibilityDomain("nft".to_string()),
        Quantity(1),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    // Register it
    let content_id = registry.register(register);
    
    // Create operations for different state transitions
    let states = vec![
        RegisterState::Locked,
        RegisterState::Active,    // Unlock
        RegisterState::Frozen,
        RegisterState::Active,    // Unfreeze
        RegisterState::Consumed,  // Terminal state
    ];
    
    let mut current_id = content_id;
    
    // Apply each state transition
    for state in states {
        let op_type = match state {
            RegisterState::Locked => RegisterOperationType::LockRegister,
            RegisterState::Active => {
                let current = registry.get_register(&current_id).unwrap();
                if current.state == RegisterState::Locked {
                    RegisterOperationType::UnlockRegister
                } else {
                    RegisterOperationType::UnfreezeRegister
                }
            },
            RegisterState::Frozen => RegisterOperationType::FreezeRegister,
            RegisterState::Consumed => RegisterOperationType::ConsumeRegister,
            _ => RegisterOperationType::Custom("unknown".to_string()),
        };
        
        let op = ContentAddressedRegisterOperation::new(
            op_type,
            current_id.clone(),
        );
        
        // Apply the operation
        current_id = registry.apply_operation(op).unwrap();
        
        // Verify the state transition
        let register = registry.get_register(&current_id).unwrap();
        assert_eq!(register.state, state);
    }
    
    // Verify that operations were recorded
    let ops = registry.find_operations_by_type(&RegisterOperationType::ConsumeRegister);
    assert_eq!(ops.len(), 1);
}

#[test]
fn test_transfer_operation() {
    // Create a registry
    let mut registry = ContentAddressedRegisterRegistry::new();
    
    // Create a register
    let register = ContentAddressedRegister::new_active(
        "test-resource-4".to_string(),
        ResourceLogic::Fungible,
        FungibilityDomain("token".to_string()),
        Quantity(500),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    // Register it
    let content_id = registry.register(register);
    
    // Create a transfer operation
    let transfer_op = ContentAddressedRegisterOperation::new(
        RegisterOperationType::TransferRegister,
        content_id.clone(),
    )
    .with_parameter("controller", serde_json::json!("new-owner"));
    
    // Apply the transfer
    let transferred_id = registry.apply_operation(transfer_op).unwrap();
    let transferred = registry.get_register(&transferred_id).unwrap();
    
    // Verify the ownership changed
    assert_eq!(transferred.controller, Some("new-owner".to_string()));
}

#[test]
fn test_operation_content_addressing() {
    // Create an operation
    let op = ContentAddressedRegisterOperation::new(
        RegisterOperationType::CreateRegister,
        ContentId::from(HashOutput::default()),
    )
    .with_parameter("name", serde_json::json!("test-op"))
    .with_domain("test-domain".to_string());
    
    // Get its content hash
    let hash = op.content_hash();
    
    // Verify it
    assert!(op.verify());
    
    // Serialize and deserialize
    let bytes = op.to_bytes();
    let restored = ContentAddressedRegisterOperation::from_bytes(&bytes).unwrap();
    
    // Verify hash matches
    assert_eq!(restored.content_hash(), hash);
    assert_eq!(restored.domain_id, "test-domain");
}

#[test]
fn test_registry_operations() {
    let mut registry = ContentAddressedRegisterRegistry::new();
    
    // Create multiple registers with different logic types
    let fungible = ContentAddressedRegister::new(
        "fungible-1".to_string(),
        ResourceLogic::Fungible,
        FungibilityDomain("token".to_string()),
        Quantity(100),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    let non_fungible = ContentAddressedRegister::new(
        "nft-1".to_string(),
        ResourceLogic::NonFungible,
        FungibilityDomain("collectible".to_string()),
        Quantity(1),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    let capability = ContentAddressedRegister::new(
        "capability-1".to_string(),
        ResourceLogic::Capability,
        FungibilityDomain("access".to_string()),
        Quantity(1),
        HashMap::new(),
        StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
    );
    
    // Register all
    registry.register(fungible);
    registry.register(non_fungible);
    registry.register(capability);
    
    // Test filtering by logic type
    let fungibles = registry.find_by_logic(&ResourceLogic::Fungible);
    let nfts = registry.find_by_logic(&ResourceLogic::NonFungible);
    let capabilities = registry.find_by_logic(&ResourceLogic::Capability);
    
    assert_eq!(fungibles.len(), 1);
    assert_eq!(nfts.len(), 1);
    assert_eq!(capabilities.len(), 1);
    
    // Test filtering by state
    let initial = registry.find_by_state(&RegisterState::Initial);
    assert_eq!(initial.len(), 3);
    
    // Clear the registry
    registry.clear();
    assert_eq!(registry.find_by_state(&RegisterState::Initial).len(), 0);
} 