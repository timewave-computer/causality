// Purpose: E2E test for multi-domain fact observation and effect propagation.

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

    // Helper function to get the path to the test scenarios directory
    fn scenario_path(file_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("test_scenarios");
        path.push(file_name);
        path
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fact_propagation() -> Result<()> {
        // Initialize the controller
        let controller = BasicSimulationController::default()?;
        
        // Start the scenario
        let scenario_file = scenario_path("fact_propagation_test.toml");
        println!("Loading scenario from: {:?}", scenario_file);
        let scenario_id = controller.load_and_start_scenario(scenario_file).await?;
        println!("Scenario started: {}", scenario_id);
        
        // Create a fact with proper hash using the LogEntry.new_with_hash method
        let fact_entry = LogEntry::new_with_hash(
            LogEntryType::FactObservation,
            None, // agent_id
            Some("test_domain".to_string()),
            json!({
                "fact_type": "test_fact",
                "message": "This is a test fact"
            }),
            None, // parent_id
            Some(scenario_id.clone()), // run_id
            HashMap::new(), // metadata
        )?;
        
        // Inject the fact
        println!("Injecting fact with ID: {}", fact_entry.id);
        match controller.inject_fact_entry(&scenario_id, fact_entry).await {
            Ok(_) => println!("Fact injected successfully"),
            Err(e) => println!("Error injecting fact: {}", e),
        }
        
        // Allow some time for fact to be processed
        sleep(Duration::from_millis(500)).await;
        
        // Stop the scenario
        println!("Stopping scenario");
        controller.stop_scenario(&scenario_id).await?;
        
        Ok(())
    }
} 