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
pub mod config;
pub mod storage;
pub mod engine;
pub mod mock;

// Re-exports
pub use log::{
    LogEntry, EntryType, EntryData,
    FactId, FactSnapshot, FactDependency, FactDependencyType
};
pub use execution::context::ExecutionContext;

// Re-export key types for public API
pub use invocation::InvocationSystem;
pub use log::fact_types::{FactType};
pub use repository::CodeRepository;
pub use resource::ResourceAllocator;

// Re-export error handling
pub use causality_error::{Result, Error, EngineError};

// Re-export content types from causality-types
pub use causality_types::{ContentId, TraceId, Timestamp};

// Re-export domain types
pub use domain::DomainId;

// Re-export engine and config
pub use engine::Engine;
pub use config::EngineConfig;
pub use storage::memory::InMemoryStorage;

// Re-export mock implementations for testing
pub use mock::MockEngine;

// Import and re-export TimeMap from causality-core
pub use causality_core::time::map::TimeMap;

// Error conversion utilities
pub mod error_conversions;

// Effect system re-exports
pub use effect::executor::EffectExecutor;
pub use effect::registry::EffectRegistry;
pub use effect::tel::TelEffectExecutor;
pub use effect::tel::TelEffectAdapter;
pub use effect::tel::{create_effect_adapter, adapter_to_core_effect, register_tel_adapter_factory};
pub use effect::tel::{TelEffectRegistry, TelEffectHandler, RegistryType, TelResourceRegistry};
pub use effect::tel::{TegExecutor, TegExecutionResult};

// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 