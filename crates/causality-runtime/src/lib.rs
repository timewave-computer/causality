//! Causality Runtime Engine
//!
//! This crate is responsible for executing Temporal Effect Graphs (TEGs).
//! It translates TEG into runtime effects, manages effect handlers, and orchestrates execution.

// Core modules
pub mod error;
pub mod registry;
pub mod concurrency;
pub mod executor;
pub mod translator;

// Re-export key components
pub use error::{RuntimeError, RuntimeResult};
pub use registry::{EffectRegistrar, EffectExecutor, AsyncEffectExecutor, EffectRegistry};
pub use registry::{BasicEffectRegistry, ThreadSafeEffectRegistry};
pub use concurrency::resource_manager::{ResourceManager, ResourceOwnership, ResourceAccessMode, ConcurrencyError};
pub use executor::{ExecutionEngine, ExecutionOptions, ExecutionStatus, ExecutionError};
pub use translator::{TegTranslator, BasicTegTranslator, TranslatorError, EffectFactory, GenericEffect, GenericEffectFactory};

// Legacy re-exports (TODO: Remove or update these as needed)
// Keep the existing re-exports from the older structure
pub use log::{
    LogEntry, EntryType, EntryData,
    FactId, FactSnapshot, FactDependency, FactDependencyType
};
pub use execution::context::ExecutionContext;
pub use invocation::InvocationSystem;
pub use log::fact_types::{FactType};
pub use repository::CodeRepository;
pub use resource::ResourceAllocator;
pub use causality_error::{Result, Error, EngineError};
pub use causality_types::{ContentId, TraceId, Timestamp};
pub use domain::DomainId;
pub use engine::Engine;
pub use config::EngineConfig;
pub use storage::memory::InMemoryStorage;
pub use mock::MockEngine;
pub use causality_core::time::map::TimeMap;

// Re-export the TEG execution function
pub use execute_teg;

/// Execute a Temporal Effect Graph
///
/// This function takes a TEG and executes it using the Causality runtime system.
/// It creates an execution engine with default components and runs the graph.
pub async fn execute_teg(teg: &causality_ir::graph::TemporalEffectGraph) -> RuntimeResult<()> {
    // Create registry and translator
    let registry = Arc::new(ThreadSafeEffectRegistry::new());
    let translator = Arc::new(BasicTegTranslator::new());
    
    // Create execution engine
    let engine = ExecutionEngine::new(registry, translator, None);
    
    // Execute the TEG
    engine.execute_teg(teg)
        .await
        .map_err(|e| e.into())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 