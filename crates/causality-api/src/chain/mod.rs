//! Chain Integration for the Causality API
//!
//! This module provides interfaces and implementations for blockchain integration,
//! including transaction submission, intent handling, and chain-specific operations.
//!
//! ## Module Organization
//!
//! The chain module is organized by functionality:
//!
//! * **Client Interfaces**: `types.rs` defines the core interfaces and types
//!   for blockchain interactions
//!
//! * **Client Implementations**: `valence_client.rs` provides implementations using
//!   the valence-domain-clients library, with `factory.rs` for client creation
//!
//! * **Transaction Management**: `transaction.rs` handles transaction creation and submission
//!
//! * **Intent Handling**: `intent.rs` and `connector.rs` handle blockchain intent processing
//!
//! * **Testing Utilities**: `mock.rs` provides test implementations of the interfaces

//-----------------------------------------------------------------------------
// Core Interfaces and Types
//-----------------------------------------------------------------------------

// Client interfaces
pub mod transaction;
pub mod types;

// Client implementations
pub mod factory;
pub mod valence_client;

// Intent handling
pub mod connector;
pub mod intent;

// Testing utilities
pub mod mock;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// Client interfaces
pub use transaction::*;
pub use types::*;

// Intent interfaces
pub use connector::*;
pub use intent::*;

// Client factories and implementations
pub use factory::*;

// Selectively re-export the valence client types
pub use valence_client::ValenceChainClient;
// Don't re-export everything to avoid namespace pollution
// pub use valence_client::*;

// Testing utilities
pub use mock::*;
