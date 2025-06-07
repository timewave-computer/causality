//! Causality CLI Library
//!
//! This crate provides command-line interface functionality for the Causality framework.
//! It can be used both as a binary and as a library.

pub mod commands;
pub mod error;

pub use commands::*;
pub use error::*; 