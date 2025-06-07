//! Command modules for the Causality CLI
//!
//! This module contains the command structure for the Causality CLI,
//! providing clean, minimal commands for working with Causality.

pub mod repl;
pub mod test_effects;
pub mod compile;
pub mod simulate;

// Re-export command structs
pub use test_effects::TestEffectsCommand;
pub use compile::CompileCommand;
pub use simulate::SimulateCommand;

// Re-export REPL command
pub use repl::*; 