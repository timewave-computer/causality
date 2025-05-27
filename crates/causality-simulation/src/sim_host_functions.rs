//! Simulation Host Functions
//!
//! Defines host functions callable from Lisp during simulation,
//! enabling features such as breakpoints and control actions.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use causality_types::async_trait::async_trait;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use anyhow::{anyhow, Result as AnyhowResult};
use causality_lisp::ExprError as OldLispError;
use causality_lisp::ValueExpr as OldLispValue;
use causality_types::{
    core::{
        id::{
            AsId, DomainId, NullifierId, ResourceId, ExprId, IntentId, NodeId, 
            TypeExprId, ValueExprId, EntityId, HandlerId,
        },
        str::Str,
        time::Timestamp,
        Effect, Resource,
    },
    expr::{
        result::{ExprError, ExprResult},
        value::ValueExpr,
        ast::Expr as TypesExpr,
    },
    serialization::{Encode, Decode},
    provider::context::TelContextInterface,
};

// Import from causality_core for ID utilities
use causality_core::{id_to_hex, create_random_id, id_from_hex};
use causality_core::extension_traits::DomainIdExt;

// Import runtime types
use causality_runtime::{
    tel::{
        context::{Context as TelContext},
        traits::{StateManager, AsExecutionContext},
    },
    state_manager::DefaultStateManager,
};

use crate::engine::BreakpointInfo;
use crate::sim_effects::{
    create_breakpoint_effect_payload,
    SIM_BREAKPOINT_EFFECT_TYPE,
    SIM_CONTROL_ACTION_CREATE_RESOURCE,
    SIM_CONTROL_ACTION_KEY,
    SIM_CONTROL_ACTION_QUERY_NULLIFIER,
    SIM_CONTROL_ACTION_QUERY_RESOURCE,
    SIM_CONTROL_ACTION_SPEND_NULLIFIER,
    SIM_CONTROL_PARAMS_KEY,
    SIM_OUTPUT_ACTION_KEY,
    SIM_OUTPUT_ERROR_KEY,
    SIM_OUTPUT_RESULT_KEY,
};

// Use the public converter from the runtime crate
use causality_runtime::tel::lisp_adapter::{from_lisp_value, LispBridgeError};

//-----------------------------------------------------------------------------
// Types & Constants
//-----------------------------------------------------------------------------

// Target signature for host functions to be stored in LispContextConfig
type SimHostFn = dyn Fn(
        Vec<ValueExpr>,
        &mut dyn TelContextInterface,
    ) -> Result<ValueExpr, ExprError>
    + Send
    + Sync;

// Removed: const HOST_FN_CREATE_BREAKPOINT_MARKER: &str = "host_create_breakpoint_marker"; (will be map key)
// Removed: pub(crate) fn get_breakpoint_marker_type_id() -> TypeExprId { ... } (unused)

//-----------------------------------------------------------------------------
// Error Conversion
//-----------------------------------------------------------------------------

// Converts OldLispError (from sim_effects or other older lisp interactions) to ExprError
fn convert_old_lisp_error_to_types_lisp_error(
    old_err: OldLispError,
) -> ExprError {
    match old_err {
        OldLispError::TypeError { message, expr } => {
            ExprError::TypeError {
                message: Str::from(format!(
                    "Lisp type error: {}{}",
                    message,
                    expr.map(|e| format!(" in expression {}", e))
                        .unwrap_or_default()
                )),
                expr: None,
            }
        }
        OldLispError::ReferenceError { name } => ExprError::UnknownSymbol {
            name: Str::from(format!("Lisp reference error: unresolved name {}", name)),
        },
        OldLispError::ExecutionError { message } => {
            ExprError::HostFunctionError {
                message: Str::from(format!("Lisp execution error: {}", message)),
                expr: None,
            }
        }
        OldLispError::PermissionError { message, resource } => {
            ExprError::HostFunctionError {
                message: Str::from(format!(
                    "Lisp permission error: {}{}",
                    message,
                    resource
                        .map(|r| format!(" for resource {}", r))
                        .unwrap_or_default()
                )),
                expr: None,
            }
        }
    }
}

// Converts LispBridgeError (from the shared from_lisp_value) to TypesLispError
impl From<LispBridgeError> for TypesLispError {
    fn from(bridge_err: LispBridgeError) -> Self {
        TypesLispError::HostFunctionError {
            message: format!("Lisp bridge conversion error: {}", bridge_err).into(),
            expr: None,
        }
    }
}

//-----------------------------------------------------------------------------
// Breakpoint Host Functions
//-----------------------------------------------------------------------------

fn sim_mark_breakpoint_host_fn(
    args: Vec<ValueExpr>,
    exec_ctx: &mut dyn TelContextInterface,
) -> Result<ValueExpr, ExprError> {
    if args.len() != 2 {
        return Err(ExprError::TypeError {
            message: Str::from("sim_mark_breakpoint expects 2 arguments"),
            expr: None,
        });
    }
    let label_str = match &args[0] {
        ValueExpr::String(s) => s.clone(),
        _ => {
            return Err(ExprError::TypeError {
                message: Str::from("Breakpoint label must be a string"),
                expr: None,
            });
        }
    };
    let id_str = match &args[1] {
        ValueExpr::String(s) => s.clone(),
        _ => {
            return Err(ExprError::TypeError {
                message: Str::from("Breakpoint id must be a string"),
                expr: None,
            });
        }
    };

    let breakpoint_info = BreakpointInfo {
        label: label_str.to_string(), // Convert Str to String if BreakpointInfo expects String
        id: id_str.to_string(),       // Convert Str to String
    };

    // create_breakpoint_effect_payload returns OldLispValue
    let old_lisp_payload =
        create_breakpoint_effect_payload(label_str.clone(), id_str.clone());
    
    // For now, create a simple ValueExpr instead of complex conversion
    let effect_payload_types_value = ValueExpr::String(Str::from("breakpoint_payload"));

    let _effect = Effect {
        id: EntityId::new(rand::random()),
        name: Str::from("breakpoint_effect"),
        domain_id: exec_ctx.get_domain_id().unwrap_or_else(DomainId::null),
        effect_type: Str::from(SIM_BREAKPOINT_EFFECT_TYPE),
        inputs: vec![],
        outputs: vec![],
        expression: None,
        timestamp: Timestamp::now(),
        resources: vec![],
        nullifiers: vec![],
        scoped_by: HandlerId::null(),
        intent_id: None,
    };

    log::info!(
        "Host function 'sim_mark_breakpoint' called: Breakpoint hit - Label: '{}', ID: '{}'.",
        breakpoint_info.label,
        breakpoint_info.id,
    );

    Ok(effect_payload_types_value) // Return the converted payload
}

//-----------------------------------------------------------------------------
// Simulation Control Host Functions
//-----------------------------------------------------------------------------

pub const HOST_FN_PROCESS_SIM_CONTROL: &str = "host_process_simulation_control"; // Keep for map key

fn host_process_simulation_control_impl(
    // Renamed to avoid conflict, will be wrapped in Arc
    args: Vec<ValueExpr>,
    exec_ctx: &mut dyn TelContextInterface,
) -> Result<ValueExpr, ExprError> {
    if args.len() != 1 {
        return Err(ExprError::TypeError {
            message: Str::from("host_process_simulation_control expects 1 argument (a map with action and params)"),
            expr: None,
        });
    }

    let control_arg_map = match &args[0] {
        ValueExpr::Map(map_expr) => map_expr,
        _ => {
            return Err(ExprError::TypeError {
                message: Str::from("Control argument must be a map"),
                expr: None,
            });
        }
    };

    let action_str_val =
        match control_arg_map.0.get(&Str::from(SIM_CONTROL_ACTION_KEY)) {
            Some(ValueExpr::String(s)) => s.clone(),
            _ => {
                return Err(ExprError::TypeError {
                    message: Str::from("Missing or invalid 'action' string in control payload"),
                    expr: None,
                });
            }
        };

    let params_val = control_arg_val
        .get(&Str::from(SIM_CONTROL_PARAMS_KEY))
        .cloned()
        .unwrap_or(ValueExpr::Nil);

    let mut output_result: Option<ValueExpr> = None;
    let mut output_error_msg: Option<String> = None;

    let action_str_as_rust_str = action_str_val.as_str();

    // Helper to block on async operations from within this sync host function.
    // This assumes that the host function is called from within a Tokio runtime context.
    let rt_handle = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(_) => {
            // Fallback: create a new small runtime. This is inefficient and should be avoided.
            // Consider logging a warning here in a real application.
            // For tests, this might be okay, but for production, the caller (Lisp interpreter integration)
            // should ensure a Tokio context is available.
            let new_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| ExprError::HostFunctionError {
                    message: format!(
                        "Failed to create fallback Tokio runtime: {}",
                        e
                    ).into(),
                    expr: None,
                })?;
            // Enter the runtime context to make block_on work.
            // This is tricky because we don't own the runtime to keep it alive beyond this scope if needed for nested calls.
            // A better approach is for the outer system to guarantee a runtime.
            // For the purpose of this edit, assuming this is sufficient for a single blocking call.
            // THIS IS A HACK for fallback. Prefer existing runtime.
            // A proper solution might involve passing the handle or using block_in_place if called from async.
            // Given SimHostFn is sync, block_on is the primary tool if an async op must be performed.
            log::warn!("Host function '{}' created a fallback Tokio runtime. This is inefficient.", HOST_FN_PROCESS_SIM_CONTROL);
            new_rt.handle().clone() // Clone the handle from the new runtime
                                    // It's better to panic if no runtime, to highlight the issue:
                                    // return Err(TypesLispError::HostFunctionError {
                                    //     message: "Host function called outside of a Tokio runtime context.".into(),
                                    //     expr: None
                                    // });
        }
    };

    if action_str_as_rust_str == SIM_CONTROL_ACTION_QUERY_RESOURCE {
        if let ValueExpr::Map(params_map_expr) = params_val {
            if let Some(ValueExpr::String(resource_id_s)) =
                params_map_expr.0.get(&Str::from("resource_id"))
            {
                let resource_id_str_ref = resource_id_s.as_str();
                let res_id_result =
                    id_utils::id_from_hex::<ResourceId>(resource_id_str_ref);

                match res_id_result {
                    Ok(res_id) => match exec_ctx.get_resource_sync(&res_id) {
                        // AsRuntimeContext via TypesTelContextInterface
                        Ok(Some(resource_data)) => {
                            match exec_ctx
                                .get_value_expr_by_id_sync(&resource_data.value)
                            {
                                // AsRuntimeContext
                                Ok(Some(val_expr)) => output_result = Some(val_expr),
                                Ok(None) => {
                                    output_error_msg = Some(format!(
                                        "ValueExpr not found for ID: {}",
                                        resource_data.value
                                    ))
                                }
                                Err(e) => {
                                    output_error_msg = Some(format!(
                                        "Error getting ValueExpr for ID {}: {}",
                                        resource_data.value, e
                                    ))
                                }
                            }
                        }
                        Ok(None) => {
                            output_error_msg =
                                Some(format!("Resource not found: {}", res_id))
                        }
                        Err(e) => {
                            output_error_msg = Some(format!(
                                "Error getting resource {}: {}",
                                res_id, e
                            ))
                        }
                    },
                    Err(e) => {
                        output_error_msg = Some(format!(
                            "Invalid ResourceId hex string '{}': {}",
                            resource_id_str_ref, e
                        ))
                    }
                }
            } else {
                output_error_msg = Some("Missing 'resource_id' string parameter for query_resource action".to_string());
            }
        } else {
            output_error_msg =
                Some("Parameters for query_resource must be a map".to_string());
        }
    } else if action_str_as_rust_str == SIM_CONTROL_ACTION_QUERY_NULLIFIER {
        if let ValueExpr::Map(params_map_expr) = params_val {
            if let Some(ValueExpr::String(nullifier_id_s)) =
                params_map_expr.0.get(&Str::from("nullifier_id"))
            {
                let nullifier_id_str_ref = nullifier_id_s.as_str();
                let nullifier_id_result =
                    id_utils::id_from_hex::<NullifierId>(nullifier_id_str_ref);

                match nullifier_id_result {
                    Ok(nullifier_id) => {
                        // is_nullifier_consumed on exec_ctx (LispHostEnvironment -> StateManager -> AsExecutionContext)
                        let is_spent_result = rt_handle.block_on(async {
                            exec_ctx.is_nullified(&nullifier_id).await
                        });
                        match is_spent_result {
                            Ok(is_spent) => {
                                output_result = Some(ValueExpr::Bool(is_spent))
                            }
                            Err(e) => {
                                output_error_msg = Some(format!(
                                    "Error checking if nullifier {} is spent: {}",
                                    nullifier_id_str_ref, e
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        output_error_msg = Some(format!(
                            "Invalid NullifierId hex string '{}': {}",
                            nullifier_id_str_ref, e
                        ))
                    }
                }
            } else {
                output_error_msg = Some("Missing 'nullifier_id' string parameter for query_nullifier action".to_string());
            }
        } else {
            output_error_msg =
                Some("Parameters for query_nullifier must be a map".to_string());
        }
    } else if action_str_as_rust_str == SIM_CONTROL_ACTION_CREATE_RESOURCE {
        // Ensure `exec_ctx` is mutable for `create_resource`
        let mut exec_ctx_mut = exec_ctx; // Already mutable
        if let ValueExpr::Map(_resource_desc_map) = params_val {
            // TODO: Parse resource_desc_map to construct the Resource properly.
            // For now, creating a dummy resource.
            let new_res = Resource {
                id: EntityId::new(rand::random()),
                name: Str::from("test_resource"),
                domain_id: exec_ctx_mut.get_domain_id().unwrap_or_else(DomainId::null),
                resource_type: Str::from("test_type"),
                quantity: 1,
                timestamp: Timestamp::now(),
            };
            // This call will be profile-gated by LispHostEnvironment
            match rt_handle.block_on(async {
                AsExecutionContext::create_resource(
                    &mut exec_ctx_mut,
                    new_res.clone(),
                )
                .await
            }) {
                Ok(created_id) => {
                    output_result = Some(ValueExpr::String(
                        id_to_hex(&created_id).into(),
                    ))
                }
                Err(e) => {
                    output_error_msg =
                        Some(format!("Failed to create resource: {}", e))
                }
            }
        } else {
            output_error_msg = Some("Parameters for create_resource must be a map describing the resource".to_string());
        }
    } else if action_str_as_rust_str == SIM_CONTROL_ACTION_SPEND_NULLIFIER {
        let mut exec_ctx_mut = exec_ctx; // Already mutable
        if let ValueExpr::Map(params_map_expr) = params_val {
            if let Some(ValueExpr::String(resource_id_s)) =
                params_map_expr.0.get(&Str::from("resource_id"))
            {
                let resource_id_str_ref = resource_id_s.as_str();
                let res_id_result =
                    id_utils::id_from_hex::<ResourceId>(resource_id_str_ref);

                match res_id_result {
                    Ok(resource_id) => {
                        let nullifier_obj =
                            causality_types::resource::Nullifier::new(
                                resource_id.clone(),
                            );
                        // This call will be profile-gated
                        match rt_handle.block_on(async {
                            AsExecutionContext::nullify_resource(
                                &mut exec_ctx_mut,
                                nullifier_obj,
                            )
                            .await
                        }) {
                            Ok(_) => {
                                output_result = Some(ValueExpr::Bool(true))
                            }
                            Err(e) => {
                                output_error_msg = Some(format!(
                                    "Failed to spend nullifier for resource {}: {}",
                                    resource_id, e
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        output_error_msg = Some(format!(
                            "Invalid ResourceId hex string '{}': {}",
                            resource_id_str_ref, e
                        ))
                    }
                }
            } else {
                output_error_msg = Some("Missing 'resource_id' string parameter for spend_nullifier action".to_string());
            }
        } else {
            output_error_msg =
                Some("Parameters for spend_nullifier must be a map".to_string());
        }
    } else {
        output_error_msg = Some(format!(
            "Unknown simulation control action: {}",
            action_str_as_rust_str
        ));
    }

    let mut result_map_data = BTreeMap::new();
    result_map_data.insert(
        Str::from(SIM_OUTPUT_ACTION_KEY),
        ValueExpr::String(action_str_val.clone()), // Ensure action_str_val is cloned if used after move
    );
    if let Some(res_val) = output_result {
        result_map_data.insert(Str::from(SIM_OUTPUT_RESULT_KEY), res_val);
    }
    if let Some(err_str) = output_error_msg {
        // Changed from output_error
        result_map_data.insert(
            Str::from(SIM_OUTPUT_ERROR_KEY),
            ValueExpr::String(err_str.into()),
        );
    }
    Ok(ValueExpr::Map(ValueExprMap(result_map_data)))
}

//-----------------------------------------------------------------------------
// Collector Function
//-----------------------------------------------------------------------------

pub fn get_simulation_host_functions() -> BTreeMap<Str, Arc<SimHostFn>> {
    let mut fns: BTreeMap<Str, Arc<SimHostFn>> = BTreeMap::new();

    fns.insert(
        Str::from_static_str("sim_mark_breakpoint"),
        Arc::new(sim_mark_breakpoint_host_fn),
    );
    fns.insert(
        Str::from_static_str(HOST_FN_PROCESS_SIM_CONTROL),
        Arc::new(host_process_simulation_control_impl),
    );
    // Add other simulation host functions here
    fns
}

//-----------------------------------------------------------------------------
// Test Utilities (Need Substantial Refactor)
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use causality_lisp::parse as lisp_parse; // To parse definitions
    use causality_runtime::state_manager::DefaultStateManager;
    use causality_runtime::tel::context::LispHostEnvironment;
    use causality_runtime::tel::graph::TelGraph; // For TelInterpreterRuntime::new
    use causality_runtime::tel::Interpreter as TelInterpreterRuntime; // For tests
    use causality_toolkit::capability_helpers::grant_execute_basic_task;
    use causality_toolkit::capability_system_lisp; // To load definitions
    use causality_types::primitive::ids::{CapabilityId, DomainId, IntentId, NodeId};
    use causality_types::interpreter_config::LispContextConfig; // For evaluate_lisp_in_context
    use std::collections::BTreeMap; // Example, but we need a more generic grant for "CoreSystemOps"
                                    // Or, we can manually construct the Lisp for granting CoreSystemOps.
    use causality_lisp::dsl::builders as dsl; // For Lisp construction

    // Use the public constant from the runtime crate
    use causality_runtime::tel::context::DIRECT_MUTATION_PROFILE;

    // Helper to create a TelInterpreter suitable for tests requiring capability checks.
    async fn create_capability_aware_test_interpreter(
        domain_id_str: &str,
    ) -> TelInterpreterRuntime {
        let mut interpreter = TelInterpreterRuntime::new(TelGraph::new());
        let test_domain_id = DomainId::new_deterministic(domain_id_str, &[0]);
        interpreter.set_domain_id(Some(test_domain_id));

        // Load capability system Lisp definitions
        let capability_lisp_payload_str =
            capability_system_lisp::generate_capability_system_payload();
        match lisp_parse(&capability_lisp_payload_str) {
            Ok(parsed_expr) => {
                if let Err(e) = interpreter.load_lisp_definitions(&parsed_expr).await
                {
                    panic!("Failed to load capability Lisp definitions for test interpreter: {:?}", e);
                }
            }
            Err(e) => {
                panic!("Failed to parse capability Lisp payload for test: {:?}", e)
            }
        }

        // Grant a standard set of "CoreSystemOps" capabilities for simulation control.
        let core_system_actions = vec![
            "create_resource",
            "query_resource",
            "update_resource_data", // Though not directly used by sim_control_impl, good to have for general core ops
            "nullify_resource",     // Ditto
            "spend_nullifier",
            "query_nullifier_status",
        ];

        let lisp_actions: Vec<TypesExpr> = core_system_actions
            .iter()
            .map(|action| dsl::str_lit(action))
            .collect();

        let mut grant_details_map_data = BTreeMap::new();
        grant_details_map_data.insert(
            Str::from("capability-type-name"),
            dsl::expr_to_value_expr_for_map(dsl::str_lit("CoreSystemOps")).unwrap(),
        );
        grant_details_map_data.insert(
            Str::from("grantee-id"),
            dsl::expr_to_value_expr_for_map(dsl::str_lit(
                &test_domain_id.to_string(),
            ))
            .unwrap(),
        );
        grant_details_map_data.insert(
            Str::from("actions"),
            dsl::expr_to_value_expr_for_map(dsl::list(lisp_actions)).unwrap(),
        );
        // For CoreSystemOps, target-resource-id and constraints are often general or checked at point of use.
        // We can add a wildcard or specific constraint if needed, e.g., domain constraint.
        // grant_details_map_data.insert(Str::from("constraints"), dsl::expr_to_value_expr_for_map(dsl::list(vec![ ... ])).unwrap());

        let grant_details_value_map = ValueExpr::Map(ValueExprMap::from(
            grant_details_map_data
                .into_iter()
                .map(|(k, v_expr)| (k, v_expr))
                .collect(),
        ));
        let grant_lisp_call = dsl::list(vec![
            dsl::sym("capability-grant"),
            grant_details_value_map.into_expr(),
        ]);
        let grant_config = LispContextConfig {
            host_function_profile: Some(Str::from("capability_profile")),
            initial_bindings: BTreeMap::new(),
            additional_host_functions: BTreeMap::new(),
        };

        match interpreter.evaluate_lisp_in_context(&grant_lisp_call, Vec::new(), &grant_config).await {
            Ok(ValueExpr::String(_cap_id_str)) => { /* Grant successful */ }
            Ok(ValueExpr::Ref(cap_id_val)) => { 
                log::debug!("Capability grant for CoreSystemOps returned Ref: {:?}", cap_id_val);
                /* Grant successful if it returns ResourceId for capability */
            }
            Ok(other) => panic!("Capability grant for CoreSystemOps failed for test setup, unexpected result: {:?}", other),
            Err(e) => panic!("Capability grant for CoreSystemOps failed for test setup: {:?}", e),
        }
        interpreter
    }

    // Mock TypesTelContextInterface for testing host functions
    struct MockSimContext {
        state_manager: Arc<DefaultStateManager>, // Using DefaultStateManager for concrete impl
        domain_id: Option<DomainId>,
        // Add other fields if needed by host functions e.g. initial_bindings for get_initial_binding
    }

    impl Default for MockSimContext {
        fn default() -> Self {
            Self {
                state_manager: Arc::new(DefaultStateManager::new()),
                domain_id: Some(DomainId::new_unique()),
            }
        }
    }

    // Implement necessary traits for MockSimContext
    // This would be a simplified LispHostEnvironment or a dedicated mock.
    // For now, we will try to use LispHostEnvironment in tests.

    fn create_test_lisp_host_env(
        profile: Option<Str>,
        initial_bindings_value_expr: BTreeMap<Str, ValueExpr>,
        additional_fns: BTreeMap<Str, Arc<SimHostFn>>,
        expr_store: Option<Arc<BTreeMap<ExprId, TypesExpr>>>, // Added expr_store
    ) -> LispHostEnvironment {
        let sm = Arc::new(DefaultStateManager::new());
        let initial_bindings_expr_result: BTreeMap<Str, ExprResult> =
            initial_bindings_value_expr
                .into_iter()
                .map(|(k, v)| (k, ExprResult::Value(v)))
                .collect();

        let final_expr_store =
            expr_store.unwrap_or_else(|| Arc::new(BTreeMap::new()));

        LispHostEnvironment::new(
            sm.clone(),
            Some(DomainId::new_unique()),
            profile,
            initial_bindings_expr_result, // Use converted bindings
            additional_fns,
            final_expr_store, // Pass expr_store
        )
    }

    #[test]
    fn test_sim_mark_breakpoint_host_fn_basic() {
        let mut env =
            create_test_lisp_host_env(None, BTreeMap::new(), BTreeMap::new(), None);
        let args = vec![
            ValueExpr::String("test_label".into()),
            ValueExpr::String("test_id".into()),
        ];
        let result = sim_mark_breakpoint_host_fn(args, &mut env);
        assert!(result.is_ok());
        if let Ok(ValueExpr::Map(map_val)) = result {
            assert_eq!(
                map_val.0.get(&Str::from("label")),
                Some(&ValueExpr::String("test_label".into()))
            );
            assert_eq!(
                map_val.0.get(&Str::from("id")),
                Some(&ValueExpr::String("test_id".into()))
            );
        } else {
            panic!("Expected a map result, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_host_process_simulation_control_query_resource() {
        // Scenario 1: Query non-existent resource (with capability to query)
        let mut interpreter_env_s1 =
            create_capability_aware_test_interpreter("domain_query_non_existent")
                .await;
        let resource_id_s1 = ResourceId::new_random();

        let mut params_map_s1 = BTreeMap::new();
        params_map_s1.insert(
            Str::from("resource_id"),
            ValueExpr::String(resource_id_s1.to_hex().into()),
        );

        let mut control_action_map_s1 = BTreeMap::new();
        control_action_map_s1.insert(
            Str::from(SIM_CONTROL_ACTION_KEY),
            ValueExpr::String(SIM_CONTROL_ACTION_QUERY_RESOURCE.into()),
        );
        control_action_map_s1.insert(
            Str::from(SIM_CONTROL_PARAMS_KEY),
            ValueExpr::Map(ValueExprMap(params_map_s1)),
        );

        let args_s1 = vec![ValueExpr::Map(ValueExprMap(
            control_action_map_s1,
        ))];

        let result_s1 =
            host_process_simulation_control_impl(args_s1, &mut interpreter_env_s1);
        assert!(
            result_s1.is_ok(),
            "Scenario 1 query failed: {:?}",
            result_s1.err()
        );

        if let Ok(ValueExpr::Map(map_val_s1)) = result_s1 {
            assert_eq!(
                map_val_s1.0.get(&Str::from(SIM_OUTPUT_ACTION_KEY)),
                Some(&ValueExpr::String(
                    SIM_CONTROL_ACTION_QUERY_RESOURCE.into()
                ))
            );
            assert!(
                map_val_s1.0.contains_key(&Str::from(SIM_OUTPUT_ERROR_KEY)),
                "Scenario 1: Expected error key for non-existent resource"
            );
            if let Some(ValueExpr::String(err_str)) =
                map_val_s1.0.get(&Str::from(SIM_OUTPUT_ERROR_KEY))
            {
                assert!(
                    err_str.as_str().contains("Resource not found"),
                    "Scenario 1: Incorrect error message: {}",
                    err_str
                );
            } else {
                panic!("Scenario 1: Expected error string for non-existent resource query.");
            }
        } else {
            panic!("Scenario 1: Expected a map result, got {:?}", result_s1);
        }

        // Scenario 2: Query existing resource (with capability to query)
        let mut interpreter_env_s2 = create_capability_aware_test_interpreter(
            "domain_query_existent_with_cap",
        )
        .await;
        let resource_id_s2 = ResourceId::new_random();
        let resource_data_s2 =
            ValueExpr::Map(ValueExprMap(BTreeMap::from([(
                Str::from("field1"),
                ValueExpr::String("value1".into()),
            )])));

        // Manually create the resource in the state manager
        let resource_s2 = Resource {
            id: resource_id_s2,
            domain: interpreter_env_s2.domain_id().unwrap(), // Associated with the interpreter's domain
            ephemeral: false,
            value: interpreter_env_s2
                .state_manager
                .lock()
                .await
                .store_value_expr(resource_data_s2.clone())
                .await
                .unwrap(),
            type_expr: ValueExprId::new_random().into(),
            static_expr: None,
        };
        interpreter_env_s2
            .state_manager
            .lock()
            .await
            .create_resource(resource_s2.clone())
            .await
            .unwrap();

        let mut params_map_s2 = BTreeMap::new();
        params_map_s2.insert(
            Str::from("resource_id"),
            ValueExpr::String(resource_id_s2.to_hex().into()),
        );

        let mut control_action_map_s2 = BTreeMap::new();
        control_action_map_s2.insert(
            Str::from(SIM_CONTROL_ACTION_KEY),
            ValueExpr::String(SIM_CONTROL_ACTION_QUERY_RESOURCE.into()),
        );
        control_action_map_s2.insert(
            Str::from(SIM_CONTROL_PARAMS_KEY),
            ValueExpr::Map(ValueExprMap(params_map_s2)),
        );

        let args_s2 = vec![ValueExpr::Map(ValueExprMap(
            control_action_map_s2,
        ))];

        let result_s2 =
            host_process_simulation_control_impl(args_s2, &mut interpreter_env_s2);
        assert!(
            result_s2.is_ok(),
            "Scenario 2 query failed: {:?}",
            result_s2.err()
        );

        if let Ok(ValueExpr::Map(map_val_s2)) = result_s2 {
            assert_eq!(
                map_val_s2.0.get(&Str::from(SIM_OUTPUT_ACTION_KEY)),
                Some(&ValueExpr::String(
                    SIM_CONTROL_ACTION_QUERY_RESOURCE.into()
                ))
            );
            assert!(
                !map_val_s2.0.contains_key(&Str::from(SIM_OUTPUT_ERROR_KEY)),
                "Scenario 2: Expected no error key, got one: {:?}",
                map_val_s2.0.get(&Str::from(SIM_OUTPUT_ERROR_KEY))
            );
            assert!(
                map_val_s2.0.contains_key(&Str::from(SIM_OUTPUT_RESULT_KEY)),
                "Scenario 2: Expected result key"
            );
            if let Some(ValueExpr::Map(returned_resource_map)) =
                map_val_s2.0.get(&Str::from(SIM_OUTPUT_RESULT_KEY))
            {
                // The host function currently returns the Resource struct itself, which gets serialized.
                // Let's check if the ID matches.
                if let Some(ValueExpr::String(id_val)) =
                    returned_resource_map.0.get(&Str::from("id"))
                {
                    assert_eq!(
                        id_val.as_str(),
                        resource_id_s2.to_hex().as_str(),
                        "Scenario 2: Returned resource ID mismatch"
                    );
                } else {
                    panic!("Scenario 2: Returned resource map does not contain 'id' field.");
                }
            } else {
                panic!("Scenario 2: Expected SIM_OUTPUT_RESULT_KEY to be a map (serialized resource), got {:?}", map_val_s2.0.get(&Str::from(SIM_OUTPUT_RESULT_KEY)));
            }
        } else {
            panic!("Scenario 2: Expected a map result, got {:?}", result_s2);
        }

        // Scenario 3: Query existing resource (no capability to query)
        // First, set up the resource with an owner domain
        let owner_domain_id_s3 =
            DomainId::new_deterministic("domain_owner_for_s3", &[0]);
        let mut owner_interpreter_s3 = TelInterpreterRuntime::new(TelGraph::new());
        owner_interpreter_s3.set_domain_id(Some(owner_domain_id_s3));
        // Minimal load for owner to create resource if capability system demands it (though direct SM access here)
        let cap_payload_s3 =
            capability_system_lisp::generate_capability_system_payload();
        owner_interpreter_s3
            .load_lisp_definitions(&lisp_parse(&cap_payload_s3).unwrap())
            .await
            .unwrap();

        let resource_id_s3 = ResourceId::new_random();
        let resource_data_s3 =
            ValueExpr::Map(ValueExprMap(BTreeMap::from([(
                Str::from("field1"),
                ValueExpr::String("value_s3".into()),
            )])));
        let resource_s3_val_id = owner_interpreter_s3
            .state_manager
            .lock()
            .await
            .store_value_expr(resource_data_s3.clone())
            .await
            .unwrap();
        let resource_s3 = Resource {
            id: resource_id_s3,
            domain: owner_domain_id_s3,
            ephemeral: false,
            value: resource_s3_val_id,
            type_expr: ValueExprId::new_random().into(),
            static_expr: None,
        };
        owner_interpreter_s3
            .state_manager
            .lock()
            .await
            .create_resource(resource_s3.clone())
            .await
            .unwrap();

        // Now, create an querier interpreter from a *different* domain, without query capability granted for CoreSystemOps
        // (create_capability_aware_test_interpreter grants it by default, so we need a more basic one or revoke)
        // For simplicity, we'll create a new interpreter and *not* grant the CoreSystemOps for querying.
        let mut querier_interpreter_s3 = TelInterpreterRuntime::new(TelGraph::new());
        let querier_domain_id_s3 =
            DomainId::new_deterministic("domain_querier_no_cap_s3", &[0]);
        querier_interpreter_s3.set_domain_id(Some(querier_domain_id_s3));
        // Load base Lisp definitions for capability system to function, but don't grant specific caps.
        querier_interpreter_s3
            .load_lisp_definitions(&lisp_parse(&cap_payload_s3).unwrap())
            .await
            .unwrap();

        let mut params_map_s3 = BTreeMap::new();
        params_map_s3.insert(
            Str::from("resource_id"),
            ValueExpr::String(resource_id_s3.to_hex().into()),
        );

        let mut control_action_map_s3 = BTreeMap::new();
        control_action_map_s3.insert(
            Str::from(SIM_CONTROL_ACTION_KEY),
            ValueExpr::String(SIM_CONTROL_ACTION_QUERY_RESOURCE.into()),
        );
        control_action_map_s3.insert(
            Str::from(SIM_CONTROL_PARAMS_KEY),
            ValueExpr::Map(ValueExprMap(params_map_s3)),
        );

        let args_s3 = vec![ValueExpr::Map(ValueExprMap(
            control_action_map_s3,
        ))];

        let result_s3 = host_process_simulation_control_impl(
            args_s3,
            &mut querier_interpreter_s3,
        );
        assert!(
            result_s3.is_ok(),
            "Scenario 3 query failed unexpectedly: {:?}",
            result_s3.err()
        );

        if let Ok(ValueExpr::Map(map_val_s3)) = result_s3 {
            assert_eq!(
                map_val_s3.0.get(&Str::from(SIM_OUTPUT_ACTION_KEY)),
                Some(&ValueExpr::String(
                    SIM_CONTROL_ACTION_QUERY_RESOURCE.into()
                ))
            );
            assert!(
                map_val_s3.0.contains_key(&Str::from(SIM_OUTPUT_ERROR_KEY)),
                "Scenario 3: Expected error key for no capability"
            );
            if let Some(ValueExpr::String(err_str)) =
                map_val_s3.0.get(&Str::from(SIM_OUTPUT_ERROR_KEY))
            {
                // The error comes from TelInterpreter's capability check.
                assert!(err_str.as_str().contains("Capability check failed"), "Scenario 3: Incorrect error message, expected capability failure: {}", err_str);
            } else {
                panic!("Scenario 3: Expected error string for no capability query.");
            }
        } else {
            panic!("Scenario 3: Expected a map result, got {:?}", result_s3);
        }
    }

    #[tokio::test]
    async fn test_host_process_simulation_control_create_resource_success() {
        let mut interpreter_env = create_capability_aware_test_interpreter(
            "test_domain_for_create_resource",
        )
        .await;

        // Dummy resource description for the test
        let mut resource_desc_map = BTreeMap::new();
        resource_desc_map.insert(
            Str::from("type"),
            ValueExpr::String("dummy_type".into()),
        );

        let mut control_action_map_data = BTreeMap::new();
        control_action_map_data.insert(
            Str::from(SIM_CONTROL_ACTION_KEY),
            ValueExpr::String(SIM_CONTROL_ACTION_CREATE_RESOURCE.into()),
        );
        control_action_map_data.insert(
            Str::from(SIM_CONTROL_PARAMS_KEY),
            ValueExpr::Map(ValueExprMap(resource_desc_map)),
        );

        let args = vec![ValueExpr::Map(ValueExprMap(
            control_action_map_data,
        ))];

        let result =
            host_process_simulation_control_impl(args, &mut interpreter_env);
        assert!(result.is_ok(), "Create resource failed: {:?}", result.err());

        if let Ok(ValueExpr::Map(map_val)) = result {
            assert_eq!(
                map_val.0.get(&Str::from(SIM_OUTPUT_ACTION_KEY)),
                Some(&ValueExpr::String(
                    SIM_CONTROL_ACTION_CREATE_RESOURCE.into()
                ))
            );
            assert!(
                map_val.0.contains_key(&Str::from(SIM_OUTPUT_RESULT_KEY)),
                "Result key missing: {:?}",
                map_val
            );
            assert!(
                !map_val.0.contains_key(&Str::from(SIM_OUTPUT_ERROR_KEY)),
                "Error key present when expecting success: {:?}",
                map_val
            );
            if let Some(ValueExpr::String(res_id_s)) =
                map_val.0.get(&Str::from(SIM_OUTPUT_RESULT_KEY))
            {
                let res_id_str_ref = res_id_s.as_str();
                assert!(
                    id_utils::id_from_hex::<ResourceId>(res_id_str_ref).is_ok(),
                    "Result is not a valid ResourceId hex"
                );
            } else {
                panic!("Expected result to be a string ResourceId hex");
            }
        } else {
            panic!(
                "Expected a map result from create_resource, got {:?}",
                result
            );
        }
    }

    // TODO: Add more tests for other actions (query_nullifier, spend_nullifier)
    // TODO: Add tests for permission denial based on profile in LispHostEnvironment for these actions.
}

// Removed: pub fn create_simulation_context(...) and some_function_that_creates_context_directly()
// as they used the old TelContext and are superseded by testing with LispHostEnvironment or mocks.

//-----------------------------------------------------------------------------
// Exported Functions for Engine
//-----------------------------------------------------------------------------

/// Creates a breakpoint marker host function definition for the engine
pub fn create_breakpoint_marker_host_fn_definition() -> (Str, Arc<dyn Fn(Vec<ValueExpr>, &mut dyn AsExprContext) -> Result<ValueExpr, ExprError> + Send + Sync>) {
    let fn_name = Str::from("create_breakpoint_marker");
    let fn_impl = Arc::new(|args: Vec<ValueExpr>, _ctx: &mut dyn AsExprContext| -> Result<ValueExpr, ExprError> {
        if args.len() != 2 {
            return Err(ExprError::TypeError {
                message: Str::from("create_breakpoint_marker expects 2 arguments"),
                expr: None,
            });
        }
        // Return a simple success marker
        Ok(ValueExpr::Bool(true))
    });
    (fn_name, fn_impl)
}
