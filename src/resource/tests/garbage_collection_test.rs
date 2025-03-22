use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::error::Result;
use crate::resource::{
    OneTimeRegisterSystem, OneTimeRegisterConfig,
    SharedEpochManager, SharedArchiveManager, SharedGarbageCollectionManager,
    GarbageCollectionConfig, CompressionFormat, RegisterId, RegisterContents,
    RegisterState, EpochId
};
use crate::types::{Address, Domain};

#[test]
fn test_gc_eligibility() -> Result<()> {
    // Create required managers
    let epoch_manager = SharedEpochManager::new();
    let archive_manager = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Zstd));
    
    // Create a custom GC config with shorter retention
    let gc_config = GarbageCollectionConfig {
        retention_epochs: 1,
        min_age_seconds: None, // No minimum age for testing
        ..Default::default()
    };
    
    let gc_manager = SharedGarbageCollectionManager::new(
        gc_config,
        Some(epoch_manager.clone()),
        Some(archive_manager.clone()),
    );
    
    // Configure the register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
        gc_manager: Some(gc_manager),
    };
    
    // Create the register system
    let mut system = OneTimeRegisterSystem::new(config)?;
    
    // Create registers in epoch 0
    let mut epoch0_ids = Vec::new();
    for i in 0..5 {
        let register_id = system.create_register(
            Address::new(&format!("owner{}", i)),
            Domain::new("test"),
            RegisterContents::with_string(&format!("Content {}", i)),
            HashMap::new(),
        )?;
        
        epoch0_ids.push(register_id);
    }
    
    // Initially, no registers should be eligible for GC
    for id in &epoch0_ids {
        assert!(!system.is_eligible_for_gc(id)?);
    }
    
    // Archive some registers
    let archived_id = epoch0_ids[0].clone();
    system.archive_register(&archived_id)?;
    
    // Consume some registers
    let consumed_id = epoch0_ids[1].clone();
    system.consume_register(&consumed_id, HashMap::new())?;
    
    // Even archived/consumed registers shouldn't be eligible yet because they're in the current epoch
    assert!(!system.is_eligible_for_gc(&archived_id)?);
    assert!(!system.is_eligible_for_gc(&consumed_id)?);
    
    // Advance to epoch 1
    system.advance_epoch()?;
    
    // Create registers in epoch 1
    let mut epoch1_ids = Vec::new();
    for i in 0..3 {
        let register_id = system.create_register(
            Address::new(&format!("owner{}", i)),
            Domain::new("test"),
            RegisterContents::with_string(&format!("Epoch 1 Content {}", i)),
            HashMap::new(),
        )?;
        
        epoch1_ids.push(register_id);
    }
    
    // Archived/consumed registers from epoch 0 should still not be eligible
    // because we're only at epoch 1 and retention is 1 epoch (so we keep epoch 0)
    assert!(!system.is_eligible_for_gc(&archived_id)?);
    assert!(!system.is_eligible_for_gc(&consumed_id)?);
    
    // Advance to epoch 2
    system.advance_epoch()?;
    
    // Now epoch 0 registers should be eligible for GC if they're archived or consumed
    assert!(system.is_eligible_for_gc(&archived_id)?);
    assert!(system.is_eligible_for_gc(&consumed_id)?);
    
    // Other registers from epoch 0 aren't eligible because they're still active
    for id in &epoch0_ids[2..] {
        assert!(!system.is_eligible_for_gc(id)?);
    }
    
    // Epoch 1 registers shouldn't be eligible yet
    for id in &epoch1_ids {
        assert!(!system.is_eligible_for_gc(id)?);
    }
    
    Ok(())
}

#[test]
fn test_garbage_collection() -> Result<()> {
    // Create required managers
    let epoch_manager = SharedEpochManager::new();
    let archive_manager = SharedArchiveManager::new_in_memory(None);
    
    // Create a custom GC config with shorter retention
    let gc_config = GarbageCollectionConfig {
        retention_epochs: 1,
        min_age_seconds: None, // No minimum age for testing
        require_archived: false, // Allow collecting consumed registers
        ..Default::default()
    };
    
    let gc_manager = SharedGarbageCollectionManager::new(
        gc_config,
        Some(epoch_manager.clone()),
        Some(archive_manager.clone()),
    );
    
    // Configure the register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
        gc_manager: Some(gc_manager),
    };
    
    // Create the register system
    let mut system = OneTimeRegisterSystem::new(config)?;
    
    // Create registers in epoch 0
    let domain = Domain::new("test");
    let mut epoch0_ids = Vec::new();
    
    for i in 0..5 {
        let register_id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Content {}", i)),
            HashMap::new(),
        )?;
        
        epoch0_ids.push(register_id);
    }
    
    // Archive two registers
    let archived_id1 = epoch0_ids[0].clone();
    let archived_id2 = epoch0_ids[1].clone();
    system.archive_register(&archived_id1)?;
    system.archive_register(&archived_id2)?;
    
    // Consume one register
    let consumed_id = epoch0_ids[2].clone();
    system.consume_register(&consumed_id, HashMap::new())?;
    
    // Advance to epoch 1
    system.advance_epoch()?;
    
    // Create registers in epoch 1
    let mut epoch1_ids = Vec::new();
    for i in 0..3 {
        let register_id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Epoch 1 Content {}", i)),
            HashMap::new(),
        )?;
        
        epoch1_ids.push(register_id);
    }
    
    // Advance to epoch 2
    system.advance_epoch()?;
    
    // Garbage collect epoch 0
    let collected_ids = system.garbage_collect_epoch(0)?;
    
    // Should have collected 3 registers (2 archived, 1 consumed)
    assert_eq!(collected_ids.len(), 3);
    assert!(collected_ids.contains(&archived_id1));
    assert!(collected_ids.contains(&archived_id2));
    assert!(collected_ids.contains(&consumed_id));
    
    // The registers should no longer be available
    assert!(system.get_register(&archived_id1)?.is_none());
    assert!(system.get_register(&archived_id2)?.is_none());
    assert!(system.get_register(&consumed_id)?.is_none());
    
    // But they should be marked as garbage collected
    assert!(system.is_garbage_collected(&archived_id1)?);
    assert!(system.is_garbage_collected(&archived_id2)?);
    assert!(system.is_garbage_collected(&consumed_id)?);
    
    // The collection time should be available
    assert!(system.get_garbage_collection_time(&archived_id1)?.is_some());
    
    // The active registers from epoch 0 should still be available
    assert!(system.get_register(&epoch0_ids[3])?.is_some());
    assert!(system.get_register(&epoch0_ids[4])?.is_some());
    
    // All epoch 1 registers should still be available
    for id in &epoch1_ids {
        assert!(system.get_register(id)?.is_some());
    }
    
    Ok(())
}

#[test]
fn test_auto_gc_on_epoch_advance() -> Result<()> {
    // Create required managers
    let epoch_manager = SharedEpochManager::new();
    let archive_manager = SharedArchiveManager::new_in_memory(None);
    
    // Create a custom GC config with auto-gc enabled
    let gc_config = GarbageCollectionConfig {
        retention_epochs: 1,
        min_age_seconds: None,
        auto_gc_on_epoch_advance: true,
        require_archived: true,
        ..Default::default()
    };
    
    let gc_manager = SharedGarbageCollectionManager::new(
        gc_config,
        Some(epoch_manager.clone()),
        Some(archive_manager.clone()),
    );
    
    // Configure the register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
        gc_manager: Some(gc_manager),
    };
    
    // Create the register system
    let mut system = OneTimeRegisterSystem::new(config)?;
    
    // Create and archive registers in epoch 0
    let domain = Domain::new("test");
    let mut archived_ids = Vec::new();
    
    for i in 0..3 {
        let register_id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Content {}", i)),
            HashMap::new(),
        )?;
        
        system.archive_register(&register_id)?;
        archived_ids.push(register_id);
    }
    
    // Advance to epoch 1 (this won't trigger GC yet)
    system.advance_epoch()?;
    
    // All registers should still be available
    for id in &archived_ids {
        assert!(system.get_register(id)?.is_some());
    }
    
    // Create and archive more registers in epoch 1
    let mut epoch1_ids = Vec::new();
    for i in 0..2 {
        let register_id = system.create_register(
            Address::new(&format!("epoch1_owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Epoch 1 Content {}", i)),
            HashMap::new(),
        )?;
        
        system.archive_register(&register_id)?;
        epoch1_ids.push(register_id);
    }
    
    // Advance to epoch 2 (this should trigger GC of epoch 0)
    system.advance_epoch()?;
    
    // Epoch 0 registers should no longer be available
    for id in &archived_ids {
        assert!(system.get_register(id)?.is_none());
        assert!(system.is_garbage_collected(id)?);
    }
    
    // Epoch 1 registers should still be available
    for id in &epoch1_ids {
        assert!(system.get_register(id)?.is_some());
    }
    
    // Advance to epoch 3 (this should trigger GC of epoch 1)
    system.advance_epoch()?;
    
    // Now epoch 1 registers should also be gone
    for id in &epoch1_ids {
        assert!(system.get_register(id)?.is_none());
        assert!(system.is_garbage_collected(id)?);
    }
    
    Ok(())
}

#[test]
fn test_gc_all_eligible() -> Result<()> {
    // Create required managers
    let epoch_manager = SharedEpochManager::new();
    let archive_manager = SharedArchiveManager::new_in_memory(None);
    
    // Create a custom GC config
    let gc_config = GarbageCollectionConfig {
        retention_epochs: 1,
        min_age_seconds: None,
        auto_gc_on_epoch_advance: false,
        ..Default::default()
    };
    
    let gc_manager = SharedGarbageCollectionManager::new(
        gc_config,
        Some(epoch_manager.clone()),
        Some(archive_manager.clone()),
    );
    
    // Configure the register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
        gc_manager: Some(gc_manager),
    };
    
    // Create the register system
    let mut system = OneTimeRegisterSystem::new(config)?;
    
    // Create registers in epochs 0, 1, 2, 3
    let domain = Domain::new("test");
    let mut register_ids_by_epoch = HashMap::new();
    
    for epoch in 0..4 {
        // Advance to the target epoch
        while system.get_current_epoch()? < epoch {
            system.advance_epoch()?;
        }
        
        let mut epoch_ids = Vec::new();
        for i in 0..3 {
            let register_id = system.create_register(
                Address::new(&format!("epoch{}_owner{}", epoch, i)),
                domain.clone(),
                RegisterContents::with_string(&format!("Epoch {} Content {}", epoch, i)),
                HashMap::new(),
            )?;
            
            // Archive the register
            system.archive_register(&register_id)?;
            epoch_ids.push(register_id);
        }
        
        register_ids_by_epoch.insert(epoch, epoch_ids);
    }
    
    // Now we're at epoch 3, and we have archived registers in epochs 0, 1, 2, 3
    
    // Epochs 0, 1 should be eligible for GC (since retention is 1 epoch)
    // Epochs 2, 3 should not be eligible
    
    // Garbage collect all eligible registers
    let collected_ids = system.garbage_collect_all_eligible()?;
    
    // Should have collected registers from epochs 0 and 1 (6 total)
    assert_eq!(collected_ids.len(), 6);
    
    // Verify epoch 0, 1 registers are gone
    for epoch in 0..2 {
        for id in &register_ids_by_epoch[&epoch] {
            assert!(system.get_register(id)?.is_none());
            assert!(system.is_garbage_collected(id)?);
        }
    }
    
    // Verify epoch 2, 3 registers still exist
    for epoch in 2..4 {
        for id in &register_ids_by_epoch[&epoch] {
            assert!(system.get_register(id)?.is_some());
            assert!(!system.is_garbage_collected(id)?);
        }
    }
    
    // Advance to epoch 4
    system.advance_epoch()?;
    
    // Collect again - should now collect epoch 2
    let collected_ids = system.garbage_collect_all_eligible()?;
    assert_eq!(collected_ids.len(), 3); // 3 registers from epoch 2
    
    // Verify epoch 2 registers are now gone
    for id in &register_ids_by_epoch[&2] {
        assert!(system.get_register(id)?.is_none());
        assert!(system.is_garbage_collected(id)?);
    }
    
    // Epoch 3 registers should still exist
    for id in &register_ids_by_epoch[&3] {
        assert!(system.get_register(id)?.is_some());
        assert!(!system.is_garbage_collected(id)?);
    }
    
    Ok(())
}

#[test]
fn test_gc_with_custom_predicate() -> Result<()> {
    // Create required managers
    let epoch_manager = SharedEpochManager::new();
    let archive_manager = SharedArchiveManager::new_in_memory(None);
    
    // Create a custom predicate that only allows GC for registers with specific metadata
    let predicate = Arc::new(|register: &crate::resource::Register| -> bool {
        register.metadata.get("can_gc").map_or(false, |v| v == "true")
    });
    
    // Create a custom GC config with the predicate
    let gc_config = GarbageCollectionConfig {
        retention_epochs: 0, // No epoch retention
        min_age_seconds: None,
        custom_gc_predicate: Some(predicate),
        ..Default::default()
    };
    
    let gc_manager = SharedGarbageCollectionManager::new(
        gc_config,
        Some(epoch_manager.clone()),
        Some(archive_manager.clone()),
    );
    
    // Configure the register system
    let config = OneTimeRegisterConfig {
        current_block_height: 1000,
        nullifier_registry: None,
        transition_system: None,
        proof_manager: None,
        migration_registry: None,
        epoch_manager: Some(epoch_manager),
        summary_manager: None,
        archive_manager: Some(archive_manager),
        gc_manager: Some(gc_manager),
    };
    
    // Create the register system
    let mut system = OneTimeRegisterSystem::new(config)?;
    
    // Create registers - some with can_gc=true, some without
    let domain = Domain::new("test");
    let mut can_gc_ids = Vec::new();
    let mut cannot_gc_ids = Vec::new();
    
    for i in 0..6 {
        let mut metadata = HashMap::new();
        
        // Even-indexed registers can be GC'd
        if i % 2 == 0 {
            metadata.insert("can_gc".to_string(), "true".to_string());
        }
        
        let register_id = system.create_register(
            Address::new(&format!("owner{}", i)),
            domain.clone(),
            RegisterContents::with_string(&format!("Content {}", i)),
            metadata,
        )?;
        
        // Archive all registers
        system.archive_register(&register_id)?;
        
        if i % 2 == 0 {
            can_gc_ids.push(register_id);
        } else {
            cannot_gc_ids.push(register_id);
        }
    }
    
    // Try to garbage collect all registers
    let collected_ids = system.garbage_collect_all_eligible()?;
    
    // Should have collected only registers with can_gc=true
    assert_eq!(collected_ids.len(), can_gc_ids.len());
    
    // Verify registers with can_gc=true are gone
    for id in &can_gc_ids {
        assert!(system.get_register(id)?.is_none());
        assert!(system.is_garbage_collected(id)?);
    }
    
    // Verify registers without can_gc=true still exist
    for id in &cannot_gc_ids {
        assert!(system.get_register(id)?.is_some());
        assert!(!system.is_garbage_collected(id)?);
    }
    
    Ok(())
} 