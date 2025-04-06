// Purpose: Implements the in-memory simulation runner.

use async_trait::async_trait;
#[allow(unused_imports)]
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
#[allow(unused_imports)]
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, info};
use std::fmt; // Import fmt for Debug impl
use anyhow::{Result, anyhow}; // Note explicit import of anyhow macro

use crate::agent::{AgentId, SimulatedAgent, SimulationAgentConfig};
use crate::runner::SimulationRunner;
use crate::scenario::Scenario;
use causality_core::resource::types::ResourceId;
use causality_types::crypto_primitives::{ContentHash, HashAlgorithm}; // Import needed types
use causality_core::effect::{Effect, EffectContext, EffectOutcome, EffectError, EffectType}; // Import EffectType
use std::any::Any; // For as_any

// When we have engine integration, import the relevant types
#[cfg(feature = "engine")]
use causality_core::resource::agent::{
    user::{UserAgent, UserAgentBuilder, UserProfile, AuthenticationMethod},
    committee::{CommitteeAgent, CommitteeAgentBuilder, CommitteeConfig},
    agent::{Agent, AgentBuilder},
    types::AgentType
};

// --- Helper to create ResourceId from string ID (copied from mocks.rs) ---
fn create_resource_id(id_str: &str) -> ResourceId {
    let hash_bytes = blake3::hash(id_str.as_bytes());
    let content_hash = ContentHash::new(
        &HashAlgorithm::Blake3.to_string(),
        hash_bytes.as_bytes().to_vec()
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

// Define the struct first
#[derive(Debug)]
pub struct InMemoryRunner {
    // Map to store handles for running agent tasks
    // Key: AgentId, Value: JoinHandle for the agent's task
    agent_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    // TODO: Add shared state if needed (e.g., shared communication channels, global clock access)
}

impl InMemoryRunner {
    /// Creates a new InMemoryRunner.
    pub fn new() -> Self {
        Self {
            agent_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SimulationRunner for InMemoryRunner {
    async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<HashMap<ResourceId, Arc<dyn Effect>>> {
        info!(scenario_name = %scenario.name, "Starting in-memory scenario");
        let mut agent_effects: HashMap<ResourceId, Arc<dyn Effect>> = HashMap::new();
        let mut agent_tasks_guard = self.agent_tasks.lock().await;

        for agent_config in &scenario.agents {
            info!(agent_id = %agent_config.id, agent_type = %agent_config.actor_type, "Starting agent task");
            
            // Create the appropriate agent type based on feature flag and agent_type
            let agent: Arc<dyn SimulatedAgent> = match agent_config.actor_type.as_str() {
                "mock_user" => {
                    #[cfg(not(feature = "engine"))]
                    {
                        // Create a simple mock user agent
                        Arc::new(MockUserAgent::new(agent_config.id.clone()))
                    }
                    
                    #[cfg(feature = "engine")]
                    {
                        // Create an engine-integrated mock user agent
                        match MockUserAgent::new(agent_config.id.clone()) {
                            Ok(agent) => Arc::new(agent),
                            Err(e) => return Err(anyhow!("Failed to create mock user agent: {}", e)),
                        }
                    }
                },
                "mock_committee" => {
                    #[cfg(not(feature = "engine"))]
                    {
                        // Create a simple mock committee agent
                        Arc::new(MockCommitteeAgent::new(agent_config.id.clone()))
                    }
                    
                    #[cfg(feature = "engine")]
                    {
                        // Create an engine-integrated mock committee agent
                        match MockCommitteeAgent::new(agent_config.id.clone()) {
                            Ok(agent) => Arc::new(agent),
                            Err(e) => return Err(anyhow!("Failed to create mock committee agent: {}", e)),
                        }
                    }
                },
                // Add other agent types as needed
                _ => return Err(anyhow!("Unsupported agent type: {}", agent_config.actor_type)),
            };

            let agent_arc_clone = agent.clone();
            let sim_config = SimulationAgentConfig {
                scenario_id: scenario.name.clone(),
                agent_config: agent_config.clone(), 
            };
            let handle = tokio::spawn(async move {
                match agent_arc_clone.run(sim_config).await { 
                    Ok(_) => info!(agent_id = %agent_arc_clone.id(), "Agent task finished."),
                    Err(e) => error!(agent_id = %agent_arc_clone.id(), error = %e, "Agent task failed."),
                }
            });

            agent_tasks_guard.insert(agent_config.id.clone(), handle);
            
            // Use the agent's own id() method which now returns ResourceId
            let resource_id = agent.id().clone();
            // Create and insert the InMemoryAgentEffect
            let effect: Arc<dyn Effect> = Arc::new(InMemoryAgentEffect::new(resource_id.clone()));
            agent_effects.insert(resource_id, effect);
        }
        
        info!(agent_count = agent_effects.len(), "In-memory agents started.");
        Ok(agent_effects)
    }

    async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        info!(scenario_name = %scenario_name, "Stopping scenario with InMemoryRunner");
        let mut agent_tasks_guard = self.agent_tasks.lock().await;
        info!(task_count = agent_tasks_guard.len(), "Aborting agent tasks");
        
        for (agent_id, task_handle) in agent_tasks_guard.iter() {
             if !task_handle.is_finished() {
                info!(%agent_id, "Aborting task...");
                task_handle.abort();
            } else {
                info!(%agent_id, "Task was already finished.");
            }
        }
        
        agent_tasks_guard.clear();

        info!("In-memory scenario stopped.");
        Ok(())
    }
}

// Removed internal MockAgent definition, using the one from agents::mocks now
