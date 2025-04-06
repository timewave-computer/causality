// Purpose: Provides the controller for managing simulation execution.

use std::collections::HashMap;
use std::sync::Arc;
use crate::runner::SimulationRunner;
use crate::scenario::Scenario;
use anyhow::{Result, anyhow};
use tracing::{info, error, warn};
use std::sync::Mutex;
use crate::runner::RunnerFactory;
use crate::runner::RunnerType;

/// The controller interface for managing simulation execution.
#[async_trait::async_trait]
pub trait SimulationController {
    /// Start a new simulation scenario.
    async fn start_scenario(&self, scenario: Arc<Scenario>) -> Result<()>;
    
    /// Stop a running simulation scenario.
    async fn stop_scenario(&self, scenario_name: &str) -> Result<()>;
    
    /// List all running simulation scenarios.
    async fn list_scenarios(&self) -> Result<Vec<String>>;
    
    /// Get the status of a specific scenario.
    async fn get_scenario_status(&self, scenario_name: &str) -> Result<ScenarioStatus>;
}

/// The status of a simulation scenario.
#[derive(Debug, Clone, PartialEq)]
pub enum ScenarioStatus {
    /// The scenario is running.
    Running,
    /// The scenario is stopped.
    Stopped,
    /// The scenario is not found.
    NotFound,
}

/// Basic implementation of the simulation controller.
pub struct BasicSimulationController {
    /// The runner for executing simulations.
    runner: Arc<dyn SimulationRunner>,
    /// The running scenarios.
    running_scenarios: Arc<tokio::sync::Mutex<HashMap<String, ScenarioState>>>,
}

/// The state of a running scenario.
struct ScenarioState {
    /// The scenario definition.
    scenario: Arc<Scenario>,
    /// The time when the scenario was started.
    start_time: chrono::DateTime<chrono::Utc>,
    /// The effect map from the scenario.
    _effect_map: HashMap<crate::agent::AgentId, Arc<dyn causality_core::effect::Effect>>,
}

impl BasicSimulationController {
    /// Create a new basic simulation controller.
    pub fn new(runner: Arc<dyn SimulationRunner>) -> Self {
        Self {
            runner,
            running_scenarios: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
    
    /// Create a new controller with the default in-memory runner
    pub fn default() -> Result<Self> {
        let factory = RunnerFactory::new();
        let runner = factory.create(RunnerType::InMemory)?;
        Ok(Self::new(runner))
    }
}

#[async_trait::async_trait]
impl SimulationController for BasicSimulationController {
    async fn start_scenario(&self, scenario: Arc<Scenario>) -> Result<()> {
        let scenario_name = scenario.name.clone();
        
        // Check if the scenario is already running
        {
            let scenarios = self.running_scenarios.lock().await;
            if scenarios.contains_key(&scenario_name) {
                return Err(anyhow!("Scenario {} is already running", scenario_name));
            }
        }
        
        // Log the start of the scenario
        info!(scenario_name = %scenario_name, "Starting simulation scenario");
        
        // Run the scenario and get the effect map
        let effect_map = match self.runner.run_scenario(scenario.clone()).await {
            Ok(effect_map) => effect_map,
            Err(e) => {
                error!(scenario_name = %scenario_name, error = %e, "Failed to run scenario");
                return Err(e);
            }
        };
        
        // Add the scenario to the running scenarios
        {
            let mut scenarios = self.running_scenarios.lock().await;
            scenarios.insert(scenario_name.clone(), ScenarioState {
                scenario: scenario.clone(),
                start_time: chrono::Utc::now(),
                _effect_map: effect_map,
            });
        }
        
        info!(scenario_name = %scenario_name, "Simulation scenario started");
        Ok(())
    }
    
    async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        // Check if the scenario is running
        {
            let scenarios = self.running_scenarios.lock().await;
            if !scenarios.contains_key(scenario_name) {
                return Err(anyhow!("Scenario {} is not running", scenario_name));
            }
        }
        
        // Log the stop of the scenario
        info!(scenario_name = %scenario_name, "Stopping simulation scenario");
        
        // Stop the scenario
        if let Err(e) = self.runner.stop_scenario(scenario_name).await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to stop scenario");
            return Err(e);
        }
        
        // Remove the scenario from the running scenarios
        {
            let mut scenarios = self.running_scenarios.lock().await;
            scenarios.remove(scenario_name);
        }
        
        info!(scenario_name = %scenario_name, "Simulation scenario stopped");
        Ok(())
    }
    
    async fn list_scenarios(&self) -> Result<Vec<String>> {
        let scenarios = self.running_scenarios.lock().await;
        Ok(scenarios.keys().cloned().collect())
    }
    
    async fn get_scenario_status(&self, scenario_name: &str) -> Result<ScenarioStatus> {
        let scenarios = self.running_scenarios.lock().await;
        if scenarios.contains_key(scenario_name) {
            Ok(ScenarioStatus::Running)
        } else {
            Ok(ScenarioStatus::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::agent::AgentConfig;

    struct MockSimulationRunner {
        pub started_scenarios: Arc<tokio::sync::Mutex<Vec<String>>>,
        pub stopped_scenarios: Arc<tokio::sync::Mutex<Vec<String>>>,
    }

    impl MockSimulationRunner {
        fn new() -> Self {
            Self {
                started_scenarios: Arc::new(tokio::sync::Mutex::new(Vec::new())),
                stopped_scenarios: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl SimulationRunner for MockSimulationRunner {
        async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<HashMap<crate::agent::AgentId, Arc<dyn causality_core::effect::Effect>>> {
            let mut started = self.started_scenarios.lock().await;
            started.push(scenario.name.clone());
            Ok(HashMap::new())
        }

        async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
            let mut stopped = self.stopped_scenarios.lock().await;
            stopped.push(scenario_name.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_start_and_stop_scenario() {
        let runner = Arc::new(MockSimulationRunner::new());
        let controller = BasicSimulationController::new(runner.clone());

        // Create a test scenario
        let scenario = Arc::new(Scenario {
            name: "test_scenario".to_string(),
            description: Some("Test scenario".to_string()),
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    actor_type: "mock_user".to_string(),
                    options: HashMap::new(),
                },
            ],
        });

        // Start the scenario
        let result = controller.start_scenario(scenario.clone()).await;
        assert!(result.is_ok());

        // Verify the scenario was started
        {
            let started = runner.started_scenarios.lock().await;
            assert_eq!(started.len(), 1);
            assert_eq!(started[0], "test_scenario");
        }

        // Get the status of the scenario
        let status = controller.get_scenario_status("test_scenario").await.unwrap();
        assert_eq!(status, ScenarioStatus::Running);

        // List the scenarios
        let scenarios = controller.list_scenarios().await.unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0], "test_scenario");

        // Stop the scenario
        let result = controller.stop_scenario("test_scenario").await;
        assert!(result.is_ok());

        // Verify the scenario was stopped
        {
            let stopped = runner.stopped_scenarios.lock().await;
            assert_eq!(stopped.len(), 1);
            assert_eq!(stopped[0], "test_scenario");
        }

        // Get the status of the scenario after stopping
        let status = controller.get_scenario_status("test_scenario").await.unwrap();
        assert_eq!(status, ScenarioStatus::NotFound);
    }
}
