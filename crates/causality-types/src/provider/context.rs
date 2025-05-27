//! Context Provider Interfaces
//!
//! Defines Service Provider Interfaces for the various contexts used within the
//! Causality framework. This module consolidates context-related traits to provide
//! a clear overview of how different operational scopes are represented.

use anyhow::Result;
use async_trait::async_trait;

use crate::primitive::ids::{DomainId, ExprId, HandlerId, ResourceId, ValueExprId};
use crate::primitive::string::Str;
use crate::core::time::Timestamp;
use crate::expr::ast::Expr as TypesExpr;
use crate::expr::result::ExprError as LispError;
use crate::expr::result::ExprResult;
use crate::expr::value::ValueExpr;
use crate::resource::{Nullifier, Resource};

//-----------------------------------------------------------------------------
// AsExprContext Trait
//-----------------------------------------------------------------------------

/// # Expression Evaluation Context (`AsExprContext`)
///
/// ## Level of Abstraction & Purpose:
/// `AsExprContext` provides the most basic, read-only view of the state necessary
/// for evaluating expressions. It operates at a low level, focusing on synchronous
/// access to immutable data like resources (as references), values, and expression ASTs.
///
/// ## Key Characteristics:
/// - **Synchronous:** All operations are expected to be non-blocking.
/// - **Read-Only (Primarily):** Focuses on data retrieval for expression inputs.
///   The `evaluate_expr` method is for evaluating sub-expressions and obtaining their
///   resulting value, not for mutating external state.
/// - **Local Scope:** Typically represents a snapshot or a localized view of state relevant
///   to a specific expression evaluation task.
///
/// ## How it Works with Others:
/// `AsExprContext` is often a foundational building block. `AsExecutionContext` extends it to add
/// mutation capabilities. An implementor of `AsRuntimeContext` or `AsExecutionContext` might
/// also directly implement `AsExprContext` or provide a way to obtain one.
pub trait AsExprContext {
    /// Get a specific field from a resource (identified by its ID), returning its `ValueExpr`.
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<Option<ValueExpr>>;

    /// Evaluate a sub-expression within the current context, returning its `ValueExpr`.
    /// This does not mutate external state.
    fn evaluate_expr(&self, expr: &TypesExpr) -> Result<ValueExpr>;

    /// Check if a resource, identified by its ID, exists and is available (e.g., not nullified locally).
    fn is_resource_available(&self, id: &ResourceId) -> Result<bool>;
}

//-----------------------------------------------------------------------------
// AsExecutionContext Trait
//-----------------------------------------------------------------------------

/// # Synchronous Execution Context (`AsExecutionContext`)
///
/// ## Level of Abstraction & Purpose:
/// `AsExecutionContext` extends `AsExprContext` to provide capabilities for synchronous
/// state mutation. It's designed for the direct, immediate execution of operations
/// or parts of an operation (e.g., within an effect handler's core logic or a transactional step)
/// where changes are applied to a local or transaction-scoped state.
///
/// ## Key Characteristics:
/// - **Synchronous:** All operations are non-blocking.
/// - **State Mutation:** Allows for creating, updating, nullifying, and locking/unlocking resources.
/// - **Transactional Scope:** Often used to manage state changes that are part of a
///   larger, potentially asynchronous, operation. The mutations here might be to an
///   in-memory representation that is later committed by a runtime.
///
/// ## How it Works with Others:
/// - Extends `AsExprContext`: Inherits all read capabilities for expression evaluation.
/// - Used by: Components performing synchronous, deterministic state changes.
/// - Relation to `AsRuntimeContext`: An `AsExecutionContext` might represent a phase
///   within a broader `AsRuntimeContext`-managed operation. For instance, an effect handler
///   called by the runtime (which operates with an `AsRuntimeContext`) might use an
///   `AsExecutionContext` internally to perform its synchronous state changes before
///   the runtime commits them or handles further async operations.
#[async_trait]
pub trait AsExecutionContext: AsExprContext + Send + Sync {
    /// Creates a new resource and makes it available in the context.
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId>;

    /// Creates a new resource derived from an existing resource with new data payload.
    /// The original resource remains immutable and unchanged.
    async fn derive_resource_data(
        &mut self,
        id: &ResourceId,
        new_data: ValueExpr,
    ) -> Result<()>;

    /// Marks a resource as nullified (permanently consumed) using a `Nullifier`.
    /// The `Nullifier` itself contains the `ResourceId`.
    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()>;

    /// Locks a resource (identified by its ID), indicating it's in use by the current transaction/operation.
    async fn lock_resource(&mut self, id: &ResourceId) -> Result<()>;

    /// Unlocks a resource (identified by its ID), releasing it from the current transaction/operation.
    async fn unlock_resource(&mut self, id: &ResourceId) -> Result<()>;

    /// Checks if a resource exists within this context (by ID), e.g., was created or added.
    async fn has_resource(&self, id: &ResourceId) -> Result<bool>;

    /// Checks if a resource, identified by its ID, has been nullified within this context.
    async fn is_nullified(&self, id: &ResourceId) -> Result<bool>;
}

//-----------------------------------------------------------------------------
// AsRuntimeContext Trait
//-----------------------------------------------------------------------------

/// # Asynchronous Runtime Context (`AsRuntimeContext`)
///
/// ## Level of Abstraction & Purpose:
/// `AsRuntimeContext` represents the broadest operational environment. It's an asynchronous
/// interface for interacting with the wider runtime system, including potentially
/// persistent storage, inter-domain communication, and global system services like time.
///
/// ## Key Characteristics:
/// - **Asynchronous:** Operations can be blocking and involve I/O (e.g., network, disk).
/// - **Global Scope:** Interacts with system-level concerns (e.g., cross-domain messaging, global clock).
/// - **State Orchestration:** Manages the lifecycle of resources at a higher level, potentially
///   involving interaction with a persistent state manager or distributed ledger.
///   Resource retrieval may involve fetching owned data (`Option<Resource>`).
///
/// ## How it Works with Others:
/// - Top-Level Operations: Typically provided to orchestrators, top-level effect handlers,
///   or services that manage end-to-end operations.
/// - May Provide Other Contexts: An implementor of `AsRuntimeContext` might provide
///   or adapt to an `AsExecutionContext` or `AsExprContext` for specific synchronous phases
///   of an operation it manages.
/// - Example: An effect handler might receive an `Arc<dyn AsRuntimeContext>`. For synchronous
///   parts of its logic, it might use a derived or separate `AsExecutionContext`.
///   For cross-domain calls or fetching definitive state, it uses the `AsRuntimeContext`.
#[async_trait]
pub trait AsRuntimeContext: Send + Sync {
    /// Retrieves an owned resource by its ID (potentially involving async I/O).
    async fn get_resource(&self, id: &ResourceId) -> Result<Option<Resource>>;

    /// Get a resource by ID (synchronous version)
    fn get_resource_sync(&self, id: &ResourceId)
        -> anyhow::Result<Option<Resource>>;

    /// Retrieves the actual ValueExpr data for a given ValueExprId.
    async fn get_value_expr_by_id(
        &self,
        id: &ValueExprId,
    ) -> Result<Option<ValueExpr>>;

    /// Get a value expression by ID (synchronous version)
    fn get_value_expr_by_id_sync(
        &self,
        id: &ValueExprId,
    ) -> anyhow::Result<Option<ValueExpr>>;

    /// Retrieves the intended input resource IDs for the current effect execution.
    /// This is context provided by the runtime for the currently dispatched effect.
    async fn get_input_resource_ids(&self) -> Result<Vec<ResourceId>>;

    /// Creates a new resource within the runtime's global state management and returns its ID.
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId>;

    /// Creates a new resource derived from an existing resource with new data payload.
    /// The original resource remains immutable and unchanged. Returns the new resource.
    async fn derive_resource_data(
        &mut self,
        id: &ResourceId,
        new_data: ValueExpr,
    ) -> Result<Resource>;

    /// Synchronous version of derive_resource_data that can be used when derivation can be done locally.
    /// Returns None if synchronous derivation is not possible, in which case callers should use the async version.
    fn derive_resource_data_sync(
        &mut self,
        _id: &ResourceId,
        _new_data: ValueExpr,
    ) -> Option<Result<Resource>> {
        None // Default implementation returns None to indicate async call is required
    }

    /// Marks a resource as nullified (permanently consumed) at the runtime level using a `Nullifier`.
    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()>;

    /// Sends a message to another domain via the runtime's messaging infrastructure.
    async fn send_message(
        &mut self,
        target_domain: DomainId,
        message_payload: ValueExpr,
    ) -> Result<()>;

    /// Gets the current causal time from the runtime's global clock.
    async fn current_time(&self) -> Result<Timestamp>;

    /// Gets the current causal time from the runtime's global clock (synchronous version).
    fn current_time_sync(&self) -> Result<Timestamp>;

    /// Retrieves an `Expr` AST by its ID (synchronous version for when ASTs are expected to be in memory).
    fn get_expr_sync(&self, id: &ExprId) -> anyhow::Result<Option<TypesExpr>>;

    /// Stores a `ValueExpr` in the runtime's state manager and returns its `ValueExprId`.
    /// This is used to ensure a `ValueExpr` exists before a `Resource` referencing it is created.
    async fn store_value_expr(&self, value_expr: ValueExpr) -> Result<ValueExprId>;
}

//-----------------------------------------------------------------------------
// StaticExprContext Trait
//-----------------------------------------------------------------------------

/// # Static Expression Context (`StaticExprContext`)
///
/// ## Level of Abstraction & Purpose:
/// `StaticExprContext` provides a more limited, read-only view specifically for
/// static expression evaluation resources like symbols and expression ASTs.
/// It is designed to be a fallback for the EvalContext in causality-lisp.
///
/// ## Key Characteristics:
/// - **Synchronous:** All operations are expected to be non-blocking.
/// - **Read-Only:** Focuses only on retrieving static data for expression evaluation.
/// - **Limited Scope:** Only provides access to symbols and expressions.
///
/// ## How it Works with Others:
/// This trait is typically implemented by components that need to provide
/// static expression evaluation context to the causality-lisp interpreter.
pub trait StaticExprContext {
    /// Get a symbol from the context by its name, returning an ExprResult.
    fn get_static_symbol(&self, name: &Str) -> Option<ExprResult>;

    /// Get an expression (AST) from the context by its ID.
    fn get_expr(&self, id: &ExprId) -> Option<&TypesExpr>;
}

//-----------------------------------------------------------------------------
// TelContextInterface Traits
//-----------------------------------------------------------------------------

/// Interface for synchronous TEL-specific context operations, primarily for Lisp host functions.
/// This trait is expected to be implemented by contexts passed to Lisp environments.
pub trait TelContextInterface: AsExprContext + Send + Sync {
    /// Get metadata for a specific handler (if any).
    fn get_handler_metadata(&self, handler_id: &HandlerId) -> Option<()>;

    /// Get the current domain ID, if any.
    fn domain_id(&self) -> Option<DomainId>;

    /// Call a registered host function by name.
    fn call_host_function(
        &mut self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Result<ValueExpr, LispError>;

    /// Get a symbol's value directly from this context.
    fn get_symbol(&self, name: &Str) -> Option<ValueExpr>;

    /// Evaluate an expression using this context's specific evaluation rules.
    /// Returns an ExprResult enum variant, not a Result<ValueExpr, Error>.
    fn evaluate(&self, expr: &TypesExpr) -> ExprResult;

    /// Get an initial binding for a symbol (typically for Lisp environments).
    fn get_initial_binding(&self, name: &Str) -> Option<ValueExpr>;

    /// Resolve a Lisp symbol to its full ExprResult, for use by the Lisp interpreter.
    /// This should perform a full hierarchical lookup if applicable.
    fn resolve_lisp_symbol(&self, name: &Str) -> Option<ExprResult>;
}

/// Asynchronous extension to TelContextInterface with async methods.
/// This trait is separated to maintain dyn-compatibility with the base TelContextInterface.
#[async_trait]
pub trait AsyncTelContextInterface: TelContextInterface {
    /// Check if a given effect, identified by its ID, has completed.
    /// Returns `Ok(true)` if the effect has completed, `Ok(false)` if it has not,
    /// or `Err(LispError)` if the effect ID is unknown or an error occurs.
    async fn is_effect_completed(
        &self,
        effect_id: &ExprId,
    ) -> Result<bool, LispError>;
}
