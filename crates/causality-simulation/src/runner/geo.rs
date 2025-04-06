// Implementation of the geo-distributed runner for the simulation system.
// This runner executes scenarios across multiple machines.

use anyhow::{Result, anyhow, Context};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use tracing::{info, error, warn};

use causality_core::resource::ResourceId;
use causality_types::DomainId;

use crate::agent::AgentId;
use crate::scenario::{Scenario, AgentConfig};
use crate::observer::ObserverRegistry;
use crate::replay::LogStorage;
use crate::runner::{SimulationRunner, RunnerState};

/// Configuration for a remote host
#[derive(Debug, Clone)]
pub struct RemoteHostConfig {
    /// Hostname or IP address
    pub hostname: String,
    /// SSH port (defaults to 22)
    pub port: u16,
    /// SSH username
    pub username: String,
    /// Path to SSH key
    pub key_path: Option<PathBuf>,
    /// Remote working directory
    pub working_dir: PathBuf,
}

impl Default for RemoteHostConfig {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            port: 22,
            username: "root".to_string(),
            key_path: None,
            working_dir: PathBuf::from("/tmp/causality-simulation"),
        }
    }
}

/// Configuration for the geo-distributed runner
#[derive(Debug, Clone)]
pub struct GeoRunnerConfig {
    /// Map of agent IDs to remote host configurations
    pub host_assignments: HashMap<AgentId, RemoteHostConfig>,
    /// Default remote host configuration
    pub default_host: RemoteHostConfig,
}

impl Default for GeoRunnerConfig {
    fn default() -> Self {
        Self {
            host_assignments: HashMap::new(),
            default_host: RemoteHostConfig::default(),
        }
    }
}

/// Runner for geo-distributed simulation
#[derive(Clone, Debug)]
pub struct GeoRunner {
    /// Runtime configuration
    config: GeoRunnerConfig,
    /// Current state of the runner
    state: Arc<Mutex<RunnerState>>,
    /// Agent process handles
    agent_handles: Arc<Mutex<HashMap<ResourceId, JoinHandle<Result<()>>>>>,
    /// Observer registry
    observer_registry: Arc<ObserverRegistry>,
    /// Log storage
    log_storage: Arc<LogStorage>,
    /// Whether the runner has been initialized
    initialized: Arc<Mutex<bool>>,
}

impl GeoRunner {
    /// Create a new geo-distributed runner
    pub fn new(
        config: GeoRunnerConfig,
        observer_registry: Arc<ObserverRegistry>,
        log_storage: Arc<LogStorage>,
    ) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(RunnerState::Initialized)),
            agent_handles: Arc::new(Mutex::new(HashMap::new())),
            observer_registry,
            log_storage,
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Get the remote host configuration for an agent
    fn get_host_config(&self, agent_id: &AgentId) -> RemoteHostConfig {
        self.config.host_assignments
            .get(agent_id)
            .cloned()
            .unwrap_or_else(|| self.config.default_host.clone())
    }

    /// Build the SSH command for a remote host
    fn build_ssh_command(&self, host: &RemoteHostConfig) -> Command {
        let mut cmd = Command::new("ssh");
        
        // Add port if not default
        if host.port != 22 {
            cmd.arg("-p").arg(host.port.to_string());
        }
        
        // Add identity file if specified
        if let Some(key_path) = &host.key_path {
            cmd.arg("-i").arg(key_path);
        }
        
        // Add hostname
        cmd.arg(format!("{}@{}", host.username, host.hostname));
        
        cmd
    }

    /// Run a command on a remote host
    async fn run_remote_command(&self, host: &RemoteHostConfig, command: &str) -> Result<String> {
        let mut cmd = self.build_ssh_command(host);
        cmd.arg(command);
        
        info!("Running remote command: {:?}", cmd);
        
        let output = tokio::process::Command::from(cmd)
            .output()
            .await
            .context("Failed to execute command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Command failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    /// Prepare environment variables for an agent
    fn prepare_agent_env_vars(&self, agent_id: &AgentId, agent_config: &AgentConfig) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        
        // Add standard variables
        env_vars.insert("AGENT_ID".to_string(), agent_id.to_string());
        env_vars.insert("AGENT_TYPE".to_string(), agent_config.actor_type.clone());
        
        // Add domain if specified
        if let Some(domain_id) = &agent_config.domain {
            env_vars.insert("DOMAIN_ID".to_string(), domain_id.to_string());
        }
        
        // Add log directory
        env_vars.insert("LOG_DIR".to_string(), "/tmp/causality-logs".to_string());
        
        env_vars
    }

    /// Build the agent command
    fn build_agent_command(&self, agent_id: &AgentId, agent_config: &AgentConfig) -> String {
        let agent_type = agent_config.actor_type.clone();
        format!(
            "nix run .#agents.{} -- --agent-id {} --run-id {}",
            agent_type,
            agent_id,
            "simulation_run"
        )
    }

    /// Deploy an agent to a remote host
    async fn deploy_agent(&self, agent_id: &AgentId, agent_config: &AgentConfig, host: &RemoteHostConfig) -> Result<()> {
        // Create working directory
        self.run_remote_command(host, &format!("mkdir -p {}", host.working_dir.display()))
            .await
            .context("Failed to create working directory")?;
        
        // Deploy the agent using nix copy
        let agent_type = agent_config.actor_type.clone();
        let nix_copy_cmd = Command::new("nix")
            .arg("copy")
            .arg("--to")
            .arg(format!("ssh://{}@{}", host.username, host.hostname))
            .arg(format!(".#agents.{}", agent_type))
            .output()
            .context("Failed to copy agent to remote host")?;
        
        if !nix_copy_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&nix_copy_cmd.stderr);
            return Err(anyhow!("Failed to copy agent: {}", stderr));
        }
        
        info!("Successfully deployed agent {} to {}", agent_id, host.hostname);
        Ok(())
    }

    /// Start an agent on a remote host
    async fn start_agent(&self, agent_id: &AgentId, agent_config: &AgentConfig, host: &RemoteHostConfig) -> Result<()> {
        // Prepare environment variables
        let env_vars = self.prepare_agent_env_vars(agent_id, agent_config);
        let env_str = env_vars.iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        
        // Build the run command
        let agent_type = agent_config.actor_type.clone();
        let run_cmd = format!(
            "cd {} && {} nix run .#agents.{} -- --agent-id {} --run-id {}",
            host.working_dir.display(),
            env_str,
            agent_type,
            agent_id,
            "simulation_run"
        );
        
        // Execute in background with nohup
        let remote_cmd = format!("nohup {} > agent_{}.log 2>&1 &", run_cmd, agent_id);
        self.run_remote_command(host, &remote_cmd)
            .await
            .context("Failed to start agent on remote host")?;
        
        info!("Successfully started agent {} on {}", agent_id, host.hostname);
        Ok(())
    }

    /// Stop an agent on a remote host
    async fn stop_agent(&self, agent_id: &AgentId, host: &RemoteHostConfig) -> Result<()> {
        // Kill the agent process using pkill
        let kill_cmd = format!("pkill -f 'nix run .#agents.* -- --agent-id {}'", agent_id);
        match self.run_remote_command(host, &kill_cmd).await {
            Ok(_) => {
                info!("Successfully stopped agent {} on {}", agent_id, host.hostname);
                Ok(())
            },
            Err(e) => {
                warn!("Error stopping agent {}: {}", agent_id, e);
                // Don't fail if the process wasn't running
                Ok(())
            }
        }
    }

    /// Collect logs from a remote host
    async fn collect_logs(&self, agent_id: &AgentId, host: &RemoteHostConfig) -> Result<()> {
        // Get log directory path from storage
        let log_storage = &self.log_storage;
        let log_storage_ref = log_storage.as_ref();
        let local_log_dir = log_storage_ref.get_log_directory()?;
        
        std::fs::create_dir_all(&local_log_dir)
            .context("Failed to create local log directory")?;
        
        // Use scp to copy logs
        let remote_log_file = format!("{}/agent_{}.log", host.working_dir.display(), agent_id);
        let local_log_file = format!("{}/agent_{}.log", local_log_dir.display(), agent_id);
        
        let scp_cmd = Command::new("scp")
            .arg(format!("{}@{}:{}", host.username, host.hostname, remote_log_file))
            .arg(&local_log_file)
            .output()
            .context("Failed to collect logs")?;
        
        if !scp_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&scp_cmd.stderr);
            warn!("Failed to collect logs for agent {}: {}", agent_id, stderr);
            // Don't fail if log collection fails
            return Ok(());
        }
        
        info!("Successfully collected logs for agent {}", agent_id);
        Ok(())
    }

    /// Run a scenario and return the list of effects
    pub async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<Vec<Arc<dyn causality_core::effect::Effect>>> {
        // First initialize the runner with the scenario
        self.initialize(&scenario).await?;
        
        // Start the scenario
        self.start(&scenario).await?;
        
        // For geo runner, we don't have actual effect objects to return
        // Return an empty vector for now
        Ok(Vec::new())
    }
    
    /// Stop a running scenario by name
    pub async fn stop_scenario(&self, _scenario_name: &str) -> Result<()> {
        // For the geo runner, we just stop all agents
        self.stop().await
    }
}

#[async_trait]
impl SimulationRunner for GeoRunner {
    /// Initialize the runner with a scenario
    async fn initialize(&self, scenario: &Scenario) -> Result<()> {
        info!(scenario_name = %scenario.name, "Initializing geo-distributed scenario");

        // Set state to Initialized
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
        // Check if initialized
        {
            let initialized = self.initialized.lock().await;
            if !*initialized {
                return Err(anyhow!("Runner not initialized"));
            }
        }

        info!(scenario_name = %scenario.name, "Starting geo-distributed scenario");

        // Set state to Running
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Running;
        }

        // Clone the scenario agents to avoid borrowing issues
        let agents = scenario.agents.clone();

        // Start each agent
        for agent_config in &agents {
            let agent_id_str = &agent_config.id;
            let agent_id = crate::agent::agent_id::from_string(agent_id_str);
            
            info!(agent_id = %agent_id_str, "Starting geo-distributed agent");
            let host_config = self.get_host_config(&agent_id);

            // Deploy and start the agent
            if let Err(e) = self.deploy_agent(&agent_id, agent_config, &host_config).await {
                error!(agent_id = %agent_id_str, error = %e, "Failed to deploy agent");
                return Err(e);
            }

            // Start the agent in a separate task to avoid blocking
            let self_clone = self.clone();
            let agent_id_clone = agent_id.clone();
            let agent_config_clone = agent_config.clone();
            let host_config_clone = host_config.clone();

            let handle = tokio::spawn(async move {
                match self_clone.start_agent(&agent_id_clone, &agent_config_clone, &host_config_clone).await {
                    Ok(_) => {
                        info!(agent_id = %agent_id_clone, "Agent started successfully");
                        Ok(())
                    },
                    Err(e) => {
                        error!(agent_id = %agent_id_clone, error = %e, "Failed to start agent");
                        Err(e)
                    }
                }
            });

            // Store the handle
            {
                let mut agent_handles = self.agent_handles.lock().await;
                agent_handles.insert(agent_id, handle);
            }
        }

        Ok(())
    }

    /// Stop the simulation
    async fn stop(&self) -> Result<()> {
        info!("Stopping geo-distributed runner");

        // Set state to Stopped
        {
            let mut state = self.state.lock().await;
            *state = RunnerState::Stopped;
        }

        // Get all agent handles
        let agent_ids: Vec<ResourceId>;
        {
            let agent_handles = self.agent_handles.lock().await;
            agent_ids = agent_handles.keys().cloned().collect();
        }

        // Stop each agent
        for agent_id in agent_ids {
            let host_config = self.get_host_config(&agent_id);
            if let Err(e) = self.stop_agent(&agent_id, &host_config).await {
                warn!(agent_id = %agent_id, error = %e, "Error stopping agent");
            }
        }

        // Clear handles
        {
            let mut agent_handles = self.agent_handles.lock().await;
            agent_handles.clear();
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

        info!("Pausing geo-distributed runner");

        // Set state to Paused
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

        info!("Resuming geo-distributed runner");

        // Set state to Running
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
