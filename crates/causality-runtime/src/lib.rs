//! Causality Runtime System
//!
//! This crate provides the runtime execution environment for the Causality framework,
//! including instruction execution, effect handling, ZK proof generation, and resource management.

pub mod error;
pub mod executor;

// Core exports
pub use error::*;
pub use executor::*;
