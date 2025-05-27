//! Configuration for Simulation
//!
//! Defines configuration structures for the TEL Simulation Engine.

//-----------------------------------------------------------------------------
// Configuration Structures
//-----------------------------------------------------------------------------

/// Placeholder for simulation configuration.
#[derive(Debug, Clone, Default)]
pub struct SimulationConfig {
    pub max_steps: Option<u64>,
    pub initial_seed: Option<u64>,
}

// For example:
// pub struct SimulationRunConfig {
//     pub max_steps: Option<u64>,
//     pub initial_seed: Option<u64>,
// }
