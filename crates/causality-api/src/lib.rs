//! Causality API
//!
//! This crate provides HTTP API server and client functionality for the Causality system,
//! including session management, transaction submission, and multi-chain interaction.

pub mod config;
pub mod handlers;
pub mod server;
pub mod session;
pub mod types;
pub mod client;

// Re-export commonly used types
pub use config::ApiConfig;
pub use session::ExecutionSession;
pub use server::Server;
pub use types::*;
pub use client::{ChainClient, TransactionResult};
