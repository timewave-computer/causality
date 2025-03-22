use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::resource::{
    OneTimeRegisterSystem, OneTimeRegisterConfig,
    Register, RegisterId, RegisterContents, RegisterState,
    EpochId, EpochManager, SharedEpochManager,
    SummaryManager, SharedSummaryManager,
    CustomStrategy, SummaryStrategy
};
use crate::types::{Address, Domain};

#[test]
fn test_register_summarization_integration() -> Result<()> {
    // Create a shared summary manager
    let summary_manager = SharedSummaryManager::new();
    
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
        summary_manager: Some(summary_manager),
    };
    
    // Create the register system
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create test registers
    let domains = ["tokens", "assets", "credentials"];
    let owners = ["user1", "user2", "user3"];
    
    for i in 0..9 {
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
        
        // For some registers, set different epoch
        if i >= 6 {
            system.set_register_epoch(&register_id, 2)?;
        }
    }
    
    // Advance epochs
    system.advance_epoch()?;
    assert_eq!(system.current_epoch()?, 1);
    
    // Consume some registers in epoch 1
    let registers = system.get_registers_for_epoch(1)?;
    assert!(registers.len() > 0);
    
    // Consume a couple of registers
    if !registers.is_empty() {
        system.consume_register(&registers[0].register_id, HashMap::new())?;
    }
    
    if registers.len() > 1 {
        system.consume_register(&registers[1].register_id, HashMap::new())?;
    }
    
    // Generate summaries for epoch 1
    let summaries = system.generate_summaries_for_epoch(1, "resource_based")?;
    
    // Should have at least one summary per domain (up to 3)
    assert!(summaries.len() > 0);
    assert!(summaries.len() <= 3);
    
    // Verify summaries
    for summary in &summaries {
        assert_eq!(summary.state, RegisterState::Summary);
        
        // Get the original registers that are summarized
        let summarized_ids = &summary.summarizes;
        assert!(!summarized_ids.is_empty());
        
        // Verify the summary
        let is_valid = system.verify_summary(&summary.register_id)?;
        assert!(is_valid);
        
        // Check that summarized registers point back to the summary
        for id in summarized_ids {
            if let Some(register) = system.get_register(id)? {
                assert_eq!(register.summarized_by, Some(summary.register_id.clone()));
            }
        }
    }
    
    // Test with a custom strategy
    // Create a custom strategy that groups by content type
    let content_type_strategy = Arc::new(CustomStrategy::new(
        "content_type_based",
        Arc::new(|register: &Register| -> Result<String> {
            let content_type = register.metadata.get("content_type")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
                
            Ok(content_type)
        }),
    ));
    
    // Register the custom strategy
    system.register_summary_strategy(content_type_strategy)?;
    
    // Generate summaries with custom strategy
    let custom_summaries = system.generate_summaries_for_epoch(1, "content_type_based")?;
    
    // Should have at most 2 summaries (one per content type)
    assert!(custom_summaries.len() <= 2);
    
    // Advance to epoch 3
    system.advance_epoch()?;
    system.advance_epoch()?;
    assert_eq!(system.current_epoch()?, 3);
    
    // Generate summaries for epoch 2
    let epoch2_summaries = system.generate_summaries_for_epoch(2, "account_based")?;
    
    // Verify epoch 2 summaries
    for summary in &epoch2_summaries {
        assert_eq!(summary.state, RegisterState::Summary);
        
        // Verify it's for epoch 2
        let epoch_str = summary.metadata.get("epoch")
            .expect("Should have epoch in metadata");
        assert_eq!(epoch_str, "2");
        
        // Verify the summary
        let is_valid = system.verify_summary(&summary.register_id)?;
        assert!(is_valid);
    }
    
    Ok(())
}

#[test]
fn test_summary_persistence_through_epochs() -> Result<()> {
    // Create a shared summary manager
    let summary_manager = SharedSummaryManager::new();
    
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
        summary_manager: Some(summary_manager),
    };
    
    // Create the register system
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create test registers in epoch 0
    for i in 0..5 {
        let domain = Domain::new("test_domain");
        let owner = Address::new("test_user");
        
        // Create the register
        system.create_register(
            owner,
            domain,
            RegisterContents::with_string(&format!("Content {}", i)),
            HashMap::new(),
        )?;
    }
    
    // Generate summaries for epoch 0
    let summaries = system.generate_summaries_for_epoch(0, "resource_based")?;
    assert_eq!(summaries.len(), 1); // One domain = one summary
    
    let summary_id = summaries[0].register_id.clone();
    
    // Advance epoch
    system.advance_epoch()?;
    
    // Add more registers in epoch 1
    for i in 0..3 {
        let domain = Domain::new("test_domain");
        let owner = Address::new("test_user");
        
        // Create the register
        system.create_register(
            owner,
            domain,
            RegisterContents::with_string(&format!("New content {}", i)),
            HashMap::new(),
        )?;
    }
    
    // Generate summaries for epoch 1
    let new_summaries = system.generate_summaries_for_epoch(1, "resource_based")?;
    assert_eq!(new_summaries.len(), 1);
    
    // Original summary should still be valid and retrievable
    let original_summary = system.get_register(&summary_id)?
        .expect("Original summary should still exist");
        
    assert_eq!(original_summary.state, RegisterState::Summary);
    
    // Verify the original summary is still valid
    let is_valid = system.verify_summary(&summary_id)?;
    assert!(is_valid);
    
    // The original summary metadata should indicate epoch 0
    let epoch_str = original_summary.metadata.get("epoch")
        .expect("Should have epoch in metadata");
    assert_eq!(epoch_str, "0");
    
    // The new summary metadata should indicate epoch 1
    let new_epoch_str = new_summaries[0].metadata.get("epoch")
        .expect("Should have epoch in metadata");
    assert_eq!(new_epoch_str, "1");
    
    Ok(())
} 