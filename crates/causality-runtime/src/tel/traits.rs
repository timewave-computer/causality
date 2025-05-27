//! TEL Interface Traits
//!
//! Defines common traits for the TEL interpreter components.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use anyhow::Result;

use causality_types::primitive::string::Str;
use causality_types::ValueExpr;
use causality_types::TypeExpr;
// use causality_types::lisp::error::LispError;
use thiserror::Error; 

// Re-export AsExprContext from causality-types
pub use causality_types::provider::context::AsExecutionContext; 

//-----------------------------------------------------------------------------
// Store Interface Traits
//-----------------------------------------------------------------------------

/// Interface for accessing and manipulating the TEL graph.
pub trait GraphStoreInterface: Send + Sync {
    // Define methods as needed, e.g.:
    // fn get_node(&self, id: &NodeId) -> Result<Option<TelNode>>;
}

/// Interface for buffered write operations on TEL data.
pub trait BufferedWriteStore: Send + Sync {
    // Define methods as needed, e.g.:
    // fn commit(&mut self) -> Result<()>;
}

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Errors that can occur during a host function call from within the TEL.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum InterpreterErrorKind {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Effect not found: {0}")]
    EffectNotFound(String),
    #[error("Nullifier not found: {0}")]
    NullifierNotFound(String),
    #[error("Nullifier already spent: {0}")]
    NullifierAlreadySpent(String),
    #[error("No graph found for domain: {0}")]
    NoGraphForDomain(String),
    #[error("No handler registered for effect: {0}")]
    NoHandlerForEffect(String),
    #[error("Invalid handler for effect: {0}")]
    InvalidHandlerForEffect(String),
    #[error("Invalid effect kind: {0}")]
    InvalidEffectKind(String),
    #[error("Invalid resource state: {0}")]
    InvalidResourceState(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Execution aborted: {0}")]
    ExecutionAborted(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),
    #[error("Max number of iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),
}

/// Errors that can occur during a host function call from within the TEL.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum HostCallError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Nullifier not found: {0}")]
    NullifierNotFound(String),
    #[error("Nullifier already spent: {0}")]
    NullifierAlreadySpent(String),
    #[error("Handler not found for effect type: {0}")]
    HandlerNotFound(Str),
    #[error("Invalid input for effect handler: {0}")]
    InvalidInput(String),
    #[error("Effect execution failed: {0}")]
    EffectFailed(String),
    #[error("Behavior execution failed: {0}")]
    BehaviorFailed(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Type checking error: {0}")]
    TypeCheckingError(String),
    #[error("Context error: {0}")]
    ContextError(String),
    #[error("Lisp bridge error: {0}")]
    LispBridgeError(String),
    #[error("Graph error: {0}")]
    GraphError(String),
    #[error("Host function argument error: {0}")]
    ArgumentError(String),
    #[error("Host function execution error: {0}")]
    ExecutionError(String),
    #[error("Host function not found: {0}")]
    FunctionNotFound(String),
    #[error("Host function permission error: {0}")]
    PermissionDenied(String),
    #[error("Host function custom error: {0}")]
    Custom(String),
}

/* // Removed to resolve E0119 conflicting implementation with lisp_adapter.rs
impl From<causality_lisp::ExprError> for HostCallError {
    fn from(lisp_err: causality_lisp::ExprError) -> Self {
        match lisp_err {
            causality_lisp::ExprError::TypeError { message, expr } => {
                HostCallError::ArgumentError(format!(
                    "Lisp type error: {}{}",
                    message,
                    expr.map(|e| format!(" in expression {}", e))
                        .unwrap_or_default()
                ))
            }
            causality_lisp::ExprError::ReferenceError { name } => {
                HostCallError::ExecutionError(format!(
                    "Lisp reference error: unresolved name {}",
                    name
                ))
            }
            causality_lisp::ExprError::ExecutionError { message } => {
                HostCallError::ExecutionError(format!(
                    "Lisp execution error: {}",
                    message
                ))
            }
            causality_lisp::ExprError::PermissionError { message, resource } => {
                HostCallError::PermissionDenied(format!(
                    "Lisp permission error: {}{}",
                    message,
                    resource
                        .map(|r| format!(" for resource {}", r))
                        .unwrap_or_default()
                ))
            }
        }
    }
}
*/

impl From<crate::tel::lisp_bridge::BridgeError> for HostCallError {
    fn from(bridge_err: crate::tel::lisp_bridge::BridgeError) -> Self {
        match bridge_err {
            crate::tel::lisp_bridge::BridgeError::LispToValueMismatch(msg)
            | crate::tel::lisp_bridge::BridgeError::ValueToLispMismatch(msg)
            | crate::tel::lisp_bridge::BridgeError::UnsupportedLispType(msg)
            | crate::tel::lisp_bridge::BridgeError::UnsupportedValueType(msg)
            | crate::tel::lisp_bridge::BridgeError::ConversionFailed(msg) => {
                HostCallError::LispBridgeError(msg)
            }
            crate::tel::lisp_bridge::BridgeError::LispExecutionError(lisp_err) => {
                HostCallError::from(lisp_err)
            }
        }
    }
}

// Removed the manual impl Display for HostCallError as thiserror::Error handles it.

// Manual impl of std::error::Error is not strictly needed if using thiserror >= 1.0.25
// and the source method is not used, but good practice to ensure it's there.
// Thiserror will generate the source() method if a variant has a #[from] or source field.
// For simple string-based errors like these, source() will default to None.
// If a specific variant should chain another error, use `#[error("...: {source}")] source: UnderlyingError`
// or `#[from] UnderlyingError`.

// AsExecutionContext is now re-exported from causality_types
// pub trait AsExecutionContext: AsExprContext + Send + Sync { ... }

// --- Simulation Mocking Primitives ---
#[derive(Clone, Debug)]
pub enum AutoMockStrategy {
    /// Always attempt to succeed the effect, using the effect's output schema to generate a default value.
    SucceedWithDefaultSchemaValue,
    /// Always fail the effect with a generic error message.
    AlwaysFail,
    // Potentially others: PromptUser, UseDefinedMockIfAvailableElseFail, etc.
}

/// Represents an explicit behavior for mocking a specific effect type.
#[derive(Clone, Debug)]
pub enum MockBehavior {
    /// Always succeed with the provided ValueExpr.
    SucceedWith(ValueExpr),
    /// Always fail with the provided error message string.
    FailWith(String),
}

//-----------------------------------------------------------------------------
// Mock Provider Interface
//-----------------------------------------------------------------------------

/// This is the MockProvider trait that should be kept and used by the interpreter.
/// Provides mock implementations for effect handlers during testing or simulation.
pub trait MockProvider: Send + Sync {
    /// Determines if an effect type should be mocked.
    fn should_mock(&self, effect_type: &Str) -> bool;

    /// Generates a mocked output for a given effect instance.
    /// This is called by the interpreter if `should_mock` returns true.
    fn mock_output(
        &self,
        effect_type: &Str,
        input: &ValueExpr, // The actual input to the effect instance
        output_schema: &TypeExpr, // The schema of the expected output
    ) -> Result<ValueExpr, causality_types::effects_core::HandlerError>;
}

//-----------------------------------------------------------------------------
// Runtime Context
//-----------------------------------------------------------------------------

/// # Higher-Level Runtime Context (`AsRuntimeContext`)
///
/// ## Level of Abstraction & Purpose:
/// `AsRuntimeContext` is envisioned to provide access to broader runtime capabilities
/// that are outside the direct scope of a single TEL graph execution but might be needed
/// by host functions or the system orchestrating the TEL interpreter.
/// This could include things like inter-domain messaging, access to persistent storage,
/// global configuration, or long-running asynchronous operations.
///
/// This trait is a placeholder for future development.
pub trait AsRuntimeContext: Send + Sync {
    // Example: Send a message to another domain (potentially asynchronously)
    // async fn send_domain_message(
    //     &mut self,
    //     target_domain: DomainId,
    //     message_payload: ValueExpr,
    // ) -> Result<(), anyhow::Error>;

    // Example: Get current global causal time from the runtime
    // async fn get_global_causal_time(&self) -> Result<u64, anyhow::Error>;

    // Example: Access a persistent key-value store
    // async fn persistent_store_get(&self, key: &str) -> Result<Option<Vec<u8>>, anyhow::Error>;
    // async fn persistent_store_put(&mut self, key: &str, value: &[u8]) -> Result<(), anyhow::Error>;

    /// Placeholder method to ensure the trait can be compiled.
    fn placeholder_runtime_method(&self) -> Result<(), anyhow::Error> {
        // log::info!("AsRuntimeContext::placeholder_runtime_method called.");
        Ok(())
        // unimplemented!("AsRuntimeContext is a placeholder and its methods are not implemented.")
    }
}

//-----------------------------------------------------------------------------
// Effect Handling Erasure
//-----------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum TelEffectOutput {
    /// Continue execution
    Continue,
    /// Halt execution with a value
    SucceedWith(ValueExpr),
    /// Fail execution with an error message
    FailWith(String),
}
