// Purpose: Implements the engine-integrated simulation runner.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{info, error};
use std::str::FromStr;
use std::any::Any;

use crate::agent::{AgentId, SimulatedAgent, SimulationAgentConfig};
use crate::scenario::Scenario;
use crate::runner::SimulationRunner;
use crate::log::adapter::engine_adapter::EngineLogAdapter;

use causality_core::resource::types::ResourceId;
use causality_core::effect::{Effect, EffectContext, EffectOutcome, EffectError, EffectType};
use causality_engine::engine::Engine;
use causality_engine::config::EngineConfig;
use causality_engine::storage::memory::InMemoryStorage;
use causality_engine::log::memory_storage::MemoryLogStorage;
use causality_types::DomainId;
use causality_core::utils::content_addressing;

/// Wrapper struct for engine agents to implement the SimulatedAgent trait
struct EngineAgentWrapper {
    id: ResourceId,
    agent_type: String,
    domain_id: DomainId,
    engine: Arc<Engine>,
}

impl EngineAgentWrapper {
    fn new(id: String, agent_type: String, domain_id: DomainId, engine: Arc<Engine>) -> Self {
        Self {
            id: ResourceId::from_str(&id).unwrap_or_else(|_| {
                // If parse fails, create a simple ResourceId with the provided ID string
                ResourceId::from_str(&format!("resource:{id}")).unwrap_or_else(|_| {
                    // Fallback to a resource with hardcoded type and id if both attempts fail
                    let hash = content_addressing::hash_string(&id);
                    ResourceId::new(hash)
                })
            }),
            agent_type,
            domain_id,
            engine,
        }
    }
}

#[async_trait]
impl SimulatedAgent for EngineAgentWrapper {
    fn id(&self) -> &AgentId {
        &self.id
    }

    async fn run(&self, _config: SimulationAgentConfig) -> Result<()> {
        // In a real implementation, we would:
        // 1. Initialize the agent in the engine
        // 2. Register agent handlers
        // 3. Connect to the domain

        info!(agent_id = %self.id, agent_type = %self.agent_type, domain = ?self.domain_id, "Engine agent running in simulation");
        
        // Just sleep for a bit to simulate the agent running
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        Ok(())
    }
}

/// Runner that integrates with the full Causality engine
pub struct EngineRunner {
    // Map to store handles for running agent tasks
    agent_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    
    // Engine instance
    engine: Arc<Engine>,
    
    // Log storage
    log_storage: Arc<MemoryLogStorage>,
}

impl EngineRunner {
    /// Creates a new Engine-integrated runner
    pub fn new() -> Self {
        let storage = Arc::new(InMemoryStorage::new());
        let log_storage = Arc::new(MemoryLogStorage::new());
        
        // Create engine with basic configuration
        let mut config = EngineConfig::default();
        
        // Configure engine for simulation use
        config.invocation_timeout_ms = 10000;
        config.enable_logging = true;
        
        let engine = Engine::with_config(config, storage, log_storage.clone())
            .expect("Failed to create engine instance");
        
        Self {
            agent_tasks: Arc::new(Mutex::new(HashMap::new())),
            engine: Arc::new(engine),
            log_storage,
        }
    }
    
    /// Creates an engine-integrated runner with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        let storage = Arc::new(InMemoryStorage::new());
        let log_storage = Arc::new(MemoryLogStorage::new());
        
        let engine = Engine::with_config(config, storage, log_storage.clone())
            .expect("Failed to create engine instance");
        
        Self {
            agent_tasks: Arc::new(Mutex::new(HashMap::new())),
            engine: Arc::new(engine),
            log_storage,
        }
    }
    
    /// Get a reference to the engine
    pub fn engine(&self) -> Arc<Engine> {
        self.engine.clone()
    }
    
    /// Get the log adapter
    pub fn get_log_adapter(&self) -> EngineLogAdapter {
        EngineLogAdapter::new()
    }
}

impl std::fmt::Debug for EngineRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineRunner")
            .field("agent_tasks", &self.agent_tasks)
            .field("engine", &"<Engine Instance>")
            .field("log_storage", &self.log_storage)
            .finish()
    }
}

#[async_trait]
impl SimulationRunner for EngineRunner {
    async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<HashMap<ResourceId, Arc<dyn Effect>>> {
        info!(scenario_name = %scenario.name, "Starting engine-integrated scenario");
        let mut agent_effects: HashMap<ResourceId, Arc<dyn Effect>> = HashMap::new();
        let mut agent_tasks_guard = self.agent_tasks.lock().await;

        // Create a unique engine domain for this scenario
        let domain_id = DomainId::new(format!("simulation_{}", scenario.name));
        
        // Setup a log adapter for the agents to use
        let _log_adapter = self.get_log_adapter();
        
        for agent_config in &scenario.agents {
            info!(agent_id = %agent_config.id, agent_type = %agent_config.actor_type, "Starting engine-integrated agent");
            
            // Create a simulated agent wrapper
            let agent: Arc<dyn SimulatedAgent> = Arc::new(EngineAgentWrapper::new(
                agent_config.id.clone(),
                agent_config.actor_type.clone(),
                domain_id.clone(),
                self.engine.clone()
            ));
            
            // Spawn the agent task
            let agent_clone = agent.clone();
            let sim_config = SimulationAgentConfig {
                scenario_id: scenario.name.clone(),
                agent_config: agent_config.clone(),
            };
            
            let handle = tokio::spawn(async move {
                match agent_clone.run(sim_config).await {
                    Ok(_) => info!(agent_id = %agent_clone.id(), "Engine agent task finished"),
                    Err(e) => error!(agent_id = %agent_clone.id(), error = %e, "Engine agent task failed"),
                }
            });
            
            // Store the task handle
            agent_tasks_guard.insert(agent_config.id.clone(), handle);
            
            // Convert the agent ID to a resource ID for the effect registry
            let resource_id = agent.id().clone();
            
            // Create an effect that represents this agent
            let effect: Arc<dyn Effect> = Arc::new(EngineAgentEffect {
                agent_id: resource_id.clone(),
                domain_id: domain_id.clone(),
            });
            
            // Add to the map of effects
            agent_effects.insert(resource_id, effect);
        }
        
        info!(agent_count = agent_effects.len(), "Engine-integrated agents started");
        Ok(agent_effects)
    }

    async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        info!(scenario_name = %scenario_name, "Stopping engine-integrated scenario");
        let mut agent_tasks_guard = self.agent_tasks.lock().await;
        
        // Abort any running agent tasks
        for (agent_id, task_handle) in agent_tasks_guard.iter() {
            if !task_handle.is_finished() {
                info!(%agent_id, "Aborting engine agent task");
                task_handle.abort();
            } else {
                info!(%agent_id, "Engine agent task was already finished");
            }
        }
        
        // Clear the tasks
        agent_tasks_guard.clear();
        info!(scenario_name = %scenario_name, "Engine-integrated scenario stopped");
        
        Ok(())
    }
}

// Engine Agent Effect implementation
#[derive(Debug)]
struct EngineAgentEffect {
    agent_id: ResourceId,
    domain_id: DomainId,
}

#[async_trait]
impl Effect for EngineAgentEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom("engine_agent".to_string())
    }
    
    async fn execute(&self, _context: &dyn EffectContext) -> Result<EffectOutcome, EffectError> {
        let mut result = HashMap::new();
        result.insert("agent_id".to_string(), self.agent_id.to_string());
        result.insert("domain_id".to_string(), self.domain_id.to_string());
        
        Ok(EffectOutcome::success(result))
    }
    
    fn description(&self) -> String {
        format!("Engine agent effect for agent {} in domain {}", self.agent_id, self.domain_id)
    }
    
    fn as_any(&self) -> &(dyn Any + 'static) {
        self
    }
} 