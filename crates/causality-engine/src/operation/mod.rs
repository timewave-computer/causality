// Operation module
//
// This module provides operation execution and context management functionality.

mod api;
mod context;
mod execution;
mod transformation;
mod verification;
mod zk;
mod types;

// Fix ambiguous re-exports by being explicit with which modules we re-export
pub use api::{OperationManager, builder};
// Export api traits with explicit names to avoid conflicts
pub use api::{ResourceRegisterTrait as ApiResourceRegisterTrait, Interpreter as ApiInterpreter};

// Export execution traits with explicit names to avoid conflicts
pub use execution::{ResourceRegisterTrait as ExecutionResourceRegisterTrait, Interpreter as ExecutionInterpreter};
pub use execution::{OperationExecutor, OperationResult, ExecutionError, execute_operation};
pub use execution::{AbstractExecutor, RegisterExecutor, ZkExecutor};

// Re-export other modules normally
pub use context::*;
pub use transformation::*;
pub use verification::*;
pub use zk::*;
pub use types::*; 