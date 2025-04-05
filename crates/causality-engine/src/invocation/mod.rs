//! Invocation system for the Causality Engine
//!
//! This module provides the invocation system for the Causality Engine,
//! which orchestrates the execution of effects and operations through
//! various patterns like direct invocation, callbacks, streaming, etc.

pub mod context;
pub mod patterns;
pub mod registry;
pub mod system;
pub mod propagation;

// Re-export key types
pub use context::InvocationContext;
pub use propagation::ContextPropagator;
pub use patterns::{
    InvocationPatternTrait,
    InvocationPatternEnum,
    DirectInvocation,
    CallbackInvocation,
    ContinuationInvocation,
    PromiseInvocation,
    StreamingInvocation,
    BatchInvocation,
};
pub use registry::{
    EffectRegistry,
    EffectHandler,
    HandlerOutput,
    HandlerInput,
};
pub use system::InvocationSystem; 