// Tests for the TEL resource system
//
// This module provides a comprehensive test framework for the
// resource operations in the Temporal Effect Language (TEL).

#![cfg(test)]

use std::sync::Arc;
use uuid::Uuid;

use crate::tel::{
    types::{ResourceId, Address, Domain},
    error::{TelError, TelResult},
    resource::{
        ResourceManager,
        Register,
        RegisterId,
        RegisterContents,
        RegisterState,
        ResourceOperation,
        ResourceOperationType,
        ZkVerifier,
        VerifierConfig,
        ResourceVmIntegration,
        VmIntegrationConfig,
        SnapshotManager,
        FileSnapshotStorage,
        RestoreMode,
        RestoreOptions,
    },
};

/// Test helper for creating resources
fn create_test_resource(
    manager: &ResourceManager,
    owner: Address,
    domain: Domain,
    contents: RegisterContents,
) -> TelResult<RegisterId> {
    manager.create_register(owner, domain, contents)
}

/// Test suite for basic resource operations
#[cfg(test)]
mod basic_operations {
    use super::*;
    
    /// Test creating a resource
    #[test]
    fn test_create_resource() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Check if the register exists
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.owner, owner);
        assert_eq!(register.domain, domain);
        assert_eq!(register.contents, contents);
        assert_eq!(register.state, RegisterState::Active);
    }
    
    /// Test updating a resource
    #[test]
    fn test_update_resource() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Update the resource
        let new_contents = RegisterContents::Text("Updated resource".to_string());
        manager.update_register(&register_id, new_contents.clone()).unwrap();
        
        // Check if the register was updated
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.contents, new_contents);
    }
    
    /// Test deleting a resource
    #[test]
    fn test_delete_resource() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Delete the resource
        manager.delete_register(&register_id).unwrap();
        
        // Check if the register is marked for deletion
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.state, RegisterState::PendingDeletion);
    }
    
    /// Test transferring a resource
    #[test]
    fn test_transfer_resource() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let new_owner = Address::try_from("0xabcdef1234567890abcdef1234567890abcdef12").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Transfer the resource
        manager.transfer_register(&register_id, &owner, new_owner.clone()).unwrap();
        
        // Check if the register has a new owner
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.owner, new_owner);
    }
    
    /// Test locking and unlocking a resource
    #[test]
    fn test_lock_unlock_resource() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Lock the resource
        manager.lock_register(&register_id).unwrap();
        
        // Check if the register is locked
        let register = manager.get_register(&register_id).unwrap();
        assert_eq!(register.state, RegisterState::Locked);
        
        // Try to update the resource (should fail)
        let new_contents = RegisterContents::Text("Updated resource".to_string());
        let result = manager.update_register(&register_id, new_contents);
        assert!(result.is_err());
        
        // Unlock the resource
        manager.unlock_register(&register_id).unwrap();
        
        // Check if the register is unlocked
        let register = manager.get_register(&register_id).unwrap();
        assert_eq!(register.state, RegisterState::Active);
        
        // Now update should succeed
        let new_contents = RegisterContents::Text("Updated resource".to_string());
        manager.update_register(&register_id, new_contents.clone()).unwrap();
        
        // Check if the register was updated
        let register = manager.get_register(&register_id).unwrap();
        assert_eq!(register.contents, new_contents);
    }
    
    /// Test querying resources
    #[test]
    fn test_query_resources() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        
        // Create multiple resources
        for i in 0..5 {
            let contents = RegisterContents::Text(format!("Test resource {}", i));
            create_test_resource(&manager, owner.clone(), domain.clone(), contents).unwrap();
        }
        
        // Query by owner
        let registers = manager.query_registers_by_owner(&owner).unwrap();
        assert_eq!(registers.len(), 5);
        
        // Query by domain
        let registers = manager.query_registers_by_domain(&domain).unwrap();
        assert_eq!(registers.len(), 5);
    }
}

/// Test suite for resource operations
#[cfg(test)]
mod operation_tests {
    use super::*;
    
    /// Test creating a resource through operations
    #[test]
    fn test_create_operation() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        let register_id = RegisterId::new();
        
        // Create the operation
        let operation = ResourceOperation {
            operation_id: uuid::Uuid::new_v4().into(),
            operation_type: ResourceOperationType::Create,
            target: register_id,
            resource_ids: vec![],
            initiator: owner.clone(),
            domain: domain.clone(),
            inputs: vec![contents.clone()],
            parameters: std::collections::HashMap::new(),
            proof: None,
            verification_key: None,
            metadata: std::collections::HashMap::new(),
        };
        
        // Apply the operation
        manager.apply_operation(operation).unwrap();
        
        // Check if the register exists
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.owner, owner);
        assert_eq!(register.domain, domain);
        assert_eq!(register.contents, contents);
        assert_eq!(register.state, RegisterState::Active);
    }
    
    /// Test updating a resource through operations
    #[test]
    fn test_update_operation() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Create the update operation
        let new_contents = RegisterContents::Text("Updated resource".to_string());
        let operation = ResourceOperation {
            operation_id: uuid::Uuid::new_v4().into(),
            operation_type: ResourceOperationType::Update,
            target: register_id,
            resource_ids: vec![],
            initiator: owner.clone(),
            domain: domain.clone(),
            inputs: vec![new_contents.clone()],
            parameters: std::collections::HashMap::new(),
            proof: None,
            verification_key: None,
            metadata: std::collections::HashMap::new(),
        };
        
        // Apply the operation
        manager.apply_operation(operation).unwrap();
        
        // Check if the register was updated
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.contents, new_contents);
    }
    
    /// Test transferring a resource through operations
    #[test]
    fn test_transfer_operation() {
        let manager = ResourceManager::new();
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let new_owner = Address::try_from("0xabcdef1234567890abcdef1234567890abcdef12").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        
        // Create the transfer operation
        let mut parameters = std::collections::HashMap::new();
        parameters.insert("recipient".to_string(), serde_json::Value::String(new_owner.to_string()));
        
        let operation = ResourceOperation {
            operation_id: uuid::Uuid::new_v4().into(),
            operation_type: ResourceOperationType::Transfer,
            target: register_id,
            resource_ids: vec![],
            initiator: owner.clone(),
            domain: domain.clone(),
            inputs: vec![],
            parameters,
            proof: None,
            verification_key: None,
            metadata: std::collections::HashMap::new(),
        };
        
        // Apply the operation
        manager.apply_operation(operation).unwrap();
        
        // Check if the register has a new owner
        let register = manager.get_register(&register_id).unwrap();
        
        assert_eq!(register.owner, new_owner);
    }
}

/// Test suite for the ZkVerifier
#[cfg(test)]
mod zk_verifier_tests {
    use super::*;
    
    /// Test verifying a proof
    #[test]
    fn test_verify_proof() {
        // Create a verifier
        let verifier = ZkVerifier::default();
        
        // Create a resource operation with a proof
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let register_id = RegisterId::new();
        let contents = RegisterContents::Text("Test resource".to_string());
        
        // Create a mock proof and verification key
        let proof = vec![1, 2, 3, 4, 5];
        let verification_key = vec![6, 7, 8, 9, 10];
        
        let operation = ResourceOperation {
            operation_id: uuid::Uuid::new_v4().into(),
            operation_type: ResourceOperationType::Create,
            target: register_id,
            resource_ids: vec![],
            initiator: owner,
            domain,
            inputs: vec![contents],
            parameters: std::collections::HashMap::new(),
            proof: Some(proof),
            verification_key: Some(verification_key),
            metadata: std::collections::HashMap::new(),
        };
        
        // Verify the operation
        let result = verifier.verify_operation(&operation).unwrap();
        
        // The mock implementation always returns true
        assert!(result.is_valid);
    }
    
    /// Test caching of verification results
    #[test]
    fn test_verification_caching() {
        // Create a verifier with caching enabled
        let config = VerifierConfig {
            enabled: true,
            timeout_ms: 1000,
            enable_caching: true,
            max_cache_size: 10,
            parallel_verification: false,
        };
        let verifier = ZkVerifier::new(config);
        
        // Create a resource operation with a proof
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let register_id = RegisterId::new();
        let contents = RegisterContents::Text("Test resource".to_string());
        
        // Create a mock proof and verification key
        let proof = vec![1, 2, 3, 4, 5];
        let verification_key = vec![6, 7, 8, 9, 10];
        
        let operation = ResourceOperation {
            operation_id: uuid::Uuid::new_v4().into(),
            operation_type: ResourceOperationType::Create,
            target: register_id,
            resource_ids: vec![],
            initiator: owner,
            domain,
            inputs: vec![contents],
            parameters: std::collections::HashMap::new(),
            proof: Some(proof),
            verification_key: Some(verification_key),
            metadata: std::collections::HashMap::new(),
        };
        
        // Verify the operation
        let result1 = verifier.verify_operation(&operation).unwrap();
        
        // Verify again, should use cache
        let result2 = verifier.verify_operation(&operation).unwrap();
        
        // Both results should be valid
        assert!(result1.is_valid);
        assert!(result2.is_valid);
    }
}

/// Test suite for the VM integration
#[cfg(test)]
mod vm_integration_tests {
    use super::*;
    use crate::tel::resource::{
        ExecutionContext,
        VmRegId,
        AccessIntent,
    };
    
    /// Test loading a resource into VM memory
    #[test]
    fn test_load_resource() {
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a VM integration
        let mut vm_integration = ResourceVmIntegration::default(manager.clone());
        
        // Create a resource
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        let resource_id = ResourceId::from_bytes(&register_id.0.as_bytes()[..]);
        
        // Create an execution context
        let ctx = ExecutionContext {
            id: "test_context".to_string(),
            initiator: owner.clone(),
            domain: domain.clone(),
        };
        
        // Load the resource into VM memory
        let vm_reg_id = vm_integration.load_resource(&resource_id, &ctx, &owner).unwrap();
        
        // Store the resource back
        vm_integration.store_resource(&vm_reg_id, &ctx, &owner).unwrap();
        
        // Commit the context
        vm_integration.commit_context(&ctx).unwrap();
    }
    
    /// Test access control with VM integration
    #[test]
    fn test_access_control() {
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a VM integration
        let mut vm_integration = ResourceVmIntegration::default(manager.clone());
        
        // Create a resource
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let non_owner = Address::try_from("0xabcdef1234567890abcdef1234567890abcdef12").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        let register_id = create_test_resource(&manager, owner.clone(), domain.clone(), contents.clone()).unwrap();
        let resource_id = ResourceId::from_bytes(&register_id.0.as_bytes()[..]);
        
        // Create an execution context
        let ctx = ExecutionContext {
            id: "test_context".to_string(),
            initiator: owner.clone(),
            domain: domain.clone(),
        };
        
        // Try to load with the wrong owner (should fail)
        let result = vm_integration.load_resource(&resource_id, &ctx, &non_owner);
        assert!(result.is_err());
        
        // Load with the correct owner (should succeed)
        let vm_reg_id = vm_integration.load_resource(&resource_id, &ctx, &owner).unwrap();
        
        // Try to store with the wrong owner (should fail)
        let result = vm_integration.store_resource(&vm_reg_id, &ctx, &non_owner);
        assert!(result.is_err());
        
        // Store with the correct owner (should succeed)
        vm_integration.store_resource(&vm_reg_id, &ctx, &owner).unwrap();
    }
}

/// Test suite for resource snapshots
#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    /// Test creating and restoring a snapshot
    #[test]
    fn test_snapshot_restore() {
        // Create a temporary directory for snapshot storage
        let temp_dir = tempdir().unwrap();
        let snapshot_dir = temp_dir.path().to_path_buf();
        
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a snapshot manager
        let storage = Box::new(FileSnapshotStorage::new(snapshot_dir));
        let snapshot_manager = SnapshotManager::new(
            manager.clone(),
            storage,
            Default::default(),
        );
        
        // Create some resources
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        
        let mut register_ids = Vec::new();
        for i in 0..5 {
            let contents = RegisterContents::Text(format!("Test resource {}", i));
            let id = create_test_resource(&manager, owner.clone(), domain.clone(), contents).unwrap();
            register_ids.push(id);
        }
        
        // Create a snapshot
        let snapshot_id = snapshot_manager.create_snapshot(
            "Test snapshot".to_string(),
            Some(&owner),
            Some(&domain),
            vec!["test".to_string()],
        ).unwrap();
        
        // Delete all resources
        for id in &register_ids {
            manager.delete_register(id).unwrap();
        }
        
        // Run garbage collection to actually remove them
        manager.run_garbage_collection().unwrap();
        
        // Restore the snapshot
        let options = RestoreOptions {
            mode: RestoreMode::Full,
            clear_existing: true,
        };
        let result = snapshot_manager.restore_snapshot(&snapshot_id, options).unwrap();
        
        // Check restore results
        assert_eq!(result.restored_registers, 5);
        assert_eq!(result.skipped_registers, 0);
        assert_eq!(result.errors.len(), 0);
        
        // Verify resources were restored
        for id in &register_ids {
            let register = manager.get_register(id).unwrap();
            assert_eq!(register.owner, owner);
            assert_eq!(register.domain, domain);
            assert_eq!(register.state, RegisterState::Active);
        }
        
        // Test selective restore
        // Delete one resource
        manager.delete_register(&register_ids[0]).unwrap();
        
        // Create another snapshot
        let snapshot_id = snapshot_manager.create_snapshot(
            "Selective test".to_string(),
            Some(&owner),
            Some(&domain),
            vec!["selective".to_string()],
        ).unwrap();
        
        // Delete all resources
        for id in &register_ids {
            manager.delete_register(id).unwrap();
        }
        
        // Run garbage collection
        manager.run_garbage_collection().unwrap();
        
        // Restore only specific registers
        let options = RestoreOptions {
            mode: RestoreMode::Selective {
                register_ids: vec![register_ids[0], register_ids[1]],
            },
            clear_existing: false,
        };
        let result = snapshot_manager.restore_snapshot(&snapshot_id, options).unwrap();
        
        // Check selective restore results
        assert_eq!(result.restored_registers, 2);
        assert_eq!(result.skipped_registers, 3);
        assert_eq!(result.errors.len(), 0);
        
        // Verify only specified resources were restored
        let register = manager.get_register(&register_ids[0]).unwrap();
        assert_eq!(register.state, RegisterState::PendingDeletion); // Was marked for deletion in the snapshot
        
        let register = manager.get_register(&register_ids[1]).unwrap();
        assert_eq!(register.state, RegisterState::Active);
        
        // Try to get a non-restored register (should fail)
        let result = manager.get_register(&register_ids[4]);
        assert!(result.is_err());
    }
    
    /// Test automatic snapshots
    #[test]
    fn test_automatic_snapshots() {
        // Create a temporary directory for snapshot storage
        let temp_dir = tempdir().unwrap();
        let snapshot_dir = temp_dir.path().to_path_buf();
        
        // Create a resource manager
        let manager = Arc::new(ResourceManager::new());
        
        // Create a snapshot manager with frequent automatic snapshots
        let storage = Box::new(FileSnapshotStorage::new(snapshot_dir));
        let config = crate::tel::resource::SnapshotScheduleConfig {
            enabled: true,
            interval: std::time::Duration::from_millis(10), // Very short interval for testing
            max_snapshots: 3,
            all_domains: true,
            domains: Vec::new(),
        };
        let snapshot_manager = SnapshotManager::new(
            manager.clone(),
            storage,
            config,
        );
        
        // Create a resource
        let owner = Address::try_from("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let domain = Domain::new("test_domain");
        let contents = RegisterContents::Text("Test resource".to_string());
        
        create_test_resource(&manager, owner.clone(), domain.clone(), contents).unwrap();
        
        // Create automatic snapshots
        // First snapshot should be created
        let snapshot_id1 = snapshot_manager.create_automatic_snapshot().unwrap();
        assert!(snapshot_id1.is_some());
        
        // Second snapshot shouldn't be created yet (too soon)
        std::thread::sleep(std::time::Duration::from_millis(5));
        let snapshot_id2 = snapshot_manager.create_automatic_snapshot().unwrap();
        assert!(snapshot_id2.is_none());
        
        // Wait and try again, should create another snapshot
        std::thread::sleep(std::time::Duration::from_millis(10));
        let snapshot_id3 = snapshot_manager.create_automatic_snapshot().unwrap();
        assert!(snapshot_id3.is_some());
        
        // Check that we can list snapshots
        let snapshots = snapshot_manager.list_snapshots().unwrap();
        assert!(snapshots.len() >= 2);
        
        // Create more snapshots to trigger pruning
        for _ in 0..5 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            snapshot_manager.create_automatic_snapshot().unwrap();
        }
        
        // Check that old snapshots were pruned
        let snapshots = snapshot_manager.list_snapshots().unwrap();
        assert!(snapshots.len() <= 3); // max_snapshots = 3
    }
} 