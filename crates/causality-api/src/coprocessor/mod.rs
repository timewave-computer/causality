//! ZK Coprocessor Module
//!
//! This module provides interfaces and implementations for interacting with
//! Zero-Knowledge proof generation coprocessors. The module is organized into
//! submodules for different aspects of ZK coprocessor functionality.

//-----------------------------------------------------------------------------
// Core Functionality
//-----------------------------------------------------------------------------

// Basic types and interfaces
pub mod traits;
pub mod types;

// Proof generation
pub mod generator;

// Service management
pub mod monitor;
pub mod pool;
pub mod retry;

// Integration with main system
pub mod integration;

// External integrations
pub mod valence_client;

//-----------------------------------------------------------------------------
// Testing Utilities
//-----------------------------------------------------------------------------

// Mock implementations for testing
pub mod mock;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Core types
pub use traits::*;
pub use types::*;

// Service components
pub use generator::*;
pub use integration::*;
pub use monitor::*;
pub use pool::*;
pub use retry::*;

// External integrations
pub use valence_client::{
    create_coprocessor_client, create_coprocessor_client_with_socket,
    ValenceCoprocessorClientWrapper,
};

// Testing utilities
pub use mock::*;
