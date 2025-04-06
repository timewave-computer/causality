// Purpose: Defines the LocalProcessRunner for running agents as separate processes.

use crate::runner::{SimulationRunner, RunnerState};
use crate::scenario::Scenario;
use causality_core::resource::types::ResourceId;
use causality_core::effect::Effect;
use std::collections::HashMap;
use std::process::{Command, Child};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tracing::{info, error, warn};
use anyhow::{Result, anyhow};

/// Runner implementation that executes agents as separate processes on the local machine.
#[derive(Clone, Debug)]
pub struct LocalProcessRunner {
    // Store child process handles to manage them (e.g., for stopping)
    // Key: AgentId (String), Value: Child process handle
    running_agents: Arc<Mutex<HashMap<String, Child>>>,
    // Current state of the runner
    state: Arc<Mutex<RunnerState>>,
    // Current scenario name
    current_scenario: Arc<Mutex<Option<String>>>,
}

impl LocalProcessRunner {
    pub fn new() -> Self {
        Self { 
            running_agents: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(RunnerState::Stopped)),
            current_scenario: Arc::new(Mutex::new(None)),
        }
    }
    
    // Helper method to set the state
    fn set_state(&self, new_state: RunnerState) -> Result<()> {
        let mut state = self.state.lock()
            .map_err(|_| anyhow!("Failed to lock state mutex"))?;
        *state = new_state;
        Ok(())
    }
    
    /// Run a scenario and return the list of effects
    pub async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<Vec<Arc<dyn Effect>>> {
        // First initialize the runner with the scenario
        self.initialize(&scenario).await?;
        
        // Start the scenario
        self.start(&scenario).await?;
        
        // For local process runner, we don't have actual effect objects to return
        // Return an empty vector for now
        Ok(Vec::new())
    }
    
    /// Stop a running scenario by name
    pub async fn stop_scenario(&self, _scenario_name: &str) -> Result<()> {
        // For the local process runner, we just stop all agents
        self.stop().await
    }
}

#[async_trait]
impl SimulationRunner for LocalProcessRunner {
    async fn initialize(&self, scenario: &Scenario) -> Result<()> {
        info!(scenario_name = %scenario.name, "Initializing scenario with LocalProcessRunner");
        
        // Set scenario name
        let mut current_scenario = self.current_scenario.lock()
            .map_err(|_| anyhow!("Failed to lock current_scenario mutex"))?;
        *current_scenario = Some(scenario.name.clone());
        
        // Set state to initialized
        self.set_state(RunnerState::Initialized)?;
        
        Ok(())
    }
    
    async fn start(&self, scenario: &Scenario) -> Result<()> {
        info!(scenario_name = %scenario.name, "Starting scenario with LocalProcessRunner");
        
        // Check if we're in the right state
        {
            let state = self.state.lock()
                .map_err(|_| anyhow!("Failed to lock state mutex"))?;
            if *state != RunnerState::Initialized {
                return Err(anyhow!("LocalProcessRunner must be initialized before starting"));
            }
        }
        
        // Set state to running
        self.set_state(RunnerState::Running)?;
        
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
                    // Set state to error
                    self.set_state(RunnerState::Error(format!("Failed to start agent process {} via Nix: {}", agent_config.id, e)))?;
                    return Err(anyhow!("Failed to start agent process {} via Nix: {}", agent_config.id, e));
                }
            }
        }
        
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // Get current scenario name
        let scenario_name = {
            let current_scenario = self.current_scenario.lock()
                .map_err(|_| anyhow!("Failed to lock current_scenario mutex"))?;
            match &*current_scenario {
                Some(name) => name.clone(),
                None => return Err(anyhow!("No scenario is currently running")),
            }
        };
        
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
        
        // Set state to stopped
        self.set_state(RunnerState::Stopped)?;
        
        // Clear current scenario
        let mut current_scenario = self.current_scenario.lock()
            .map_err(|_| anyhow!("Failed to lock current_scenario mutex"))?;
        *current_scenario = None;

        Ok(())
    }
    
    async fn pause(&self) -> Result<()> {
        info!("Pausing LocalProcessRunner");
        
        // Check if we're in the right state
        {
            let state = self.state.lock()
                .map_err(|_| anyhow!("Failed to lock state mutex"))?;
            if *state != RunnerState::Running {
                return Err(anyhow!("LocalProcessRunner must be running before pausing"));
            }
        }
        
        // Set state to paused
        self.set_state(RunnerState::Paused)?;
        
        // TODO: Implement actual pause logic for child processes
        // This would involve sending SIGSTOP to all processes
        
        Ok(())
    }
    
    async fn resume(&self) -> Result<()> {
        info!("Resuming LocalProcessRunner");
        
        // Check if we're in the right state
        {
            let state = self.state.lock()
                .map_err(|_| anyhow!("Failed to lock state mutex"))?;
            if *state != RunnerState::Paused {
                return Err(anyhow!("LocalProcessRunner must be paused before resuming"));
            }
        }
        
        // Set state to running
        self.set_state(RunnerState::Running)?;
        
        // TODO: Implement actual resume logic for child processes
        // This would involve sending SIGCONT to all processes
        
        Ok(())
    }
    
    fn get_state(&self) -> RunnerState {
        match self.state.lock() {
            Ok(guard) => (*guard).clone(),
            Err(_) => {
                error!("Failed to lock state mutex, defaulting to Error state");
                RunnerState::Error("Failed to get state".to_string())
            }
        }
    }
}

// Placeholder for Effect trait if needed temporarily
// struct PlaceholderEffect;
// impl Effect for PlaceholderEffect { ... }
