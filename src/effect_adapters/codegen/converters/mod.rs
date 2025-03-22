//! Code Converters for Effect Adapters
//!
//! This module provides converters for translating code between different
//! target languages and platforms, particularly for zero-knowledge proof systems.

pub mod risc_v_to_succinct;

pub use risc_v_to_succinct::RiscVToSuccinctConverter;
pub use risc_v_to_succinct::SuccinctProgram; 