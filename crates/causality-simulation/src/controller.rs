// Purpose: Provides the controller for managing simulation execution.

use std::collections::HashMap;
use std::sync::Arc;
use crate::runner::{SimulationRunner, RunnerType, RunnerFactory, SimulationRunnerEnum, RunnerState};
use crate::scenario::Scenario;
use anyhow::{Result, anyhow, Context};
use tracing::{info, error, warn};
use std::sync::Mutex;
use crate::observer::ObserverRegistry;
// Comment out the unresolved import for now
// use crate::invariant::{InvariantObserver, InvariantResult};
use crate::agent::AgentId;
use crate::replay::{LogEntry, AsyncLogStorageAdapter};
use serde_json::Value;
use toml;

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
    
    /// Get invariant violations for a scenario
    async fn get_invariant_violations(&self, scenario_name: &str) -> Result<Vec<String>>;
    
    /// Inject a fact into a running simulation
    async fn inject_fact(&self, scenario_name: &str, fact_data: Value) -> Result<()>;
    
    /// Query the state of an agent in a running simulation
    async fn query_agent_state(&self, scenario_name: &str, agent_id: &AgentId, query: &str) -> Result<Value>;
    
    /// Pause a running simulation
    async fn pause_scenario(&self, scenario_name: &str) -> Result<()>;
    
    /// Resume a paused simulation
    async fn resume_scenario(&self, scenario_name: &str) -> Result<()>;
    
    /// Get logs for a running scenario
    async fn get_scenario_logs(&self, scenario_name: &str, limit: Option<usize>) -> Result<Vec<LogEntry>>;
    
    /// Get the observer registry
    fn observer_registry(&self) -> Arc<ObserverRegistry>;
}

/// The status of a simulation scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioStatus {
    /// The scenario is running.
    Running,
    /// The scenario is paused.
    Paused,
    /// The scenario is stopped.
    Stopped,
    /// The scenario is not found.
    NotFound,
}

/// Basic implementation of the simulation controller.
pub struct BasicSimulationController {
    /// The runner that executes the simulation
    runner: Arc<tokio::sync::Mutex<SimulationRunnerEnum>>,
    /// Map of running scenarios
    running_scenarios: Arc<tokio::sync::Mutex<HashMap<String, ScenarioState>>>,
    /// Observer registry for recording simulation events
    observer_registry: Arc<ObserverRegistry>,
    /// Log storage adapter
    log_storage: Arc<AsyncLogStorageAdapter>,
}

/// The state of a running scenario.
struct ScenarioState {
    /// The scenario definition.
    scenario: Arc<Scenario>,
    /// The time when the scenario was started.
    start_time: chrono::DateTime<chrono::Utc>,
    /// Current status of the scenario
    status: ScenarioStatus,
}

impl BasicSimulationController {
    /// Create a new simulation controller
    pub fn new(runner: SimulationRunnerEnum, log_storage: Arc<AsyncLogStorageAdapter>) -> Self {
        Self {
            runner: Arc::new(tokio::sync::Mutex::new(runner)),
            running_scenarios: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            observer_registry: Arc::new(ObserverRegistry::new()),
            log_storage,
        }
    }
    
    /// Create a new controller with the default in-memory runner
    pub fn default() -> Result<Self> {
        let log_storage = Arc::new(AsyncLogStorageAdapter::new_temp()?);
        let factory = RunnerFactory::default()?;
        
        // Create an in-memory runner by default
        let runner = factory.create_runner(RunnerType::InMemory)?;
        
        Ok(Self::new(runner, log_storage))
    }
    
    /// Get the observer registry
    pub fn observer_registry(&self) -> Arc<ObserverRegistry> {
        self.observer_registry.clone()
    }
    
    /// Set up invariant checking for a scenario
    fn setup_invariant_checking(&self, _scenario: &Arc<Scenario>) -> Result<()> {
        // Simplified version that doesn't depend on InvariantObserver
        Ok(())
    }
    
    /// Create a fact injection log entry
    async fn create_fact_injection_entry(&self, scenario_name: &str, fact_data: Value) -> Result<LogEntry> {
        // Create a log entry for the fact injection
        let mut metadata = HashMap::new();
        metadata.insert("scenario_name".to_string(), scenario_name.to_string());
        metadata.insert("type".to_string(), "fact_injection".to_string());
        
        let entry = LogEntry::new(
            crate::replay::LogEntryType::FactObservation,
            None,  // No specific agent
            None,  // No specific domain
            fact_data.clone(),
            None,  // No parent
            None,  // Will use the default run ID
            metadata,
        )?;
        
        Ok(entry)
    }
    
    /// Process a fact injection
    async fn process_fact_injection(&self, scenario_name: &str, fact_data: Value) -> Result<()> {
        // Create the log entry for the fact
        let entry = self.create_fact_injection_entry(scenario_name, fact_data.clone()).await?;
        
        // Store the log entry
        self.log_storage.store_entry(&entry).await?;
        
        // Notify observers about the fact injection
        self.observer_registry.notify_log_entry(entry);
        
        // Let the runner know about the fact (this would typically involve forwarding it to agents)
        // In a real implementation, this would use a specific runner API to distribute the fact
        info!(scenario_name = %scenario_name, "Fact injected into simulation");
        
        Ok(())
    }
    
    /// Process an agent state query
    async fn process_agent_query(&self, scenario_name: &str, agent_id: &AgentId, query: &str) -> Result<Value> {
        // In a real implementation, this would use a specific runner API to query agent state
        // For now, we just return a placeholder response
        
        info!(
            scenario_name = %scenario_name, 
            agent_id = %agent_id, 
            query = %query, 
            "Agent state query executed"
        );
        
        // Create a query log entry to record this operation
        let query_data = serde_json::json!({
            "agent_id": agent_id.to_string(),
            "query": query,
        });
        
        // Create a log entry for the query
        let mut metadata = HashMap::new();
        metadata.insert("scenario_name".to_string(), scenario_name.to_string());
        metadata.insert("type".to_string(), "agent_query".to_string());
        
        let entry = LogEntry::new(
            crate::replay::LogEntryType::AgentState,
            Some(agent_id.clone()),
            None,  // No specific domain
            query_data,
            None,  // No parent
            None,  // Will use the default run ID
            metadata,
        )?;
        
        // Store the query in the log
        self.log_storage.store_entry(&entry).await?;
        
        // Return a mock response for now
        // In a real implementation, this would come from the agent
        Ok(serde_json::json!({
            "status": "ok",
            "query": query,
            "agent_id": agent_id.to_string(),
            "result": {
                "placeholder": "This is a placeholder response",
                "timestamp": chrono::Utc::now().to_string(),
            }
        }))
    }
    
    /// Load a scenario from a file and start it
    pub async fn load_and_start_scenario(&self, scenario_path: std::path::PathBuf) -> Result<String> {
        // Load the scenario from file
        let scenario_content = std::fs::read_to_string(&scenario_path)
            .context(format!("Failed to read scenario file: {:?}", scenario_path))?;
        
        // Parse the scenario from TOML
        let scenario: Scenario = toml::from_str(&scenario_content)
            .context(format!("Failed to parse scenario from TOML: {:?}", scenario_path))?;
        
        // Start the scenario
        self.start_scenario(Arc::new(scenario.clone())).await?;
        
        // Return the scenario name
        Ok(scenario.name)
    }
    
    /// Stop a running simulation scenario - proxy to the trait implementation
    pub async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        // Forward to the trait implementation
        <Self as SimulationController>::stop_scenario(self, scenario_name).await
    }
}

#[async_trait::async_trait]
impl SimulationController for BasicSimulationController {
    /// Start a new simulation scenario
    async fn start_scenario(&self, scenario: Arc<Scenario>) -> Result<()> {
        let scenario_name = scenario.name.clone();
        info!(scenario_name = %scenario_name, "Starting scenario");
        
        // Check if already running
        let mut scenarios = self.running_scenarios.lock().await;
        if scenarios.contains_key(&scenario_name) {
            return Err(anyhow!("Scenario {} is already running", scenario_name));
        }
        
        // Set up invariant checking
        self.setup_invariant_checking(&scenario)?;
        
        // Acquire lock on the runner
        let mut runner_guard = self.runner.lock().await;
        
        // Initialize the runner
        if let Err(e) = runner_guard.initialize(&scenario).await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to initialize runner");
            return Err(anyhow!("Failed to initialize runner: {}", e));
        }
        
        // Start the runner
        if let Err(e) = runner_guard.start(&scenario).await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to start runner");
            return Err(anyhow!("Failed to initialize runner: {}", e));
        }
        
        // Create scenario state
        let scenario_state = ScenarioState {
            scenario: scenario.clone(),
            start_time: chrono::Utc::now(),
            status: ScenarioStatus::Running,
        };
        
        // Add to running scenarios
        scenarios.insert(scenario_name.clone(), scenario_state);
        
        info!(scenario_name = %scenario_name, "Scenario started successfully");
        Ok(())
    }
    
    /// Stop a running simulation scenario
    async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        info!(scenario_name = %scenario_name, "Stopping scenario");
        
        // Check if running
        let mut scenarios = self.running_scenarios.lock().await;
        if !scenarios.contains_key(scenario_name) {
            return Err(anyhow!("Scenario {} is not running", scenario_name));
        }
        
        // Get the scenario state
        let scenario_state = scenarios.get(scenario_name).unwrap();
        
        // Stop the runner
        let mut runner_guard = self.runner.lock().await;
        
        // First, properly stop the scenario using the runner's stop_scenario method
        if let Err(e) = runner_guard.stop_scenario(scenario_name).await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to stop scenario with runner");
            return Err(anyhow!("Failed to stop scenario with runner: {}", e));
        }
        
        // Then stop the runner
        if let Err(e) = runner_guard.stop().await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to stop runner");
            return Err(anyhow!("Failed to stop runner: {}", e));
        }
        
        // Remove from running scenarios
        scenarios.remove(scenario_name);
        
        info!(scenario_name = %scenario_name, "Scenario stopped successfully");
        Ok(())
    }
    
    /// List all running simulation scenarios.
    async fn list_scenarios(&self) -> Result<Vec<String>> {
        let scenarios = self.running_scenarios.lock().await;
        let scenario_names: Vec<String> = scenarios.keys().cloned().collect();
        Ok(scenario_names)
    }
    
    /// Get the status of a specific scenario.
    async fn get_scenario_status(&self, scenario_name: &str) -> Result<ScenarioStatus> {
        let scenarios = self.running_scenarios.lock().await;
        if let Some(state) = scenarios.get(scenario_name) {
            Ok(state.status.clone())
        } else {
            // If the scenario is not in our running list, assume it's not running
            Ok(ScenarioStatus::Stopped)
        }
    }
    
    /// Get invariant violations for a scenario
    async fn get_invariant_violations(&self, scenario_name: &str) -> Result<Vec<String>> {
        // Simplified implementation that doesn't depend on invariant_violations
        // Always return an empty vec since we're not tracking violations currently
        info!(scenario_name = %scenario_name, "Returning empty invariant violations list");
        Ok(Vec::new())
    }
    
    /// Inject a fact into a running simulation
    async fn inject_fact(&self, scenario_name: &str, fact_data: Value) -> Result<()> {
        // Check if the scenario is running
        let scenarios = self.running_scenarios.lock().await;
        if !scenarios.contains_key(scenario_name) {
            return Err(anyhow!("Scenario {} is not running", scenario_name));
        }
        
        // Check if the scenario is paused
        let scenario_state = scenarios.get(scenario_name).unwrap();
        if scenario_state.status == ScenarioStatus::Paused {
            return Err(anyhow!("Scenario {} is paused, cannot inject facts while paused", scenario_name));
        }
        
        // Process the fact injection
        self.process_fact_injection(scenario_name, fact_data).await
    }
    
    /// Query the state of an agent in a running simulation
    async fn query_agent_state(&self, scenario_name: &str, agent_id: &AgentId, query: &str) -> Result<Value> {
        // Check if the scenario is running
        let scenarios = self.running_scenarios.lock().await;
        if !scenarios.contains_key(scenario_name) {
            return Err(anyhow!("Scenario {} is not running", scenario_name));
        }
        
        // Process the agent state query
        self.process_agent_query(scenario_name, agent_id, query).await
    }
    
    /// Pause a running simulation
    async fn pause_scenario(&self, scenario_name: &str) -> Result<()> {
        info!(scenario_name = %scenario_name, "Pausing scenario");
        
        // Check if running
        let scenarios = self.running_scenarios.lock().await;
        if !scenarios.contains_key(scenario_name) {
            return Err(anyhow!("Scenario {} is not running", scenario_name));
        }
        
        // Pause the runner
        let mut runner_guard = self.runner.lock().await;
        if let Err(e) = runner_guard.pause().await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to pause runner");
            return Err(anyhow!("Failed to pause runner: {}", e));
        }
        
        // Update scenario state
        let mut scenarios = self.running_scenarios.lock().await;
        let scenario_state = scenarios.get_mut(scenario_name).unwrap();
        scenario_state.status = ScenarioStatus::Paused;
        
        info!(scenario_name = %scenario_name, "Scenario paused successfully");
        Ok(())
    }
    
    /// Resume a paused simulation
    async fn resume_scenario(&self, scenario_name: &str) -> Result<()> {
        info!(scenario_name = %scenario_name, "Resuming scenario");
        
        // Check if running and paused
        let scenarios = self.running_scenarios.lock().await;
        if !scenarios.contains_key(scenario_name) {
            return Err(anyhow!("Scenario {} is not running", scenario_name));
        }
        
        let state = scenarios.get(scenario_name).unwrap();
        if state.status != ScenarioStatus::Paused {
            return Err(anyhow!("Scenario {} is not paused", scenario_name));
        }
        
        // Resume the runner
        let mut runner_guard = self.runner.lock().await;
        if let Err(e) = runner_guard.resume().await {
            error!(scenario_name = %scenario_name, error = %e, "Failed to resume runner");
            return Err(anyhow!("Failed to resume runner: {}", e));
        }
        
        // Update scenario state
        let mut scenarios = self.running_scenarios.lock().await;
        let scenario_state = scenarios.get_mut(scenario_name).unwrap();
        scenario_state.status = ScenarioStatus::Running;
        
        info!(scenario_name = %scenario_name, "Scenario resumed successfully");
        Ok(())
    }
    
    /// Get logs for a running scenario
    async fn get_scenario_logs(&self, scenario_name: &str, limit: Option<usize>) -> Result<Vec<LogEntry>> {
        // Check if the scenario exists
        if !self.log_storage.scenario_exists(scenario_name).await? {
            return Err(anyhow!("Scenario {} not found in logs", scenario_name));
        }
        
        // Get logs for the scenario
        let entries = self.log_storage.get_entries_for_scenario(scenario_name, limit).await?;
        Ok(entries)
    }
    
    /// Get the observer registry
    fn observer_registry(&self) -> Arc<ObserverRegistry> {
        self.observer_registry.clone()
    }
}

/// Mock implementation of a simulation runner for testing the controller.
#[derive(Debug, Clone)]
struct MockSimulationRunner {
    pub state: Arc<Mutex<RunnerState>>,
    pub started_scenarios: Arc<tokio::sync::Mutex<Vec<String>>>,
    pub stopped_scenarios: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl MockSimulationRunner {
    /// Create a new mock runner in the Stopped state.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RunnerState::Stopped)),
            started_scenarios: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            stopped_scenarios: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    
    /// Run a scenario and return effects
    pub async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<Vec<Arc<dyn causality_core::effect::Effect>>> {
        // Initialize and start the scenario
        self.initialize(&scenario).await?;
        self.start(&scenario).await?;
        
        // Return an empty vector of effects
        Ok(Vec::new())
    }
    
    /// Stop a running scenario by name
    pub async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        // Add to stopped scenarios
        let mut stopped = self.stopped_scenarios.lock().await;
        stopped.push(scenario_name.to_string());
        
        // Update state to stopped
        let mut state = self.state.lock().map_err(|e| anyhow!("Failed to lock state: {:?}", e))?;
        *state = RunnerState::Stopped;
        
        info!("Mock runner stopped scenario {}", scenario_name);
        Ok(())
    }
}

#[async_trait::async_trait]
impl SimulationRunner for MockSimulationRunner {
    async fn initialize(&self, scenario: &Scenario) -> Result<()> {
        let mut state = self.state.lock().map_err(|e| anyhow!("Failed to lock state: {:?}", e))?;
        *state = RunnerState::Initialized;
        info!("Mock runner initialized with scenario {}", scenario.name);
        Ok(())
    }
    
    async fn start(&self, scenario: &Scenario) -> Result<()> {
        {
            let mut state = self.state.lock().map_err(|e| anyhow!("Failed to lock state: {:?}", e))?;
            *state = RunnerState::Running;
        }
        
        let mut started = self.started_scenarios.lock().await;
        started.push(scenario.name.clone());
        info!("Mock runner started scenario {}", scenario.name);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut state = self.state.lock().map_err(|e| anyhow!("Failed to lock state: {:?}", e))?;
        *state = RunnerState::Stopped;
        info!("Mock runner stopped");
        Ok(())
    }
    
    async fn pause(&self) -> Result<()> {
        let mut state = self.state.lock().map_err(|e| anyhow!("Failed to lock state: {:?}", e))?;
        *state = RunnerState::Paused;
        info!("Mock runner paused");
        Ok(())
    }
    
    async fn resume(&self) -> Result<()> {
        let mut state = self.state.lock().map_err(|e| anyhow!("Failed to lock state: {:?}", e))?;
        *state = RunnerState::Running;
        info!("Mock runner resumed");
        Ok(())
    }
    
    fn get_state(&self) -> RunnerState {
        match self.state.lock() {
            Ok(guard) => (*guard).clone(),
            Err(e) => {
                error!("Failed to lock state mutex: {:?}, defaulting to Error state", e);
                RunnerState::Error("Failed to get state".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{RunnerState, InMemoryRunner, SimulationRunnerEnum};
    use std::sync::Arc;
    
    /// Mock implementation of LogStorage for testing
    pub struct MockLogStorage {
        run_id: String,
    }
    
    impl MockLogStorage {
        pub fn new() -> Self {
            Self {
                run_id: "test-run-id".to_string(),
            }
        }
    }
    
    impl crate::replay::LogStorageTrait for MockLogStorage {
        fn run_id(&self) -> String {
            self.run_id.clone()
        }
        
        fn store_entry(&self, _entry: &crate::replay::LogEntry) -> crate::replay::Result<()> {
            Ok(())
        }
        
        fn get_entries(&self, _filter: Option<&crate::replay::LogFilter>) -> crate::replay::Result<Vec<crate::replay::LogEntry>> {
            Ok(Vec::new())
        }
        
        fn scenario_exists(&self, _scenario_name: &str) -> crate::replay::Result<bool> {
            Ok(true)
        }
        
        fn get_scenarios(&self) -> crate::replay::Result<Vec<String>> {
            Ok(vec!["test-scenario".to_string()])
        }
    }
    
    #[tokio::test]
    async fn test_start_scenario() {
        // Create the controller for testing
        let mock_runner = MockSimulationRunner::new();
        
        // Create a temp log storage for testing
        let log_storage = Arc::new(AsyncLogStorageAdapter::new_temp().unwrap());
        
        let controller = BasicSimulationController::new(
            SimulationRunnerEnum::InMemory(Arc::new(InMemoryRunner::new())),
            log_storage
        );
        
        // Create a test scenario
        let scenario = Arc::new(Scenario {
            name: "test_scenario".to_string(),
            description: Some("A test scenario".to_string()),
            simulation_mode: crate::scenario::SimulationMode::InMemory,
            agents: Vec::new(),
            initial_state: None,
            invariants: None,
        });
        
        // Start the scenario
        let result = controller.start_scenario(scenario.clone()).await;
        
        // Verify the result
        assert!(result.is_ok(), "Failed to start scenario: {:?}", result);
        
        // Check that the scenario is marked as running
        let scenarios = controller.running_scenarios.lock().await;
        assert!(scenarios.contains_key(&scenario.name));
        assert_eq!(scenarios.get(&scenario.name).unwrap().status, ScenarioStatus::Running);
    }
}
