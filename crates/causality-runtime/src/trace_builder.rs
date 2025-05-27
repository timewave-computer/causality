// crates/causality-runtime/src/trace_builder.rs
// Purpose: Provides functionality to build ExecutionTrace objects.

use std::collections::{BTreeMap, HashSet};
use anyhow::Result;

use causality_types::{
    core::{
        id::{ResourceId, EffectId},
        resource_conversion::ToValueExpr,
    },
    expr::value::ValueExpr,
    trace::{EffectDetail, ExecutionTrace},
    state::ResourceState,
};
use crate::state_manager::StateManager;

/// Input required to build an ExecutionTrace.
pub struct TraceBuilderInput<'a> {
    pub state_manager: &'a dyn StateManager,
    pub executed_effects_ids: Vec<EffectId>, // Ordered list of executed effect IDs
    pub effect_details_map: BTreeMap<EffectId, EffectDetail>, // Details for each executed effect
    pub final_state_resource_ids: HashSet<ResourceId>, // IDs of resources for which final state is required
    pub context_values_map: BTreeMap<String, ValueExpr>, // Context values for the trace
    // Relevant resource IDs for full details can be derived from final_state_resource_ids and effect_details_map.
    // Relevant expr IDs for definitions can be derived from effect_details_map and fetched Resource.static_expr.
}

/// Builds an ExecutionTrace based on the provided input and state manager.
pub async fn build_execution_trace(input: TraceBuilderInput<'_>) -> Result<ExecutionTrace> {
    let mut final_resource_states = BTreeMap::new();
    let mut resource_details_map = BTreeMap::new();
    let mut expr_definitions_map = BTreeMap::new();
    let mut all_relevant_resource_ids = input.final_state_resource_ids.clone();
    let mut all_relevant_expr_ids = HashSet::new();

    // Populate final_resource_states and collect resource IDs from final states
    for res_id in &input.final_state_resource_ids {
        // StateManager needs a method like get_resource_state_sync or similar.
        // For now, assume we fetch the full resource and derive its state if not nullified.
        // Or, ExecutionTrace.final_resource_states is populated by the caller based on their knowledge.
        // Let's assume for now StateManager can give ResourceState directly.
        // This part might need StateManager to evolve or the caller to provide final_resource_states directly.
        // Defaulting to Available if present, assuming StateManager tracks active resources.
        if let Some(resource) = input.state_manager.get_resource_sync(res_id)? {
            // Determine state: If resource exists and not nullified, it's Available.
            // This is a simplification; actual state might be more complex (Locked, Consumed via nullifier).
            // ExecutionTrace expects ResourceState enum.
            if !input.state_manager.is_nullified(res_id).await? {
                 final_resource_states.insert(*res_id, ResourceState::Available);
            } else {
                 final_resource_states.insert(*res_id, ResourceState::Consumed); // Or other appropriate state
            }
            resource_details_map.insert(*res_id, resource.clone());
            // For now, we'll skip static validation since the Resource struct no longer has static_expr
            // In a full implementation, static validation would be handled differently
            
            // Convert resource to ValueExpr for evaluation if needed
            let _resource_value = resource.to_value_expr();
        } else {
            // If a resource ID is in final_state_resource_ids but not found, it implies an issue
            // or it means it was consumed and removed. For now, let's assume it implies Consumed.
            final_resource_states.insert(*res_id, ResourceState::Consumed);
        }
    }

    // Collect resource and expression IDs from effect details
    for detail in input.effect_details_map.values() {
        for res_id in &detail.inputs {
            all_relevant_resource_ids.insert(*res_id);
        }
        for res_id in &detail.outputs {
            all_relevant_resource_ids.insert(*res_id);
        }
        for expr_id in &detail.constraints {
            all_relevant_expr_ids.insert(*expr_id);
        }
    }

    // Populate resource_details for all relevant resources
    for res_id in &all_relevant_resource_ids {
        if !resource_details_map.contains_key(res_id) {
            if let Some(resource) = input.state_manager.get_resource(res_id).await? {
                resource_details_map.insert(*res_id, resource.clone());
                // For now, we'll skip static validation since the Resource struct no longer has static_expr
                // In a full implementation, static validation would be handled differently
                
                // Convert resource to ValueExpr for evaluation if needed
                let _resource_value = resource.to_value_expr();
            }
            // If resource not found, it might have been ephemeral or consumed.
            // Not adding to resource_details_map if not found.
        }
    }
    
    // Populate expr_definitions for all relevant expressions
    for expr_id in &all_relevant_expr_ids {
        if let Some(expr) = input.state_manager.get_expr_sync(expr_id)? {
            expr_definitions_map.insert(*expr_id, expr);
        } else {
            // If expr not found, it's an issue, trace might be incomplete.
            // Return an error.
            return Err(anyhow::anyhow!(
                "Required expression with ID {:?} not found in state manager.",
                expr_id
            ));
        }
    }

    Ok(ExecutionTrace {
        executed_effects: input.executed_effects_ids,
        final_resource_states, // This should be populated accurately by the caller or SM
        effect_details: input.effect_details_map,
        expr_definitions: expr_definitions_map,
        context_values: input.context_values_map,
        resource_details: resource_details_map,
    })
} 