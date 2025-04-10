//! TEL effect integration module
//!
//! This module provides integration between the TEL language's effect system
//! and the causality-core effect system, including execution, registration,
//! and handler management.

pub mod adapter;
pub mod executor;
pub mod factory;
pub mod registry;
pub mod resource;
pub mod teg_executor;
pub mod teg_resource;

#[cfg(test)]
pub mod tests;

pub use adapter::TelEffectAdapter;
pub use executor::TelEffectExecutor;
pub use factory::{create_tel_effect, create_effect_adapter, adapter_to_core_effect, register_tel_adapter_factory};
pub use resource::TelResourceRegistry;
pub use registry::{TelEffectRegistry, TelEffectHandler, RegistryType};
pub use teg_executor::{TegExecutor, TegExecutionResult}; 