// Purpose: Defines the interface for simulation runners and provides implementations.

use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::scenario::Scenario;
use crate::observer::ObserverRegistry;
use crate::replay::LogStorage;
use causality_core::resource::ResourceId;
use causality_core::effect::Effect;

/// The state of a simulation runner.
#[derive(Debug, Clone, PartialEq)]
pub enum RunnerState {
    /// The runner is stopped.
    Stopped,
    /// The runner is initialized but not started.
    Initialized,
    /// The runner is running.
    Running,
    /// The runner is paused.
    Paused,
    /// The runner has encountered an error.
    Error(String),
}

/// Trait that defines the interface for simulation runners.
#[async_trait]
pub trait SimulationRunner: Send + Sync {
    /// Initialize the runner with a scenario
    async fn initialize(&self, scenario: &Scenario) -> Result<()>;
    
    /// Start the simulation
    async fn start(&self, scenario: &Scenario) -> Result<()>;
    
    /// Stop the simulation
    async fn stop(&self) -> Result<()>;
    
    /// Pause the simulation
    async fn pause(&self) -> Result<()>;
    
    /// Resume the simulation
    async fn resume(&self) -> Result<()>;
    
    /// Get the current state of the runner
    fn get_state(&self) -> RunnerState;
}

// Different runner implementations
pub mod in_memory;
pub mod local_process;
pub mod geo;

#[cfg(test)]
pub mod geo_tests;

#[cfg(test)]
pub mod in_memory_test;

#[cfg(test)]
pub mod isolated_test;

// Engine module is enabled when the engine feature is active
#[cfg(feature = "engine")]
pub mod engine;

// Re-export the concrete implementations
pub use in_memory::InMemoryRunner;
pub use local_process::LocalProcessRunner;
pub use geo::{GeoRunner, GeoRunnerConfig, RemoteHostConfig};

// Engine runner is exported when the feature is enabled
#[cfg(feature = "engine")]
pub use engine::EngineRunner;

/// Enum for different types of simulation runners.
#[derive(Clone)]
pub enum SimulationRunnerEnum {
    /// In-memory runner
    InMemory(Arc<InMemoryRunner>),
    /// Local process runner
    Local(Arc<LocalProcessRunner>),
    /// Geo-distributed runner
    Geo(Arc<GeoRunner>),
    /// Engine runner (only available with the engine feature)
    #[cfg(feature = "engine")]
    Engine(Arc<engine::EngineRunner>),
}

impl SimulationRunnerEnum {
    /// Run a simulation scenario and return the list of effects
    pub async fn run_scenario(&self, scenario: Arc<Scenario>) -> Result<Vec<Arc<dyn Effect>>> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.run_scenario(scenario).await,
            SimulationRunnerEnum::Local(runner) => runner.run_scenario(scenario).await,
            SimulationRunnerEnum::Geo(runner) => runner.run_scenario(scenario).await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.run_scenario(scenario).await,
        }
    }
    
    /// Stop a running scenario by name
    pub async fn stop_scenario(&self, scenario_name: &str) -> Result<()> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.stop_scenario(scenario_name).await,
            SimulationRunnerEnum::Local(runner) => runner.stop_scenario(scenario_name).await,
            SimulationRunnerEnum::Geo(runner) => runner.stop_scenario(scenario_name).await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.stop_scenario(scenario_name).await,
        }
    }
}

#[async_trait]
impl SimulationRunner for SimulationRunnerEnum {
    async fn initialize(&self, scenario: &Scenario) -> Result<()> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.initialize(scenario).await,
            SimulationRunnerEnum::Local(runner) => runner.initialize(scenario).await,
            SimulationRunnerEnum::Geo(runner) => runner.initialize(scenario).await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.initialize(scenario).await,
        }
    }
    
    async fn start(&self, scenario: &Scenario) -> Result<()> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.start(scenario).await,
            SimulationRunnerEnum::Local(runner) => runner.start(scenario).await,
            SimulationRunnerEnum::Geo(runner) => runner.start(scenario).await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.start(scenario).await,
        }
    }
    
    async fn stop(&self) -> Result<()> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.stop().await,
            SimulationRunnerEnum::Local(runner) => runner.stop().await,
            SimulationRunnerEnum::Geo(runner) => runner.stop().await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.stop().await,
        }
    }
    
    async fn pause(&self) -> Result<()> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.pause().await,
            SimulationRunnerEnum::Local(runner) => runner.pause().await,
            SimulationRunnerEnum::Geo(runner) => runner.pause().await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.pause().await,
        }
    }
    
    async fn resume(&self) -> Result<()> {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.resume().await,
            SimulationRunnerEnum::Local(runner) => runner.resume().await,
            SimulationRunnerEnum::Geo(runner) => runner.resume().await,
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.resume().await,
        }
    }
    
    fn get_state(&self) -> RunnerState {
        match self {
            SimulationRunnerEnum::InMemory(runner) => runner.get_state(),
            SimulationRunnerEnum::Local(runner) => runner.get_state(),
            SimulationRunnerEnum::Geo(runner) => runner.get_state(),
            #[cfg(feature = "engine")]
            SimulationRunnerEnum::Engine(runner) => runner.get_state(),
        }
    }
}

/// Types of runners available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerType {
    /// In-memory runner (default)
    InMemory,
    /// Local process runner
    LocalProcess,
    /// Geo-distributed runner
    GeoDistributed,
    /// Engine runner (only available with the engine feature)
    #[cfg(feature = "engine")]
    Engine,
}

// Factory for creating runners
pub struct RunnerFactory {
    /// Observer registry for all runners
    observer_registry: Arc<ObserverRegistry>,
    /// Log storage for all runners
    log_storage: Arc<LogStorage>,
}

impl RunnerFactory {
    pub fn new(observer_registry: Arc<ObserverRegistry>, log_storage: Arc<LogStorage>) -> Self {
        Self {
            observer_registry,
            log_storage,
        }
    }
    
    /// Create a new factory with default observer registry and log storage
    pub fn default() -> Result<Self> {
        let observer_registry = Arc::new(ObserverRegistry::new());
        let log_storage = Arc::new(LogStorage::new_temp()?);
        Ok(Self::new(observer_registry, log_storage))
    }
    
    /// Create a runner based on the runner type.
    pub fn create_runner(&self, runner_type: RunnerType) -> Result<SimulationRunnerEnum> {
        match runner_type {
            RunnerType::InMemory => {
                let mut runner = InMemoryRunner::new();
                runner.observer_registry = self.observer_registry.clone();
                runner.log_storage = self.log_storage.clone();
                Ok(SimulationRunnerEnum::InMemory(Arc::new(runner)))
            },
            RunnerType::LocalProcess => {
                let mut runner = LocalProcessRunner::new();
                // Add observer registry and log storage if needed
                Ok(SimulationRunnerEnum::Local(Arc::new(runner)))
            },
            RunnerType::GeoDistributed => {
                let config = GeoRunnerConfig::default();
                let runner = GeoRunner::new(
                    config,
                    self.observer_registry.clone(),
                    self.log_storage.clone(),
                );
                Ok(SimulationRunnerEnum::Geo(Arc::new(runner)))
            },
            #[cfg(feature = "engine")]
            RunnerType::Engine => {
                tracing::info!("Creating engine-integrated runner");
                let runner = engine::EngineRunner::new(
                    self.observer_registry.clone(),
                    self.log_storage.clone(),
                );
                Ok(SimulationRunnerEnum::Engine(Arc::new(runner)))
            },
        }
    }
} 