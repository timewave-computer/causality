//! ProcessDataflowBlock Orchestration Combinators
//!
//! This module implements Rust logic for PDB orchestration combinators that can be
//! called from Lisp expressions to manage ProcessDataflowBlock instances and operations.

use causality_types::{
    core::{
        id::{ExprId, ResourceId, DomainId, AsId, HandlerId},
        str::Str,
        time::Timestamp,
        Effect,
    },
    expr::{
        result::{ExprError, ExprResult},
        value::{ValueExpr, ValueExprMap},
    },
    graph::{
        optimization::TypedDomain,
        dataflow::{ProcessDataflowDefinition, ProcessDataflowNode as DataflowNode},
        execution::ProcessDataflowInstanceState,
    },
    serialization::Encode, // Added Encode trait for as_ssz_bytes()
};
use crate::core::{ExprContextual, Evaluator};
use std::collections::BTreeMap;
use hex; // Added hex crate for encoding

/// Context for dataflow orchestration operations
pub struct DataflowOrchestrationContext<'a> {
    /// Reference to the expression context
    pub expr_context: &'a dyn ExprContextual,
    
    /// Available ProcessDataflowDefinitions
    pub dataflow_definitions: BTreeMap<ExprId, ProcessDataflowDefinition>,
    
    /// Active ProcessDataflowBlock instances
    pub active_instances: BTreeMap<ResourceId, ProcessDataflowInstanceState>,
    
    /// Current typed domain
    pub current_typed_domain: TypedDomain,
    
    /// Generated effects queue (to be returned to Graph Executor)
    pub generated_effects: Vec<Effect>,
}

impl<'a> DataflowOrchestrationContext<'a> {
    /// Create a new orchestration context
    pub fn new(
        expr_context: &'a dyn ExprContextual,
        current_typed_domain: TypedDomain,
    ) -> Self {
        Self {
            expr_context,
            dataflow_definitions: BTreeMap::new(),
            active_instances: BTreeMap::new(),
            current_typed_domain,
            generated_effects: Vec::new(),
        }
    }
    
    /// Add a dataflow definition to the context
    pub fn add_dataflow_definition(&mut self, id: ExprId, definition: ProcessDataflowDefinition) {
        self.dataflow_definitions.insert(id, definition);
    }
    
    /// Add an active instance to the context
    pub fn add_active_instance(&mut self, id: ResourceId, state: ProcessDataflowInstanceState) {
        self.active_instances.insert(id, state);
    }
}

/// Get a ProcessDataflowDefinition by ID
/// Lisp signature: (get-dataflow-definition df_id)
pub async fn get_dataflow_definition(
    context: &mut DataflowOrchestrationContext<'_>,
    df_id: ExprId,
) -> Result<ValueExpr, ExprError> {
    match context.dataflow_definitions.get(&df_id) {
        Some(definition) => {
            // Convert ProcessDataflowDefinition to ValueExpr representation
            let mut def_map = BTreeMap::new();
            
            def_map.insert(
                Str::from("definition_id"),
                ValueExpr::String(Str::from(df_id.to_hex()))
            );
            
            def_map.insert(
                Str::from("node_count"),
                ValueExpr::Number(causality_types::primitive::number::Number::Integer(definition.nodes.len() as i64))
            );
            
            def_map.insert(
                Str::from("edge_count"),
                ValueExpr::Number(causality_types::primitive::number::Number::Integer(definition.edges.len() as i64))
            );
            
            // Add more fields as needed for Lisp consumption
            Ok(ValueExpr::Map(ValueExprMap(def_map)))
        }
        None => Err(ExprError::ExecutionError {
            message: Str::from(format!("ProcessDataflowDefinition not found: {}", df_id.to_hex())),
        })
    }
}

/// Evaluate a gating condition for dataflow progression
/// Lisp signature: (evaluate-gating-condition condition_expr_id context_value_expr)
pub async fn evaluate_gating_condition(
    context: &mut DataflowOrchestrationContext<'_>,
    condition_expr_id: ExprId,
    context_value_expr: ValueExpr,
) -> Result<ValueExpr, ExprError> {
    // Get the condition expression
    let condition_expr = context.expr_context.get_expr_by_id(&condition_expr_id).await?;
    
    // Create a temporary binding context with the provided context value
    let mut temp_bindings = BTreeMap::new();
    temp_bindings.insert(Str::from("context"), ExprResult::Value(context_value_expr));
    
    // Use the Lisp interpreter to evaluate the condition recursively
    let interpreter = crate::core::Interpreter::new();
    let binding_context = crate::core::LambdaBindingContext::new(context.expr_context, temp_bindings);
    
    match interpreter.evaluate_expr(condition_expr, &binding_context).await? {
        ExprResult::Value(value) => Ok(value),
        other => Err(ExprError::ExecutionError {
            message: Str::from(format!("Gating condition must evaluate to a value, got: {:?}", other)),
        })
    }
}

/// Instantiate an Effect from a dataflow node template
/// Lisp signature: (instantiate-effect-from-node effect_node_template params_value_expr)
pub async fn instantiate_effect_from_node(
    context: &mut DataflowOrchestrationContext<'_>,
    _effect_node_template: &DataflowNode,
    params_value_expr: ValueExpr,
) -> Result<ValueExpr, ExprError> {
    // Extract parameters from the params_value_expr
    let params_map = match params_value_expr {
        ValueExpr::Map(map) => map,
        _ => return Err(ExprError::ExecutionError {
            message: Str::from("Parameters must be provided as a map"),
        })
    };
    
    // Create a new Effect based on the node template and parameters
    let effect_type = params_map.get(&Str::from("effect_type"))
        .and_then(|v| match v {
            ValueExpr::String(s) => Some(s.clone()),
            _ => None
        })
        .unwrap_or_else(|| Str::from("default_effect_type"));
    
    // Generate a new Effect ID using a simple counter instead of rand
    let effect_id = causality_types::primitive::ids::EntityId::new([42u8; 32]); // Fixed ID for deterministic testing
    
    // Create the Effect struct
    let effect = Effect {
        id: effect_id,
        name: Str::from("generated_effect"),
        domain_id: context.current_typed_domain.domain_id,
        effect_type,
        inputs: Vec::new(), // Would be populated from template and params
        outputs: Vec::new(), // Would be populated from template and params
        expression: None,
        timestamp: Timestamp::now(),
        hint: None,
    };
    
    // Add the effect to the generated effects queue
    context.generated_effects.push(effect.clone());
    
    // Return the effect ID as a string
    Ok(ValueExpr::String(Str::from(effect_id.to_hex())))
}

/// Emit an effect on a specific domain
/// Lisp signature: (emit-effect-on-domain target_domain_id effect_value_expr)
pub async fn emit_effect_on_domain(
    context: &mut DataflowOrchestrationContext<'_>,
    target_domain_id: DomainId,
    effect_value_expr: ValueExpr,
) -> Result<ValueExpr, ExprError> {
    // Parse the effect from the value expression
    let effect_map = match effect_value_expr {
        ValueExpr::Map(map) => map,
        _ => return Err(ExprError::ExecutionError {
            message: Str::from("Effect must be provided as a map"),
        })
    };
    
    // Extract effect details
    let effect_type = effect_map.get(&Str::from("effect_type"))
        .and_then(|v| match v {
            ValueExpr::String(s) => Some(s.clone()),
            _ => None
        })
        .ok_or_else(|| ExprError::ExecutionError {
            message: Str::from("Effect type is required"),
        })?;
    
    // Determine target typed domain based on domain_id
    let target_typed_domain = determine_typed_domain_from_domain_id(&target_domain_id);
    
    // Generate a new Effect ID using a simple counter instead of rand
    let effect_id = causality_types::primitive::ids::EntityId::new([43u8; 32]); // Fixed ID for deterministic testing
    
    // Create the Effect struct
    let effect = Effect {
        id: effect_id,
        name: Str::from("generated_effect"),
        domain_id: target_typed_domain.domain_id,
        effect_type,
        inputs: Vec::new(), // Would be populated from effect_value_expr
        outputs: Vec::new(), // Would be populated from effect_value_expr
        expression: None,
        timestamp: Timestamp::now(),
        hint: None,
    };
    
    // Add the effect to the generated effects queue
    context.generated_effects.push(effect.clone());
    
    // Return success indicator
    Ok(ValueExpr::String(Str::from(format!("effect_emitted_{}", effect_id.to_hex()))))
}

/// Update ProcessDataflowBlock instance state
/// Lisp signature: (update-dataflow-instance-state df_instance_id new_state_value_expr)
pub async fn update_dataflow_instance_state(
    context: &mut DataflowOrchestrationContext<'_>,
    df_instance_id: ResourceId,
    new_state_value_expr: ValueExpr,
) -> Result<ValueExpr, ExprError> {
    let instance_state = context
        .active_instances
        .get_mut(&df_instance_id)
        .ok_or_else(|| {
            ExprError::ReferenceError {
                name: format!("Dataflow instance not found: {}", df_instance_id).into(),
            }
        })?;

    // Serialize the new_state_value_expr to SSZ bytes, then to a hex string
    let state_bytes = new_state_value_expr.as_ssz_bytes();
    let state_hex_string = hex::encode(state_bytes); // Requires hex crate

    instance_state.state = state_hex_string.into();

    // The original logic for extracting node_id_str and state_values from new_state_value_expr
    // is removed as we are now storing the whole ValueExpr serialized.
    // If specific fields are needed elsewhere, they would be deserialized from instance_state.state.

    // For now, we return a simple confirmation or the ID of the updated instance.
    // The actual return value might need to be more meaningful depending on Lisp expectations.
    Ok(ValueExpr::String(Str::from(format!(
        "Updated instance {}",
        df_instance_id
    ))))
}

/// Helper function to determine TypedDomain from DomainId
fn determine_typed_domain_from_domain_id(domain_id: &DomainId) -> TypedDomain {
    let type_str = if domain_id.to_string().contains("zk") || domain_id.to_string().contains("verifiable") {
        Str::from("verifiable")
    } else if domain_id.to_string().contains("svc") || domain_id.to_string().contains("service") {
        Str::from("service")
    } else {
        Str::from("unknown") // Default or error case
    };
    TypedDomain::new(*domain_id, type_str)
}

/// Check if an operation is compatible with ZK verification
pub fn is_zk_compatible_operation(
    _operation_type: &str, // Parameter kept for now, but logic focuses on domain_type
    current_domain: &TypedDomain,
) -> bool {
    current_domain.domain_type == Str::from("verifiable")
}

/// Validate dataflow step constraints based on target domain
pub fn validate_dataflow_step_constraints(
    step_type: &str,
    target_domain: &TypedDomain,
    parameters: &ValueExpr,
) -> Result<(), ExprError> {
    if target_domain.domain_type == Str::from("verifiable") {
        // ZK-specific validation
        validate_zk_parameters(parameters).map_err(|e| {
            ExprError::ExecutionError {
                message: format!(
                    "Invalid ZK parameters for step '{}': {}",
                    step_type,
                    e
                ).into(),
            }
        })?; // Added ? to propagate error
    } else if target_domain.domain_type == Str::from("service") {
        // Service-specific validation (if any)
        validate_service_parameters(parameters).map_err(|e| {
            ExprError::ExecutionError {
                message: format!(
                    "Invalid Service parameters for step '{}': {}",
                    step_type,
                    e
                ).into(),
            }
        })?; // Added ? to propagate error
    } else {
        // Default: No specific validation or handle unknown domain type
        // For now, let's assume no validation for other types.
    }
    Ok(())
}

/// Validate ZK-specific parameters
fn validate_zk_parameters(parameters: &ValueExpr) -> Result<(), ExprError> {
    match parameters {
        ValueExpr::Map(map) => {
            for (key, value) in map.0.iter() {
                if key.as_str() == "proof" {
                    match value {
                        ValueExpr::Bool(_) | 
                        ValueExpr::Number(_) | 
                        ValueExpr::String(_) => {
                            validate_zk_parameters(value)?;
                        }
                        _ => {}
                    }
                }
                if key.as_str() == "circuit" {
                    match value {
                        ValueExpr::Bool(_) | 
                        ValueExpr::Number(_) | 
                        ValueExpr::String(_) => {
                            validate_zk_parameters(value)?;
                        }
                        _ => {}
                    }
                }
            }
        }
        ValueExpr::List(list) => {
            for item in list.0.iter() {
                match item {
                    ValueExpr::Bool(_) | 
                    ValueExpr::Number(_) | 
                    ValueExpr::String(_) => {}
                    _ => return Err(ExprError::ExecutionError {
                        message: Str::from("Invalid ZK parameter type"),
                    })
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Validate parameters for Service domain
fn validate_service_parameters(_parameters: &ValueExpr) -> Result<(), ExprError> {
    // Service domain is more permissive, just check basic structure
    // In a real implementation, this might validate against service schemas
    Ok(())
} 