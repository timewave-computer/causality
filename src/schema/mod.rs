//! Schema Evolution System
//!
//! This module provides a system for defining schemas, handling schema
//! evolution, and safely migrating data between schema versions.

use std::fmt;

pub mod definition;
pub mod evolution;
pub mod migration;
pub mod registry;
pub mod safe_state;

pub use definition::{Schema, SchemaField, SchemaType, SchemaVersion};
pub use evolution::{EvolutionRule, EvolutionRules, SchemaChange, ChangeType};
pub use migration::{MigrationStrategy, MigrationHandler, MigrationEngine};
pub use registry::{MigrationRegistry, SharedMigrationRegistry, create_migration_registry};
pub use safe_state::{
    SafeStateManager, SharedSafeStateManager, SafeStateOptions, 
    SafeStateStrategy, SafeStateStatus, SchemaTransaction
};

/// Result type for schema operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in schema operations
#[derive(Debug)]
pub enum Error {
    /// Error in schema definition
    Definition(String),
    /// Error in schema evolution
    Evolution(String),
    /// Error in schema migration
    Migration(String),
    /// Error in schema serialization/deserialization
    Serialization(String),
    /// Error in schema validation
    Validation(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Definition(msg) => write!(f, "Schema definition error: {}", msg),
            Error::Evolution(msg) => write!(f, "Schema evolution error: {}", msg),
            Error::Migration(msg) => write!(f, "Schema migration error: {}", msg),
            Error::Serialization(msg) => write!(f, "Schema serialization error: {}", msg),
            Error::Validation(msg) => write!(f, "Schema validation error: {}", msg),
        }
    }
}

impl std::error::Error for Error {} 