//! Effect runtime implementation for the Causality Engine
//!
//! This module implements the EffectRuntime interface defined in the causality-effects crate.
//! It provides the concrete implementation for executing effects in the Causality system.

pub mod runtime;
pub mod registry;
pub mod capability;
pub mod executor;
pub mod content_addressable_executor;
pub mod resource;

/// Re-export all public items from submodules
pub use runtime::{EngineEffectRuntime, EngineCapabilityVerifier, EngineEffectRuntimeFactory};
pub use registry::EffectRegistry;
pub use capability::CapabilityManager;
pub use executor::EffectExecutor;
pub use content_addressable_executor::{
    ContentAddressableExecutor,
    SecuritySandbox,
    DefaultSecuritySandbox,
    ExecutionContext,
    CodeRepository,
    CodeEntry,
    ContentAddressed,
    Value,
    ContextId,
    ExecutionEvent,
};
pub use resource::{
    ResourceEffectManager,
    ResourceCapabilityVerifier,
    ResourceQueryEffect,
    ResourceStoreEffect,
    ResourceGetEffect,
    ResourceDeleteEffect,
}; 