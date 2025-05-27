//! Command modules for the Causality CLI
//!
//! This module contains the command structure for the Causality CLI,
//! providing clean, minimal commands for working with Causality.

//-----------------------------------------------------------------------------
// Command Module
//-----------------------------------------------------------------------------

// Existing command module

pub mod intent;
pub mod zk;

// New command module
pub mod debug;
pub mod project;
pub mod simulate;

//-----------------------------------------------------------------------------
// Re-exports

//-----------------------------------------------------------------------------

// Re-export existing commands
pub use intent::IntentCommand;
pub use zk::ZkCommands;

// Re-export new commands
pub use debug::{handle_debug_command, DebugCommands};
pub use project::{handle_project_command, ProjectCommands};
pub use simulate::{handle_simulate_command, SimulateCommands};
