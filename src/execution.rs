// Execution module for Causality Content-Addressed Code system
//
// This module provides functionality for executing content-addressed code
// with comprehensive tracing, context management, and resource control.

pub mod context;
pub mod executor;
pub mod trace;
pub mod replay;
pub mod security;

// Re-export core types
pub use context::{ExecutionContext, CallFrame, ExecutionEvent, ContextId};
pub use executor::{ContentAddressableExecutor, ExecutionError};
pub use trace::ExecutionTracer;
pub use replay::ExecutionReplayer;
pub use security::SecuritySandbox; 