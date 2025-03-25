// Execution engine for transaction processing
// Original file: src/execution/mod.rs

// Execution module for Causality
//
// This module provides the execution framework for content-addressable code,
// including context management, tracing, and replay functionality.

pub mod context;
pub mod executor;
pub mod replay;
pub mod trace;
pub mod security;

// Re-export key types
pub use context::{
    ContextId, ExecutionContext, ExecutionEvent, Value, 
    CallFrame, ExecutionError
};
pub use executor::{ContentAddressableExecutor};
pub use trace::{ExecutionTracer};
pub use replay::{ExecutionReplayer};
pub use security::{SecuritySandbox}; 