//! Database abstraction layer for Causality
//!
//! This module provides database traits and implementations for the Causality system.

// Module definitions
pub mod types;
pub mod memory;
#[cfg(feature = "rocks")]
pub mod rocks;
pub mod factory;

// Re-export the factory and common types
pub use factory::DbFactory;
pub use types::{Database, DbConfig, DbError, BatchOp, DbIterator}; 