//! Runtime-related components for the ZK subsystem
//!
//! This module contains runtime APIs, execution context, and integration with
//! the ZK proving system. It provides the primary interfaces for:
//! - Generating witnesses from execution traces
//! - Submitting proofs to the Valence Coprocessor
//! - Verifying proofs for Causality executions
//! - Dynamic expression execution in ZK circuits

//-----------------------------------------------------------------------------
// Runtime Module Definitions
//-----------------------------------------------------------------------------

mod api;
mod api_dynamic;
pub mod core;
mod dynamic_processor;
mod effect;
mod expression_integration;

//-----------------------------------------------------------------------------
// Public Exports
//-----------------------------------------------------------------------------

pub use api::*;
pub use core::*;
pub use dynamic_processor::*;
pub use effect::*;
pub use expression_integration::*;
