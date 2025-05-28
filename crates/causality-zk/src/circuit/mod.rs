//! ZK Circuit Module
//!
//! This module defines the circuit interface and entry points for ZK proof generation,
//! including both circuit definition and execution entry points. The implementation
//! is compatible with the Valence Coprocessor for off-chain proof generation.

//-----------------------------------------------------------------------------
// ZK Circuit Module
//-----------------------------------------------------------------------------

pub mod core;

// Entry module is only available when not targeting the host

#[cfg(not(feature = "host"))]
pub mod entry;

pub use self::core::{build_circuit, CircuitBuilder, CircuitSpec};

// Only re-export entry items when sp1 feature is enabled in the future
#[cfg(all(not(feature = "host"), feature = "sp1"))]
pub struct SP1FutureSupport; // Placeholder for future SP1 support

// Core ZK Circuit functionality for the Causality System
//
// This module provides the circuit types and implementations that are used in the ZK proof system.

// -------- Circuit Reference Implementation and Types --------

// Configure implementation types based on the feature gates

// Forward Circuit exports from the appropriate implementation
#[cfg(feature = "host")]
// pub use self::host_runtime::*; // Unused import - commented out

// Host runtime is a mock implementation for testing on the host
#[cfg(feature = "host")]
mod host_runtime;

// SP1 implementation for real proofs
#[cfg(all(not(feature = "host"), feature = "sp1"))]
pub mod sp1_runtime;

// Default to a mock implementation if no other feature is specified
#[cfg(all(not(feature = "host"), not(feature = "sp1")))]
pub mod mock_runtime;
