// Purpose: E2E test for cross-domain effects propagation across multiple domains.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;
    use std::collections::HashMap;
    use serde_json::json;
    use tokio::time::sleep;
    use anyhow::Result;
    
    use causality_simulation::controller::BasicSimulationController;
    use causality_simulation::controller::SimulationController;
    use causality_simulation::replay::{LogEntry, LogEntryType};
    use causality_simulation::agent::agent_id;

    // Helper function to get the path to the test scenarios directory
    fn scenario_path(file_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("test_scenarios");
        path.push(file_name);
        path
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cross_domain_effects() -> Result<()> {
        // Initialize the controller
        let controller = BasicSimulationController::default()?;
        
        // Start the scenario
        let scenario_file = scenario_path("cross_domain_effects_test.toml");
        println!("Loading scenario from: {:?}", scenario_file);
        let scenario_id = controller.load_and_start_scenario(scenario_file).await?;
        println!("Scenario started: {}", scenario_id);
        
        // Step 1: Create a transaction in Domain A
        let mut tx_a_metadata = HashMap::new();
        tx_a_metadata.insert("scenario_name".to_string(), scenario_id.clone());
        tx_a_metadata.insert("type".to_string(), "transaction".to_string());
        
        let transaction_a = json!({
            "action": "transfer",
            "source_account": "account_1", 
            "target_account": "account_2",
            "amount": 100,
            "domain": "DomainA",
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        let tx_a_entry = LogEntry::new_with_hash(
            LogEntryType::DomainEvent,
            Some(agent_id::from_string("domain-a-agent")),
            Some("DomainA".to_string()),
            transaction_a.clone(),
            None,
            Some(scenario_id.clone()),
            tx_a_metadata,
        )?;
        
        // Submit the Domain A transaction
        controller.inject_fact_entry(&scenario_id, tx_a_entry.clone()).await?;
        println!("Domain A transaction injected");
        
        // Allow time for processing
        sleep(Duration::from_millis(100)).await;
        
        // Step 2: Create a cross-domain transaction from Domain A to Domain B
        let mut cross_tx_metadata = HashMap::new();
        cross_tx_metadata.insert("scenario_name".to_string(), scenario_id.clone());
        cross_tx_metadata.insert("type".to_string(), "cross_domain_transaction".to_string());
        
        let cross_domain_tx = json!({
            "action": "cross_domain_transfer",
            "source_domain": "DomainA",
            "target_domain": "DomainB",
            "source_account": "account_1",
            "target_account": "ext_account_1",
            "amount": 200,
            "reference": tx_a_entry.id,
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        let cross_tx_entry = LogEntry::new_with_hash(
            LogEntryType::AgentAction,
            Some(agent_id::from_string("domain-a-agent")),
            Some("DomainA".to_string()),
            cross_domain_tx.clone(),
            Some(tx_a_entry.id.clone()),
            Some(scenario_id.clone()),
            cross_tx_metadata,
        )?;
        
        // Submit the cross-domain transaction
        controller.inject_fact_entry(&scenario_id, cross_tx_entry.clone()).await?;
        println!("Cross-domain transaction injected");
        
        // Allow time for processing
        sleep(Duration::from_millis(100)).await;
        
        // Step 3: Create a fact observation in Domain B that confirms receipt
        let mut receipt_metadata = HashMap::new();
        receipt_metadata.insert("scenario_name".to_string(), scenario_id.clone());
        receipt_metadata.insert("type".to_string(), "transaction_receipt".to_string());
        
        let receipt_fact = json!({
            "action": "receipt",
            "source_domain": "DomainA",
            "source_transaction": cross_tx_entry.id,
            "received_amount": 200,
            "target_account": "ext_account_1",
            "status": "confirmed",
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        let receipt_entry = LogEntry::new_with_hash(
            LogEntryType::FactObservation,
            Some(agent_id::from_string("domain-b-agent")),
            Some("DomainB".to_string()),
            receipt_fact.clone(),
            Some(cross_tx_entry.id.clone()),
            Some(scenario_id.clone()),
            receipt_metadata,
        )?;
        
        // Submit the receipt fact
        controller.inject_fact_entry(&scenario_id, receipt_entry.clone()).await?;
        println!("Domain B receipt fact injected");
        
        // Allow time for processing
        sleep(Duration::from_millis(100)).await;
        
        // Step 4: Create a multi-domain effect from Domain B to both Domain A and C
        let mut multi_effect_metadata = HashMap::new();
        multi_effect_metadata.insert("scenario_name".to_string(), scenario_id.clone());
        multi_effect_metadata.insert("type".to_string(), "multi_domain_effect".to_string());
        
        let multi_domain_effect = json!({
            "action": "multi_domain_effect",
            "source_domain": "DomainB",
            "target_domains": ["DomainA", "DomainC"],
            "effect_type": "notification",
            "data": {
                "message": "Processing complete",
                "reference": receipt_entry.id
            },
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        let multi_effect_entry = LogEntry::new_with_hash(
            LogEntryType::AgentAction,
            Some(agent_id::from_string("domain-b-agent")),
            Some("DomainB".to_string()),
            multi_domain_effect.clone(),
            Some(receipt_entry.id.clone()),
            Some(scenario_id.clone()),
            multi_effect_metadata,
        )?;
        
        // Submit the multi-domain effect
        controller.inject_fact_entry(&scenario_id, multi_effect_entry.clone()).await?;
        println!("Multi-domain effect injected");
        
        // Allow time for processing
        sleep(Duration::from_millis(100)).await;
        
        // Step 5: Create an acknowledgment fact from Domain C
        let mut ack_metadata = HashMap::new();
        ack_metadata.insert("scenario_name".to_string(), scenario_id.clone());
        ack_metadata.insert("type".to_string(), "acknowledgment".to_string());
        
        let ack_fact = json!({
            "action": "acknowledge",
            "source_domain": "DomainB",
            "source_effect": multi_effect_entry.id,
            "status": "processed",
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        let ack_entry = LogEntry::new_with_hash(
            LogEntryType::FactObservation,
            Some(agent_id::from_string("domain-c-agent")),
            Some("DomainC".to_string()),
            ack_fact.clone(),
            Some(multi_effect_entry.id.clone()),
            Some(scenario_id.clone()),
            ack_metadata,
        )?;
        
        // Submit the acknowledgment fact
        controller.inject_fact_entry(&scenario_id, ack_entry).await?;
        println!("Domain C acknowledgment fact injected");
        
        // Allow time for all processing to complete
        sleep(Duration::from_millis(500)).await;
        
        // Retrieve all log entries for this scenario
        let logs = controller.get_scenario_logs(&scenario_id, None).await?;
        println!("Retrieved {} log entries", logs.len());
        
        // Verify the causal chain integrity
        let mut domain_a_txs = 0;
        let mut domain_b_receipts = 0;
        let mut domain_c_acks = 0;
        let mut cross_domain_txs = 0;
        let mut multi_domain_effects = 0;
        
        for entry in &logs {
            if let Some(domain) = &entry.domain {
                if domain == "DomainA" {
                    domain_a_txs += 1;
                } else if domain == "DomainB" {
                    if entry.entry_type == LogEntryType::FactObservation {
                        domain_b_receipts += 1;
                    }
                } else if domain == "DomainC" {
                    domain_c_acks += 1;
                }
            }
            
            if let Some(metadata_type) = entry.metadata.get("type") {
                if metadata_type == "cross_domain_transaction" {
                    cross_domain_txs += 1;
                } else if metadata_type == "multi_domain_effect" {
                    multi_domain_effects += 1;
                }
            }
        }
        
        println!("Domain A transactions: {}", domain_a_txs);
        println!("Domain B receipts: {}", domain_b_receipts);
        println!("Domain C acknowledgments: {}", domain_c_acks);
        println!("Cross-domain transactions: {}", cross_domain_txs);
        println!("Multi-domain effects: {}", multi_domain_effects);
        
        // Verify that all expected event types occurred
        assert!(domain_a_txs > 0, "No Domain A transactions found");
        assert!(domain_b_receipts > 0, "No Domain B receipts found");
        assert!(domain_c_acks > 0, "No Domain C acknowledgments found");
        assert!(cross_domain_txs > 0, "No cross-domain transactions found");
        assert!(multi_domain_effects > 0, "No multi-domain effects found");
        
        // Verify causal chains by examining parent references
        let mut has_complete_chain = false;
        
        // Use a reference to logs to avoid moving it
        for entry in &logs {
            if entry.domain.as_ref().map_or(false, |d| d == "DomainC") {
                if let Some(parent_id) = &entry.parent_id {
                    // Find the parent entry
                    let parent_entry = logs.iter().find(|e| e.id == *parent_id);
                    if let Some(parent) = parent_entry {
                        if parent.domain.as_ref().map_or(false, |d| d == "DomainB") {
                            has_complete_chain = true;
                            break;
                        }
                    }
                }
            }
        }
        
        assert!(has_complete_chain, "No complete causal chain found from Domain A to Domain C");
        
        // Stop the scenario
        println!("Stopping scenario");
        controller.stop_scenario(&scenario_id).await?;
        
        println!("Cross-domain effects test completed successfully");
        
        Ok(())
    }
} 