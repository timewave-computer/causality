//! Command modules for the Causality CLI
//!
//! This module contains the command structure for the Causality CLI,
//! providing clean, minimal commands for working with Causality.

pub mod diagnostics;
pub mod intent;
pub mod project;
pub mod repl;
pub mod simulate;
pub mod test_effects;
pub mod visualizer;
pub mod zk;

// Re-export REPL command
pub use repl::*; 