// Tests for the geo-distributed runner implementation.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::collections::HashMap;

    use causality_types::DomainId;
    use crate::observer::ObserverRegistry;
    use crate::replay::LogStorage;
    use crate::scenario::{Scenario, AgentConfig, SimulationMode};
    use crate::runner::RunnerState;
    use crate::runner::SimulationRunner;
    use crate::runner::geo::{GeoRunner, GeoRunnerConfig, RemoteHostConfig};

    #[tokio::test]
    async fn test_geo_runner_initialization() {
        // Create a simple test scenario
        let scenario = Scenario {
            name: "test_scenario".to_string(),
            description: Some("Test scenario for GeoRunner".to_string()),
            simulation_mode: SimulationMode::GeoDistributed,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    actor_type: "test_agent".to_string(),
                    domain: Some(DomainId::from("test_domain")),
                }
            ],
            initial_state: None,
            invariants: None,
        };

        // Create log storage
        let log_dir = PathBuf::from("/tmp/test-logs");
        let run_id = Some("test-run-123".to_string());
        let log_storage = Arc::new(LogStorage::new(log_dir, run_id).unwrap());

        // Create observer registry
        let observer_registry = Arc::new(ObserverRegistry::new());

        // Create GeoRunner config
        let config = GeoRunnerConfig::default();

        // Create GeoRunner
        let runner = GeoRunner::new(
            config,
            observer_registry,
            log_storage,
        );

        // Initialize the runner with the scenario
        let _ = runner.initialize(&scenario).await.unwrap();

        // Check that the runner is initialized
        assert_eq!(runner.get_state(), RunnerState::Initialized);
    }

    #[tokio::test]
    async fn test_geo_runner_state_management() {
        // Create a simple test scenario
        let scenario = Scenario {
            name: "test_scenario".to_string(),
            description: Some("Test scenario for GeoRunner".to_string()),
            simulation_mode: SimulationMode::GeoDistributed,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    actor_type: "test_agent".to_string(),
                    domain: Some(DomainId::from("test_domain")),
                }
            ],
            initial_state: None,
            invariants: None,
        };

        // Create log storage
        let log_storage = Arc::new(LogStorage::new_temp().unwrap());

        // Create observer registry
        let observer_registry = Arc::new(ObserverRegistry::new());

        // Create GeoRunner config with an empty host assignment to avoid SSH operations
        let config = GeoRunnerConfig {
            host_assignments: HashMap::new(),
            default_host: RemoteHostConfig::default(),
        };

        // Create GeoRunner
        let runner = GeoRunner::new(
            config,
            observer_registry,
            log_storage,
        );

        // Verify initial state
        assert_eq!(runner.get_state(), RunnerState::Initialized);
        
        // Check if we can initialize the runner
        let _ = runner.initialize(&scenario).await.unwrap();
        assert_eq!(runner.get_state(), RunnerState::Initialized);
        
        // Note: We won't test start/pause/resume/stop here since they attempt SSH connections
        // and we're just testing the state management logic at this point
    }
}
