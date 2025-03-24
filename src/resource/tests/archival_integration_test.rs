use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::resource::{
    OneTimeRegisterSystem, OneTimeRegisterConfig,
    SharedEpochManager, SharedSummaryManager, SharedArchiveManager,
    Register, RegisterId, RegisterContents, RegisterState, CompressionFormat,
    ArchiveReference
};
use crate::types::{Address, Domain};

#[test]
fn test_register_archival_integration() -> Result<()> {
    // Create a shared archive manager (in-memory for tests)
    let archive_manager = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Zstd));
    
    // Create a shared epoch manager
    let epoch_manager = SharedEpochManager::new();
    
    // Create a shared summary manager
    let summary_manager = SharedSummaryManager::new();
    
    // Configure the one-time register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: Some(summary_manager),
        archive_manager: Some(archive_manager),
    };
    
    // Create the register system
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create test registers
    let domains = ["tokens", "assets", "credentials"];
    let owners = ["user1", "user2", "user3"];
    let mut register_ids = Vec::new();
    
    for i in 0..6 {
        let domain = Domain::new(domains[i % 3]);
        let owner = Address::new(owners[i % 3]);
        let mut metadata = HashMap::new();
        
        metadata.insert("content_type".to_string(), 
            format!("type{}", i % 2 + 1));
        
        // Create the register
        let register_id = system.create_register(
            owner,
            domain,
            RegisterContents::with_string(&format!("Content for register {}", i)),
            metadata,
        )?;
        
        register_ids.push(register_id);
        
        // Put some registers in different epochs
        if i >= 3 {
            system.set_register_epoch(&register_id, 1)?;
        }
    }
    
    // Archive a single register
    let register_id = &register_ids[0];
    let archive_ref = system.archive_register(register_id)?;
    
    // Check that the register is now archived
    let register = system.get_register(register_id)?.unwrap();
    assert_eq!(register.state, RegisterState::Archived);
    assert!(register.archive_reference.is_some());
    
    // Verify the archive reference matches
    let ref_from_register = register.archive_reference.unwrap();
    assert_eq!(ref_from_register.epoch, archive_ref.epoch);
    assert_eq!(ref_from_register.archive_hash, archive_ref.archive_hash);
    
    // Verify we can retrieve from archive
    let archived = system.retrieve_from_archive(&archive_ref)?.unwrap();
    assert_eq!(archived.register_id, *register_id);
    assert_eq!(archived.state, RegisterState::Archived);
    
    // Archive an entire epoch
    let archive_refs = system.archive_epoch(0)?;
    
    // Should have archived the remaining registers in epoch 0 (2 more, since one is already archived)
    assert_eq!(archive_refs.len(), 2);
    
    // Verify all epoch 0 registers are archived
    for i in 0..3 {
        let reg = system.get_register(&register_ids[i])?.unwrap();
        assert_eq!(reg.state, RegisterState::Archived);
        assert!(reg.archive_reference.is_some());
    }
    
    // Epoch 1 registers should still be active
    for i in 3..6 {
        let reg = system.get_register(&register_ids[i])?.unwrap();
        assert_eq!(reg.state, RegisterState::Active);
        assert!(reg.archive_reference.is_none());
    }
    
    // Archive epoch 1
    let archive_refs = system.archive_epoch(1)?;
    assert_eq!(archive_refs.len(), 3); // 3 registers in epoch 1
    
    // Now all registers should be archived
    for id in &register_ids {
        let reg = system.get_register(id)?.unwrap();
        assert_eq!(reg.state, RegisterState::Archived);
    }
    
    // Verify integrity check works
    for id in &register_ids {
        let reg = system.get_register(id)?.unwrap();
        let archive_ref = reg.archive_reference.as_ref().unwrap();
        
        let verified = system.verify_archive(archive_ref)?;
        assert!(verified, "Archive integrity check failed for register {}", id);
    }
    
    Ok(())
}

#[test]
fn test_archival_with_consumed_registers() -> Result<()> {
    // Create a shared archive manager
    let archive_manager = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Gzip));
    
    // Create a shared epoch manager
    let epoch_manager = SharedEpochManager::new();
    
    // Configure the one-time register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
    };
    
    // Create the register system
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create test registers
    let domain = Domain::new("test_domain");
    let owner = Address::new("test_owner");
    
    // Create three registers
    let id1 = system.create_register(
        owner.clone(),
        domain.clone(),
        RegisterContents::with_string("Register 1"),
        HashMap::new(),
    )?;
    
    let id2 = system.create_register(
        owner.clone(),
        domain.clone(),
        RegisterContents::with_string("Register 2"),
        HashMap::new(),
    )?;
    
    let id3 = system.create_register(
        owner.clone(),
        domain.clone(),
        RegisterContents::with_string("Register 3"),
        HashMap::new(),
    )?;
    
    // Consume one register
    system.consume_register(&id1, HashMap::new())?;
    
    // Lock one register
    system.lock_register(&id2)?;
    
    // Archive all registers in epoch 0
    let archive_refs = system.archive_epoch(0)?;
    assert_eq!(archive_refs.len(), 3); // All three registers
    
    // Verify all registers are archived
    let reg1 = system.get_register(&id1)?.unwrap();
    let reg2 = system.get_register(&id2)?.unwrap();
    let reg3 = system.get_register(&id3)?.unwrap();
    
    assert_eq!(reg1.state, RegisterState::Archived);
    assert_eq!(reg2.state, RegisterState::Archived);
    assert_eq!(reg3.state, RegisterState::Archived);
    
    // Retrieve from archive and check original states are preserved
    let archived1 = system.retrieve_from_archive(reg1.archive_reference.as_ref().unwrap())?.unwrap();
    let archived2 = system.retrieve_from_archive(reg2.archive_reference.as_ref().unwrap())?.unwrap();
    let archived3 = system.retrieve_from_archive(reg3.archive_reference.as_ref().unwrap())?.unwrap();
    
    assert_eq!(archived1.register_id, id1);
    assert_eq!(archived2.register_id, id2);
    assert_eq!(archived3.register_id, id3);
    
    // Original states are preserved in the archive metadata
    assert_eq!(archived1.metadata.get("original_state").map(|s| s.as_str()), Some("Consumed"));
    assert_eq!(archived2.metadata.get("original_state").map(|s| s.as_str()), Some("Locked"));
    assert_eq!(archived3.metadata.get("original_state").map(|s| s.as_str()), Some("Active"));
    
    Ok(())
}

#[test]
fn test_multiple_epoch_archival() -> Result<()> {
    // Create managers
    let archive_manager = SharedArchiveManager::new_in_memory(None);
    let epoch_manager = SharedEpochManager::new();
    
    // Configure the one-time register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
    };
    
    // Create the register system
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create registers in epoch 0
    let domain = Domain::new("test_domain");
    let mut epoch0_ids = Vec::new();
    
    for i in 0..3 {
        let id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Epoch 0 Register {}", i)),
            HashMap::new(),
        )?;
        
        epoch0_ids.push(id);
    }
    
    // Advance to epoch 1
    system.advance_epoch()?;
    
    // Create registers in epoch 1
    let mut epoch1_ids = Vec::new();
    
    for i in 0..2 {
        let id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Epoch 1 Register {}", i)),
            HashMap::new(),
        )?;
        
        epoch1_ids.push(id);
    }
    
    // Archive epoch 0
    let archive_refs_0 = system.archive_epoch(0)?;
    assert_eq!(archive_refs_0.len(), 3);
    
    // Verify epoch 0 registers are archived
    for id in &epoch0_ids {
        let reg = system.get_register(id)?.unwrap();
        assert_eq!(reg.state, RegisterState::Archived);
    }
    
    // Verify epoch 1 registers are still active
    for id in &epoch1_ids {
        let reg = system.get_register(id)?.unwrap();
        assert_eq!(reg.state, RegisterState::Active);
    }
    
    // Advance to epoch 2
    system.advance_epoch()?;
    
    // Create registers in epoch 2
    let mut epoch2_ids = Vec::new();
    
    for i in 0..2 {
        let id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Epoch 2 Register {}", i)),
            HashMap::new(),
        )?;
        
        epoch2_ids.push(id);
    }
    
    // Archive epoch 1
    let archive_refs_1 = system.archive_epoch(1)?;
    assert_eq!(archive_refs_1.len(), 2);
    
    // Verify epoch 1 registers are now archived
    for id in &epoch1_ids {
        let reg = system.get_register(id)?.unwrap();
        assert_eq!(reg.state, RegisterState::Archived);
    }
    
    // Epoch 2 registers should still be active
    for id in &epoch2_ids {
        let reg = system.get_register(id)?.unwrap();
        assert_eq!(reg.state, RegisterState::Active);
    }
    
    // Verify we can retrieve all archived registers from all epochs
    for id in &epoch0_ids {
        let reg = system.get_register(id)?.unwrap();
        let archive_ref = reg.archive_reference.as_ref().unwrap();
        
        let retrieved = system.retrieve_from_archive(archive_ref)?.unwrap();
        assert_eq!(retrieved.register_id, *id);
        assert_eq!(retrieved.state, RegisterState::Archived);
    }
    
    for id in &epoch1_ids {
        let reg = system.get_register(id)?.unwrap();
        let archive_ref = reg.archive_reference.as_ref().unwrap();
        
        let retrieved = system.retrieve_from_archive(archive_ref)?.unwrap();
        assert_eq!(retrieved.register_id, *id);
        assert_eq!(retrieved.state, RegisterState::Archived);
    }
    
    Ok(())
} 
