// Purpose: Tests for the InMemoryRunner implementation
//
// This file contains tests to verify the InMemoryRunner works correctly with interior mutability

use std::sync::Arc;
use std::collections::HashMap;

use tokio::time::sleep;
use tokio::time::Duration;
use anyhow::Result;
use async_trait::async_trait;
use tempfile::tempdir;
use tokio::time::timeout;

use crate::runner::{RunnerState, SimulationRunner};
use crate::runner::in_memory::{InMemoryRunner, AgentFactory, EffectFactory};
use crate::observer::{ObserverRegistry};
use crate::scenario::Scenario;
use crate::replay::LogStorage;
use crate::agent::{AgentId, SimulatedAgent};
use causality_core::effect::{Effect, EffectContext, EffectType};
use causality_core::effect::outcome::EffectOutcome;

// Isolated test module to avoid conflicts with other tests
#[cfg(test)]
mod in_memory_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock agent factory that tracks if it was called
    struct TestAgentFactory {
        was_called: Arc<AtomicBool>,
    }

    impl TestAgentFactory {
        fn new() -> Self {
            Self {
                was_called: Arc::new(AtomicBool::new(false)),
            }
        }

        fn was_called(&self) -> bool {
            self.was_called.load(Ordering::SeqCst)
        }
    }

    impl AgentFactory for TestAgentFactory {
        fn create_agent(&self, _config: &crate::scenario::AgentConfig) -> Result<Arc<dyn SimulatedAgent>> {
            self.was_called.store(true, Ordering::SeqCst);
            
            // Create a mock agent
            let agent_id = crate::agent::agent_id::from_string("test-agent");
            let agent = MockAgent::new(agent_id);
            Ok(Arc::new(agent))
        }
    }

    // Mock effect factory
    struct TestEffectFactory {
        was_called: Arc<AtomicBool>,
    }

    impl TestEffectFactory {
        fn new() -> Self {
            Self {
                was_called: Arc::new(AtomicBool::new(false)),
            }
        }

        fn was_called(&self) -> bool {
            self.was_called.load(Ordering::SeqCst)
        }
    }

    impl EffectFactory for TestEffectFactory {
        fn create_effect(&self, _config: &crate::scenario::AgentConfig) -> Result<Arc<dyn Effect>> {
            self.was_called.store(true, Ordering::SeqCst);
            
            // Return a mock effect
            Ok(Arc::new(MockEffect {}))
        }
    }

    // Simple mock agent implementation
    struct MockAgent {
        id: AgentId,
        on_run: Arc<AtomicBool>,
        on_shutdown: Arc<AtomicBool>,
    }

    impl MockAgent {
        fn new(id: AgentId) -> Self {
            Self {
                id,
                on_run: Arc::new(AtomicBool::new(false)),
                on_shutdown: Arc::new(AtomicBool::new(false)),
            }
        }

        fn was_run(&self) -> bool {
            self.on_run.load(Ordering::SeqCst)
        }

        fn was_shutdown(&self) -> bool {
            self.on_shutdown.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl SimulatedAgent for MockAgent {
        async fn run(&self, _config: crate::agent::SimulationAgentConfig) -> Result<()> {
            self.on_run.store(true, Ordering::SeqCst);
            
            // Just sleep a bit to simulate work
            sleep(Duration::from_millis(100)).await;
            Ok(())
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        async fn shutdown(&self) -> Result<()> {
            self.on_shutdown.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    // Mock effect implementation
    #[derive(Debug)]
    struct MockEffect {}

    #[async_trait]
    impl Effect for MockEffect {
        fn effect_type(&self) -> EffectType {
            EffectType::Custom("test".to_string())
        }

        fn description(&self) -> String {
            "Mock effect for testing".to_string()
        }

        async fn execute(
            &self,
            _context: &dyn EffectContext,
        ) -> std::result::Result<EffectOutcome, causality_core::effect::EffectError> {
            // Use the builder or factory methods to create a success outcome
            Ok(causality_core::effect::outcome::EffectOutcome::success(HashMap::new()))
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // Basic test scenario builder
    fn create_test_scenario() -> Scenario {
        let mut scenario = Scenario {
            name: "test-scenario".to_string(),
            description: Some("Test scenario for in-memory runner".to_string()),
            simulation_mode: crate::scenario::SimulationMode::InMemory,
            agents: Vec::new(),
            initial_state: None,
            invariants: None,
        };
        
        // Add a mock agent to the scenario
        let agent_config = crate::scenario::AgentConfig {
            id: "test-agent".to_string(),
            actor_type: "mock".to_string(),
            domain: None,
        };
        
        // Manually add the agent to the scenario
        scenario.agents.push(agent_config);
        
        scenario
    }

    // Create a test runner with mocks
    fn create_test_runner() -> (
        InMemoryRunner,
        Arc<TestAgentFactory>,
        Arc<TestEffectFactory>,
        Arc<ObserverRegistry>,
        Arc<LogStorage>,
    ) {
        let temp_dir = tempdir().unwrap();
        let log_storage = Arc::new(LogStorage::new(temp_dir.path(), None).unwrap());
        let observer_registry = Arc::new(ObserverRegistry::new());
        
        let agent_factory = Arc::new(TestAgentFactory::new());
        let effect_factory = Arc::new(TestEffectFactory::new());
        
        let runner = InMemoryRunner::with_components(
            observer_registry.clone(),
            log_storage.clone(),
            agent_factory.clone(),
            effect_factory.clone(),
        );
        
        (runner, agent_factory, effect_factory, observer_registry, log_storage)
    }

    #[tokio::test]
    async fn test_in_memory_runner_lifecycle() -> Result<()> {
        // Wrap the test in a timeout to prevent hanging
        timeout(Duration::from_secs(10), async {
            let (runner, agent_factory, _effect_factory, _observer_registry, _log_storage) = create_test_runner();
            let scenario = create_test_scenario();
            
            // Thread-safe way to share the runner
            let runner = Arc::new(runner);
            
            // Initialize the runner
            runner.initialize(&scenario).await?;
            
            // After initialization, state should be Initialized
            assert_eq!(runner.get_state(), RunnerState::Initialized);
            
            // Check if agent factory was called
            assert!(agent_factory.was_called());
            
            // Start the simulation
            runner.start(&scenario).await?;
            
            // After starting, state should be Running
            assert_eq!(runner.get_state(), RunnerState::Running);
            
            // Pause the simulation
            runner.pause().await?;
            
            // After pausing, state should be Paused
            assert_eq!(runner.get_state(), RunnerState::Paused);
            
            // Resume the simulation
            runner.resume().await?;
            
            // After resuming, state should be Running
            assert_eq!(runner.get_state(), RunnerState::Running);
            
            // Stop the simulation
            runner.stop().await?;
            
            // After stopping, state should be Stopped
            assert_eq!(runner.get_state(), RunnerState::Stopped);
            
            Ok(())
        }).await?
    }
} 