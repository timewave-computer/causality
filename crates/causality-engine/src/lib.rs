// Causality Engine library
//
// This crate provides the execution engine for the Causality platform,
// including operation execution, context management, and effect handling.

// Core modules
pub mod log;
pub mod invocation;
pub mod execution;
pub mod operation;
pub mod effect;
pub mod resource;
pub mod domain;
pub mod repository;

// Re-exports
pub use log::{
    LogEntry, EntryType, EntryData,
    FactId, FactSnapshot, FactDependency, FactDependencyType
};
pub use operation::Operation;
pub use execution::context::ExecutionContext;

// Re-export key types for public API
pub use invocation::InvocationSystem;
pub use log::fact::{FactType};
pub use repository::CodeRepository;
pub use resource::ResourceAllocator;

// Re-export error handling
pub use causality_error::{Result, Error, EngineError};

// Re-export content types from causality-types
pub use causality_types::{ContentId, TraceId, Timestamp};

// Re-export domain types
pub use domain::DomainId;

// Import and re-export TimeMap from causality-core
pub use causality_core::time::map::TimeMap;

// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 