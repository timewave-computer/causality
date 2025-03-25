use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterState, Domain,
    SummaryManager, SharedSummaryManager, ResourceBasedStrategy,
    AccountBasedStrategy, TypeBasedStrategy, CustomStrategy,
    SummaryStrategy, EpochId, BlockHeight
};
use crate::types::Address;

/// Create test registers with different characteristics for testing summarization
fn create_test_registers() -> Vec<Register> {
    let mut registers = Vec::new();
    
    // Create registers with different domains, owners, and states
    for i in 1..10 {
        let domain = if i < 4 {
            Domain::new("tokens")
        } else if i < 7 {
            Domain::new("assets")
        } else {
            Domain::new("credentials")
        };
        
        let owner = if i % 3 == 0 {
            Address::new("user1")
        } else if i % 3 == 1 {
            Address::new("user2")
        } else {
            Address::new("user3")
        };
        
        let state = if i % 4 == 0 {
            RegisterState::Consumed
        } else if i % 4 == 1 {
            RegisterState::Locked
        } else {
            RegisterState::Active
        };
        
        let mut metadata = HashMap::new();
        
        let content_type = if i % 2 == 0 {
            "json"
        } else {
            "text"
        };
        
        metadata.insert("content_type".to_string(), content_type.to_string());
        
        let register = Register {
            register_id: ContentId::new_unique(),
            owner: owner.clone(),
            domain: domain.clone(),
            contents: RegisterContents::with_string(&format!("Test content {}", i)),
            state,
            created_at: 1000 + i as u64,
            updated_at: 1000 + i as u64,
            version: 1,
            metadata,
            archive_reference: None,
            summarizes: Vec::new(),
            summarized_by: None,
            successors: Vec::new(),
            predecessors: Vec::new(),
        };
        
        registers.push(register);
    }
    
    registers
}

#[test]
fn test_resource_based_summary_generation() -> Result<()> {
    // Create test data
    let registers = create_test_registers();
    let summary_manager = SummaryManager::new();
    
    // Generate summaries using resource-based strategy
    let summaries = summary_manager.generate_summaries(
        &registers,
        "resource_based",
        1, // epoch
        1000, // block height
    )?;
    
    // There should be 3 summaries (one for each domain)
    assert_eq!(summaries.len(), 3);
    
    // Check for tokens domain summary
    let tokens_summary = summaries.iter()
        .find(|s| s.metadata.get("summary_group_key") == Some(&"tokens".to_string()))
        .expect("Should have a summary for tokens domain");
    
    // Verify the summary
    assert_eq!(tokens_summary.state, RegisterState::Summary);
    assert_eq!(tokens_summary.owner, Address::system_address());
    assert_eq!(tokens_summary.domain, Domain::new("tokens"));
    
    // Verify the summary validation
    let tokens_registers: Vec<Register> = registers.iter()
        .filter(|r| r.domain == Domain::new("tokens"))
        .cloned()
        .collect();
        
    let is_valid = summary_manager.verify_summary(tokens_summary, &tokens_registers)?;
    assert!(is_valid);
    
    Ok(())
}

#[test]
fn test_account_based_summary_generation() -> Result<()> {
    // Create test data
    let registers = create_test_registers();
    let summary_manager = SummaryManager::new();
    
    // Generate summaries using account-based strategy
    let summaries = summary_manager.generate_summaries(
        &registers,
        "account_based",
        1, // epoch
        1000, // block height
    )?;
    
    // There should be 3 summaries (one for each owner)
    assert_eq!(summaries.len(), 3);
    
    // Check for user1 summary
    let user1_summary = summaries.iter()
        .find(|s| s.metadata.get("summary_group_key") == Some(&"user1".to_string()))
        .expect("Should have a summary for user1");
    
    // Verify the summary
    assert_eq!(user1_summary.state, RegisterState::Summary);
    
    // Check summary contents
    let summary_text = user1_summary.contents.as_string();
    assert!(summary_text.contains("Account summary for user1"));
    
    Ok(())
}

#[test]
fn test_custom_summary_strategy() -> Result<()> {
    // Create a custom strategy that groups by creation time range
    let grouping_fn = Arc::new(|register: &Register| -> Result<String> {
        let time_range = if register.created_at < 1005 {
            "early"
        } else {
            "late"
        };
        
        Ok(time_range.to_string())
    });
    
    // Create the custom strategy
    let strategy = Arc::new(CustomStrategy::new(
        "time_based",
        grouping_fn,
    ));
    
    // Create test data and summary manager
    let registers = create_test_registers();
    let summary_manager = SummaryManager::new();
    
    // Register the custom strategy
    summary_manager.register_strategy(strategy)?;
    
    // Generate summaries using the custom strategy
    let summaries = summary_manager.generate_summaries(
        &registers,
        "time_based",
        1, // epoch
        1000, // block height
    )?;
    
    // Should have summaries for early and late
    assert!(summaries.len() == 2);
    
    // Verify summary structure
    for summary in &summaries {
        assert_eq!(summary.state, RegisterState::Summary);
        assert!(summary.contents.as_string().contains("Custom summary for group"));
    }
    
    Ok(())
}

#[test]
fn test_summary_records() -> Result<()> {
    // Create test data
    let registers = create_test_registers();
    let summary_manager = SummaryManager::new();
    
    // Generate summaries
    let summaries = summary_manager.generate_summaries(
        &registers,
        "resource_based",
        2, // epoch
        1500, // block height
    )?;
    
    // Get summary record for first summary
    let summary = &summaries[0];
    let summary_record = summary_manager.get_summary_record(&summary.register_id)?
        .expect("Summary record should exist");
    
    // Verify summary record
    assert_eq!(summary_record.summary_id, summary.register_id);
    assert_eq!(summary_record.epoch, 2);
    assert_eq!(summary_record.block_height, 1500);
    assert_eq!(summary_record.domain, summary.domain);
    assert_eq!(summary_record.summarized_register_ids.len(), summary.summarizes.len());
    
    Ok(())
} 
