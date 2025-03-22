// Tests for the epoch management system

use std::collections::HashSet;

use crate::error::Result;
use crate::resource::{
    RegisterId, EpochId, BlockHeight,
    EpochManager, SharedEpochManager, ArchivalPolicy,
    SummaryStrategy, ArchiveLocation,
};
use crate::types::Domain;

#[test]
fn test_epoch_lifecycle() -> Result<()> {
    // Create a new epoch manager
    let manager = EpochManager::new();
    
    // Initial epoch should be 1
    assert_eq!(manager.current_epoch()?, 1);
    
    // Initial boundary should be at block 0
    assert_eq!(manager.get_epoch_boundary(1)?, Some(0));
    
    // Advance epochs a few times
    manager.advance_epoch(100)?;
    manager.advance_epoch(200)?;
    manager.advance_epoch(300)?;
    
    // Current epoch should now be 4
    assert_eq!(manager.current_epoch()?, 4);
    
    // Check boundaries
    assert_eq!(manager.get_epoch_boundary(1)?, Some(0));
    assert_eq!(manager.get_epoch_boundary(2)?, Some(100));
    assert_eq!(manager.get_epoch_boundary(3)?, Some(200));
    assert_eq!(manager.get_epoch_boundary(4)?, Some(300));
    
    // Non-existent epoch should return None
    assert_eq!(manager.get_epoch_boundary(5)?, None);
    
    Ok(())
}

#[test]
fn test_register_epoch_mapping() -> Result<()> {
    // Create a new epoch manager
    let manager = EpochManager::new();
    
    // Create some register IDs
    let reg1 = RegisterId::new_unique();
    let reg2 = RegisterId::new_unique();
    let reg3 = RegisterId::new_unique();
    
    // Register registers in different epochs
    manager.register_in_epoch(reg1.clone(), 1)?;
    
    manager.advance_epoch(100)?; // Epoch 2
    manager.register_in_epoch(reg2.clone(), 2)?;
    
    manager.advance_epoch(200)?; // Epoch 3
    manager.register_in_current_epoch(reg3.clone())?;
    
    // Get registers by epoch
    let epoch1_regs = manager.get_registers_in_epoch(1)?;
    let epoch2_regs = manager.get_registers_in_epoch(2)?;
    let epoch3_regs = manager.get_registers_in_epoch(3)?;
    
    // Verify register assignments
    assert_eq!(epoch1_regs.len(), 1);
    assert!(epoch1_regs.contains(&reg1));
    
    assert_eq!(epoch2_regs.len(), 1);
    assert!(epoch2_regs.contains(&reg2));
    
    assert_eq!(epoch3_regs.len(), 1);
    assert!(epoch3_regs.contains(&reg3));
    
    // Register another in the current epoch
    let reg4 = RegisterId::new_unique();
    manager.register_in_current_epoch(reg4.clone())?;
    
    // Verify it was added to the current epoch
    let updated_epoch3_regs = manager.get_registers_in_epoch(3)?;
    assert_eq!(updated_epoch3_regs.len(), 2);
    assert!(updated_epoch3_regs.contains(&reg3));
    assert!(updated_epoch3_regs.contains(&reg4));
    
    Ok(())
}

#[test]
fn test_epoch_determination_by_block() -> Result<()> {
    // Create a new epoch manager with custom boundaries
    let manager = EpochManager::new();
    
    // Set up epoch boundaries
    // Epoch 1: blocks 0-99
    // Epoch 2: blocks 100-249
    // Epoch 3: blocks 250-399
    // Epoch 4: blocks 400+
    manager.set_epoch_boundary(1, 0)?;
    manager.set_epoch_boundary(2, 100)?;
    manager.set_epoch_boundary(3, 250)?;
    manager.set_epoch_boundary(4, 400)?;
    
    // Test block to epoch mapping
    assert_eq!(manager.get_epoch_for_block(0)?, 1);
    assert_eq!(manager.get_epoch_for_block(50)?, 1);
    assert_eq!(manager.get_epoch_for_block(99)?, 1);
    
    assert_eq!(manager.get_epoch_for_block(100)?, 2);
    assert_eq!(manager.get_epoch_for_block(175)?, 2);
    assert_eq!(manager.get_epoch_for_block(249)?, 2);
    
    assert_eq!(manager.get_epoch_for_block(250)?, 3);
    assert_eq!(manager.get_epoch_for_block(300)?, 3);
    assert_eq!(manager.get_epoch_for_block(399)?, 3);
    
    assert_eq!(manager.get_epoch_for_block(400)?, 4);
    assert_eq!(manager.get_epoch_for_block(500)?, 4);
    
    // Test a block beyond defined boundaries (should return the latest epoch)
    assert_eq!(manager.get_epoch_for_block(1000)?, 4);
    
    Ok(())
}

#[test]
fn test_archival_policy_configuration() -> Result<()> {
    // Create a new epoch manager
    let manager = EpochManager::new();
    
    // Check default policy
    let default_policy = manager.get_archival_policy()?;
    assert_eq!(default_policy.keep_epochs, 2);
    assert_eq!(default_policy.prune_after, 3);
    
    // Create a custom policy
    let custom_policy = ArchivalPolicy {
        keep_epochs: 4,
        prune_after: 6,
        summary_strategy: SummaryStrategy::SummarizeByAccount,
        archive_location: ArchiveLocation::LocalStorage("/custom/path".to_string()),
    };
    
    // Update policy
    manager.set_archival_policy(custom_policy.clone())?;
    
    // Verify policy was updated
    let updated_policy = manager.get_archival_policy()?;
    assert_eq!(updated_policy.keep_epochs, 4);
    assert_eq!(updated_policy.prune_after, 6);
    match updated_policy.summary_strategy {
        SummaryStrategy::SummarizeByAccount => (),
        _ => panic!("Unexpected summary strategy"),
    }
    match updated_policy.archive_location {
        ArchiveLocation::LocalStorage(path) => {
            assert_eq!(path, "/custom/path");
        },
        _ => panic!("Unexpected archive location"),
    }
    
    Ok(())
}

#[test]
fn test_garbage_collection_eligibility() -> Result<()> {
    // Create a new epoch manager with a specific policy
    let policy = ArchivalPolicy {
        keep_epochs: 2,
        prune_after: 5,  // GC after 5 epochs
        summary_strategy: SummaryStrategy::SummarizeByResource,
        archive_location: ArchiveLocation::LocalStorage("./archives".to_string()),
    };
    
    let manager = EpochManager::with_config(1, 0, policy);
    
    // Advance several epochs
    for i in 1..10 {
        manager.advance_epoch(i * 100)?;
    }
    
    // Current epoch should be 10
    assert_eq!(manager.current_epoch()?, 10);
    
    // Check eligibility
    // Epoch 1: age = 9, should be eligible (prune_after = 5)
    assert!(manager.is_epoch_eligible_for_gc(1)?);
    
    // Epoch 4: age = 6, should be eligible
    assert!(manager.is_epoch_eligible_for_gc(4)?);
    
    // Epoch 5: age = 5, should be eligible (exactly at threshold)
    assert!(manager.is_epoch_eligible_for_gc(5)?);
    
    // Epoch 6: age = 4, should not be eligible
    assert!(!manager.is_epoch_eligible_for_gc(6)?);
    
    // Epoch 9: age = 1, should not be eligible
    assert!(!manager.is_epoch_eligible_for_gc(9)?);
    
    Ok(())
}

#[test]
fn test_register_removal() -> Result<()> {
    // Create a new epoch manager
    let manager = EpochManager::new();
    
    // Create some registers in epoch 1
    let reg1 = RegisterId::new_unique();
    let reg2 = RegisterId::new_unique();
    
    manager.register_in_current_epoch(reg1.clone())?;
    manager.register_in_current_epoch(reg2.clone())?;
    
    // Verify registers are in epoch 1
    let epoch1_regs = manager.get_registers_in_epoch(1)?;
    assert_eq!(epoch1_regs.len(), 2);
    assert!(epoch1_regs.contains(&reg1));
    assert!(epoch1_regs.contains(&reg2));
    
    // Remove a register (simulation of GC)
    let removed = manager.remove_register(&reg1, 1)?;
    assert!(removed);
    
    // Verify register was removed
    let updated_epoch1_regs = manager.get_registers_in_epoch(1)?;
    assert_eq!(updated_epoch1_regs.len(), 1);
    assert!(!updated_epoch1_regs.contains(&reg1));
    assert!(updated_epoch1_regs.contains(&reg2));
    
    // Try to remove non-existent register
    let non_existent = RegisterId::new_unique();
    let removed = manager.remove_register(&non_existent, 1)?;
    assert!(!removed);
    
    // Try to remove from non-existent epoch
    let removed = manager.remove_register(&reg2, 99)?;
    assert!(!removed);
    
    Ok(())
}

#[test]
fn test_shared_epoch_manager() -> Result<()> {
    // Create a shared epoch manager
    let shared_manager = SharedEpochManager::new();
    
    // Test operations on the shared manager
    assert_eq!(shared_manager.current_epoch()?, 1);
    
    // Advance epoch
    shared_manager.advance_epoch(100)?;
    assert_eq!(shared_manager.current_epoch()?, 2);
    
    // Register a register
    let reg_id = RegisterId::new_unique();
    shared_manager.register_in_current_epoch(reg_id.clone())?;
    
    // Get registers in epoch 2
    let epoch2_regs = shared_manager.get_registers_in_epoch(2)?;
    assert_eq!(epoch2_regs.len(), 1);
    assert!(epoch2_regs.contains(&reg_id));
    
    // Test policy modification
    let policy = ArchivalPolicy {
        keep_epochs: 3,
        prune_after: 7,
        summary_strategy: SummaryStrategy::SummarizeByType,
        archive_location: ArchiveLocation::LocalStorage("./shared_archives".to_string()),
    };
    
    shared_manager.set_archival_policy(policy.clone())?;
    
    let updated_policy = shared_manager.get_archival_policy()?;
    assert_eq!(updated_policy.keep_epochs, 3);
    assert_eq!(updated_policy.prune_after, 7);
    
    Ok(())
}

#[test]
fn test_concurrent_epoch_access() -> Result<()> {
    use std::thread;
    
    // Create a shared epoch manager
    let shared_manager = SharedEpochManager::new();
    let arc_manager = shared_manager.inner();
    
    // Spawn multiple threads to register registers in parallel
    let mut handles = Vec::new();
    for i in 0..5 {
        let manager_clone = arc_manager.clone();
        let handle = thread::spawn(move || {
            let reg_id = RegisterId::new_unique();
            manager_clone.register_in_current_epoch(reg_id.clone()).unwrap();
            reg_id
        });
        handles.push(handle);
    }
    
    // Collect register IDs from threads
    let mut register_ids = Vec::new();
    for handle in handles {
        register_ids.push(handle.join().unwrap());
    }
    
    // Verify all registers were added to epoch 1
    let epoch1_regs = shared_manager.get_registers_in_epoch(1)?;
    assert_eq!(epoch1_regs.len(), 5);
    
    for reg_id in register_ids {
        assert!(epoch1_regs.contains(&reg_id));
    }
    
    Ok(())
} 