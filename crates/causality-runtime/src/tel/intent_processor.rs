// Purpose: Processes Intents, validates them, and generates an initial EffectGraph.

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

use causality_types::{
    core::{
        id::{EntityId, ExprId, ResourceId, NodeId, HandlerId, EdgeId, TypeExprId, AsId},
        str::Str as CausalityStr,
    },
    tel::{
        EffectGraph, Edge, EdgeKind, ResourceRef,
    },
    effect::{
        types::Effect,
        intent::Intent,
        handler::Handler,
    },
    expr::{value::ValueExpr, result::ExprError, ValueExprMap, ValueExprVec},
    graph::{
        execution::GraphExecutionContext,
        optimization::TypedDomain,
    },
    resource::flow::ResourceFlow,
    primitive::time::Timestamp,
};

use causality_core::utils::expr::{value_expr_as_list, value_expr_as_record, value_expr_as_string, value_expr_as_bool, value_expr_as_int};
use causality_core::id_from_hex;

use crate::{
    tel::{
        interpreter::{Interpreter as LispInterpreterService, LispContextConfig, LispEvaluator},
    },
};

// Type alias for a complex host function type
#[allow(dead_code)]
type LispHostFn = Arc<dyn Fn(Vec<ValueExpr>) -> Result<ValueExpr, ExprError> + Send + Sync>;

// Helper functions to create unique IDs
fn create_unique_resource_id() -> ResourceId {
    let mut bytes = [0u8; 32];
    let uuid = Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();
    bytes[0..16].copy_from_slice(uuid_bytes);
    ResourceId::new(bytes)
}

fn create_unique_node_id() -> NodeId {
    let mut bytes = [0u8; 32];
    let uuid = Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();
    bytes[0..16].copy_from_slice(uuid_bytes);
    NodeId::new(bytes)
}

fn create_unique_handler_id() -> HandlerId {
    let mut bytes = [0u8; 32];
    let uuid = Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();
    bytes[0..16].copy_from_slice(uuid_bytes);
    HandlerId::new(bytes)
}

fn create_unique_edge_id() -> EdgeId {
    let mut bytes = [0u8; 32];
    let uuid = Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();
    bytes[0..16].copy_from_slice(uuid_bytes);
    EdgeId::new(bytes)
}

// Helper trait for hex conversion
trait FromHex: Sized {
    fn from_hex(s: &str) -> Result<Self, &'static str>;
}

// Implement FromHex for all ID types
impl<T: AsId> FromHex for T {
    fn from_hex(s: &str) -> Result<Self, &'static str> {
        id_from_hex(s)
    }
}

#[derive(Debug)]
pub struct IntentProcessor {
    lisp_service: Arc<LispInterpreterService>,
}

// Helper to convert Handler to ValueExpr::Record
fn create_handler_details_record(handler: &causality_types::core::Handler) -> ValueExpr {
    let mut record_map = BTreeMap::new();
    record_map.insert(CausalityStr::from_static_str(":id"), ValueExpr::String(CausalityStr::from(handler.id.to_hex())));
    record_map.insert(CausalityStr::from_static_str(":name"), ValueExpr::String(handler.name.clone()));
    record_map.insert(CausalityStr::from_static_str(":domain_id"), ValueExpr::String(CausalityStr::from(handler.domain_id.to_hex())));
    record_map.insert(CausalityStr::from_static_str(":handles_type"), ValueExpr::String(handler.handles_type.clone()));
    record_map.insert(CausalityStr::from_static_str(":priority"), ValueExpr::Number(causality_types::expression::value::Number::new_integer(handler.priority as i64)));
    if let Some(ref expr_id) = handler.expression {
        record_map.insert(CausalityStr::from_static_str(":expression_id"), ValueExpr::String(CausalityStr::from(expr_id.to_hex())));
    } else {
        record_map.insert(CausalityStr::from_static_str(":expression_id"), ValueExpr::Nil);
    }
    
    ValueExpr::Record(ValueExprMap(record_map))
}

impl IntentProcessor {
    pub fn new(lisp_service: Arc<LispInterpreterService>) -> Self {
        Self { lisp_service }
    }

    pub async fn process_intent(
        &self,
        intent: &causality_types::effect::intent::Intent, // Updated path
        initial_graph_context: &GraphExecutionContext, // Context for intent's own evaluation
    ) -> Result<EffectGraph> {
        log::info!("Processing intent {:?} (context mode: {:?})", intent.id, initial_graph_context.interpreter_mode);

        let dynamic_expr_id = match intent.expression {
            Some(expr_id) => expr_id,
            None => return Err(anyhow!("Intent {:?} has no expression to evaluate for graph generation.", intent.id)),
        };

        let intent_domain = match initial_graph_context.domain_id {
            Some(domain) => domain,
            None => return Err(anyhow!("Intent processing requires a domain_id in the context"))
        };

        let expr_ast_result = {
            // Create a binding to prevent temporary value dropped error
            let state_manager = self.lisp_service.state_manager();
            let sm_guard = state_manager.lock().await;
            sm_guard.get_expr_sync(&dynamic_expr_id)
                .map_err(|e| anyhow!("StateManager error fetching intent expr {:?}: {}", dynamic_expr_id, e))
                .and_then(|opt_expr| opt_expr.ok_or_else(|| anyhow!("Intent Expr AST not found for ID: {:?}", dynamic_expr_id)))
        };

        match expr_ast_result {
            Ok(expr_ast) => {
                let mut initial_bindings = BTreeMap::new();
                initial_bindings.insert(
                    CausalityStr::from_static_str("*current-intent-id*"),
                    ValueExpr::String(CausalityStr::from(intent.id.to_hex()))
                );
                initial_bindings.insert(
                    CausalityStr::from_static_str("*current-intent-payload*"),
                    ValueExpr::Map(ValueExprMap(BTreeMap::new()))
                );
                // TODO M2.2.d: Add other relevant context from initial_graph_context to bindings or provide via host functions.
                // For example, make `initial_graph_context.visible_resources` queryable.

                let mut additional_host_functions: BTreeMap<CausalityStr, Arc<dyn Fn(Vec<ValueExpr>) -> Result<ValueExpr, ExprError> + Send + Sync>> = BTreeMap::new();
                
                let lisp_service_clone_for_resources = self.lisp_service.clone();
                let host_fn_get_resources: Arc<dyn Fn(Vec<ValueExpr>) -> Result<ValueExpr, ExprError> + Send + Sync> = 
                    Arc::new(move |args: Vec<ValueExpr>| -> Result<ValueExpr, ExprError> {
                        if !args.is_empty() {
                            return Err(ExprError::ExecutionError { 
                                message: CausalityStr::from("get-visible-resource-refs-for-intent takes no arguments"),
                            });
                        }
                        // Create bindings to prevent temporary value dropped error
                        let state_manager = lisp_service_clone_for_resources.state_manager();
                        let sm_guard = futures::executor::block_on(state_manager.lock());
                        let domain_id_for_lookup = intent_domain;
                        let resource_refs = futures::executor::block_on(sm_guard.get_all_resources_by_domain(&domain_id_for_lookup))
                            .map_err(|e| ExprError::ExecutionError { 
                                message: CausalityStr::from(format!("Error fetching resources for domain: {}", e)),
                            })?;
                        let resource_ref_values: Vec<ValueExpr> = resource_refs
                            .iter()
                            .map(|rr| ValueExpr::String(CausalityStr::from(rr.0.to_hex())))
                            .collect();
                        Ok(ValueExpr::List(ValueExprVec(resource_ref_values)))
                    });
                additional_host_functions.insert(
                    CausalityStr::from_static_str("get-visible-resource-refs-for-intent"), 
                    host_fn_get_resources
                );

                let lisp_service_clone_for_handler_details = self.lisp_service.clone();
                let host_fn_get_handler_details: Arc<dyn Fn(Vec<ValueExpr>) -> Result<ValueExpr, ExprError> + Send + Sync> = 
                    Arc::new(move |args: Vec<ValueExpr>| -> Result<ValueExpr, ExprError> {
                        if args.len() != 1 {
                            return Err(ExprError::ExecutionError { 
                                message: CausalityStr::from("get-handler-details expects 1 argument (handler-id-string)"),
                            });
                        }
                        let handler_id_str_val = args.first().unwrap();
                        match value_expr_as_string(handler_id_str_val) {
                            Some(handler_id_c_str) => {
                                match <HandlerId as AsId>::from_hex(handler_id_c_str.as_str()) {
                                    Ok(handler_id) => {
                                        // Create bindings to prevent temporary value dropped error
                                        let state_manager = lisp_service_clone_for_handler_details.state_manager();
                                        let sm_guard = futures::executor::block_on(state_manager.lock());
                                        match futures::executor::block_on(sm_guard.get_handler(&handler_id)) {
                                            Ok(Some(handler_arc)) => Ok(create_handler_details_record(&handler_arc)),
                                            Ok(None) => Ok(ValueExpr::Nil), // Handler not found
                                            Err(e) => Err(ExprError::ExecutionError { 
                                                message: CausalityStr::from(format!("Error fetching handler details: {}", e)),
                                            }),
                                        }
                                    }
                                    Err(e) => Err(ExprError::ExecutionError { 
                                        message: CausalityStr::from(format!("Invalid HandlerId format: {}, error: {}", handler_id_c_str, e)),
                                    }),
                                }
                            }
                            None => Err(ExprError::ExecutionError { 
                                message: CausalityStr::from("get-handler-details argument must be a string HandlerId"),
                            }),
                        }
                    });
                additional_host_functions.insert(
                    CausalityStr::from_static_str("get-handler-details"), 
                    host_fn_get_handler_details
                );

                let lisp_service_clone_for_find_handlers = self.lisp_service.clone();
                let intent_domain_clone_for_find_handlers = intent_domain;
                let host_fn_find_handlers: Arc<dyn Fn(Vec<ValueExpr>) -> Result<ValueExpr, ExprError> + Send + Sync> = 
                    Arc::new(move |args: Vec<ValueExpr>| -> Result<ValueExpr, ExprError> {
                        if args.len() != 1 {
                            return Err(ExprError::ExecutionError { 
                                message: CausalityStr::from("find-handlers-for-effect-type expects 1 argument (effect-type-string)"),
                            });
                        }
                        let effect_type_val = args.first().unwrap();
                        match value_expr_as_string(effect_type_val) {
                            Some(effect_type_c_str) => {
                                // Create bindings to prevent temporary value dropped error
                                let state_manager = lisp_service_clone_for_find_handlers.state_manager();
                                let sm_guard = futures::executor::block_on(state_manager.lock());
                                match futures::executor::block_on(sm_guard.get_all_handlers_by_domain(&intent_domain_clone_for_find_handlers)) {
                                    Ok(all_handlers) => {
                                        let matching_handlers: Vec<ValueExpr> = all_handlers.iter()
                                            .filter(|h| h.handles_type == *effect_type_c_str) 
                                            .map(create_handler_details_record)
                                            .collect();
                                        Ok(ValueExpr::List(ValueExprVec(matching_handlers)))
                                    }
                                    Err(e) => Err(ExprError::ExecutionError { 
                                        message: CausalityStr::from(format!("Error fetching all handlers for find: {}", e)),
                                    }),
                                }
                            }
                            None => Err(ExprError::ExecutionError { 
                                message: CausalityStr::from("find-handlers-for-effect-type argument must be a string"),
                            }),
                        }
                    });
                additional_host_functions.insert(
                    CausalityStr::from_static_str("find-handlers-for-effect-type"), 
                    host_fn_find_handlers
                );

                let lisp_config = LispContextConfig {
                    host_function_profile: Some(CausalityStr::from_static_str("intent_processing")),
                    initial_bindings,
                    additional_host_functions,
                };
                
                // Intents might not take direct arguments in the same way effects do; payload is in bindings.
                let lisp_args = vec![]; 

                match self.lisp_service.evaluate_lisp_in_context(&expr_ast, lisp_args, &lisp_config).await {
                    Ok(eval_result) => {
                        log::debug!("Intent {:?} dynamic_expr eval OK: {:?}", intent.id, eval_result);
                        // Convention: eval_result is a data structure (Lisp record) describing the initial graph.
                        // TODO M2.2.a: Define this convention robustly.
                        //
                        // Proposed Lisp Output Structure:
                        // (record
                        //   (':effects (list
                        //               (record (':id "temp-effect-1") ; Symbolic ID for intra-definition linking
                        //                       (':type "effect-type-A")
                        //                       (':payload (record ...))
                        //                       (':inputs (list "resource-id-1" (record (':ref "temp-out-X")))) ; ResourceId string or ref to another temp output ID
                        //                       (':outputs (list (record (':id "temp-out-A1") (':type_expr "type-expr-id-for-A1"))
                        //                                       (record (':id "temp-out-A2") (':type_expr "type-expr-id-for-A2"))))
                        //                       (':dynamic_expr "expr-id-for-effect-logic") ; Optional ExprId string
                        //                       (':scoped_handler "handler-id-for-scope")   ; Optional HandlerId string
                        //                       (':constraints (list "expr-id-constraint-1"))) ; Optional list of ExprId strings
                        //               ;; ... more effects
                        //              ))
                        //   (':edges (list
                        //             (record (':source "temp-effect-1") ; Symbolic ID or existing NodeId string
                        //                     (':target "temp-effect-2") ; Symbolic ID or existing NodeId string
                        //                     (':kind (record (':type "Next") (':node_id "temp-effect-2")))) ; Or (':node_id_ref "temp-effect-2")
                        //                                                                                     ; Specific fields depend on EdgeKind type
                        //                                                                                     ; e.g. Consumes: (':resource_ref "some-resource-id") or (':resource_ref (record (':ref "temp-out-A1")))
                        //                                                                                     ;      Applies: (':handler_id "some-handler-id")
                        //             ;; ... more edges
                        //            ))
                        //   (':handlers (list ; Optional: if intents can propose handlers for the graph
                        //                (record (':id "temp-handler-1")      ; Symbolic ID for intra-definition linking (if needed by edges, e.g. Applies)
                        //                        (':name "MyCustomHandler")    ; Optional descriptive name (string)
                        //                        (':effect_type "effect-type-A") ; String identifier of the effect type this handler targets
                        //                        (':priority 10)               ; Integer priority, higher wins
                        //                        (':constraints (list "expr-id-constraint-h1")) ; Optional list of ExprId strings for handler applicability
                        //                        (':dynamic_expr "expr-id-for-handler-logic")) ; Required ExprId string for handler logic
                        //                ;; ... more handlers
                        //               ))
                        // )

                        let mut effects = Vec::new();
                        let mut edges = Vec::new();
                        let mut handlers_vec: Vec<causality_types::tel::Handler> = Vec::new(); // Vector for parsed handlers
                        let mut temp_id_to_node_id_map: BTreeMap<CausalityStr, NodeId> = BTreeMap::new();
                        let mut temp_id_to_handler_id_map: BTreeMap<CausalityStr, HandlerId> = BTreeMap::new(); // For temp handler IDs
                        let mut temp_output_id_to_resource_id_map: BTreeMap<CausalityStr, (ResourceId, TypeExprId)> = BTreeMap::new();

                        if let ValueExpr::Record(ValueExprMap(graph_def_record_map)) = eval_result {
                            // Parse :effects
                            if let Some(effects_list_val) = graph_def_record_map.get(&CausalityStr::from_static_str(":effects")) {
                                if let Some(effect_defs) = value_expr_as_list(effects_list_val) {
                                    for effect_def_val in effect_defs {
                                        if let Some(effect_fields) = value_expr_as_record(effect_def_val) {
                                            let temp_effect_id_str = effect_fields.get(&CausalityStr::from_static_str(":id"))
                                                .and_then(value_expr_as_string)
                                                .cloned();
                                            
                                            let effect_type_str = effect_fields.get(&CausalityStr::from_static_str(":type"))
                                                .and_then(value_expr_as_string);
                                            
                                            let _payload_val = effect_fields.get(&CausalityStr::from_static_str(":payload"))
                                                .cloned()
                                                .unwrap_or(ValueExpr::Nil);

                                            let dynamic_expr_id = effect_fields.get(&CausalityStr::from_static_str(":dynamic_expr"))
                                                .and_then(value_expr_as_string)
                                                .and_then(|s| <ExprId as AsId>::from_hex(s.as_str()).ok());
                                            
                                            let scoped_handler_id = effect_fields.get(&CausalityStr::from_static_str(":scoped_handler"))
                                                .and_then(value_expr_as_string)
                                                .and_then(|s| <HandlerId as AsId>::from_hex(s.as_str()).ok());

                                            let mut constraints_vec = Vec::new();
                                            if let Some(constraints_list_val) = effect_fields.get(&CausalityStr::from_static_str(":constraints")) {
                                                if let Some(constraints_list) = value_expr_as_list(constraints_list_val) {
                                                    for constraint_val in constraints_list {
                                                        if let Some(s) = value_expr_as_string(constraint_val) {
                                                            match <ExprId as AsId>::from_hex(s.as_str()) {
                                                                Ok(expr_id) => constraints_vec.push(expr_id),
                                                                Err(_) => log::warn!("Invalid ExprId format for a constraint in intent Lisp for effect temp_id {:?}: {}", temp_effect_id_str, s),
                                                            }
                                                        } else {
                                                            log::warn!("Non-string value found in ':constraints' list for effect temp_id {:?}", temp_effect_id_str);
                                                        }
                                                    }
                                                } else {
                                                    log::warn!("Value for ':constraints' is not a list for effect temp_id {:?}", temp_effect_id_str);
                                                }
                                            }
                                        
                                            let mut inputs_vec = Vec::new();
                                            if let Some(inputs_list_val) = effect_fields.get(&CausalityStr::from_static_str(":inputs")) {
                                                if let Some(inputs_list) = value_expr_as_list(inputs_list_val) {
                                                    for input_val in inputs_list {
                                                        let mut resolved_input_res_id: Option<ResourceId> = None;
                                                        if let Some(s) = value_expr_as_string(input_val) {
                                                            // Direct ResourceId string
                                                            match <ResourceId as AsId>::from_hex(s.as_str()) {
                                                                Ok(res_id) => resolved_input_res_id = Some(res_id),
                                                                Err(_) => log::warn!("Invalid ResourceId format for an input string in intent Lisp for effect temp_id {:?}: {}", temp_effect_id_str, s),
                                                            }
                                                        } else if let Some(input_ref_record) = value_expr_as_record(input_val) {
                                                            // Symbolic reference like (record (':ref "temp-out-X"))
                                                            if let Some(ref_temp_id_str) = input_ref_record.get(&CausalityStr::from_static_str(":ref")).and_then(value_expr_as_string) {
                                                                if let Some((res_id, _type_expr_id)) = temp_output_id_to_resource_id_map.get(ref_temp_id_str) {
                                                                    resolved_input_res_id = Some(*res_id);
                                                                } else {
                                                                    log::warn!("Unresolved ':ref' ID '{}' for an input in intent Lisp for effect temp_id {:?}", ref_temp_id_str, temp_effect_id_str);
                                                                }
                                                            } else {
                                                                log::warn!("Missing or invalid ':ref' string in input reference record for effect temp_id {:?}", temp_effect_id_str);
                                                            }
                                                        } else {
                                                            log::warn!("Invalid value type in ':inputs' list for effect temp_id {:?}. Expected ResourceId string or ref record.", temp_effect_id_str);
                                                        }
                                                        if let Some(res_id) = resolved_input_res_id {
                                                            inputs_vec.push(res_id);
                                                        }
                                                    }
                                                } else {
                                                    log::warn!("Value for ':inputs' is not a list for effect temp_id {:?}", temp_effect_id_str);
                                                }
                                            }

                                            let mut outputs_vec: Vec<ResourceFlow> = Vec::new();
                                            if let Some(outputs_list_val) = effect_fields.get(&CausalityStr::from_static_str(":outputs")) {
                                                if let Some(outputs_list) = value_expr_as_list(outputs_list_val) {
                                                    for output_def_val in outputs_list {
                                                        if let Some(output_fields) = value_expr_as_record(output_def_val) {
                                                            let temp_output_id_str = output_fields.get(&CausalityStr::from_static_str(":id"))
                                                                .and_then(value_expr_as_string)
                                                                .cloned();
                                                            let type_expr_id_str = output_fields.get(&CausalityStr::from_static_str(":type_expr"))
                                                                .and_then(value_expr_as_string);

                                                            if let (Some(temp_id), Some(type_expr_s)) = (temp_output_id_str, type_expr_id_str) {
                                                                match <TypeExprId as AsId>::from_hex(type_expr_s.as_str()) {
                                                                    Ok(type_expr_id) => {
                                                                        outputs_vec.push(ResourceFlow::new(
                                                                            CausalityStr::from("output_resource"), // resource_type
                                                                            1, // quantity  
                                                                            intent_domain, // domain_id
                                                                        ));
                                                                        temp_output_id_to_resource_id_map.insert(temp_id, (create_unique_resource_id(), type_expr_id));
                                                                    }
                                                                    Err(_) => log::warn!("Invalid TypeExprId format for an output's ':type_expr' in intent Lisp for effect temp_id {:?}, output temp_id '{}'", temp_effect_id_str, temp_id),
                                                                }
                                                            } else {
                                                                log::warn!("Missing ':id' or ':type_expr' for an output definition in intent Lisp for effect temp_id {:?}", temp_effect_id_str);
                                                            }
                                                        } else {
                                                            log::warn!("Non-record value found in ':outputs' list for effect temp_id {:?}", temp_effect_id_str);
                                                        }
                                                    }
                                                } else {
                                                     log::warn!("Value for ':outputs' is not a list for effect temp_id {:?}", temp_effect_id_str);
                                                }
                                            }

                                            let actual_node_id = create_unique_node_id();
                                            if let Some(temp_id) = temp_effect_id_str {
                                                temp_id_to_node_id_map.insert(temp_id, actual_node_id);
                                            }
                                            
                                            let effect_type = effect_type_str.cloned().unwrap_or_else(|| {
                                                log::warn!("Effect type is missing for effect with temp_id {:?}, defaulting to empty string.", temp_effect_id_str);
                                                CausalityStr::default()
                                            });

                                            let effect = Effect {
                                                id: EntityId::new(actual_node_id.inner()), // Convert NodeId to EntityId
                                                name: CausalityStr::from(format!("effect_{}", actual_node_id.to_hex())),
                                                domain_id: intent_domain,    
                                                effect_type,
                                                inputs: vec![], // TODO: Convert inputs_vec to ResourceFlow
                                                outputs: outputs_vec,
                                                expression: dynamic_expr_id,
                                                timestamp: Timestamp::now(),
                                                hint: None, // Can be set based on intent.hint if needed
                                            };
                                            effects.push(effect);
                                        } else {
                                             log::warn!("Intent Lisp result: Expected a record for effect definition, got: {:?}", effect_def_val);
                                        }
                                    }
                                }
                            }

                            // Parse :edges
                            if let Some(edges_list_val) = graph_def_record_map.get(&CausalityStr::from_static_str(":edges")) {
                                if let Some(edge_defs) = value_expr_as_list(edges_list_val) {
                                    for edge_def_val in edge_defs {
                                        if let Some(edge_fields) = value_expr_as_record(edge_def_val) {
                                            let source_str_opt = edge_fields.get(&CausalityStr::from_static_str(":source")).and_then(value_expr_as_string);
                                            let target_str_opt = edge_fields.get(&CausalityStr::from_static_str(":target")).and_then(value_expr_as_string);
                                            let kind_val_opt = edge_fields.get(&CausalityStr::from_static_str(":kind"));
                                            let metadata_val_opt = edge_fields.get(&CausalityStr::from_static_str(":metadata")).cloned();

                                            let resolve_node_id_spec = |node_spec_val: Option<CausalityStr>| -> Option<NodeId> {
                                                match node_spec_val {
                                                    Some(s) => {
                                                        temp_id_to_node_id_map.get(&s).cloned()
                                                    },
                                                    None => None,
                                                }
                                            };
                                            
                                            let source_node_id_opt = resolve_node_id_spec(source_str_opt.cloned());
                                            let target_node_id_opt = resolve_node_id_spec(target_str_opt.cloned());

                                            if let (Some(source_node_id), Some(target_node_id), Some(kind_val)) = (source_node_id_opt, target_node_id_opt, kind_val_opt) {
                                                match parse_edge_kind_from_lisp(kind_val, &temp_id_to_node_id_map, &temp_id_to_handler_id_map, &temp_output_id_to_resource_id_map) {
                                                    Ok(edge_kind) => {
                                                        edges.push(Edge {
                                                            id: create_unique_edge_id(),
                                                            source: source_node_id,
                                                            target: target_node_id,
                                                            kind: edge_kind,
                                                            metadata: metadata_val_opt.map_or_else(BTreeMap::new, |val| {
                                                                let mut map = BTreeMap::new();
                                                                map.insert(CausalityStr::from_static_str("metadata"), val);
                                                                map
                                                            }),
                                                        });
                                                    }
                                                    Err(e) => log::warn!("Failed to parse EdgeKind for edge from {:?} to {:?}: {}. Skipping edge.", source_node_id, target_node_id, e),
                                                }
                                            } else {
                                                log::warn!("Missing :source, :target, or :kind for an edge definition. Source: {:?}, Target: {:?}. Skipping edge.", source_str_opt, target_str_opt);
                                            }
                                        }
                                    }
                                }
                            }

                            // Parse :handlers
                            if let Some(handlers_list_val) = graph_def_record_map.get(&CausalityStr::from_static_str(":handlers")) {
                                if let Some(handler_defs) = value_expr_as_list(handlers_list_val) {
                                    for handler_def_val in handler_defs {
                                        if let Some(handler_fields) = value_expr_as_record(handler_def_val) {
                                            let temp_handler_id_str = handler_fields.get(&CausalityStr::from_static_str(":id")).and_then(value_expr_as_string);
                                            let _name_str_opt = handler_fields.get(&CausalityStr::from_static_str(":name")).and_then(value_expr_as_string);
                                            let effect_type_str_opt = handler_fields.get(&CausalityStr::from_static_str(":handles_type")).and_then(value_expr_as_string);
                                            let priority_opt = handler_fields.get(&CausalityStr::from_static_str(":priority")).and_then(value_expr_as_int);
                                            let dynamic_expr_id_str_opt = handler_fields.get(&CausalityStr::from_static_str(":expression_id")).and_then(value_expr_as_string);
                                            let _ephemeral_opt = handler_fields.get(&CausalityStr::from_static_str(":ephemeral")).and_then(value_expr_as_bool);

                                            let mut constraints_vec: Vec<ExprId> = Vec::new();
                                            if let Some(constraints_list_val) = handler_fields.get(&CausalityStr::from_static_str(":constraints")) {
                                                if let Some(constraint_val_list) = value_expr_as_list(constraints_list_val) {
                                                    for constraint_val in constraint_val_list {
                                                        if let Some(s) = value_expr_as_string(constraint_val) {
                                                            match <ExprId as AsId>::from_hex(s.as_str()) {
                                                                Ok(expr_id) => constraints_vec.push(expr_id),
                                                                Err(_) => log::warn!("Invalid ExprId format for a constraint in handler constraints: {}", s),
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            if let (Some(effect_type_str), Some(priority_i64), Some(dynamic_expr_id_str)) = 
                                                (effect_type_str_opt, priority_opt, dynamic_expr_id_str_opt) {
                                                match <ExprId as AsId>::from_hex(dynamic_expr_id_str.as_str()) {
                                                    Ok(dynamic_expr_id) => {
                                                        let mut actual_handler_id = create_unique_handler_id();
                                                        if let Some(temp_id) = temp_handler_id_str {
                                                            actual_handler_id = *temp_id_to_handler_id_map.entry(*temp_id).or_insert_with(create_unique_handler_id);
                                                        }

                                                        let temp_handler = causality_types::tel::Handler {
                                                            id: EntityId::new(actual_handler_id.inner()),
                                                            name: CausalityStr::from(format!("handler_{}", actual_handler_id.to_hex())),
                                                            domain_id: intent_domain,
                                                            handles_type: *effect_type_str,
                                                            priority: priority_i64 as u32,
                                                            expression: Some(dynamic_expr_id),
                                                            timestamp: Timestamp::now(),
                                                        };
                                                        handlers_vec.push(temp_handler);
                                                    }
                                                    Err(e) => log::warn!("Invalid ExprId hex string for handler's :dynamic_expr '{}': {}. Skipping handler.", dynamic_expr_id_str, e),
                                                }
                                            } else {
                                                 log::warn!("Missing required fields (:handles_type, :priority, :expression_id) for handler with temp_id {:?}. Skipping handler.", temp_handler_id_str);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if eval_result != ValueExpr::Nil {
                            log::warn!("Intent {:?} dynamic_expr eval result was not a record as expected for graph generation: {:?}", intent.id, eval_result);
                            return Err(anyhow!("Intent dynamic_expr did not return a record for graph definition."));
                        }
                        
                        // Create a TEL Intent from our resource Intent

                        // Map the existing intent (resource type) to the unified Intent type
                        let tel_intent = causality_types::effect::intent::Intent {
                            id: intent.id,
                            name: intent.name,
                            domain_id: intent_domain,
                            priority: 1, // Default priority
                            inputs: vec![], // Parse from intent  
                            outputs: vec![], // Parse from intent  
                            expression: intent.expression, // Match types (Option<ExprId>)
                            timestamp: Timestamp::now(),
                            hint: None, // No hint for now
                        };

                        // Use in graph creation
                        let graph = EffectGraph {
                            id: EntityId::new(intent.id.inner()),
                            domain_id: intent_domain,
                            nodes: BTreeMap::new(), // TODO: Convert effects to nodes
                            edges: BTreeMap::new(), // TODO: Convert edges to BTreeMap
                            metadata: BTreeMap::new(),
                        };
                        Ok(graph)
                    }
                    Err(e) => {
                        log::error!("Error evaluating dynamic_expr for intent {:?}: {}", intent.id, e);
                        Err(anyhow!("Intent dynamic_expr evaluation failed: {}", e))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
}

// Helper function to parse EdgeKind from Lisp ValueExpr
// This needs to be defined in this file, or imported if it's common.
// For now, define it locally.
fn parse_edge_kind_from_lisp(
    kind_val: &ValueExpr,
    temp_node_ids: &BTreeMap<CausalityStr, NodeId>,
    temp_handler_ids: &BTreeMap<CausalityStr, HandlerId>,
    temp_output_ids: &BTreeMap<CausalityStr, (ResourceId, TypeExprId)>,
) -> Result<EdgeKind> {
    let kind_record = value_expr_as_record(kind_val)
        .ok_or_else(|| anyhow!("EdgeKind definition must be a record. Got: {:?}", kind_val))?;

    let kind_type_str = kind_record.get(&CausalityStr::from_static_str(":type"))
        .and_then(value_expr_as_string)
        .ok_or_else(|| anyhow!("EdgeKind record missing ':type' field"))?;

    let resolve_node_id = |key: &CausalityStr| -> Result<NodeId> {
        kind_record.get(key)
            .and_then(|val| {
                if let Some(s) = value_expr_as_string(val) {
                    temp_node_ids.get(s.as_str()).cloned().or_else(|| <NodeId as AsId>::from_hex(s.as_str()).ok())
                } else if let Some(rec) = value_expr_as_record(val) {
                                            rec.get(&CausalityStr::from_static_str(":ref")).and_then(value_expr_as_string).and_then(|s_ref| temp_node_ids.get(s_ref.as_str()).cloned())
                } else { None }
            })
            .ok_or_else(|| anyhow!("Missing or invalid NodeId for key '{:?}' in EdgeKind", key))
    };

    let resolve_handler_id = |key: &CausalityStr| -> Result<HandlerId> {
        kind_record.get(key)
            .and_then(|val| {
                 if let Some(s) = value_expr_as_string(val) {
                    temp_handler_ids.get(s.as_str()).cloned().or_else(|| <HandlerId as AsId>::from_hex(s.as_str()).ok())
                } else if let Some(rec) = value_expr_as_record(val) {
                                            rec.get(&CausalityStr::from_static_str(":ref")).and_then(value_expr_as_string).and_then(|s_ref| temp_handler_ids.get(s_ref.as_str()).cloned())
                } else { None }
            })
            .ok_or_else(|| anyhow!("Missing or invalid HandlerId for key '{:?}' in EdgeKind", key))
    };
    
    let resolve_resource_ref = |key: &CausalityStr| -> Result<ResourceRef> {
        kind_record.get(key)
            .and_then(|val| {
                if let Some(s) = value_expr_as_string(val) {
                    temp_output_ids.get(s.as_str()).map(|(res_id, _)| (*res_id).into())
                     .or_else(|| <ResourceId as AsId>::from_hex(s.as_str()).ok().map(Into::into))
                } else if let Some(rec) = value_expr_as_record(val) {
                    rec.get(&CausalityStr::from_static_str(":ref"))
                        .and_then(value_expr_as_string)
                        .and_then(|ref_s| temp_output_ids.get(ref_s.as_str()).map(|(res_id, _)| (*res_id).into()))
                } else { None }
            })
            .ok_or_else(|| anyhow!("Missing or invalid ResourceRef for key '{:?}' in EdgeKind", key))
    };

    match kind_type_str.as_ref() {
        b"ControlFlow" => Ok(EdgeKind::ControlFlow),
        b"Next" => Ok(EdgeKind::Next(resolve_node_id(&CausalityStr::from_static_str(":node_id"))?)),
        b"DependsOn" => Ok(EdgeKind::DependsOn(resolve_node_id(&CausalityStr::from_static_str(":dependency_node_id"))?)),
        b"Consumes" => Ok(EdgeKind::Consumes(resolve_resource_ref(&CausalityStr::from_static_str(":resource_ref"))?)),
        b"Produces" => Ok(EdgeKind::Produces(resolve_resource_ref(&CausalityStr::from_static_str(":resource_ref"))?)),
        b"Applies" => Ok(EdgeKind::Applies(resolve_handler_id(&CausalityStr::from_static_str(":handler_id"))?)),
        b"ScopedBy" => Ok(EdgeKind::ScopedBy(resolve_handler_id(&CausalityStr::from_static_str(":handler_id"))?)),
        b"Override" => Ok(EdgeKind::Override(resolve_handler_id(&CausalityStr::from_static_str(":handler_id"))?)),
        _ => Err(anyhow!("Unknown EdgeKind type: {}", kind_type_str.to_string())),
    }
}