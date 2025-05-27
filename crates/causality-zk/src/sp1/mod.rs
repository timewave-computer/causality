//-----------------------------------------------------------------------------
// SP1 Target-Specific Module
//-----------------------------------------------------------------------------
//
// This module contains code specifically for the SP1 RISC-V target.
// It provides simplified implementations for the SP1 environment that doesn't
// support async/await or other features available in the host environment.

#![cfg(feature = "sp1")]

pub mod circuit_stub;
pub mod format;
pub mod sync;
pub mod verification;
