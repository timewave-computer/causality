// Purpose: Defines the standard SimulatedAgent trait and related types for simulation agents.

use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use crate::scenario::AgentConfig; // Correct path
// Use ResourceId from causality-core as agents are resources.
use causality_core::resource::ResourceId;
use causality_types::crypto_primitives::ContentHash; // Needed for potential hash creation

/// Unique identifier for an agent in the simulation.
pub type AgentId = ResourceId;

/// Configuration passed to an agent when it's started by a runner.
#[derive(Debug, Clone)]
pub struct SimulationAgentConfig {
    // Information from the Scenario's AgentConfig
    pub scenario_id: String, // ID of the scenario run
    pub agent_config: AgentConfig,
    // Information provided by the runner
    // e.g., communication endpoints, shared state handles
    // TODO: Define specific fields needed by agents (e.g., message queues, peer info)
}

/// Standard interface for all agents participating in a simulation.
/// This trait is implemented by the agent's *simulation logic* or the runner
/// responsible for managing the agent process/task.
#[async_trait]
pub trait SimulatedAgent: Send + Sync + 'static {
    /// Starts the agent's execution logic based on the provided config.
    /// This might involve running an event loop, starting a process, etc.
    async fn run(&self, config: SimulationAgentConfig) -> Result<()>;

    /// Returns the unique identifier of this agent.
    fn id(&self) -> &AgentId;

    /// Called by the controller/runner to gracefully shut down the agent.
    async fn shutdown(&self) -> Result<()> {
        // Default implementation does nothing.
        // Agents/runners that need specific cleanup logic should override this.
        Ok(())
    }
} 