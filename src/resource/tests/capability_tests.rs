use std::collections::HashSet;
use std::sync::Arc;

use crate::address::{Address, AddressGenerator};
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterMetadata,
    capability::{CapabilityId, Right, Restrictions, CapabilityRegistry, ResourceCapability},
    capability_api::{ResourceAPI, ResourceIntent, ResourceOperation, TransferIntent, SwapIntent},
    capability_chain::{CapabilityChain, ComposedIntent, MultiTransferIntent},
    register_service::InMemoryRegisterService,
    CapabilityRepository, CapabilityError,
};

#[test]
fn test_basic_capability() {
    let address_gen = AddressGenerator::new();
    let capability_registry = CapabilityRegistry::new(address_gen.clone());
    
    let issuer = address_gen.generate_unique();
    let holder = address_gen.generate_unique();
    
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    
    let restrictions = Restrictions::default();
    
    let capability = ResourceCapability::new(
        &address_gen,
        issuer.clone(),
        holder.clone(),
        rights,
        restrictions,
    );
    
    assert!(capability.has_right(&Right::Read));
    assert!(capability.has_right(&Right::Write));
    assert!(!capability.has_right(&Right::Delete));
    assert!(capability.is_valid());
}

#[test]
fn test_capability_delegation() {
    let address_gen = AddressGenerator::new();
    let capability_registry = Arc::new(CapabilityRegistry::new(address_gen.clone()));
    
    let issuer = address_gen.generate_unique();
    let holder = address_gen.generate_unique();
    let delegate = address_gen.generate_unique();
    
    // Create a set of rights
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    rights.insert(Right::Delegate);
    
    // Create a root capability
    let parent_id = capability_registry
        .create_capability(issuer.clone(), holder.clone(), rights.clone(), Restrictions::default())
        .unwrap();
    
    // Reduce rights for delegation
    let mut delegated_rights = HashSet::new();
    delegated_rights.insert(Right::Read);
    
    // Delegate the capability
    let child_id = capability_registry
        .delegate(
            &parent_id,
            &holder,
            delegate.clone(),
            delegated_rights,
            Restrictions::default(),
        )
        .unwrap();
    
    // Verify the delegation
    let child = capability_registry.get(&child_id).unwrap();
    assert_eq!(child.holder(), &delegate);
    assert!(child.has_right(&Right::Read));
    assert!(!child.has_right(&Right::Write));
    assert!(!child.has_right(&Right::Delegate));
}

#[test]
fn test_capability_verification() {
    let address_gen = AddressGenerator::new();
    let capability_registry = Arc::new(CapabilityRegistry::new(address_gen.clone()));
    
    let issuer = address_gen.generate_unique();
    let holder = address_gen.generate_unique();
    
    // Create register ID
    let register_id = RegisterId::new(address_gen.generate_unique());
    
    // Create a set of rights
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    
    // Create resource scope restriction
    let mut resource_scope = HashSet::new();
    resource_scope.insert(register_id.clone());
    
    let restrictions = Restrictions {
        resource_scope: Some(resource_scope),
        ..Restrictions::default()
    };
    
    // Create capability
    let capability_id = capability_registry
        .create_capability(issuer.clone(), holder.clone(), rights, restrictions)
        .unwrap();
    
    // Verify the capability for read operation on the register
    capability_registry
        .verify(
            &capability_id,
            &holder,
            &Right::Read,
            Some(&register_id),
            None,
        )
        .unwrap();
    
    // Create another register ID outside the scope
    let other_register_id = RegisterId::new(address_gen.generate_unique());
    
    // Verification should fail for a register outside the scope
    assert!(capability_registry
        .verify(
            &capability_id,
            &holder,
            &Right::Read,
            Some(&other_register_id),
            None,
        )
        .is_err());
}

#[test]
fn test_resource_api() {
    let address_gen = AddressGenerator::new();
    let capability_registry = Arc::new(CapabilityRegistry::new(address_gen.clone()));
    let register_service = Arc::new(InMemoryRegisterService::new());
    
    let api = ResourceAPI::new(register_service.clone(), capability_registry.clone());
    
    let issuer = address_gen.generate_unique();
    let holder = address_gen.generate_unique();
    
    // Create a capability with all rights
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    rights.insert(Right::Create);
    rights.insert(Right::Delete);
    rights.insert(Right::UpdateMetadata);
    
    let capability_id = capability_registry
        .create_capability(issuer.clone(), holder.clone(), rights, Restrictions::default())
        .unwrap();
    
    // Create a register
    let register_id = RegisterId::new(address_gen.generate_unique());
    let register = Register::new(
        register_id.clone(),
        RegisterContents::new(b"test data".to_vec()),
        RegisterMetadata::default(),
    );
    
    // Use the API to create the register
    api.create(&capability_id, &holder, register.clone()).unwrap();
    
    // Read the register back
    let read_register = api.read(&capability_id, &holder, &register_id).unwrap();
    assert_eq!(read_register.id(), &register_id);
    assert_eq!(read_register.contents().data(), b"test data");
    
    // Update the register
    let new_contents = RegisterContents::new(b"updated data".to_vec());
    api.update(&capability_id, &holder, &register_id, new_contents).unwrap();
    
    // Read again to verify the update
    let updated_register = api.read(&capability_id, &holder, &register_id).unwrap();
    assert_eq!(updated_register.contents().data(), b"updated data");
}

#[test]
fn test_transfer_intent() {
    let address_gen = AddressGenerator::new();
    let capability_registry = Arc::new(CapabilityRegistry::new(address_gen.clone()));
    let register_service = Arc::new(InMemoryRegisterService::new());
    
    let api = ResourceAPI::new(register_service.clone(), capability_registry.clone());
    
    let issuer = address_gen.generate_unique();
    let holder = address_gen.generate_unique();
    let recipient = address_gen.generate_unique();
    
    // Create a capability with all rights including delegation
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    rights.insert(Right::Create);
    rights.insert(Right::Delete);
    rights.insert(Right::UpdateMetadata);
    rights.insert(Right::Delegate);
    
    let capability_id = capability_registry
        .create_capability(issuer.clone(), holder.clone(), rights, Restrictions::default())
        .unwrap();
    
    // Create a register
    let register_id = RegisterId::new(address_gen.generate_unique());
    let register = Register::new(
        register_id.clone(),
        RegisterContents::new(b"transferable data".to_vec()),
        RegisterMetadata::default(),
    );
    
    // Create the register
    api.create(&capability_id, &holder, register.clone()).unwrap();
    
    // Create a transfer intent
    let transfer = TransferIntent {
        capability_id: capability_id.clone(),
        current_holder: holder.clone(),
        register_id: register_id.clone(),
        recipient: recipient.clone(),
    };
    
    // Execute the transfer
    let recipient_cap_id = api.execute_intent(&transfer).unwrap();
    
    // Try reading the register using the recipient's capability
    let read_register = api
        .read(&recipient_cap_id, &recipient, &register_id)
        .unwrap();
    
    assert_eq!(read_register.id(), &register_id);
    assert_eq!(read_register.contents().data(), b"transferable data");
}

#[test]
fn test_multi_transfer_intent() {
    let address_gen = AddressGenerator::new();
    let capability_registry = Arc::new(CapabilityRegistry::new(address_gen.clone()));
    let register_service = Arc::new(InMemoryRegisterService::new());
    
    let api = ResourceAPI::new(register_service.clone(), capability_registry.clone());
    
    let issuer = address_gen.generate_unique();
    let holder = address_gen.generate_unique();
    let recipient1 = address_gen.generate_unique();
    let recipient2 = address_gen.generate_unique();
    let recipient3 = address_gen.generate_unique();
    
    // Create a capability with all rights including delegation
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    rights.insert(Right::Create);
    rights.insert(Right::Delete);
    rights.insert(Right::UpdateMetadata);
    rights.insert(Right::Delegate);
    
    let capability_id = capability_registry
        .create_capability(issuer.clone(), holder.clone(), rights, Restrictions::default())
        .unwrap();
    
    // Create a register
    let register_id = RegisterId::new(address_gen.generate_unique());
    let register = Register::new(
        register_id.clone(),
        RegisterContents::new(b"multi-transfer data".to_vec()),
        RegisterMetadata::default(),
    );
    
    // Create the register
    api.create(&capability_id, &holder, register.clone()).unwrap();
    
    // Create the initial transfer
    let first_transfer = TransferIntent {
        capability_id: capability_id.clone(),
        current_holder: holder.clone(),
        register_id: register_id.clone(),
        recipient: recipient1.clone(),
    };
    
    // Create subsequent transfers
    let subsequent_transfers = vec![
        (recipient1.clone(), register_id.clone(), recipient2.clone()),
        (recipient2.clone(), register_id.clone(), recipient3.clone()),
    ];
    
    // Create multi-transfer intent
    let multi_transfer = MultiTransferIntent::new(first_transfer, subsequent_transfers);
    
    // Execute the multi-transfer
    let capability_ids = api.execute_intent(&multi_transfer).unwrap();
    
    // Verify we got three capability IDs
    assert_eq!(capability_ids.len(), 3);
    
    // Check that the final recipient can access the register
    let read_register = api
        .read(&capability_ids[2], &recipient3, &register_id)
        .unwrap();
    
    assert_eq!(read_register.id(), &register_id);
    assert_eq!(read_register.contents().data(), b"multi-transfer data");
}

#[test]
fn test_capability_chain() {
    let address_gen = AddressGenerator::new();
    let capability_registry = Arc::new(CapabilityRegistry::new(address_gen.clone()));
    
    let issuer = address_gen.generate_unique();
    let holder1 = address_gen.generate_unique();
    let holder2 = address_gen.generate_unique();
    let holder3 = address_gen.generate_unique();
    
    // Create a root capability with all rights
    let mut rights = HashSet::new();
    rights.insert(Right::Read);
    rights.insert(Right::Write);
    rights.insert(Right::Delegate);
    
    let root_id = capability_registry
        .create_capability(issuer.clone(), holder1.clone(), rights.clone(), Restrictions::default())
        .unwrap();
    
    // Delegate to holder2 with reduced rights
    let mut rights2 = HashSet::new();
    rights2.insert(Right::Read);
    rights2.insert(Right::Delegate);
    
    let level2_id = capability_registry
        .delegate(&root_id, &holder1, holder2.clone(), rights2, Restrictions::default())
        .unwrap();
    
    // Delegate to holder3 with further reduced rights
    let mut rights3 = HashSet::new();
    rights3.insert(Right::Read);
    
    let leaf_id = capability_registry
        .delegate(&level2_id, &holder2, holder3.clone(), rights3, Restrictions::default())
        .unwrap();
    
    // Create a capability chain from the leaf
    let chain = CapabilityChain::from_leaf(leaf_id.clone(), capability_registry.clone()).unwrap();
    
    // Verify the chain
    assert_eq!(chain.len(), 3);
    assert_eq!(chain.leaf_id(), &leaf_id);
    assert_eq!(chain.root_id(), Some(&root_id));
    
    // Verify the chain integrity
    chain.verify().unwrap();
    
    // Verify the chain is valid for read operation by holder3
    chain.is_valid_for(&holder3, &Right::Read, None).unwrap();
    
    // Verify the chain is not valid for write operation by holder3
    assert!(chain.is_valid_for(&holder3, &Right::Write, None).is_err());
}

#[test]
fn test_capability_creation() {
    let resource_id = "document:123";
    let resource_type = "document";
    let issuer = Address::from("issuer:0x1234");
    let holder = Address::from("holder:0x5678");
    
    let rights = vec![Right::Read, Right::Write];
    
    let capability = ResourceCapability::new(
        resource_id,
        resource_type,
        issuer.clone(),
        holder.clone(),
        rights.clone(),
    );
    
    assert_eq!(capability.resource_id(), resource_id);
    assert_eq!(capability.resource_type(), resource_type);
    assert_eq!(capability.issuer(), &issuer);
    assert_eq!(capability.holder(), &holder);
    
    // Check rights
    for right in &rights {
        assert!(capability.has_right(right));
    }
    
    // Shouldn't have rights not granted
    assert!(!capability.has_right(&Right::Delete));
    assert!(!capability.has_right(&Right::Transfer));
    assert!(!capability.has_right(&Right::Delegate));
}

#[test]
fn test_capability_delegation() {
    let resource_id = "document:123";
    let resource_type = "document";
    let issuer = Address::from("issuer:0x1234");
    let holder = Address::from("holder:0x5678");
    let delegate = Address::from("delegate:0x9abc");
    
    let rights = vec![
        Right::Read,
        Right::Write,
        Right::Delete,
        Right::Delegate,
    ];
    
    let capability = ResourceCapability::new(
        resource_id,
        resource_type,
        issuer.clone(),
        holder.clone(),
        rights.clone(),
    );
    
    // Delegate with reduced rights
    let delegated_rights = vec![Right::Read];
    let delegated = capability.delegate(
        delegate.clone(),
        delegated_rights.clone(),
        None,
    ).unwrap();
    
    // Check delegated capability properties
    assert_eq!(delegated.resource_id(), resource_id);
    assert_eq!(delegated.resource_type(), resource_type);
    assert_eq!(delegated.issuer(), &holder); // holder of parent becomes issuer
    assert_eq!(delegated.holder(), &delegate);
    
    // Check delegated rights
    assert!(delegated.has_right(&Right::Read));
    assert!(!delegated.has_right(&Right::Write));
    assert!(!delegated.has_right(&Right::Delete));
    assert!(!delegated.has_right(&Right::Delegate));
}

#[test]
fn test_delegation_without_delegate_right() {
    let resource_id = "document:123";
    let resource_type = "document";
    let issuer = Address::from("issuer:0x1234");
    let holder = Address::from("holder:0x5678");
    let delegate = Address::from("delegate:0x9abc");
    
    // Create capability without delegate right
    let rights = vec![Right::Read, Right::Write];
    
    let capability = ResourceCapability::new(
        resource_id,
        resource_type,
        issuer.clone(),
        holder.clone(),
        rights.clone(),
    );
    
    // Try to delegate
    let delegated_rights = vec![Right::Read];
    let result = capability.delegate(
        delegate.clone(),
        delegated_rights.clone(),
        None,
    );
    
    // Should fail because the capability doesn't have delegate right
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, CapabilityError::AccessDenied(_)));
    }
}

#[test]
fn test_delegation_with_invalid_rights() {
    let resource_id = "document:123";
    let resource_type = "document";
    let issuer = Address::from("issuer:0x1234");
    let holder = Address::from("holder:0x5678");
    let delegate = Address::from("delegate:0x9abc");
    
    // Create capability with only read and delegate rights
    let rights = vec![Right::Read, Right::Delegate];
    
    let capability = ResourceCapability::new(
        resource_id,
        resource_type,
        issuer.clone(),
        holder.clone(),
        rights.clone(),
    );
    
    // Try to delegate write right (which parent doesn't have)
    let delegated_rights = vec![Right::Read, Right::Write];
    let result = capability.delegate(
        delegate.clone(),
        delegated_rights.clone(),
        None,
    );
    
    // Should fail because trying to delegate a right not possessed
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, CapabilityError::InvalidRights(_)));
    }
}

#[test]
fn test_capability_repository() {
    let resource_id = "document:123";
    let resource_type = "document";
    let issuer = Address::from("issuer:0x1234");
    let holder = Address::from("holder:0x5678");
    
    let rights = vec![Right::Read, Right::Write];
    
    let capability = ResourceCapability::new(
        resource_id,
        resource_type,
        issuer.clone(),
        holder.clone(),
        rights.clone(),
    );
    
    let mut repo = CapabilityRepository::new();
    
    // Register capability
    let cap_ref = repo.register(capability);
    
    // Get capability by ID
    let retrieved = repo.get(cap_ref.id()).expect("Should find capability");
    assert_eq!(retrieved.id(), cap_ref.id());
    
    // Get capabilities for resource
    let resource_caps = repo.get_for_resource(resource_id);
    assert_eq!(resource_caps.len(), 1);
    assert_eq!(resource_caps[0].id(), cap_ref.id());
    
    // Get capabilities for holder
    let holder_caps = repo.get_for_holder(&holder);
    assert_eq!(holder_caps.len(), 1);
    assert_eq!(holder_caps[0].id(), cap_ref.id());
    
    // Validate capability (should succeed)
    let validated = repo.validate(cap_ref.id()).expect("Validation should succeed");
    assert_eq!(validated.id(), cap_ref.id());
    
    // Revoke capability
    repo.revoke(cap_ref.id()).expect("Revocation should succeed");
    
    // Try to validate revoked capability (should fail)
    let result = repo.validate(cap_ref.id());
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, CapabilityError::Revoked(_)));
    }
}

#[test]
fn test_capability_restrictions() {
    let resource_id = "document:123";
    let resource_type = "document";
    let issuer = Address::from("issuer:0x1234");
    let holder = Address::from("holder:0x5678");
    
    let rights = vec![Right::Read, Right::Write, Right::Delegate];
    
    // Create time-restricted capability
    let mut capability = ResourceCapability::new(
        resource_id,
        resource_type,
        issuer.clone(),
        holder.clone(),
        rights.clone(),
    );
    
    // Set expiration time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let expiration = now + 3600; // 1 hour from now
    
    let mut restrictions = Restrictions::new();
    restrictions.add_expiration(expiration);
    
    capability.set_restrictions(restrictions);
    
    // Create repository and register capability
    let mut repo = CapabilityRepository::new();
    let cap_ref = repo.register(capability);
    
    // Validate (should succeed)
    let result = repo.validate(cap_ref.id());
    assert!(result.is_ok());
    
    // Manually expire the capability
    let expired_restrictions = {
        let mut r = Restrictions::new();
        r.add_expiration(now - 10); // 10 seconds in the past
        r
    };
    
    let cap = cap_ref.capability_mut();
    cap.set_restrictions(expired_restrictions);
    
    // Try to validate expired capability (should fail)
    let result = repo.validate(cap_ref.id());
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, CapabilityError::Expired(_)));
    }
} 
