// Purpose: Defines the LocalProcessRunner for running agents as separate processes.

use crate::runner::SimulationRunner;
use crate::scenario::Scenario;
use causality_core::resource::types::ResourceId;
use causality_core::effect::Effect;
use std::collections::HashMap;
use std::process::{Command, Child};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tracing::{info, error, warn};
use anyhow::{Result, anyhow};

/// Runs simulation agents as separate local processes.
#[derive(Debug, Default)]
pub struct LocalProcessRunner {
    // Store child process handles to manage them (e.g., for stopping)
    // Key: AgentId (String), Value: Child process handle
    running_agents: Arc<Mutex<HashMap<String, Child>>>,
}

impl LocalProcessRunner {
    pub fn new() -> Self {
        Self { 
            running_agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SimulationRunner for LocalProcessRunner {
    async fn run_scenario(
        &self,
        scenario: Arc<Scenario>,
    ) -> Result<HashMap<ResourceId, Arc<dyn Effect>>> {
        info!(scenario_name = %scenario.name, "Running scenario with LocalProcessRunner");
        
        let agent_handles: HashMap<ResourceId, Arc<dyn Effect>> = HashMap::new();
        let mut running_agents_guard = self.running_agents.lock()
            .map_err(|_| anyhow!("Failed to lock running_agents mutex"))?;

        for agent_config in &scenario.agents {
            info!(agent_id = %agent_config.id, agent_type = %agent_config.actor_type, "Preparing Nix command for agent");
            
            // Map agent type to Nix flake app name
            let nix_app_name = match agent_config.actor_type.as_str() {
                "mock_user" => "agent-user",
                "mock_committee" => "agent-committee",
                // Add mappings for real agent types later
                _ => {
                    error!(agent_type = %agent_config.actor_type, "Unsupported agent type for local process runner");
                    continue;
                }
            };

            // Construct the Nix command
            // Assumes the flake is in the current directory (.)
            let nix_flake_ref = format!(".#{}", nix_app_name);
            
            let mut cmd = Command::new("nix");
            cmd.arg("run")
               .arg(&nix_flake_ref)
               .arg("--") // Separator for arguments passed to the agent app
               .arg("--agent-id")
               .arg(&agent_config.id);
               // TODO: Pass other necessary config (e.g., controller address, domain info) 
               //       to the agent app via command-line arguments.

            info!(command = ?cmd, "Spawning agent process via Nix");

            // Spawn the child process
            match cmd.spawn() {
                Ok(child) => {
                    info!(agent_id = %agent_config.id, pid = child.id(), "Agent process started via Nix");
                    // TODO: Implement proper Effect creation/return
                    // For now, just store the child handle for stopping
                    running_agents_guard.insert(agent_config.id.clone(), child);
                }
                Err(e) => {
                    error!(agent_id = %agent_config.id, error = %e, command = ?cmd, "Failed to start agent process via Nix");
                    // Attempt to stop already started agents before returning error
                    for (_id, mut child_to_kill) in running_agents_guard.drain() {
                        warn!(agent_id = _id, "Stopping previously started agent due to error");
                        if let Err(kill_err) = child_to_kill.kill() {
                            error!(agent_id = _id, error = %kill_err, "Failed to kill agent process during cleanup");
                        }
                    }
                    return Err(anyhow!("Failed to start agent process {} via Nix: {}", agent_config.id, e));
                }
            }
        }

        // TODO: Return map of ResourceId -> Arc<dyn Effect>
        // Needs a way to represent the running process as an Effect.
        Ok(HashMap::new()) // Placeholder
    }

    async fn stop_scenario(
        &self,
        scenario_name: &str,
    ) -> Result<()> {
        info!(scenario_name = %scenario_name, "Stopping scenario with LocalProcessRunner");
        
        let mut running_agents_guard = self.running_agents.lock()
            .map_err(|_| anyhow!("Failed to lock running_agents mutex"))?;

        info!(agent_count = running_agents_guard.len(), "Attempting to stop agent processes");

        for (agent_id, child) in running_agents_guard.iter_mut() {
            match child.kill() {
                Ok(_) => info!(%agent_id, "Successfully sent kill signal to agent process"),
                Err(e) => {
                    error!(%agent_id, error = %e, "Failed to kill agent process");
                }
            }
        }
        
        running_agents_guard.clear();

        Ok(())
    }
}

// Placeholder for Effect trait if needed temporarily
// struct PlaceholderEffect;
// impl Effect for PlaceholderEffect { ... }
