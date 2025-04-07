//! Simulation framework for Causality with both standalone and engine-integrated modes
//!
//! This crate supports two operation modes through feature flags:
//!
//! 1. "standalone" (default): Provides a simplified mock implementation
//!    that can run independently without the full engine.
//!
//! 2. "engine": Integrates with the `causality-engine` crate to provide
//!    a more realistic simulation environment.
//!
//! # Known issues with the engine integration:
//! 
//! - Field name mismatches between simulation and engine APIs
//! - Structure differences in log entries
//! - Missing required fields in engine implementation
//!
//! See the README.md for more details.

#![deny(unsafe_code)]

/// The scenario module for defining simulation scenarios
pub mod scenario;

/// The agent module for defining simulation agents
pub mod agent;

/// The runner module for executing simulations
pub mod runner;

/// The controller module for managing simulation execution
pub mod controller;

/// The log module for interacting with the causality log system
pub mod log;

/// The observer module for subscribing to simulation events
pub mod observer;

/// The observe module for additional observer extensions
pub mod observe;

/// The replay module for replay capabilities
pub mod replay;

/// The invariant module for checking simulation rules
pub mod invariant;

/// The CLI module for command-line interface
pub mod cli;

#[cfg(test)]
pub mod controller_tests;

#[cfg(test)]
pub mod cli_tests;

#[cfg(test)]
mod invariant_test;

// No need for a separate tests module since we have modules like
// geo_tests and isolated_test already defined in runner/mod.rs

// Re-export the public API
pub use scenario::{Scenario, AgentConfig, InvariantConfig, InitialFact};
pub use agent::{SimulatedAgent, AgentId, SimulationAgentConfig};
pub use runner::{SimulationRunner, RunnerType, RunnerFactory};
pub use controller::SimulationController;
pub use observer::{Observer, LogFilter};
pub use invariant::{InvariantChecker, InvariantObserver, InvariantResult, InvariantType};
