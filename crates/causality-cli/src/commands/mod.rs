//! Command modules for the Causality CLI
//!
//! This module contains the command structure for the Causality CLI,
//! providing clean, minimal commands for working with Causality.

pub mod repl;
pub mod test_effects;
pub mod compile;
pub mod simulate;
pub mod zk;
pub mod submit;

// Re-export command structs
pub use simulate::SimulateCommand;
pub use zk::ProveCommand;
pub use submit::SubmitCommand;

// Re-export REPL command
pub use repl::*; 
