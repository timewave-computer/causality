// Purpose: Defines the interface for simulation runners and provides implementations.

use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::scenario::Scenario;
use causality_core::resource::types::ResourceId;
use causality_core::effect::Effect;

/// Trait that defines the interface for simulation runners.
#[async_trait]
pub trait SimulationRunner: Send + Sync {
    /// Run a simulation scenario
    async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<HashMap<ResourceId, Arc<dyn Effect>>>;
    
    /// Stop a running scenario by name
    async fn stop_scenario(&self, scenario_name: &str) -> Result<()>;
}

// Different runner implementations
pub mod in_memory;
pub mod local_process;

// Engine module is enabled when the engine feature is active
#[cfg(feature = "engine")]
pub mod engine;

// Re-export the concrete implementations
pub use in_memory::InMemoryRunner;
pub use local_process::LocalProcessRunner;

// Engine runner is exported when the feature is enabled
#[cfg(feature = "engine")]
pub use engine::EngineRunner;

/// Types of runners available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerType {
    /// In-memory runner (default)
    InMemory,
    /// Local process runner
    LocalProcess,
    /// Engine runner (only available with the engine feature)
    #[cfg(feature = "engine")]
    Engine,
}

// Factory for creating runners
pub struct RunnerFactory;

impl RunnerFactory {
    pub fn new() -> Self {
        Self
    }
    
    pub fn create(&self, runner_type: RunnerType) -> Result<Arc<dyn SimulationRunner>> {
        match runner_type {
            RunnerType::InMemory => Ok(Arc::new(InMemoryRunner::new())),
            RunnerType::LocalProcess => Ok(Arc::new(LocalProcessRunner::new())),
            #[cfg(feature = "engine")]
            RunnerType::Engine => {
                tracing::info!("Creating engine-integrated runner");
                let runner = engine::EngineRunner::new();
                Ok(Arc::new(runner))
            },
        }
    }
} 