//-----------------------------------------------------------------------------
// ZK Context Provider
//-----------------------------------------------------------------------------
//
// This module defines the interface for providing context values to dynamic

// expressions in the ZK guest environment.
extern crate alloc;
use alloc::{format, string::String, vec::Vec};
use log; // Added for log::trace!

// use causality_types::anyhow; // anyhow::Result is used by AsExprContext impl, not ZkContextProvider itself
use causality_types::primitive::ids::{ExprId, ResourceId};
use causality_types::serialization::{Encode, SimpleSerialize};
// Define our own resource_id_from_str function instead of using causality_core
fn resource_id_from_str(hex_str: &str) -> Result<ResourceId, String> {
    if hex_str.len() != 64 {
        return Err(format!("Invalid hex string length for ResourceId: {} (expected 64)", hex_str.len()));
    }
    
    let bytes = match hex::decode(hex_str) {
        Ok(b) => b,
        Err(e) => return Err(format!("Failed to decode hex: {}", e)),
    };
    
    if bytes.len() != 32 {
        return Err(format!("Decoded bytes length is not 32: {}", bytes.len()));
    }
    
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(ResourceId::from(arr))
}
use causality_types::primitive::string::Str;
use causality_types::expr::ast::{Expr, Atom};
use causality_types::expr::value::ValueExpr;
use causality_types::provider::context::AsExprContext;
use causality_lisp::core::ExprContextual;
use causality_types::expr::result::ExprError;
use causality_types::expr::result::ExprResult;
use causality_types::expr::result::TypeErrorData;

use crate::core::Error as ZkError; // Use this for ZkContextProvider
                                   // use crate::core::RuntimeEnvironment; // This was causing an error, remove if not used here
                                   // use crate::trace::tracer::Tracer; // This was causing an error, remove if not used here
use crate::witness::core::WitnessData as CoreWitnessData; // Aliased import

// For ZkEvalContext and its AsExprContext impl
use causality_types::anyhow::{
    self, Result,
};

use std::collections::BTreeMap; // Added for BTreeMap
// use tokio::runtime::Runtime; // This was causing an error, remove if not used here

use causality_types::trace::ExecutionTrace;

//-----------------------------------------------------------------------------
// Conversion Helpers
//-----------------------------------------------------------------------------

fn zk_error_to_lisp_error(err: ZkError) -> ExprError {
    ExprError::ExecutionError {
        message: Str::from(format!("ZK Context Error: {:?}", err)),
    }
}

// Converts causality_types::expr::value::ValueExpr to causality_types::expr::ast::Expr (aliased as Expr)
// This typically means wrapping the ValueExpr as a Literal (Atom) in the AST.
#[allow(dead_code)]
fn value_expr_to_ast_atom(val: &ValueExpr) -> Result<Atom, ExprError> {
    match val {
        ValueExpr::Unit => Ok(Atom::Nil),
        ValueExpr::Bool(b) => Ok(Atom::Boolean(*b)),
        ValueExpr::String(s) => Ok(Atom::String(*s)),
        ValueExpr::Number(n) => {
            match n {
                causality_types::primitive::number::Number::Integer(i) => Ok(Atom::Integer(*i)),
                _ => Err(ExprError::TypeError(Box::new(TypeErrorData {
                    message: Str::from(format!("Cannot convert non-integer Number {:?} to ast::Atom literal directly", n)),
                    expr: None,
                }))),
            }
        }
        _ => {
            Err(ExprError::TypeError(Box::new(TypeErrorData {
                message: Str::from(format!("Unsupported ValueExpr to ast::Expr::Atom conversion for: {:?}", val)),
                expr: None,
            })))
        }
    }
}

// Converts causality_types::expr::ast::Expr (aliased as Expr) to causality_types::expr::value::ValueExpr
// This typically only works if the ast::Expr is an Atom.
#[allow(dead_code)]
fn ast_expr_to_value_expr(expr: &Expr) -> Result<ValueExpr, ExprError> {
    match expr {
        Expr::Atom(Atom::Nil) => Ok(ValueExpr::Unit),
        Expr::Atom(Atom::Boolean(b)) => Ok(ValueExpr::Bool(*b)),
        Expr::Atom(Atom::String(s)) => Ok(ValueExpr::String(*s)), // ValueExpr::String takes Str
        Expr::Atom(Atom::Integer(i)) => Ok(ValueExpr::Number(causality_types::primitive::number::Number::new_integer(*i))),
        _ => Err(ExprError::TypeError(Box::new(TypeErrorData {
            message: Str::from(format!("Unsupported ast::Expr to ValueExpr conversion. Only Atoms can be converted: {:?}", expr)),
            expr: None,
        }))),
    }
}

// Converts causality_types::expr::result::ExprResult to causality_types::expr::ast::Expr (aliased as Expr)
#[allow(dead_code)]
fn expr_result_to_ast_expr(res: &ExprResult) -> Result<Expr, ExprError> {
    match res {
        ExprResult::Atom(atom) => Ok(Expr::Atom(atom.clone())),
        ExprResult::Value(val) => {
            let atom = value_expr_to_ast_atom(val)?;
            Ok(Expr::Atom(atom))
        }
        ExprResult::Bool(b) => Ok(Expr::Atom(Atom::Boolean(*b))),
        ExprResult::Unit => Ok(Expr::Atom(Atom::Nil)),
        _ => Err(ExprError::TypeError(Box::new(TypeErrorData {
            message: Str::from(format!(
                "Unsupported ExprResult to ast::Expr conversion for: {:?}",
                res
            )),
            expr: None,
        }))),
    }
}

//-----------------------------------------------------------------------------
// Context Provider Type
//-----------------------------------------------------------------------------

/// Interface for providing contextual values in the ZK environment
pub trait ZkContextProvider: Send + Sync {
    /// Get a named context value (e.g., current_timestamp)
    fn get_context_value(&self, name: &str) -> Result<ValueExpr, ZkError>;

    /// Check if a resource is available
    fn is_resource_available(&self, id: &ResourceId) -> Result<bool, ZkError>;

    /// Get a field from a resource
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<ValueExpr, ZkError>;

    /// Get a Lisp expression from the context provider
    fn get_lisp_expr(&self, id: &ExprId) -> Option<&Expr>;
}

/// ZK witness-based context provider
///
/// This implementation of ZkContextProvider sources values from witness data
/// that has been provided to the ZK circuit.
#[derive(Clone)] // Added Clone for ZkEvalContext if it needs to own this
#[allow(dead_code)]
pub struct WitnessContextProvider {
    /// Reference to the core witness data
    core_witness_data: CoreWitnessData,
    /// Store for Lisp expressions, derived from ExecutionTrace
    expr_store: BTreeMap<ExprId, Expr>, // Expr is TypesExpr
    // Store the deserialized trace to avoid re-deserializing for every ZkContextProvider method call.
    // This assumes ExecutionTrace is reasonably sized to keep in memory.
    deserialized_trace: ExecutionTrace,
}

/// Simplified witness data structure for the ZK environment
///
/// This is a minimal representation of witness data for use in the ZK guest.
/// It focuses on providing access to the key-value pairs and resources that
/// expressions might need during evaluation.
#[derive(Clone, Debug)]
pub struct WitnessData {
    /// Context values (timestamps, block heights, etc.)
    pub context_values: Vec<(String, ValueExpr)>,

    /// Resources available in this witness
    pub resources: Vec<ResourceData>,
}

impl SimpleSerialize for WitnessData {}

impl Encode for WitnessData {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.context_values.as_ssz_bytes());
        bytes.extend(self.resources.as_ssz_bytes());
        bytes
    }
}

/// Resource data within a witness
#[derive(Clone, Debug)]
pub struct ResourceData {
    /// Resource ID
    pub id: ResourceId,

    /// Resource fields
    pub fields: Vec<(String, ValueExpr)>,
}

impl SimpleSerialize for ResourceData {}

impl Encode for ResourceData {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.fields.as_ssz_bytes());
        bytes
    }
}

//-----------------------------------------------------------------------------
// Implementation
//-----------------------------------------------------------------------------

impl WitnessContextProvider {
    /// Create a new witness-based context provider
    pub fn new(core_witness_data: CoreWitnessData) -> Result<Self, ZkError> {
        // For now, create a default ExecutionTrace since we don't have SSZ implementation
        let execution_trace = ExecutionTrace::new();
        // TODO: Implement proper deserialization when ExecutionTrace has SSZ support

        let expr_store = execution_trace.expr_definitions.clone();

        Ok(Self {
            core_witness_data,
            expr_store,
            deserialized_trace: execution_trace,
        })
    }

    // Method to access the Lisp expression store, used by ZkEvalContext
    pub fn get_expr_from_store(&self, id: &ExprId) -> Option<&Expr> {
        self.expr_store.get(id)
    }
}

impl ZkContextProvider for WitnessContextProvider {
    fn get_context_value(&self, name: &str) -> Result<ValueExpr, ZkError> {
        self.deserialized_trace
            .context_values
            .get(name)
            .cloned()
            .ok_or_else(|| {
                ZkError::InvalidInput(format!(
                    "Context value '{}' not found in trace",
                    name
                ))
            })
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool, ZkError> {
        match self.deserialized_trace.final_resource_states.get(id) {
            Some(state) => {
                Ok(*state == causality_types::state::ResourceState::Available)
            }
            None => Ok(false), // If not in final states, assume not available or consumed
        }
    }

    fn get_resource_field(
        &self,
        id: &ResourceId,
        field_name_str: &str,
    ) -> Result<ValueExpr, ZkError> {
        match self.deserialized_trace.resource_details.get(id) {
            Some(resource) => {
                // Accessing fields from a Resource struct. Resource struct has specific fields like
                // id, domain, ephemeral, value (ValueExprId), type_expr (TypeExprId), static_expr (Option<ExprId>).
                // It does not have arbitrary string-keyed fields.
                // If field_name_str refers to one of these, we need specific logic.
                // If it refers to a field *within* the ValueExpr pointed to by resource.value, that's different.
                // Let's assume for now it tries to match one of the fixed field names of the Resource struct.
                // This is a simplification.
                match field_name_str {
                    "id" => Ok(ValueExpr::String(resource.id.to_string().into())),
                    "domain_id" => {
                        Ok(ValueExpr::String(resource.domain_id.to_string().into()))
                    }
                    "name" => Ok(ValueExpr::String(resource.name)),
                    "resource_type" => Ok(ValueExpr::String(resource.resource_type)),
                    "quantity" => Ok(ValueExpr::Number(causality_types::primitive::number::Number::Integer(resource.quantity as i64))),
                    "timestamp" => {
                        Ok(ValueExpr::Number(causality_types::primitive::number::Number::Integer(resource.timestamp.as_millis() as i64)))
                    }
                    // If we want to access fields of the resource's *value* (which is a ValueExprId referring to a ValueExpr)
                    // we'd need to fetch that ValueExpr from the state_manager (if stored there) or have it in trace.
                    // This ZkContextProvider doesn't have direct access to the state_manager to resolve ValueExprId.
                    // This indicates a gap or that "get_resource_field" has a different expectation.
                    _ => {
                        Err(ZkError::FieldNotFound(field_name_str.to_string(), *id))
                    }
                }
            }
            None => Err(ZkError::ResourceNotFound(*id)),
        }
    }

    fn get_lisp_expr(&self, id: &ExprId) -> Option<&Expr> {
        self.expr_store.get(id)
    }
}

/// Implementation of EvalContext that adapts a ZkContextProvider
///
/// This adapter implements the EvalContext trait required by the Lisp
/// interpreter, sourcing its values from a ZkContextProvider.
pub struct ZkEvalContext<'a> {
    /// Provider for context values
    provider: &'a WitnessContextProvider,
}

impl<'a> ZkEvalContext<'a> {
    /// Create a new ZK eval context with the given provider
    pub fn new(provider: &'a WitnessContextProvider) -> Self {
        Self { provider }
    }

    // Direct method to get TypeExpr, used by wasm.rs before calling interpreter
    pub fn get_actual_expr(&self, id: &ExprId) -> Option<&Expr> {
        self.provider.get_lisp_expr(id) // Use the method from ZkContextProvider trait
    }
}

// Downcasting not needed in ZK environment
impl<'a> causality_types::provider::context::AsExprContext for ZkEvalContext<'a> {
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<Option<ValueExpr>> {
        log::trace!(
            "ZkEvalContext::get_resource_field id: {:?}, field: {}",
            id,
            field
        );
        self.provider
            .get_resource_field(id, field)
            .map(Some) // Ensure it returns Option<ValueExpr>
            .map_err(|e| anyhow::anyhow!("ZkContextProvider error in get_resource_field: {:?}", e))
    }

    fn evaluate_expr(&self, _expr: &Expr) -> Result<ValueExpr> {
        // In a real implementation, this would evaluate the expression
        // For now, return a default value
        Ok(ValueExpr::Nil)
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        log::trace!("ZkEvalContext::is_resource_available id: {:?}", id);
        self.provider.is_resource_available(id).map_err(|e| {
            anyhow::anyhow!(
                "ZkContextProvider error in is_resource_available: {:?}",
                e
            )
        })
    }
}

impl<'a> causality_types::provider::context::StaticExprContext
    for ZkEvalContext<'a>
{
    fn get_static_symbol(&self, _name: &Str) -> Option<ExprResult> {
        None
    }

    fn get_expr(&self, id: &ExprId) -> Option<&Expr> {
        self.provider.get_lisp_expr(id) // Use the method from ZkContextProvider trait
    }
}

/// A struct that owns both the context provider and the evaluation context
pub struct OwnedZkEvalContext {
    #[allow(dead_code)]
    provider: WitnessContextProvider,
    ctx: ZkEvalContext<'static>, // Use 'static as we'll convert the reference
}

impl OwnedZkEvalContext {
    /// Create a new owned ZK eval context from core witness data
    pub fn new(core_witness_data: CoreWitnessData) -> Result<Self, ZkError> {
        let provider_instance = WitnessContextProvider::new(core_witness_data)?;
        // Create a 'static reference to a WitnessContextProvider for ZkEvalContext<'static>.
        // This requires leaking memory, which is acceptable in some ZK guest contexts
        // where the execution is single-shot and memory is reclaimed afterwards.
        // OwnedZkEvalContext will also hold its own copy of provider_instance.
        let static_provider_ref: &'static WitnessContextProvider =
            Box::leak(Box::new(provider_instance.clone()));
        let ctx = ZkEvalContext::new(static_provider_ref);
        Ok(Self {
            provider: provider_instance,
            ctx,
        })
    }

    // Helper to get a mutable reference if needed, though ExprContextual mostly takes &self
}

/// Implement AsExprContext for OwnedZkEvalContext
impl AsExprContext for OwnedZkEvalContext {
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<Option<ValueExpr>> {
        log::trace!(
            "OwnedZkEvalContext::get_resource_field id: {:?}, field: {}",
            id,
            field
        );
        self.ctx.get_resource_field(id, field) //.map_err(|e| anyhow::anyhow!(e))
    }

    fn evaluate_expr(&self, expr: &Expr) -> Result<ValueExpr> {
        log::trace!("OwnedZkEvalContext::evaluate_expr expr: {:?}", expr);
        self.ctx.evaluate_expr(expr) //.map_err(|e| anyhow::anyhow!(e))
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        log::trace!("OwnedZkEvalContext::is_resource_available id: {:?}", id);
        self.ctx.is_resource_available(id) //.map_err(|e| anyhow::anyhow!(e))
    }
}

/// Forward ExprContextual implementation to the contained context
#[async_trait::async_trait]
impl ExprContextual for OwnedZkEvalContext {
    async fn get_symbol(&self, name: &Str) -> Option<ExprResult> {
        self.ctx.get_symbol(name).await
    }

    async fn try_call_host_function(
        &self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, ExprError>> {
        self.ctx.try_call_host_function(fn_name, args).await
    }
    
    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError> {
        self.ctx.get_expr_by_id(id).await
    }
    
    async fn is_effect_completed(&self, effect_id: &ExprId) -> Result<bool, ExprError> {
        self.ctx.is_effect_completed(effect_id).await
    }
    
    async fn define_symbol(&self, name: Str, value: ExprResult) -> Result<(), ExprError> {
        self.ctx.define_symbol(name, value).await
    }
}

impl causality_types::provider::context::StaticExprContext for OwnedZkEvalContext {
    fn get_static_symbol(&self, name: &Str) -> Option<ExprResult> {
        self.ctx.get_static_symbol(name)
    }

    fn get_expr(&self, id: &ExprId) -> Option<&Expr> {
        self.ctx.get_expr(id)
    }
}

/// Helper function to create a witness-based context for ZK evaluation
pub fn create_witness_context(
    core_witness_data: CoreWitnessData,
) -> Result<OwnedZkEvalContext, ZkError> {
    OwnedZkEvalContext::new(core_witness_data)
}

/// Helper to create witness data from raw components
pub fn create_witness_data(
    context_values: Vec<(String, ValueExpr)>,
    resources: Vec<ResourceData>,
) -> WitnessData {
    WitnessData {
        context_values,
        resources,
    }
}

/// Helper to create resource data
pub fn create_resource_data(
    id: ResourceId,
    fields: Vec<(String, ValueExpr)>,
) -> ResourceData {
    ResourceData { id, fields }
}

// Implement constructors and accessors for WitnessData
impl Default for WitnessData {
    fn default() -> Self {
        Self::new()
    }
}

impl WitnessData {
    /// Create a new, empty witness data structure
    pub fn new() -> Self {
        Self {
            context_values: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Add a context value
    pub fn add_context_value(&mut self, name: String, value: ValueExpr) {
        self.context_values.push((name, value));
    }

    /// Add a resource
    pub fn add_resource(&mut self, resource: ResourceData) {
        self.resources.push(resource);
    }
}

// Implement constructors and accessors for ResourceData
impl ResourceData {
    /// Create a new resource data structure
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            fields: Vec::new(),
        }
    }

    /// Add a field
    pub fn add_field(&mut self, name: String, value: ValueExpr) {
        self.fields.push((name, value));
    }
}

// This is the ExprContextual impl to KEEP
#[async_trait::async_trait]
impl<'a> causality_lisp::core::ExprContextual for ZkEvalContext<'a> {
    async fn get_symbol(&self, name: &Str) -> Option<ExprResult> {
        match self.provider.get_context_value(&name.as_string()) {
            Ok(value_expr) => Some(ExprResult::Value(value_expr)),
            Err(_) => None,
        }
    }
    
    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError> {
        self.provider.get_lisp_expr(id).ok_or(ExprError::ExecutionError {
            message: format!("Expression with ID '{}' not found", id).into(),
        })
    }
    
    async fn is_effect_completed(&self, _effect_id: &ExprId) -> Result<bool, ExprError> {
        // In ZK context, we don't have effects, so return completed
        Ok(true)
    }
    
    async fn define_symbol(&self, _name: Str, _value: ExprResult) -> Result<(), ExprError> {
        // ZK context is immutable, can't define new symbols at runtime
        Err(ExprError::ExecutionError {
            message: "Cannot define symbols in ZK context".into(),
        })
    }

    async fn try_call_host_function(
        &self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, ExprError>> {
        let fn_name_str_val = fn_name.to_string();
        match fn_name_str_val.as_str() {
            "get-context-value" => {
                if args.len() != 1 {
                    return Some(Err(ExprError::ExecutionError {
                        message: "get-context-value expects 1 argument".into(),
                    }));
                }
                match &args[0] {
                    ValueExpr::String(key_s) => Some(
                        self.provider.get_context_value(&key_s.as_string()).map_err(
                            |e| ExprError::ExecutionError {
                                message: format!(
                                    "Host fn get-context-value failed: {:?}",
                                    e
                                )
                                .into(),
                            },
                        ),
                    ),
                    _ => Some(Err(ExprError::TypeError(Box::new(TypeErrorData {
                        message: "get-context-value argument must be a string".into(),
                        expr: None,
                    })))),
                }
            }
            "is-resource-available" => {
                if args.len() != 1 {
                    return Some(Err(ExprError::ExecutionError {
                        message: "is-resource-available expects 1 argument".into(),
                    }));
                }
                match &args[0] {
                    ValueExpr::String(ref id_s) => {
                        match resource_id_from_str(id_s.as_string().as_str()) { 
                            Ok(res_id) => Some(self.provider.is_resource_available(&res_id)
                                .map(ValueExpr::Bool)
                                .map_err(zk_error_to_lisp_error)),
                            Err(e) => Some(Err(ExprError::ExecutionError { 
                                message: format!("Invalid ResourceId string: {}", e).into() 
                            })),
                        }
                    }
                    _ => Some(Err(ExprError::TypeError(Box::new(TypeErrorData { 
                        message: "is-resource-available argument must be a ResourceId string".into(), 
                        expr: None 
                    })))),
                }
            }
            "get-resource-field" => {
                if args.len() != 2 {
                    return Some(Err(ExprError::ExecutionError {
                        message: "get-resource-field expects 2 arguments".into(),
                    }));
                }
                let res_id = match &args[0] {
                    ValueExpr::String(s) => {
                        match resource_id_from_str(s.as_string().as_str()) {
                            Ok(id) => id,
                            Err(e) => {
                                return Some(Err(ExprError::ExecutionError {
                                    message: format!("Failed to parse resource ID from string '{}': {}", s, e).into(),
                                }));
                            }
                        }
                    }
                    _ => return Some(Err(ExprError::TypeError(Box::new(TypeErrorData {
                        message: "get-resource-field arg0 (resource_id) must be a string".into(),
                        expr: None,
                    })))),
                };
                let field_name_val_str = match &args[1] {
                    ValueExpr::String(s) => s.as_string(),
                    _ => return Some(Err(ExprError::TypeError(Box::new(TypeErrorData {
                        message: "get-resource-field arg1 (field_name) must be a string".into(),
                        expr: None,
                    })))),
                };
                Some(
                    self.provider
                        .get_resource_field(&res_id, field_name_val_str.as_str())
                        .map_err(|e| ExprError::ExecutionError {
                            message: format!(
                                "Host fn get-resource-field failed: {:?}",
                                e
                            )
                            .into(),
                        }),
                )
            }
            _ => None,
        }
    }
}
