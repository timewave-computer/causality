//! Witness Generation for ZK Proofs
//!
//! This module defines the witness generation logic for ZK proofs, including
//! building witnesses from execution traces and managing the witness context.
//! Witnesses contain all the required inputs for ZK proof generation while
//! maintaining deterministic serialization format for circuit compatibility.

//-----------------------------------------------------------------------------
// Witness Components
//-----------------------------------------------------------------------------

pub mod context_provider;
pub mod core;
pub mod generator;
pub mod ssz_input;
pub mod benchmark;

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub mod zk_tests;

//-----------------------------------------------------------------------------
// Core ID Re-exports
//-----------------------------------------------------------------------------

pub use crate::core::WitnessId;

//-----------------------------------------------------------------------------
// Witness Context Provider
//-----------------------------------------------------------------------------

pub use context_provider::{ResourceData, WitnessContextProvider};

//-----------------------------------------------------------------------------
// Witness Registry and Generation
//-----------------------------------------------------------------------------

pub use self::core::{
    build_witness_from_trace, AsWitness, PublicInputs, WitnessData,
};
pub use core::WitnessRegistry;

//-----------------------------------------------------------------------------
// Essential Types
//-----------------------------------------------------------------------------

pub use causality_types::trace::ExecutionTrace;

pub use generator::WitnessGenerator;
pub use ssz_input::{SszCircuitInput, SszInputMetadata, SszInputType};

/// Convenience function to create a witness generator
pub fn create_witness_generator() -> WitnessGenerator {
    WitnessGenerator::new()
}

/// Convenience function to generate circuit inputs from a witness generator
pub fn generate_circuit_inputs(generator: &WitnessGenerator) -> causality_types::anyhow::Result<Vec<SszCircuitInput>> {
    generator.generate_circuit_inputs()
}

/// Convenience function to generate a Merkle root from a witness generator
pub fn generate_merkle_root(generator: &WitnessGenerator) -> causality_types::anyhow::Result<[u8; 32]> {
    generator.generate_merkle_root()
}
