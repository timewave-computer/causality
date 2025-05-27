//! Deployment Module for ZK Circuit Management
//!
//! This module handles circuit deployment, verification key management,
//! and registration with proving systems.

//-----------------------------------------------------------------------------
// Deployment Components
//-----------------------------------------------------------------------------

mod core;
#[cfg(feature = "host")]
mod wasm;

pub use core::*;
#[cfg(feature = "host")]
pub use wasm::*;
