//! Runtime module for effect system
//!
//! This module provides the runtime components for executing effects in the system.
//! It defines the EffectRuntime interface and related structures for executing effects,
//! verifying capabilities, and managing handlers.

pub mod context;
pub mod core;
pub mod error;
pub mod runtime;
pub mod types;

// Re-exports for convenience
pub use context::{Context, ContextValue, ContextExt};
pub use error::{EffectError, EffectResult};
pub use runtime::{EffectRuntime, EffectRuntimeBase, CapabilityVerifier, EffectRuntimeFactory};
pub use types::{Effect, EffectTypeId};
pub use core::handler::EffectHandler; 