// Causality Core Crate
//
// This crate provides the core functionality for the causality platform.
// It includes the effect system, resource management, and other core components.

// Core modules
pub mod effect; // New simplified effect system based on ADR-034
pub mod capability; // Preserving old capability module for backward compatibility
pub mod resource; // Preserving old resource module for backward compatibility
pub mod resource_types;
// pub mod collection; // Temporarily disabled until implemented

// Support modules
// pub mod time; // Temporarily disabled while we fix the compatibility issues
pub mod concurrency;
// pub mod integration;
pub mod serialization;
pub mod verification;
// pub mod observation; // Temporarily disabled while we fix the compatibility issues

// Utility modules
pub mod error;
pub mod log;
pub mod id_utils;
pub mod zk;
// Use the causality-crypto crate instead of defining a local module
pub use causality_crypto as crypto;

// Re-exports
pub use effect::*;
// Use tracing instead of log
pub use tracing::{debug, error, info, trace, warn};

// Reexport key types 