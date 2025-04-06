// Purpose: Integration tests for the SimulationController.

use causality_simulation::controller::BasicSimulationController;
use std::path::PathBuf;
use std::time::Duration;
use tokio;
use tokio::time::timeout;
use anyhow::Result;

// Helper function to get the path to the test scenarios directory
fn scenario_path(scenario_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR points to the crate root, so just append the relative path to scenarios
    path.push("tests");
    path.push("scenarios");
    path.push(scenario_name);
    println!("Constructed scenario path (final attempt): {:?}", path);
    path
}

#[tokio::test]
async fn test_basic_in_memory_scenario_start_stop() -> Result<()> {
    // Wrap the entire test in a timeout
    timeout(Duration::from_secs(10), async {
        // 1. Setup: Initialize the controller
        let controller = BasicSimulationController::default()?;
        let scenario_file = "basic_in_memory.toml";
        let scenario_full_path = scenario_path(scenario_file);
        let expected_scenario_id = "basic-in-memory-test";

        // Ensure the scenario file exists before running
        assert!(scenario_full_path.exists(), "Scenario file does not exist: {:?}", scenario_full_path);

        // 2. Action: Load and start the scenario
        println!("Attempting to start scenario: {}", expected_scenario_id);
        let result = controller.load_and_start_scenario(scenario_full_path).await;
        println!("Start scenario result: {:?}", result);
        
        // 3. Assert: Check if scenario started successfully
        assert!(result.is_ok(), "Failed to start scenario: {:?}", result.err());
        let running_scenario_id = result.unwrap();
        assert_eq!(running_scenario_id, expected_scenario_id, "Started scenario ID does not match expected");

        // Small delay to allow agents to run briefly (optional, for observing logs if needed)
        // tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // 4. Action: Stop the scenario
        println!("Attempting to stop scenario: {}", running_scenario_id);
        let stop_result = controller.stop_scenario(&running_scenario_id).await;
        println!("Stop scenario result: {:?}", stop_result);

        // 5. Assert: Check if scenario stopped successfully
        assert!(stop_result.is_ok(), "Failed to stop scenario: {:?}", stop_result.err());

        Ok(())
    }).await?
} 