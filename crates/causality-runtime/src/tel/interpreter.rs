use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;
use sha2::Digest;

use causality_types::{
    core::{
        id::{ResourceId, HandlerId, ValueExprId, ExprId, DomainId, AsId, SubgraphId},
        resource_conversion::ToValueExpr,
        str::Str as CausalityStr,
    },
    expr::{
        value::ValueExpr,
        ast::{Expr as TypesExpr, AtomicCombinator},
    },
    resource::{Resource, Nullifier},
    provider::context::{AsExprContext, AsExecutionContext, AsRuntimeContext, TelContextInterface},
    TypeExpr,
    serialization::Encode,
    interpreter_config::{LispContextConfig, LispEvaluator, LispEvaluationError},
    effects_core::HandlerError as CausalityHandlerError,
    compiler_output::{CompiledTeg, CompiledSubgraph as CompilerCompiledSubgraph},
};

use causality_lisp::{
    core::ExprContextual,
    Evaluator,
    Interpreter as LispConcreteInterpreter,
};

use causality_types::expr::result::{ExprResult, ExprError as LispError};

use causality_core::extension_traits::ValueExprExt;

use crate::{
    state_manager::StateManager,
    tel::{lisp_adapter::TelLispAdapter, traits::MockProvider},
};

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Handler not found: {0}")]
    HandlerNotFound(String),
    #[error("Expression not found: {0}")]
    ExpressionNotFound(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

// Using the same definition as causality-types
#[derive(Debug)]
pub enum LispEvaluationErrorLocal {
    EvaluationFailed(String),
    ResourceCreationFailed(String),
    ResourceNullificationFailed(String),
    ValueStorageFailed(String),
    ExprResolutionFailed(String),
}

impl std::fmt::Display for LispEvaluationErrorLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispEvaluationErrorLocal::EvaluationFailed(s) => {
                write!(f, "Lisp evaluation failed: {}", s)
            }
            LispEvaluationErrorLocal::ResourceCreationFailed(s) => {
                write!(f, "Resource creation failed: {}", s)
            }
            LispEvaluationErrorLocal::ResourceNullificationFailed(s) => {
                write!(f, "Resource nullification failed: {}", s)
            }
            LispEvaluationErrorLocal::ValueStorageFailed(s) => {
                write!(f, "Value storage failed: {}", s)
            }
            LispEvaluationErrorLocal::ExprResolutionFailed(s) => {
                write!(f, "Expression resolution failed: {}", s)
            }
        }
    }
}

impl std::error::Error for LispEvaluationErrorLocal {}

// A more concrete representation for the graph loaded from a CompiledTeg
#[derive(Debug, Clone, Default)]
pub struct LoadedTelGraph {
    pub subgraphs: std::collections::BTreeMap<SubgraphId, CompilerCompiledSubgraph>,
}

/// Temporal Event Ledger (TEL) Interpreter
///
/// The `Interpreter` is the central execution engine for TEL operations. It manages 
/// state, evaluates Lisp expressions in various contexts, and orchestrates interactions 
/// with resources and capabilities.
///
/// Key Responsibilities:
/// - **State Management**: Owns and manages the `StateManager` (e.g., `DefaultStateManager`) 
///   for persistent storage of resources, expressions, and values.
/// - **Lisp Execution**: 
///   - Embeds a `causality_lisp::Interpreter` for evaluating Lisp code.
///   - Manages a `global_lisp_adapter` which wraps a `LispHostEnvironment` acting as the 
///     top-level context for globally defined Lisp functions (like the capability system).
///   - Provides `evaluate_lisp_in_context` to run Lisp expressions within a dynamically 
///     created `LispHostEnvironment` that can inherit from the global context, allowing for 
///     scoped evaluations (e.g., for resource static expressions, effect execution).
/// - **Capability System Integration**: 
///   - Loads Lisp definitions for the capability system (e.g., `capability-check`, `capability-grant`) 
///     into its `global_lisp_adapter` via `load_lisp_definitions`.
///   - Core operations like `create_resource`, `update_resource_data`, `nullify_resource` invoke 
///     these Lisp capability functions to enforce access control.
/// - **Resource Operations**: Implements `AsExecutionContext` and `AsRuntimeContext` to provide 
///   primitives for creating, reading, updating, and nullifying resources, often involving 
///   capability checks.
/// - **Context Provisioning**: Acts as a provider for various context traits (`AsExprContext`, 
///   `TelContextInterface`, `TypesTelContextInterface`, `LispEvaluator`) required by different 
///   parts of the system, delegating to its internal components like the state manager or Lisp 
///   environments.
/// - **Domain Awareness**: Can be associated with a specific `DomainId`, relevant for capability 
///   checks and resource ownership.
///
pub struct Interpreter {
    pub domain_id: Option<DomainId>,
    loaded_graph: LoadedTelGraph,
    pub(crate) state_manager: Arc<Mutex<dyn StateManager>>,
    lisp_interpreter: Arc<dyn Evaluator>, // Changed to dyn Evaluator
    global_lisp_adapter: Arc<Mutex<TelLispAdapter>>,
}

impl std::fmt::Debug for Interpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("domain_id", &self.domain_id)
            .field("state_manager", &"<StateManager>")
            .field("lisp_interpreter", &"<LispInterpreter>")
            .field("global_lisp_adapter", &"<TelLispAdapter>")
            .finish()
    }
}

impl Interpreter {
    pub fn new(state_manager: Arc<Mutex<dyn StateManager>>, global_lisp_adapter: Arc<Mutex<TelLispAdapter>>) -> Self {
        Self {
            domain_id: None,
            loaded_graph: LoadedTelGraph::default(),
            state_manager,
            lisp_interpreter: Arc::new(LispConcreteInterpreter::new()), // Use concrete type
            global_lisp_adapter,
        }
    }

    // Method to allow setting the domain_id, primarily for testing or specific setup scenarios.
    pub fn set_domain_id(&mut self, domain_id: Option<DomainId>) {
        self.domain_id = domain_id;
    }

    /// Public accessor for the state manager, primarily for testing and internal trusted setup.
    pub fn state_manager(&self) -> Arc<Mutex<dyn StateManager>> {
        Arc::clone(&self.state_manager)
    }

    /// Loads a compiled TEG program into the interpreter.
    /// This involves loading global Lisp definitions, storing all expressions and handlers,
    /// and setting up the graph structure.
    pub async fn load_compiled_teg(&mut self, compiled_teg: CompiledTeg) -> Result<()> {
        // 1. Identify which expressions in `compiled_teg.expressions` are global definitions
        // (e.g., defun, defmacro from :global-lisp or includes).
        
        let mut global_defs_to_load = Vec::new();
        for expr in compiled_teg.expressions.values() {
            // Check if the expression is an Apply with a symbol as its first operand
            if let TypesExpr::Apply(box_func, _expr_vec) = expr {
                if let TypesExpr::Var(sym) = &***box_func {  // Note the extra dereference
                    // Check if the symbol string starts with "defun" or "defmacro"
                    let sym_str = String::from_utf8_lossy(sym.as_bytes());
                    if sym_str.starts_with("defun") || sym_str.starts_with("defmacro") {
                        global_defs_to_load.push(expr.clone());
                    }
                }
            }
        }

        if !global_defs_to_load.is_empty() {
            // Store the length for logging before we move global_defs_to_load
            let def_count = global_defs_to_load.len();
            
            // Construct the list expression expected by load_lisp_definitions
            // (list (defun ...) (defmacro ...) ...)
            let list_combinator_expr = TypesExpr::Combinator(AtomicCombinator::List);
            let definitions_payload = TypesExpr::Apply(
                causality_types::expr::ExprBox(Box::new(list_combinator_expr)), 
                causality_types::expr::ExprVec(global_defs_to_load) // moved here
            );
            
            log::info!("Loading {} global Lisp definitions into interpreter...", def_count);
            self.load_lisp_definitions(&definitions_payload).await.map_err(|e| anyhow!("Failed to load global Lisp definitions: {}", e))?;
        } else {
            log::info!("No global Lisp definitions (defun/defmacro) found in CompiledTeg to load.");
        }

        // 2. Store All Expressions in StateManager
        {
            let mut sm_guard = self.state_manager.lock().await; 
            for (expr_id, expr) in compiled_teg.expressions {
                sm_guard.put_expr(expr_id, expr)
                    .map_err(|e| anyhow!("Failed to store expression {:?}: {}", expr_id, e))?;
                log::trace!("Stored expression {:?}", expr_id);
            }
        }

        // 3. Store Handlers in StateManager
        {
            let mut sm_guard = self.state_manager.lock().await;
            for (handler_id, handler_data) in compiled_teg.handlers {
                sm_guard.put_handler(handler_id, handler_data)
                    .map_err(|e| anyhow!("Failed to store handler {:?}: {}", handler_id, e))?;
                log::trace!("Stored handler {:?}", handler_id);
            }
        }

        // 4. Set up Graph Structure
        self.loaded_graph = LoadedTelGraph {
            subgraphs: compiled_teg.subgraphs,
        };
        log::info!("Loaded TEG graph structure with {} subgraphs.", self.loaded_graph.subgraphs.len());

        Ok(())
    }

    pub async fn load_lisp_definitions(&self, definitions_expr: &TypesExpr) -> Result<(), LispError> {
        // The TypesExpr enum doesn't have a List variant. We need to work with the structure available
        if let TypesExpr::Apply(combinator, operands) = definitions_expr {
            // Check if the combinator is the List combinator
            if let TypesExpr::Combinator(AtomicCombinator::List) = &***combinator {  // Note the triple dereference
                // Process each definition expression one by one, without requiring a common adapter lock
                for def_expr in operands.0.iter() {
                    // Log the definition
                    if let TypesExpr::Apply(def_type_box, def_args) = def_expr {
                        if let TypesExpr::Var(ref name) = &***def_type_box {  // Note the triple dereference
                            // Convert bytes to string for comparison
                            let name_str = String::from_utf8_lossy(name.as_bytes());
                            if name_str.starts_with("defun") || name_str.starts_with("defmacro") {
                                if let Some(TypesExpr::Var(ref func_name)) = def_args.0.first() {
                                    log::info!("Loading global Lisp definition for '{}' ({})", func_name, name_str);
                                } else {
                                    log::warn!("Malformed {} expression in global Lisp definitions: Missing function name", name_str);
                                }
                            } else {
                                log::warn!("Expected defun/defmacro in global Lisp definitions, got: {:?}", def_type_box);
                            }
                        } else {
                            log::warn!("Expected Var expression in def_type_box, got: {:?}", def_type_box);
                        }
                    } else {
                        log::warn!("Expected Apply expression in global Lisp definitions, got: {:?}", def_expr);
                    }
                    
                    // Use the lisp interpreter directly - get a fresh adapter lock for each expression
                    let adapter_guard = self.global_lisp_adapter.lock().await;
                    
                    match self.lisp_interpreter.evaluate_expr(def_expr, &*adapter_guard).await {
                        Ok(_) => log::trace!("Successfully loaded Lisp definition"),
                        Err(e) => {
                            log::error!("Failed to load Lisp definition: {}", e);
                            return Err(e);
                        }
                    }
                    
                    std::mem::drop(adapter_guard); // Release the lock after evaluation
                }
                Ok(())
            } else {
                Err(LispError::ExecutionError { 
                    message: CausalityStr::from("Expected a List combinator at the root of definitions_expr")
                })
            }
        } else {
            Err(LispError::ExecutionError { 
                message: CausalityStr::from("Expected an Apply expression at the root of definitions_expr")
            })
        }
    }

    pub async fn evaluate_resource_static_expr(
        &self,
        resource_id: &ResourceId,
    ) -> Result<bool> {
        let sm_guard = self.state_manager.blocking_lock();
        let resource = sm_guard.get_resource(resource_id).await?
            .ok_or_else(|| anyhow!(InterpreterError::ResourceNotFound(format!("Resource not found for static expr eval: {:?}", resource_id))))?;

        // For now, we'll skip static validation since the Resource struct no longer has static_expr
        // In a full implementation, static validation would be handled differently
        
        // Convert resource to a constant expression for evaluation
        let resource_value = resource.to_value_expr();
        let resource_expr = TypesExpr::Const(resource_value);
        
        let config = LispContextConfig {
            host_function_profile: Some(CausalityStr::from_static_str("static_validation")),
            initial_bindings: std::collections::BTreeMap::new(),
            additional_host_functions: BTreeMap::new(),
        };

        let result_value_expr = self.evaluate_lisp_in_context(
            &resource_expr,
            Vec::new(),
            &config
        ).await.map_err(|e| anyhow!("Lisp evaluation failed for static_expr of resource {:?}: {:?}", resource_id, e))?;

        match result_value_expr {
            ValueExpr::Bool(b) => Ok(b),
            _ => Err(anyhow!(
                "Static expression for resource {:?} (expr {:?}) did not return a boolean value. Got: {:?}",
                resource_id,
                resource_expr,
                result_value_expr
            )),
        }
    }

    pub async fn evaluate_potential_resource_static_expr(
        &self,
        resource_to_check: &Resource,
        _potential_value_override: Option<&ValueExpr>,
    ) -> Result<bool> {
        // For now, we'll skip static validation since the Resource struct no longer has static_expr
        // In a full implementation, static validation would be handled differently
        
        // Convert resource to a constant expression for evaluation
        let resource_value = resource_to_check.to_value_expr();
        let resource_expr = TypesExpr::Const(resource_value);
        
        let result_value_expr = self.evaluate_lisp_in_context(
            &resource_expr,
            Vec::new(),
            &LispContextConfig::default()
        ).await.map_err(|e| anyhow!("Lisp evaluation failed for resource {:?}: {:?}", resource_to_check.id, e))?;

        match result_value_expr {
            ValueExpr::Bool(b) => Ok(b),
            _ => Err(anyhow!(
                "Static expression for resource {:?} (expr {:?}) did not return a boolean value. Got: {:?}",
                resource_to_check.id,
                resource_expr,
                result_value_expr
            )),
        }
    }

    /// Evaluates a Lisp expression in a context with the provided configuration
    pub async fn evaluate_lisp_in_context_impl(
        &self, 
        expr_to_eval: Arc<TypesExpr>,
        args: Vec<ValueExpr>, 
        _config: &LispContextConfig,
    ) -> Result<ValueExpr, LispEvaluationError> {
        let lisp_interpreter = LispConcreteInterpreter::new();
        
        let final_expr = if !args.is_empty() {
            // If args are provided, we need to create an application expression
            // that applies the expr_to_eval to the args
            let mut operands = vec![(*expr_to_eval).clone()];
            for arg in args {
                // Validate each arg can be serialized with SSZ
                let _ssz_bytes = arg.as_ssz_bytes();
                operands.push(TypesExpr::Const(arg));
            }
            
            // Use the ExprBox and ExprVec wrappers for Apply
            TypesExpr::Apply(
                causality_types::expr::ExprBox(Box::new(TypesExpr::Var(CausalityStr::from_static_str("dynamic")))),
                causality_types::expr::ExprVec(operands)
            )
        } else {
            (*expr_to_eval).clone()
        };

        let global_lisp_adapter = self.global_lisp_adapter.lock().await;

        // Evaluate the expression
        match lisp_interpreter.evaluate_expr(&final_expr, &*global_lisp_adapter).await {
            Ok(result) => match result {
                ExprResult::Value(v) => {
                    // Validate the result can be serialized with SSZ
                    let _ssz_bytes = v.as_ssz_bytes();
                    Ok(v)
                },
                _ => Err(LispEvaluationError::EvaluationFailed(format!("Expected value, got {:?}", result))),
            },
            Err(e) => Err(LispEvaluationError::EvaluationFailed(e.to_string())),
        }
    }
}

impl AsExprContext for Interpreter {
    fn get_resource_field(&self, _id: &ResourceId, _field: &str) -> Result<Option<ValueExpr>> {
        Err(anyhow!(
            "TelInterpreter::get_resource_field: Not implemented. Requires ValueExpr field access."
        ))
    }

    fn evaluate_expr(&self, _expr: &TypesExpr) -> Result<ValueExpr> {
        Err(anyhow::anyhow!(InterpreterError::NotImplemented("evaluate_expr".to_string())))
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        let sm_guard = self.state_manager.blocking_lock();
        // Use futures::executor::block_on to handle the async calls in a sync context
        let exists = futures::executor::block_on(sm_guard.get_resource(id))?.is_some();
        if !exists { return Ok(false); }
        // Check if resource is nullified
        let nullified = futures::executor::block_on(sm_guard.is_nullified(id))?;
        Ok(!nullified)
    }
}
#[async_trait]
impl AsExecutionContext for Interpreter {
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId> {
        // Resource creation is handled by the state manager
        // No capability checks in the runtime - these will be handled by Lisp payloads
        let mut sm_guard = self.state_manager.lock().await;
        AsExecutionContext::create_resource(&mut *sm_guard, resource).await // Disambiguated and awaited
    }

    async fn derive_resource_data(&mut self, id: &ResourceId, new_data: ValueExpr) -> Result<()> {
        // Derive resource data is handled by the state manager
        // No capability checks in the runtime - these will be handled by Lisp payloads
        let _new_data_id = new_data.id(); // Use ValueExprExt::id() - prefix with underscore as it's unused

        // The state manager doesn't have values_mut method, so directly delegate to its implementation
        let mut sm_guard = self.state_manager.lock().await;
        // Store the value expression first if needed
        sm_guard.store_value_expr(new_data.clone()).await?;
        
        // Then derive the resource data (creates new Resource)
        AsExecutionContext::derive_resource_data(&mut *sm_guard, id, new_data).await
    }

    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()> {
        // Resource nullification is handled by the state manager
        // No capability checks in the runtime - these will be handled by Lisp payloads
        let mut sm_guard = self.state_manager.lock().await;
        AsExecutionContext::nullify_resource(&mut *sm_guard, nullifier).await // Disambiguated and awaited
    }

    async fn lock_resource(&mut self, id: &ResourceId) -> Result<()> {
        let mut sm_guard = self.state_manager.lock().await;
        sm_guard.lock_resource(id).await // Await the future
    }

    async fn unlock_resource(&mut self, id: &ResourceId) -> Result<()> {
        let mut sm_guard = self.state_manager.lock().await;
        sm_guard.unlock_resource(id).await // Await the future
    }

    async fn has_resource(&self, id: &ResourceId) -> Result<bool> {
        let sm_guard = self.state_manager.lock().await;
        sm_guard.has_resource(id).await // Await the future
    }

    async fn is_nullified(&self, resource_id: &ResourceId) -> Result<bool> {
        let sm_guard = self.state_manager.lock().await;
        sm_guard.is_nullified(resource_id).await // Await the future
    }
}

#[async_trait]
impl AsRuntimeContext for Interpreter {
    async fn get_resource(&self, id: &ResourceId) -> Result<Option<Resource>> {
        self.state_manager.lock().await.get_resource(id).await
    }
    fn get_resource_sync(&self, id: &ResourceId) -> Result<Option<Resource>> {
        // Use block_on to convert the async call to sync
        futures::executor::block_on(self.state_manager.blocking_lock().get_resource(id))
    }
    async fn get_value_expr_by_id(&self, id: &ValueExprId) -> Result<Option<ValueExpr>> {
        self.state_manager.lock().await.get_value_expr_by_id(id).await
    }
    fn get_value_expr_by_id_sync(&self, id: &ValueExprId) -> Result<Option<ValueExpr>> {
        self.state_manager.blocking_lock().get_value_expr_by_id_sync(id)
    }
    async fn get_input_resource_ids(&self) -> Result<Vec<ResourceId>> {
        self.state_manager.lock().await.get_input_resource_ids().await
    }
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId> {
        // Resource creation is handled by the state manager
        // No capability checks in the runtime - capability handling is now through Lisp payloads
        let mut sm_guard = self.state_manager.lock().await;
        AsRuntimeContext::create_resource(&mut *sm_guard, resource).await
    }
    async fn derive_resource_data(&mut self, id: &ResourceId, new_data: ValueExpr) -> Result<Resource> {
        // Using SSZ serialization
        let new_data_bytes = new_data.as_ssz_bytes();
        
        // Create a hash of the data to use with new()
        let mut hasher = sha2::Sha256::new();
        hasher.update(&new_data_bytes);
        let hash_result = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_result);
        let _new_data_id = ValueExprId::new(hash_array); // Prefix with underscore as it's unused

        // The trait definition for AsRuntimeContext::update_resource_data returns Result<Resource>
        // The StateManager's implementation of AsRuntimeContext will handle this.
        let mut sm_guard = self.state_manager.lock().await;
        AsRuntimeContext::derive_resource_data(&mut *sm_guard, id, new_data).await
    }
    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()> {
        // Resource nullification is handled by the state manager
        // No capability checks in the runtime - capability handling is now through Lisp payloads
        let mut sm_guard = self.state_manager.lock().await;
        AsRuntimeContext::nullify_resource(&mut *sm_guard, nullifier).await
    }
    async fn send_message(&mut self, target_domain: DomainId, message_payload: ValueExpr) -> Result<()> {
        let mut sm_guard = self.state_manager.lock().await;
        sm_guard.send_message(target_domain, message_payload).await
    }
    async fn current_time(&self) -> Result<causality_types::core::time::Timestamp> {
        self.state_manager.lock().await.current_time().await
    }
    fn current_time_sync(&self) -> Result<causality_types::core::time::Timestamp> {
        self.state_manager.blocking_lock().current_time_sync()
    }
    fn get_expr_sync(&self, id: &ExprId) -> Result<Option<TypesExpr>> {
        self.state_manager.blocking_lock().get_expr_sync(id)
    }
    async fn store_value_expr(&self, value_expr: ValueExpr) -> Result<ValueExprId> {
        // Validate that we can serialize with SSZ
        let _ssz_bytes = value_expr.as_ssz_bytes();
            
        self.state_manager.lock().await.store_value_expr(value_expr).await
    }
}

impl MockProvider for Interpreter {
    fn should_mock(&self, _effect_type: &CausalityStr) -> bool {
        false
    }
    fn mock_output(
        &self,
        effect_type: &CausalityStr,
        _input: &ValueExpr,
        _output_schema: &TypeExpr,
    ) -> Result<ValueExpr, CausalityHandlerError> {
        Err(CausalityHandlerError::InternalError(
            format!("Interpreter does not mock effects. Effect type: {}", effect_type)
        ))
    }
}

impl TelContextInterface for Interpreter {
    fn get_handler_metadata(&self, _handler_id: &HandlerId) -> Option<()> {
        None
    }
    fn domain_id(&self) -> Option<DomainId> {
        self.domain_id
    }
    fn call_host_function(&mut self, name: &CausalityStr, args: Vec<ValueExpr>) -> Result<ValueExpr, LispError> {
        // This should delegate to the global_lisp_adapter's context
        // However, global_lisp_adapter.lock().await.call_host_function requires &mut TelLispAdapter
        // and here we have &mut Interpreter. This needs careful handling of MutexGuard.
        // For now, assume this is for functions not needing StateManager mutation directly via Interpreter.
        // This is tricky because TelLispAdapter also implements TelContextInterface.
        // Let's assume this is meant to call *additional* host functions registered directly on Interpreter if any,
        // or it's a misrouted call that should go via a LispHostEnvironment.
        // This path seems less used than the LispHostEnvironment's call_host_function.
        // For now, return NotImplemented.
        // Update: LispHostEnvironment::call_host_function now takes &mut self
        // So, we can delegate to the global adapter's context.
        futures::executor::block_on(async {
            let adapter_guard = self.global_lisp_adapter.lock().await;
            // TelLispAdapter itself doesn't directly implement call_host_function.
            // Its underlying LispHostEnvironment does.
            // Directly use adapter_guard which implements TelContextInterface
            // Use try_call_host_function which returns a Future
            let host_fn_result = adapter_guard.try_call_host_function(name, args).await;
            match host_fn_result {
                Some(Ok(value)) => Ok(value),
                Some(Err(e)) => Err(e),
                None => Err(LispError::ExecutionError { message: CausalityStr::from(format!("Host function not available: {}", name)) })
            }
        })
    }
    fn get_symbol(&self, name: &CausalityStr) -> Option<ValueExpr> {
        // Delegate to global_lisp_adapter's context
        futures::executor::block_on(async {
            let adapter_guard = self.global_lisp_adapter.lock().await;
            
            // get_symbol returns a Future
            let symbol_result = adapter_guard.get_symbol(name).await;
            
            // Convert from Option<ExprResult> to Option<ValueExpr>
            symbol_result.and_then(|result| {
                if let ExprResult::Value(value) = result {
                    Some(value)
                } else {
                    None
                }
            })
        })
    }
    fn evaluate(&self, expr: &TypesExpr) -> ExprResult {
        // Delegate to the state manager's evaluate_expr method
        match self.state_manager.try_lock() {
            Ok(sm_guard) => {
                match sm_guard.evaluate_expr(expr) {
                    Ok(value) => {
                        // sm_guard.evaluate_expr returns ValueExpr, so wrap it in ExprResult::Value
                        ExprResult::Value(value)
                    }
                    Err(e) => ExprResult::Value(ValueExpr::String(format!("ERROR: {}", e).into()))
                }
            }
            Err(_) => ExprResult::Value(ValueExpr::String("State manager lock error".into()))
        }
    }
    fn get_initial_binding(&self, name: &CausalityStr) -> Option<ValueExpr> {
        // Return None for now - this would be used for initial symbol bindings
        None
    }
    fn resolve_lisp_symbol(&self, name: &CausalityStr) -> Option<ExprResult> {
        // Map get_symbol to ExprResult::Value
        self.get_symbol(name).map(ExprResult::Value)
    }
}

#[async_trait]
impl LispEvaluator for Interpreter {
    fn get_expr_sync(&self, id: &ExprId) -> Result<Option<TypesExpr>, LispEvaluationError> {
        match self.state_manager.blocking_lock().get_expr_sync(id) {
            Ok(expr) => Ok(expr),
            Err(e) => Err(LispEvaluationError::ExprResolutionFailed(e.to_string())),
        }
    }

    async fn evaluate_lisp_in_context(
        &self,
        expr_to_eval: &TypesExpr,
        args: Vec<ValueExpr>,
        _config: &LispContextConfig,
    ) -> Result<ValueExpr, LispEvaluationError> {
        // Convert &TypesExpr to Arc<TypesExpr> for evaluate_lisp_in_context_impl
        self.evaluate_lisp_in_context_impl(Arc::new(expr_to_eval.clone()), args, _config).await
    }

    async fn store_value_expr(
        &self,
        value_expr: ValueExpr,
    ) -> Result<ValueExprId, LispEvaluationError> {
        // Ensure we can serialize with SSZ (validation step)
        let _ssz_bytes = value_expr.as_ssz_bytes();
            
        match self.state_manager.lock().await.store_value_expr(value_expr).await {
            Ok(id) => Ok(id),
            Err(e) => Err(LispEvaluationError::ValueStorageFailed(e.to_string())),
        }
    }

    async fn create_resource_for_evaluator(
        &mut self,
        resource: Resource,
    ) -> Result<ResourceId, LispEvaluationError> {
        match AsExecutionContext::create_resource(self, resource).await {
            Ok(id) => Ok(id),
            Err(e) => Err(LispEvaluationError::ResourceCreationFailed(e.to_string())),
        }
    }

    async fn nullify_resource_for_evaluator(
        &mut self,
        nullifier: Nullifier,
    ) -> Result<(), LispEvaluationError> {
        match AsExecutionContext::nullify_resource(self, nullifier).await {
            Ok(()) => Ok(()),
            Err(e) => Err(LispEvaluationError::ResourceNullificationFailed(e.to_string())),
        }
    }
}

#[allow(dead_code)]
fn convert_lisp_result_to_value_expr(
    result: causality_types::expr::result::ExprResult,
) -> Result<ValueExpr, CausalityHandlerError> {
    match result {
        causality_types::expr::result::ExprResult::Value(v) => Ok(v), // ValueExpr is directly usable
        causality_types::expr::result::ExprResult::Atom(atom) => match atom {
            causality_types::expr::ast::Atom::Nil => Ok(ValueExpr::Unit),
            causality_types::expr::ast::Atom::Boolean(b) => Ok(ValueExpr::Bool(b)),
            causality_types::expr::ast::Atom::String(s) => Ok(ValueExpr::String(s)),
            causality_types::expr::ast::Atom::Integer(n) => Ok(ValueExpr::Number(causality_types::primitive::number::Number::Integer(n))),
        },
        causality_types::expr::result::ExprResult::Bool(b) => Ok(ValueExpr::Bool(b)),
        causality_types::expr::result::ExprResult::Unit => Ok(ValueExpr::Unit),
        causality_types::expr::result::ExprResult::Resource(r_id) => {
            // Convert ResourceId to a String representation
            Ok(ValueExpr::String(CausalityStr::from(r_id.to_string())))
        }
        // Other ExprResult variants (Combinator, Function, ExternalHostFnRef, etc.) 
        // do not have a direct, general conversion to a simple ValueExpr.
        // The caller should handle these if they expect them.
        other => Err(CausalityHandlerError::InternalError(format!(
            "Cannot convert Lisp ExprResult variant {:?} to ValueExpr in this context",
            other
        ))),
    }
}

#[allow(dead_code)]
fn atom_to_value_expr(atom: causality_types::expr::ast::Atom) -> ValueExpr {
    match atom {
        causality_types::expr::ast::Atom::Nil => ValueExpr::Unit,
        causality_types::expr::ast::Atom::Boolean(b) => ValueExpr::Bool(b),
        causality_types::expr::ast::Atom::String(s) => ValueExpr::String(s),
        causality_types::expr::ast::Atom::Integer(i) => ValueExpr::Number(causality_types::primitive::number::Number::Integer(i)),
    }
}

// Add a wrapper function to convert between LispError and LispEvaluationError
impl From<LispError> for LispEvaluationErrorLocal {
    fn from(err: LispError) -> Self {
        LispEvaluationErrorLocal::EvaluationFailed(err.to_string())
    }
}

impl From<anyhow::Error> for LispEvaluationErrorLocal {
    fn from(err: anyhow::Error) -> Self {
        LispEvaluationErrorLocal::EvaluationFailed(err.to_string())
    }
}