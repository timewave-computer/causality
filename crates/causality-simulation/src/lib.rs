//! Simplified causality-simulation lib.rs

// pub mod engine;  // Temporarily disabled
pub mod history;
pub mod mocking;
pub mod sim_effects;
pub mod error;
// Temporarily disable problematic module until API compatibility is fixed
// pub mod sim_host_functions;

// Re-export key types
// pub use engine::SimulationEngine;  // Temporarily disabled
pub use history::{SimulationHistory, SimulationSnapshot};
pub use error::{SimulationError, SimulationResult};

// Re-export state manager types
pub use causality_runtime::state_manager::{DefaultStateManager, StateManager};

// Re-export commonly used types for simulation
pub use causality_types::{
    core::{Resource, Effect, Intent, Handler},
    expr::value::ValueExpr,
    core::id::{DomainId, ResourceId, EntityId},
};

// Re-export simulation-specific types
pub use sim_effects::*;
// Temporarily disabled until API compatibility is fixed
// pub use sim_host_functions::*;
