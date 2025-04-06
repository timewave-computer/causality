// Purpose: Defines the Scenario struct and related types for simulation configuration.

use serde::Deserialize;
use std::collections::HashMap;
use causality_types::DomainId;
use serde_json::Value; // Import serde_json::Value

/// Defines the execution mode for the simulation.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SimulationMode {
    InMemory,
    LocalProcess,
    GeoDistributed,
}

/// Configuration for a single agent in the simulation.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct AgentConfig {
    pub id: String, // Agent ID
    #[serde(rename = "type")]
    pub actor_type: String, // e.g., "User", "Committee"
    pub domain: Option<DomainId>,
    // TODO: Add other agent-specific config fields (e.g., path to binary for local process)
}

/// Represents an initial fact to set up in the simulation environment.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InitialFact {
    pub domain: DomainId,
    pub fact: Value, // Use serde_json::Value for arbitrary data
}

/// Configuration for simulation invariants.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InvariantConfig {
    pub no_negative_balances: Option<bool>,
    // Add other invariant checks as needed
}

/// Represents the entire configuration for a simulation run.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Scenario {
    pub name: String,
    pub description: Option<String>,
    pub simulation_mode: SimulationMode,
    pub agents: Vec<AgentConfig>,
    pub initial_state: Option<HashMap<DomainId, Vec<InitialFact>>>,
    pub invariants: Option<InvariantConfig>,
}
