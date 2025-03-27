// obligation.rs - Tests for the obligation manager
//
// This file contains tests for the obligation manager which tracks and
// enforces capability obligations for agents.

use crate::resource_types::ResourceId;
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::resource::agent::types::{AgentId, AgentType, AgentState};
use crate::resource::agent::agent::{Agent, AgentImpl, AgentBuilder};
use crate::resource::agent::obligation::{
    Obligation, ObligationId, ObligationType, ObligationStatus,
    ObligationManager, ObligationError, CapabilityObligation
};

use std::collections::HashMap;
use std::time::Duration;
use tokio;
use chrono::{Utc, TimeZone};

// Helper function to create a test agent ID
fn create_test_agent_id(name: &str) -> AgentId {
    AgentId::from_content_hash(ContentHash::calculate(name.as_bytes()))
}

#[tokio::test]
async fn test_obligation_creation_and_tracking() {
    let agent_id = create_test_agent_id("test-agent");
    let manager = ObligationManager::new();
    
    // Create various types of obligations
    let use_within_time = Obligation::new(
        "cap1",
        agent_id.clone(),
        ObligationType::UseWithinTime(Duration::from_secs(3600)),
        agent_id.clone(),
    );
    
    let use_max_times = Obligation::new(
        "cap2",
        agent_id.clone(),
        ObligationType::UseMaximumTimes(5),
        agent_id.clone(),
    );
    
    let use_min_times = Obligation::new(
        "cap3",
        agent_id.clone(),
        ObligationType::UseMinimumTimes(3),
        agent_id.clone(),
    );
    
    let report_usage = Obligation::new(
        "cap4",
        agent_id.clone(),
        ObligationType::ReportUsage {
            to_agent: create_test_agent_id("admin"),
            due: Utc::now() + chrono::Duration::days(7),
        },
        agent_id.clone(),
    );
    
    // Add obligations to the manager
    let ob1_id = manager.add_obligation(use_within_time).await.unwrap();
    let ob2_id = manager.add_obligation(use_max_times).await.unwrap();
    let ob3_id = manager.add_obligation(use_min_times).await.unwrap();
    let ob4_id = manager.add_obligation(report_usage).await.unwrap();
    
    // Get all obligations for the agent
    let agent_obligations = manager.get_agent_obligations(&agent_id).await.unwrap();
    assert_eq!(agent_obligations.len(), 4);
    
    // Get obligations for specific capabilities
    let cap1_obligations = manager.get_capability_obligations("cap1").await.unwrap();
    assert_eq!(cap1_obligations.len(), 1);
    
    let cap2_obligations = manager.get_capability_obligations("cap2").await.unwrap();
    assert_eq!(cap2_obligations.len(), 1);
    
    // Verify the obligations by ID
    let ob1 = manager.get_obligation(&ob1_id).await.unwrap();
    assert!(matches!(ob1.obligation_type(), ObligationType::UseWithinTime(_)));
    
    let ob2 = manager.get_obligation(&ob2_id).await.unwrap();
    assert!(matches!(ob2.obligation_type(), ObligationType::UseMaximumTimes(_)));
    
    let ob3 = manager.get_obligation(&ob3_id).await.unwrap();
    assert!(matches!(ob3.obligation_type(), ObligationType::UseMinimumTimes(_)));
    
    let ob4 = manager.get_obligation(&ob4_id).await.unwrap();
    assert!(matches!(ob4.obligation_type(), ObligationType::ReportUsage { .. }));
}

#[tokio::test]
async fn test_obligation_fulfillment_and_violations() {
    let agent_id = create_test_agent_id("test-agent");
    let manager = ObligationManager::new();
    
    // Add a use-within-time obligation
    let time_obligation_id = manager.add_obligation_to_capability(
        "time-cap",
        &agent_id,
        ObligationType::UseWithinTime(Duration::from_secs(60)),
    ).await.unwrap();
    
    // Add a max-times obligation
    let max_obligation_id = manager.add_obligation_to_capability(
        "max-cap",
        &agent_id,
        ObligationType::UseMaximumTimes(2),
    ).await.unwrap();
    
    // Add a min-times obligation
    let min_obligation_id = manager.add_obligation_to_capability(
        "min-cap",
        &agent_id,
        ObligationType::UseMinimumTimes(3),
    ).await.unwrap();
    
    // Record usage for time-cap - should fulfill that obligation
    manager.record_capability_usage("time-cap", &agent_id).await.unwrap();
    
    // Check if fulfilled
    let time_ob = manager.get_obligation(&time_obligation_id).await.unwrap();
    assert!(time_ob.is_fulfilled());
    
    // Record usage for max-cap twice - should still be active
    manager.record_capability_usage("max-cap", &agent_id).await.unwrap();
    manager.record_capability_usage("max-cap", &agent_id).await.unwrap();
    
    let max_ob = manager.get_obligation(&max_obligation_id).await.unwrap();
    assert!(max_ob.is_active());
    
    // Record one more usage - should violate the obligation
    manager.record_capability_usage("max-cap", &agent_id).await.unwrap();
    
    let max_ob = manager.get_obligation(&max_obligation_id).await.unwrap();
    assert!(max_ob.is_violated());
    
    // Record usage for min-cap twice - should still need one more
    manager.record_capability_usage("min-cap", &agent_id).await.unwrap();
    manager.record_capability_usage("min-cap", &agent_id).await.unwrap();
    
    let min_ob = manager.get_obligation(&min_obligation_id).await.unwrap();
    assert!(min_ob.is_active());
    
    // Record one more usage - should fulfill the obligation
    manager.record_capability_usage("min-cap", &agent_id).await.unwrap();
    
    let min_ob = manager.get_obligation(&min_obligation_id).await.unwrap();
    assert!(min_ob.is_fulfilled());
}

#[tokio::test]
async fn test_obligation_deadline_enforcement() {
    let agent_id = create_test_agent_id("test-agent");
    let manager = ObligationManager::new();
    
    // Create an obligation with a past deadline
    let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
    
    let overdue_obligation = Obligation::new(
        "overdue-cap",
        agent_id.clone(),
        ObligationType::RevokeAt(one_hour_ago),
        agent_id.clone(),
    );
    
    let overdue_id = manager.add_obligation(overdue_obligation).await.unwrap();
    
    // Check overdue obligations
    let overdue_obs = manager.get_overdue_obligations().await.unwrap();
    assert_eq!(overdue_obs.len(), 1);
    
    // Enforce obligations
    let result = manager.enforce_obligations().await.unwrap();
    
    // Check for revocations
    assert_eq!(result.revocations.len(), 1);
    let (revoke_agent, revoke_cap, revoke_ob) = &result.revocations[0];
    assert_eq!(revoke_agent, &agent_id);
    assert_eq!(revoke_cap, "overdue-cap");
    assert_eq!(revoke_ob, &overdue_id);
    
    // Create an obligation that will become overdue after a short time
    let future_obligation = Obligation::new(
        "future-cap",
        agent_id.clone(),
        ObligationType::UseWithinTime(Duration::from_millis(50)),
        agent_id.clone(),
    );
    
    let future_id = manager.add_obligation(future_obligation).await.unwrap();
    
    // Wait for the obligation to become overdue
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check and enforce overdue obligations
    let overdue = manager.check_overdue_obligations().await.unwrap();
    assert_eq!(overdue.len(), 1);
    
    // Check that the obligation is now violated
    let violated_ob = manager.get_obligation(&future_id).await.unwrap();
    assert!(violated_ob.is_violated());
}

#[tokio::test]
async fn test_obligation_reporting() {
    let agent_id = create_test_agent_id("test-agent");
    let manager = ObligationManager::new();
    
    // Add various obligations with different statuses
    let ob1 = Obligation::new(
        "cap1",
        agent_id.clone(),
        ObligationType::UseWithinTime(Duration::from_secs(3600)),
        agent_id.clone(),
    );
    
    let mut ob2 = Obligation::new(
        "cap2",
        agent_id.clone(),
        ObligationType::UseMaximumTimes(5),
        agent_id.clone(),
    );
    ob2.fulfill("Used as required");
    
    let mut ob3 = Obligation::new(
        "cap3",
        agent_id.clone(),
        ObligationType::UseMinimumTimes(3),
        agent_id.clone(),
    );
    ob3.violate("Failed to use minimum times");
    
    let mut ob4 = Obligation::new(
        "cap4",
        agent_id.clone(),
        ObligationType::DoNotDelegate,
        agent_id.clone(),
    );
    ob4.waive("Policy changed", agent_id.clone());
    
    // Add all obligations
    manager.add_obligation(ob1).await.unwrap();
    manager.add_obligation(ob2).await.unwrap();
    manager.add_obligation(ob3).await.unwrap();
    manager.add_obligation(ob4).await.unwrap();
    
    // Generate a summary
    let summary = manager.generate_summary().await.unwrap();
    
    // Check summary counts
    assert_eq!(summary.total, 4);
    assert_eq!(summary.active, 1);
    assert_eq!(summary.fulfilled, 1);
    assert_eq!(summary.violated, 1);
    assert_eq!(summary.waived, 1);
    
    // Generate an agent report
    let report = manager.generate_agent_report(&agent_id).await.unwrap();
    
    // Report should contain the summary information
    assert!(report.contains("Total Obligations: 4"));
    assert!(report.contains("Active: 1"));
    assert!(report.contains("Fulfilled: 1"));
    assert!(report.contains("Violated: 1"));
    assert!(report.contains("Waived: 1"));
}

#[tokio::test]
async fn test_obligation_custom_types() {
    let agent_id = create_test_agent_id("test-agent");
    let manager = ObligationManager::new();
    
    // Create a custom obligation type
    let mut params = HashMap::new();
    params.insert("review_before_use".to_string(), "true".to_string());
    params.insert("deadline".to_string(), (Utc::now().timestamp() + 86400).to_string());
    
    let custom_obligation = Obligation::new(
        "custom-cap",
        agent_id.clone(),
        ObligationType::Custom {
            obligation_type: "review".to_string(),
            parameters: params,
        },
        agent_id.clone(),
    );
    
    let custom_id = manager.add_obligation(custom_obligation).await.unwrap();
    
    // Verify the custom obligation
    let custom_ob = manager.get_obligation(&custom_id).await.unwrap();
    
    if let ObligationType::Custom { obligation_type, parameters } = custom_ob.obligation_type() {
        assert_eq!(obligation_type, "review");
        assert_eq!(parameters.get("review_before_use"), Some(&"true".to_string()));
    } else {
        panic!("Expected Custom obligation type");
    }
    
    // Check if it has a deadline
    assert!(custom_ob.has_deadline());
    assert!(custom_ob.deadline().is_some());
}

#[tokio::test]
async fn test_obligation_removal() {
    let agent_id = create_test_agent_id("test-agent");
    let manager = ObligationManager::new();
    
    // Add an obligation
    let obligation = Obligation::new(
        "remove-cap",
        agent_id.clone(),
        ObligationType::UseWithinTime(Duration::from_secs(3600)),
        agent_id.clone(),
    );
    
    let obligation_id = manager.add_obligation(obligation).await.unwrap();
    
    // Verify it was added
    let obligations = manager.get_agent_obligations(&agent_id).await.unwrap();
    assert_eq!(obligations.len(), 1);
    
    // Remove the obligation
    manager.remove_obligation(&obligation_id).await.unwrap();
    
    // Verify it was removed
    let obligations = manager.get_agent_obligations(&agent_id).await.unwrap();
    assert_eq!(obligations.len(), 0);
    
    // Attempt to get the removed obligation should fail
    let result = manager.get_obligation(&obligation_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_obligation_with_agents() {
    // Create test agents
    let agent1 = AgentBuilder::new()
        .agent_type(AgentType::User)
        .state(AgentState::Active)
        .build();
    
    let agent2 = AgentBuilder::new()
        .agent_type(AgentType::Operator)
        .state(AgentState::Active)
        .build();
    
    let manager = ObligationManager::new();
    
    // Add capabilities with obligations
    let obligation_id = manager.add_obligation_to_capability(
        "restricted-cap",
        agent1.agent_id(),
        ObligationType::UseMaximumTimes(1),
    ).await.unwrap();
    
    // Record first usage - should be allowed
    manager.record_capability_usage("restricted-cap", agent1.agent_id()).await.unwrap();
    
    // Check if another usage would be allowed
    let can_use = manager.check_capability_usage("restricted-cap", agent1.agent_id()).await.unwrap();
    assert!(!can_use, "Should not allow usage beyond the maximum");
    
    // Another agent should still be able to use the capability
    let can_use_agent2 = manager.check_capability_usage("restricted-cap", agent2.agent_id()).await.unwrap();
    assert!(can_use_agent2, "Different agent should be able to use the capability");
}

#[tokio::test]
async fn test_obligation_effects() {
    let agent_id = create_test_agent_id("test-agent");
    
    // Create effects for various obligation operations
    let create_effect = ObligationEffect::create(
        agent_id.clone(),
        "effect-cap",
        ObligationType::UseWithinTime(Duration::from_secs(3600)),
    );
    
    assert!(matches!(create_effect.effect_type, ObligationEffectType::Create));
    assert_eq!(create_effect.capability_id, "effect-cap");
    assert!(create_effect.obligation_type.is_some());
    
    let obligation_id = ObligationId::new("test-id");
    
    let update_effect = ObligationEffect::update(
        agent_id.clone(),
        "effect-cap",
        obligation_id.clone(),
        ObligationStatus::Fulfilled {
            when: Utc::now(),
            how: "Fulfilled via effect".to_string(),
        },
    );
    
    assert!(matches!(update_effect.effect_type, ObligationEffectType::Update(_)));
    assert_eq!(update_effect.capability_id, "effect-cap");
    assert_eq!(update_effect.obligation_id, Some(obligation_id.clone()));
    
    let remove_effect = ObligationEffect::remove(
        agent_id.clone(),
        "effect-cap",
        obligation_id.clone(),
    );
    
    assert!(matches!(remove_effect.effect_type, ObligationEffectType::Remove));
    assert_eq!(remove_effect.capability_id, "effect-cap");
    assert_eq!(remove_effect.obligation_id, Some(obligation_id));
    
    let usage_effect = ObligationEffect::record_usage(
        agent_id.clone(),
        "effect-cap",
    );
    
    assert!(matches!(usage_effect.effect_type, ObligationEffectType::RecordUsage));
    assert_eq!(usage_effect.capability_id, "effect-cap");
    assert!(usage_effect.obligation_id.is_none());
} 