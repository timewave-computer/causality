// Purpose: Implements the in-memory simulation runner.

use async_trait::async_trait;
#[allow(unused_imports)]
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
#[allow(unused_imports)]
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use std::fmt; // Import fmt for Debug impl
use anyhow::{Result, anyhow}; // Note explicit import of anyhow macro
use chrono::Utc;
use tokio::sync::Mutex as AsyncMutex;

use crate::agent::{AgentId, SimulatedAgent, SimulationAgentConfig};
use crate::runner::{SimulationRunner, RunnerState};
use crate::scenario::Scenario;
use causality_core::resource::types::ResourceId;
use causality_types::crypto_primitives::{ContentHash, HashAlgorithm}; // Import needed types
use causality_core::effect::{Effect, EffectContext, EffectOutcome, EffectError, EffectType}; // Import EffectType
use std::any::Any; // For as_any
use crate::observer::{Observer, ObserverRegistry};
use crate::replay::LogStorage;
use causality_core::effect::outcome::{EffectStatus, ResultData};
use causality_core::effect::EffectResult;
use causality_types::ContentId;
use causality_types::content_addressing::content_hash_from_bytes;

// When we have engine integration, import the relevant types
#[cfg(feature = "engine")]
use causality_core::resource::agent::{
    user::{UserAgent, UserAgentBuilder, UserProfile, AuthenticationMethod},
    committee::{CommitteeAgent, CommitteeAgentBuilder, CommitteeConfig},
    agent::{Agent, AgentBuilder},
    types::AgentType
};

// --- Helper to create ResourceId from string ID (copied from mocks.rs) ---
/// Create a resource ID from a string
fn create_resource_id(id_str: &str) -> ResourceId {
    let hash_output = causality_types::content_addressing::content_hash_from_bytes(id_str.as_bytes());
    let content_hash = ContentHash::new(
        &HashAlgorithm::Blake3.to_string(),
        hash_output.as_bytes().to_vec()
    );
    ResourceId::with_name(content_hash, id_str.to_string())
}
// --- End Helper ---

// --- Simple mock agent implementations for standalone mode ---
#[cfg(not(feature = "engine"))]
struct MockUserAgent {
    agent_id: AgentId,
}

#[cfg(not(feature = "engine"))]
impl MockUserAgent {
    fn new(id_str: String) -> Self {
        Self {
            agent_id: create_resource_id(&id_str),
        }
    }
}

#[cfg(not(feature = "engine"))]
#[async_trait]
impl SimulatedAgent for MockUserAgent {
    async fn run(&self, _config: SimulationAgentConfig) -> Result<()> {
        info!(agent_id = %self.agent_id, "MockUserAgent started");
        // In a real implementation, this would run a user simulation loop
        // For now, we just return immediately as a placeholder
        Ok(())
    }

    fn id(&self) -> &AgentId {
        &self.agent_id
    }

    async fn shutdown(&self) -> Result<()> {
        info!(agent_id = %self.agent_id, "MockUserAgent shutdown");
        Ok(())
    }
}

#[cfg(not(feature = "engine"))]
struct MockCommitteeAgent {
    agent_id: AgentId,
}

#[cfg(not(feature = "engine"))]
impl MockCommitteeAgent {
    fn new(id_str: String) -> Self {
        Self {
            agent_id: create_resource_id(&id_str),
        }
    }
}

#[cfg(not(feature = "engine"))]
#[async_trait]
impl SimulatedAgent for MockCommitteeAgent {
    async fn run(&self, _config: SimulationAgentConfig) -> Result<()> {
        info!(agent_id = %self.agent_id, "MockCommitteeAgent started");
        // In a real implementation, this would run a committee simulation loop
        // For now, we just return immediately as a placeholder
        Ok(())
    }

    fn id(&self) -> &AgentId {
        &self.agent_id
    }

    async fn shutdown(&self) -> Result<()> {
        info!(agent_id = %self.agent_id, "MockCommitteeAgent shutdown");
        Ok(())
    }
}
// --- End standalone mock agent implementations ---

// --- Engine-integrated mock agents ---
#[cfg(feature = "engine")]
struct MockUserAgent {
    agent_id: AgentId,
    core_agent: UserAgent,
}

#[cfg(feature = "engine")]
impl MockUserAgent {
    fn new(id_str: String) -> Result<Self> {
        let agent_id = create_resource_id(&id_str);
        
        // Create a UserAgent using the official builder pattern
        let user_agent = UserAgentBuilder::new()
            .with_metadata("name", &id_str)
            .with_display_name(id_str.clone())
            .with_auth_method(AuthenticationMethod::PublicKey { 
                public_key: format!("pk_{}", id_str)
            })
            .build()?;
        
        Ok(Self {
            agent_id,
            core_agent: user_agent,
        })
    }
}

#[cfg(feature = "engine")]
#[async_trait]
impl SimulatedAgent for MockUserAgent {
    async fn run(&self, _config: SimulationAgentConfig) -> Result<()> {
        info!(agent_id = %self.agent_id, "Engine-integrated MockUserAgent started");
        
        // Simulate some user activity
        for i in 0..3 {
            info!(agent_id = %self.agent_id, iteration = i, "User performing actions");
            // We could perform actual operations here, but for now just simulate activity
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        info!(agent_id = %self.agent_id, "Engine-integrated MockUserAgent finished");
        Ok(())
    }

    fn id(&self) -> &AgentId {
        &self.agent_id
    }

    async fn shutdown(&self) -> Result<()> {
        info!(agent_id = %self.agent_id, "Engine-integrated MockUserAgent shutting down");
        Ok(())
    }
}

#[cfg(feature = "engine")]
struct MockCommitteeAgent {
    agent_id: AgentId,
    core_agent: CommitteeAgent,
}

#[cfg(feature = "engine")]
impl MockCommitteeAgent {
    fn new(id_str: String) -> Result<Self> {
        let agent_id = create_resource_id(&id_str);
        
        // Create a CommitteeAgent using the official builder pattern
        let committee_config = CommitteeConfig {
            domain: format!("domain_{}", id_str),
            quorum_percentage: 67,
            max_size: 5,
            min_votes: 2,
            protocol_version: "1.0".to_string(),
        };
        
        let committee_agent = CommitteeAgentBuilder::new()
            .with_metadata("name", &id_str)
            .with_config(committee_config)
            .build()?;
        
        Ok(Self {
            agent_id,
            core_agent: committee_agent,
        })
    }
}

#[cfg(feature = "engine")]
#[async_trait]
impl SimulatedAgent for MockCommitteeAgent {
    async fn run(&self, _config: SimulationAgentConfig) -> Result<()> {
        info!(agent_id = %self.agent_id, "Engine-integrated MockCommitteeAgent started");
        
        // Simulate committee activities
        info!(agent_id = %self.agent_id, "Committee processing proposals");
        // Just simulate a delay for now
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        info!(agent_id = %self.agent_id, "Engine-integrated MockCommitteeAgent finished");
        Ok(())
    }

    fn id(&self) -> &AgentId {
        &self.agent_id
    }

    async fn shutdown(&self) -> Result<()> {
        info!(agent_id = %self.agent_id, "Engine-integrated MockCommitteeAgent shutting down");
        Ok(())
    }
}
// --- End engine-integrated mock agents ---

// --- Simple Effect representation for InMemoryRunner --- 
#[derive(Clone)]
struct InMemoryAgentEffect {
    agent_id: ResourceId,
}

impl InMemoryAgentEffect {
    fn new(agent_id: ResourceId) -> Self {
        Self { agent_id }
    }
}

impl fmt::Debug for InMemoryAgentEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryAgentEffect")
         .field("agent_id", &self.agent_id)
         .finish()
    }
}

// Implement the Effect trait
#[async_trait]
impl Effect for InMemoryAgentEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom("InMemoryAgentEffect".to_string())
    }

    fn description(&self) -> String {
        format!("Represents a running in-memory agent ({})", self.agent_id)
    }

    async fn execute(
        &self,
        _context: &dyn EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        Ok(EffectOutcome::success(HashMap::new()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
// --- End InMemoryAgentEffect --- 

/// Mock effect for testing
#[derive(Debug, Clone)]
pub struct MockEffect {
    id: String,
    name: String,
    metadata: HashMap<String, String>,
}

impl MockEffect {
    pub fn new(id: String, name: &str, metadata: HashMap<String, String>) -> Self {
        Self {
            id,
            name: name.to_string(),
            metadata,
        }
    }
}

#[async_trait]
impl Effect for MockEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(self.name.clone())
    }
    
    fn description(&self) -> String {
        format!("Mock effect {} for testing", self.name)
    }
    
    async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        let mut data = HashMap::new();
        data.insert("status".to_string(), "success".to_string());
        data.insert("id".to_string(), self.id.clone());
        
        // Add any metadata to the outcome
        for (key, value) in &self.metadata {
            data.insert(key.clone(), value.clone());
        }
        
        Ok(EffectOutcome::success(data))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct InMemoryRunner {
    /// Tasks for all running agents
    #[allow(clippy::type_complexity)]
    agent_tasks: Arc<Mutex<HashMap<AgentId, (tokio::task::JoinHandle<()>, Arc<dyn SimulatedAgent>)>>>,
    /// Flag indicating if the runner has been initialized
    initialized: Arc<Mutex<bool>>,
    /// The current state of the runner
    state: Arc<Mutex<RunnerState>>,
    /// Agent factory for creating agent instances
    agent_factory: Arc<dyn AgentFactory>,
    /// Effect factory for creating agent effects
    effect_factory: Arc<dyn EffectFactory>,
    /// Observer registry for simulation events
    pub observer_registry: Arc<ObserverRegistry>,
    /// Log storage for recording simulation events
    pub log_storage: Arc<LogStorage>,
}

// Implement Debug manually for InMemoryRunner
impl std::fmt::Debug for InMemoryRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryRunner")
            .field("initialized", &self.initialized)
            .field("state", &self.state)
            .field("observer_registry", &self.observer_registry)
            .field("log_storage", &self.log_storage)
            .finish_non_exhaustive()
    }
}

/// Factory for creating agents
#[async_trait]
pub trait AgentFactory: Send + Sync {
    /// Create an agent from a config
    fn create_agent(&self, config: &crate::scenario::AgentConfig) -> Result<Arc<dyn SimulatedAgent>>;
}

/// Factory for creating effects
pub trait EffectFactory: Send + Sync {
    /// Create an effect for an agent
    fn create_effect(&self, config: &crate::scenario::AgentConfig) -> Result<Arc<dyn Effect>>;
}

/// Default implementation of AgentFactory
pub struct DefaultAgentFactory;

impl AgentFactory for DefaultAgentFactory {
    fn create_agent(&self, config: &crate::scenario::AgentConfig) -> Result<Arc<dyn SimulatedAgent>> {
        // For testing, create a simple mock agent
        // Convert the string ID to a ResourceId
        let agent_id = crate::agent::agent_id::from_string(&config.id);
        Ok(Arc::new(MockAgent::new(agent_id)))
    }
}

/// Default implementation of EffectFactory
pub struct DefaultEffectFactory;

impl EffectFactory for DefaultEffectFactory {
    fn create_effect(&self, config: &crate::scenario::AgentConfig) -> Result<Arc<dyn Effect>> {
        // For testing, create a simple mock effect
        Ok(Arc::new(MockEffect::new(
            config.id.clone(),
            &config.actor_type,
            HashMap::new()
        )))
    }
}

/// Mock agent for testing
pub struct MockAgent {
    id: AgentId,
}

impl MockAgent {
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait]
impl SimulatedAgent for MockAgent {
    async fn run(&self, _config: SimulationAgentConfig) -> Result<()> {
        // Mock implementation that just completes successfully
        Ok(())
    }

    fn id(&self) -> &AgentId {
        &self.id
    }
}

impl InMemoryRunner {
    /// Create a new in-memory runner
    pub fn new() -> Self {
        Self {
            agent_tasks: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
            state: Arc::new(Mutex::new(RunnerState::Stopped)),
            agent_factory: Arc::new(DefaultAgentFactory),
            effect_factory: Arc::new(DefaultEffectFactory),
            observer_registry: Arc::new(ObserverRegistry::new()),
            log_storage: Arc::new(LogStorage::new_temp().unwrap()),
        }
    }
    
    /// Create a new in-memory runner with custom components
    pub fn with_components(
        observer_registry: Arc<ObserverRegistry>,
        log_storage: Arc<LogStorage>,
        agent_factory: Arc<dyn AgentFactory>,
        effect_factory: Arc<dyn EffectFactory>,
    ) -> Self {
        Self {
            agent_tasks: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
            state: Arc::new(Mutex::new(RunnerState::Stopped)),
            agent_factory,
            effect_factory,
            observer_registry,
            log_storage,
        }
    }

    /// Run a scenario and return the list of effects
    pub async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<Vec<Arc<dyn causality_core::effect::Effect>>> {
        // First initialize the runner with the scenario
        self.initialize(&scenario).await?;
        
        // Start the scenario
        self.start(&scenario).await?;
        
        // For in-memory runner, we don't have actual effect objects to return
        // Return an empty vector for now
        Ok(Vec::new())
    }
    
    /// Stop a running scenario by name
    pub async fn stop_scenario(&self, _scenario_name: &str) -> Result<()> {
        // For the in-memory runner, we just stop all agents
        self.stop().await
    }
}

#[async_trait]
impl SimulationRunner for InMemoryRunner {
    /// Initialize the runner with a scenario
    async fn initialize(&self, scenario: &Scenario) -> Result<()> {
        info!(scenario_name = %scenario.name, "Initializing in-memory scenario");
        
        // Process each agent configuration in the scenario
        for agent_config in scenario.agents.iter() {
            let agent_id = agent_config.id.clone();
            info!(agent_id = %agent_id, "Creating in-memory agent");
            
            // Create an agent from the config using the factory
            let _agent = match self.agent_factory.create_agent(agent_config) {
                Ok(agent) => agent,
                Err(e) => {
                    error!(agent_id = %agent_id, error = %e, "Failed to create agent");
                    return Err(anyhow!("Failed to create agent {}: {}", agent_id, e));
                }
            };
            
            // We don't store the agent yet, we'll do that in the start method
        }
        
        // Update state using interior mutability
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Initialized;
        }
        
        // Set initialized flag
        {
            let mut initialized = self.initialized.lock().await;
            *initialized = true;
        }
        
        Ok(())
    }
    
    /// Start the simulation
    async fn start(&self, scenario: &Scenario) -> Result<()> {
        // Check if the runner is initialized
        {
            let initialized = self.initialized.lock().await;
            if !*initialized {
                return Err(anyhow!("Runner must be initialized before starting"));
            }
        }
        
        // Update state to Running
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Running;
        }
        
        info!(scenario_name = %scenario.name, "Starting in-memory scenario");
        
        // Initialize mutable state to track agents and effects
        let mut agent_effects: HashMap<ResourceId, Arc<dyn Effect>> = HashMap::new();
        let mut agent_tasks_guard = self.agent_tasks.lock().await;
        
        // Process each agent configuration separately without borrowing scenario
        for agent_config in scenario.agents.iter().cloned() {
            let agent_id = agent_config.id.clone();
            info!(agent_id = %agent_id, "Starting in-memory agent");
            
            // Create an agent from the config
            let agent = match self.agent_factory.create_agent(&agent_config) {
                Ok(agent) => agent,
                Err(e) => {
                    error!(agent_id = %agent_id, error = %e, "Failed to create agent");
                    return Err(anyhow!("Failed to create agent {}: {}", agent_id, e));
                }
            };
            
            // Set up the agent's effect and environment
            let effect = match self.effect_factory.create_effect(&agent_config) {
                Ok(effect) => effect,
                Err(e) => {
                    error!(agent_id = %agent_id, error = %e, "Failed to create effect for agent");
                    return Err(anyhow!("Failed to create effect for agent {}: {}", agent_id, e));
                }
            };
            
            // Store the effect for the agent
            let agent_resource_id = crate::agent::agent_id::from_string(&agent_id);
            agent_effects.insert(agent_resource_id.clone(), effect.clone());
            
            // Create the agent's configuration
            let agent_simulation_config = SimulationAgentConfig {
                scenario_id: scenario.name.clone(),
                agent_config: agent_config.clone(),
                observer_registry: self.observer_registry.clone(),
                log_storage: self.log_storage.clone(),
                run_id: self.log_storage.run_id().to_string(),
            };
            
            // Create and start a task for the agent
            let agent_clone = agent.clone();
            let agent_id_clone = agent_id.clone(); // Clone for the closure
            let agent_task = tokio::spawn(async move {
                if let Err(e) = agent_clone.run(agent_simulation_config).await {
                    error!(agent_id = %agent_id_clone, error = %e, "Agent execution failed");
                }
            });
            
            // Store the task and agent using ResourceId for key
            let resource_id = crate::agent::agent_id::from_string(&agent_id);
            agent_tasks_guard.insert(resource_id, (agent_task, agent));
        }
        
        Ok(())
    }
    
    /// Stop the simulation
    async fn stop(&self) -> Result<()> {
        info!("Stopping in-memory runner");
        
        // Update state to Stopped
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Stopped;
        }
        
        let mut agent_tasks_guard = self.agent_tasks.lock().await;
        info!(task_count = agent_tasks_guard.len(), "Aborting agent tasks");
        
        // Shut down each agent
        for (agent_id, (task, agent)) in agent_tasks_guard.drain() {
            info!(agent_id = %agent_id, "Shutting down agent");
            
            // Call the agent's shutdown method if available
            if let Err(e) = agent.shutdown().await {
                warn!(agent_id = %agent_id, error = %e, "Error during agent shutdown");
            }
            
            // Abort the task
            task.abort();
        }
        
        Ok(())
    }
    
    /// Pause the simulation
    async fn pause(&self) -> Result<()> {
        // Check current state
        {
            let state = self.state.lock().await;
            if *state != RunnerState::Running {
                return Err(anyhow!("Cannot pause: runner is not in running state"));
            }
        }
        
        info!("Pausing in-memory runner");
        
        // Update state to Paused
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Paused;
        }
        
        // In a real implementation, would need to signal agents to pause
        Ok(())
    }
    
    /// Resume the simulation
    async fn resume(&self) -> Result<()> {
        // Check current state
        {
            let state = self.state.lock().await;
            if *state != RunnerState::Paused {
                return Err(anyhow!("Cannot resume: runner is not in paused state"));
            }
        }
        
        info!("Resuming in-memory runner");
        
        // Update state to Running
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Running;
        }
        
        // In a real implementation, would need to signal agents to resume
        Ok(())
    }
    
    /// Get the current state of the runner
    fn get_state(&self) -> RunnerState {
        // Get a lock on the state and clone it
        match self.state.try_lock() {
            Ok(state) => state.clone(),
            Err(_) => RunnerState::Error("Failed to access runner state".to_string()),
        }
    }
}

// Function to create an effect ID using content_hash_from_bytes
fn create_effect_id(agent_id: &str, effect_name: &str) -> String {
    let id_str = format!("agent-{}-effect-{}", agent_id, effect_name);
    let hash = content_hash_from_bytes(id_str.as_bytes());
    let content_id = ContentId::from(hash);
    content_id.to_string()
}

// Replace the blake3 usage in the effect implementation
pub fn create_test_effect(agent_id: &str, effect_name: &str) -> Result<Arc<dyn Effect>> {
    let id_str = format!("agent-{}-effect-{}", agent_id, effect_name);
    let hash = content_hash_from_bytes(id_str.as_bytes());
    let content_id = ContentId::from(hash);
    
    // Add required metadata
    let mut metadata = HashMap::new();
    metadata.insert("algorithm".to_string(), HashAlgorithm::Blake3.to_string());
    metadata.insert("agent_id".to_string(), agent_id.to_string());
    metadata.insert("effect_name".to_string(), effect_name.to_string());
    
    // Create a basic effect implementation
    let effect = Arc::new(MockEffect::new(
        content_id.to_string(),
        effect_name,
        metadata
    ));
    
    Ok(effect)
}

// End of file
