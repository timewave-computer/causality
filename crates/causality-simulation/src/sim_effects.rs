//! Simulation Effects
//!
//! Defines constants and helpers for simulation-specific TEL effects,
//! including breakpoints and control mechanisms.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use causality_types::{
    core::{
        id::{EntityId, DomainId, IntentId, NodeId, ExprId, HandlerId, AsId},
        str::Str,
        time::Timestamp,
        resource::ResourceFlow,
        Effect,
    },
    tel::optimization::TypedDomain,
    ValueExpr,
    expr::value::ValueExprMap,
};
use rand;
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Simulation Breakpoint Effect
//-----------------------------------------------------------------------------

/// Type identifier for a simulation breakpoint effect.
/// When a TEL Effect with this `effect_type` is encountered, and a specific handler
/// is registered in the SimulationEngine, it will trigger a breakpoint.
pub const SIM_BREAKPOINT_EFFECT_TYPE: &str = "simulation_breakpoint_v1";

/// Resource type for markers created in the TelContext when a breakpoint is hit.
/// The SimulationEngine will look for resources of this type.
pub const SIM_BREAKPOINT_MARKER_RESOURCE_TYPE: &str =
    "simulation_breakpoint_marker_v1";

/// Key for the 'label' field in a breakpoint effect's payload and marker resource.
pub const BREAKPOINT_PAYLOAD_LABEL_KEY: &str = "label";
/// Key for the 'id' field in a breakpoint effect's payload and marker resource.
pub const BREAKPOINT_PAYLOAD_ID_KEY: &str = "id";

/// Creates the payload for a simulation breakpoint effect.
/// Returns a ValueExpr::Map directly.
pub fn create_breakpoint_effect_payload(label: Str, id: Str) -> ValueExpr {
    let mut payload_map = BTreeMap::new();
    payload_map.insert(
        Str::from(BREAKPOINT_PAYLOAD_LABEL_KEY),
        ValueExpr::String(label),
    );
    payload_map.insert(Str::from(BREAKPOINT_PAYLOAD_ID_KEY), ValueExpr::String(id));
    ValueExpr::Map(ValueExprMap(payload_map))
}

/// Creates a simulation breakpoint effect.
/// This effect, when processed, will trigger a breakpoint in the simulation engine.
pub fn create_breakpoint_effect(
    node_id: NodeId,
    domain_id: DomainId,
    intent_id: IntentId,
    _label: String,
    _id: String,
) -> Effect {
    Effect {
        id: EntityId::new(node_id.inner()),
        name: Str::from("simulation_breakpoint"),
        domain_id,
        effect_type: Str::from(SIM_BREAKPOINT_EFFECT_TYPE),
        inputs: vec![],
        outputs: vec![],
        expression: None,
        timestamp: Timestamp::now(),
        resources: vec![],
        nullifiers: vec![],
        scoped_by: HandlerId::null(),
        intent_id: Some(ExprId::new(intent_id.inner())),
        source_typed_domain: TypedDomain::default(),
        target_typed_domain: TypedDomain::default(),
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    }
}

//-----------------------------------------------------------------------------
// Simulation Control Effect
//-----------------------------------------------------------------------------

/// Type identifier for a simulation control effect.
/// This effect type is used for simulation control operations like stepping, pausing, etc.
pub const SIM_CONTROL_EFFECT_TYPE: &str = "simulation_control_v1";

/// Creates a simulation control effect.
/// This effect is used for controlling simulation execution.
pub fn create_control_effect(
    node_id: NodeId,
    domain_id: DomainId,
    intent_id: IntentId,
    _action: String,
    _params: Option<ValueExpr>,
) -> Effect {
    Effect {
        id: EntityId::new(node_id.inner()),
        name: Str::from("simulation_control"),
        domain_id,
        effect_type: Str::from(SIM_CONTROL_EFFECT_TYPE),
        inputs: vec![],
        outputs: vec![],
        expression: None,
        timestamp: Timestamp::now(),
        resources: vec![],
        nullifiers: vec![],
        scoped_by: HandlerId::null(),
        intent_id: Some(ExprId::new(intent_id.inner())),
        source_typed_domain: TypedDomain::default(),
        target_typed_domain: TypedDomain::default(),
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    }
}

//-----------------------------------------------------------------------------
// Resource Creation Effect
//-----------------------------------------------------------------------------

/// Type identifier for a resource creation effect.
pub const RESOURCE_CREATE_EFFECT_TYPE: &str = "resource_create_v1";

/// Creates a resource creation effect.
/// This effect creates a new resource in the simulation.
pub fn create_resource_creation_effect(
    node_id: NodeId,
    domain_id: DomainId,
    intent_id: IntentId,
    resource_type: String,
    _initial_data: ValueExpr,
) -> Effect {
    Effect {
        id: EntityId::new(node_id.inner()),
        name: Str::from("resource_creation"),
        domain_id,
        effect_type: Str::from(RESOURCE_CREATE_EFFECT_TYPE),
        inputs: vec![],
        outputs: vec![
            ResourceFlow::new(Str::from(resource_type), 1, domain_id)
        ],
        expression: None,
        timestamp: Timestamp::now(),
        resources: vec![],
        nullifiers: vec![],
        scoped_by: HandlerId::null(),
        intent_id: Some(ExprId::new(intent_id.inner())),
        source_typed_domain: TypedDomain::default(),
        target_typed_domain: TypedDomain::default(),
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    }
}

//-----------------------------------------------------------------------------
// Payload Creation Helpers for Simulation Control
//-----------------------------------------------------------------------------

// Payload keys for SIM_CONTROL_EFFECT_TYPE
/// Key for the 'action' field in a control effect's payload.
/// Value example: "query_resource", "query_context_value".
pub const SIM_CONTROL_ACTION_KEY: &str = "action";
/// Key for action-specific parameters (e.g., resource ID for "query_resource").
/// The value for this key will itself often be a MapExpr.
pub const SIM_CONTROL_PARAMS_KEY: &str = "params";

// Predefined action strings
pub const SIM_CONTROL_ACTION_QUERY_RESOURCE: &str = "query_resource";
pub const SIM_CONTROL_ACTION_QUERY_NULLIFIER: &str = "query_nullifier";
pub const SIM_CONTROL_ACTION_CREATE_RESOURCE: &str = "create_resource";
pub const SIM_CONTROL_ACTION_SPEND_NULLIFIER: &str = "spend_nullifier";
// Add more actions as needed, e.g., "get_current_step", "list_active_handlers"

/// Resource type for markers created in the TelContext when a control effect produces an output.
/// The SimulationEngine will look for resources of this type to relay data back.
pub const SIM_OUTPUT_MARKER_RESOURCE_TYPE: &str = "simulation_output_marker_v1";

/// Type identifier for the output marker effect.
pub const SIM_OUTPUT_MARKER_EFFECT_TYPE: &str = "SimOutputMarkerEffect";

// Content keys for SIM_OUTPUT_MARKER_RESOURCE_TYPE
/// Key for the original action that prompted this output.
pub const SIM_OUTPUT_ACTION_KEY: &str = "action";
/// Key for the result/data of the action.
pub const SIM_OUTPUT_RESULT_KEY: &str = "result";
/// Key for an error message if the action failed.
pub const SIM_OUTPUT_ERROR_KEY: &str = "sim_output_error";

/// Creates a generic simulation control effect.
///
/// # Arguments
/// * `effect_id` - The ID for this new effect instance.
/// * `intent_id` - The ID of the intent this effect belongs to (can be a dummy if not relevant).
/// * `action` - A string identifying the control action to perform (e.g., "query_resource").
/// * `params` - A `ValueExpr` (often a `ValueExpr::Map`) containing parameters for the action.
/// * `scoped_handler_id` - Optional ID of a handler to scope for this effect.
pub fn create_simulation_control_effect(
    effect_id: NodeId,
    _intent_id: NodeId, // Or causality_types::primitive::ids::IntentId if more appropriate
    action: String,
    params: ValueExpr,
    _scoped_handler_id: Option<NodeId>, // Or causality_types::primitive::ids::HandlerId
) -> Effect {
    let mut payload_map = BTreeMap::new();
    payload_map.insert(
        Str::from(SIM_CONTROL_ACTION_KEY),
        ValueExpr::String(Str::from(action)),
    );
    payload_map.insert(Str::from(SIM_CONTROL_PARAMS_KEY), params);

    Effect {
        id: causality_types::primitive::ids::EntityId::new(effect_id.inner()),
        name: Str::from(SIM_CONTROL_EFFECT_TYPE),
        domain_id: DomainId::null(),
        effect_type: Str::from(SIM_CONTROL_EFFECT_TYPE),
        inputs: Vec::new(),
        outputs: Vec::new(),
        expression: None,
        timestamp: causality_types::core::time::Timestamp::now(),
        resources: Vec::new(),
        nullifiers: Vec::new(),
        scoped_by: causality_types::primitive::ids::HandlerId::null(),
        intent_id: Some(ExprId::new(rand::random())),
        source_typed_domain: TypedDomain::default(),
        target_typed_domain: TypedDomain::default(),
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    }
}

/// Creates the content for a simulation output marker resource.
pub fn create_simulation_output_marker_content(
    action: &str,
    result: Option<ValueExpr>,
    error: Option<String>,
) -> ValueExpr {
    let mut map_data = BTreeMap::new();
    map_data.insert(
        Str::from(SIM_OUTPUT_ACTION_KEY),
        ValueExpr::String(Str::from(action)),
    );
    map_data.insert(
        Str::from(SIM_OUTPUT_ERROR_KEY),
        match error {
            Some(e) => ValueExpr::String(Str::from(e)),
            None => ValueExpr::Nil,
        },
    );
    map_data.insert(
        Str::from(SIM_OUTPUT_RESULT_KEY),
        match result {
            Some(r) => r,
            None => ValueExpr::Nil,
        },
    );

    ValueExpr::Map(ValueExprMap(map_data))
}

/// Create a simulation output marker effect with the given content.
/// This effect is typically handled by the simulation engine to record outputs.
pub fn create_simulation_output_marker_effect(_content: ValueExpr) -> Effect {
    Effect {
        id: causality_types::primitive::ids::EntityId::new(rand::random()),
        name: Str::from(SIM_OUTPUT_MARKER_EFFECT_TYPE),
        domain_id: DomainId::null(),
        effect_type: Str::from(SIM_OUTPUT_MARKER_EFFECT_TYPE),
        inputs: vec![],
        outputs: vec![],
        expression: None,
        timestamp: causality_types::core::time::Timestamp::now(),
        resources: vec![],
        nullifiers: vec![],
        scoped_by: causality_types::primitive::ids::HandlerId::null(),
        intent_id: Some(ExprId::new(rand::random())),
        source_typed_domain: TypedDomain::default(),
        target_typed_domain: TypedDomain::default(),
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    }
}

/// Creates the payload for a `SIM_CONTROL_EFFECT_TYPE` effect.
pub fn create_sim_control_effect_payload(
    action: &str,
    params: Option<ValueExpr>,
) -> ValueExpr {
    let mut payload_map = BTreeMap::new();
    payload_map.insert(
        Str::from(SIM_CONTROL_ACTION_KEY),
        ValueExpr::String(Str::from(action)),
    );
    if let Some(params) = params {
        payload_map.insert(Str::from(SIM_CONTROL_PARAMS_KEY), params);
    }
    ValueExpr::Map(ValueExprMap(payload_map))
}

//-----------------------------------------------------------------------------
// Simulation effect types
//-----------------------------------------------------------------------------

/// Enumeration of simulation effect types
pub enum SimulationEffectType {
    /// Breakpoint effect used to pause simulation
    Breakpoint,
    /// Control effect used to interact with the simulation
    SimControl,
}

impl SimulationEffectType {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            SimulationEffectType::Breakpoint => {
                SIM_BREAKPOINT_EFFECT_TYPE.to_string()
            }
            SimulationEffectType::SimControl => SIM_CONTROL_EFFECT_TYPE.to_string(),
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

// Note: Tests have been temporarily removed due to API changes in the unified types.
// They need to be updated to work with the new Effect struct that doesn't have a payload field.
