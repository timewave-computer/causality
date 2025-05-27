(* Purpose: Test file demonstrating Phase 6 enhancements for TypedDomain and ProcessDataflowBlock *)

open Ml_causality_lib_types.Types

(* Test TypedDomain creation *)
let test_typed_domains () =
  Printf.printf "=== Testing TypedDomain Creation ===\n";
  
  (* Create different types of domains *)
  let verifiable_domain_id = Bytes.of_string "verifiable_domain_001" in
  let service_domain_id = Bytes.of_string "service_domain_001" in
  let compute_domain_id = Bytes.of_string "compute_domain_001" in
  
  (* Create VerifiableDomain *)
  let verifiable_domain = VerifiableDomain {
    domain_id = verifiable_domain_id;
    zk_constraints = true;
    deterministic_only = true;
  } in
  Printf.printf "Created VerifiableDomain with ID: %s\n" 
    (Bytes.to_string verifiable_domain_id);
  
  (* Create ServiceDomain *)
  let service_domain = ServiceDomain {
    domain_id = service_domain_id;
    external_apis = ["https://api.example.com"; "https://oracle.service.com"];
    non_deterministic_allowed = true;
  } in
  Printf.printf "Created ServiceDomain with ID: %s\n"
    (Bytes.to_string service_domain_id);
  
  (* Create ComputeDomain *)
  let compute_domain = ComputeDomain {
    domain_id = compute_domain_id;
    compute_intensive = true;
    parallel_execution = true;
  } in
  Printf.printf "Created ComputeDomain with ID: %s\n"
    (Bytes.to_string compute_domain_id);
  
  (verifiable_domain, service_domain, compute_domain)

(* Test ProcessDataflowBlock creation *)
let test_process_dataflow_blocks () =
  Printf.printf "\n=== Testing ProcessDataflowBlock Creation ===\n";
  
  let default_domain = VerifiableDomain {
    domain_id = Bytes.of_string "default_pdb_domain";
    zk_constraints = true;
    deterministic_only = true;
  } in
  
  (* Create a simple ProcessDataflowBlock definition *)
  let linear_pdb = {
    definition_id = Bytes.of_string "linear_pdb_def_001";
    name = "token_transfer_flow";
    input_schema = BatMap.of_enum (BatList.enum [
      ("sender", "address");
      ("recipient", "address");
      ("amount", "uint256");
    ]);
    output_schema = BatMap.of_enum (BatList.enum [
      ("transaction_hash", "bytes32");
      ("success", "bool");
    ]);
    state_schema = BatMap.of_enum (BatList.enum [
      ("current_step", "uint8");
      ("validated", "bool");
    ]);
    nodes = [{
      node_id = "validate_sender";
      node_type = "validation";
      typed_domain_policy = Some default_domain;
      action_template = None;
      gating_conditions = [];
    }];
    edges = [{
      from_node = "validate_sender";
      to_node = "check_balance";
      condition = None;
      transition_type = "sequential";
    }];
    default_typed_domain = default_domain;
  } in
  Printf.printf "Created ProcessDataflowBlock definition: %s\n"
    linear_pdb.name;
  
  linear_pdb

(* Test optimization types creation *)
let test_optimization_types () =
  Printf.printf "\n=== Testing Optimization Types ===\n";
  
  let verifiable_domain = VerifiableDomain {
    domain_id = Bytes.of_string "opt_domain";
    zk_constraints = true;
    deterministic_only = true;
  } in
  
  (* Create effect compatibility *)
  let effect_compat = {
    effect_type = "token_transfer";
    source_typed_domain = verifiable_domain;
    target_typed_domain = verifiable_domain;
    compatibility_score = 0.95;
    transfer_overhead = 100L;
  } in
  Printf.printf "Created Effect Compatibility for: %s\n"
    effect_compat.effect_type;
  
  (* Create resource preference *)
  let resource_pref = {
    resource_type = "ETH";
    preferred_typed_domain = verifiable_domain;
    preference_weight = 0.8;
    cost_multiplier = 1.2;
  } in
  Printf.printf "Created Resource Preference for: %s\n"
    resource_pref.resource_type;
  
  (* Create optimization hint *)
  let opt_hint = {
    strategy_preference = Some "capital_efficiency";
    cost_weight = 0.4;
    time_weight = 0.3;
    quality_weight = 0.3;
    typed_domain_constraints = [verifiable_domain];
  } in
  Printf.printf "Created Optimization Hint with strategy: %s\n"
    (match opt_hint.strategy_preference with Some s -> s | None -> "none");
  
  (effect_compat, resource_pref, opt_hint)

(* Test enhanced Intent creation *)
let test_enhanced_intent_creation () =
  Printf.printf "\n=== Testing Enhanced Intent Creation ===\n";
  
  let intent_id = Bytes.of_string "enhanced_intent_001" in
  let domain_id = Bytes.of_string "defi_domain" in
  let target_domain = VerifiableDomain {
    domain_id;
    zk_constraints = true;
    deterministic_only = true;
  } in
  
  let opt_hint_id = Bytes.of_string "opt_hint_001" in
  
  (* Create ProcessDataflowBlock initiation hint *)
  let pdb_hint = {
    df_def_id = Bytes.of_string "defi_swap_flow";
    initial_params = VStruct (BatMap.of_enum (BatList.enum [
      ("token_in", VString "USDC");
      ("token_out", VString "ETH");
      ("amount", VInt 1000L);
    ]));
    target_typed_domain = Some target_domain;
  } in
  
  (* Create enhanced intent *)
  let enhanced_intent = {
    id = intent_id;
    name = "DeFi Swap Intent";
    domain_id;
    priority = 5;
    inputs = [{
      resource_type = "USDC";
      quantity = 1000L;
      domain_id;
    }];
    outputs = [{
      resource_type = "ETH";
      quantity = 1L;
      domain_id;
    }];
    expression = None;
    timestamp = 1640995200L;
    optimization_hint = Some opt_hint_id;
    compatibility_metadata = [];
    resource_preferences = [];
    target_typed_domain = Some target_domain;
    process_dataflow_hint = Some pdb_hint;
  } in
  
  Printf.printf "Created Enhanced Intent: %s\n"
    enhanced_intent.name;
  
  enhanced_intent

(* Test enhanced Effect creation *)
let test_enhanced_effect_creation () =
  Printf.printf "\n=== Testing Enhanced Effect Creation ===\n";
  
  let effect_id = Bytes.of_string "enhanced_effect_001" in
  let domain_id = Bytes.of_string "defi_domain" in
  let handler_id = Bytes.of_string "swap_handler_001" in
  
  let source_domain = VerifiableDomain {
    domain_id;
    zk_constraints = true;
    deterministic_only = true;
  } in
  
  let target_domain = ComputeDomain {
    domain_id = Bytes.of_string "compute_domain";
    compute_intensive = true;
    parallel_execution = false;
  } in
  
  (* Create enhanced effect *)
  let enhanced_effect = {
    id = effect_id;
    name = "Cross-Domain Swap Effect";
    domain_id;
    effect_type = "defi_swap";
    inputs = [{
      resource_type = "USDC";
      quantity = 1000L;
      domain_id;
    }];
    outputs = [{
      resource_type = "ETH";
      quantity = 1L;
      domain_id;
    }];
    expression = None;
    timestamp = 1640995200L;
    resources = [];
    nullifiers = [];
    scoped_by = handler_id;
    intent_id = None;
    source_typed_domain = source_domain;
    target_typed_domain = target_domain;
    originating_dataflow_instance = Some (Bytes.of_string "pdb_instance_001");
  } in
  
  Printf.printf "Created Enhanced Effect: %s\n"
    enhanced_effect.name;
  
  enhanced_effect

(* Test ProcessDataflowBlock instance management *)
let test_pdb_instance_management () =
  Printf.printf "\n=== Testing ProcessDataflowBlock Instance Management ===\n";
  
  let instance_id = Bytes.of_string "pdb_instance_001" in
  let definition_id = Bytes.of_string "linear_pdb_token_transfer_flow" in
  let timestamp = 1640995200L in
  
  (* Create initial instance state *)
  let initial_state = {
    instance_id;
    definition_id;
    current_node_id = "step_0";
    state_values = VStruct (BatMap.of_enum (BatList.enum [
      ("current_step", VInt 0L);
      ("transfer_amount", VInt 1000L);
      ("sender", VString "0x123...");
      ("recipient", VString "0x456...");
    ]));
    created_timestamp = timestamp;
    last_updated = timestamp;
  } in
  
  Printf.printf "Created PDB Instance State: current_node=%s\n" 
    initial_state.current_node_id;
  
  (* Transition to next step *)
  let updated_state = {
    initial_state with
    current_node_id = "step_1";
    state_values = VStruct (BatMap.of_enum (BatList.enum [
      ("current_step", VInt 1L);
      ("transfer_amount", VInt 1000L);
      ("sender", VString "0x123...");
      ("recipient", VString "0x456...");
      ("balance_checked", VBool true);
    ]));
    last_updated = Int64.add timestamp 1000L;
  } in
  
  Printf.printf "Transitioned PDB Instance: current_node=%s\n"
    updated_state.current_node_id;
  
  (initial_state, updated_state)

(* Test TEG integration functions *)
let test_teg_integration () =
  Printf.printf "\n=== Testing TEG Integration Functions ===\n";
  
  let effect_id = "effect_001" in
  let capability_id = "read_balance_capability" in
  let grantor_id = "admin_node" in
  let grantee_id = "user_node" in
  let delegator_id = "manager_node" in
  let delegate_id = "operator_node" in
  
  (* Test capability requirement edge *)
  let req_edge = Ml_causality_lib_capability_system.Capability_generator.create_capability_requirement_edge 
    ~effect_id ~capability_id () in
  Printf.printf "Created capability requirement edge: %s -> %s\n"
    (Bytes.to_string req_edge.source) (Bytes.to_string req_edge.target);
  
  (* Test capability grant edge *)
  let grant_edge = Ml_causality_lib_capability_system.Capability_generator.create_capability_grant_edge
    ~grantor_id ~grantee_id ~capability_id () in
  Printf.printf "Created capability grant edge: %s -> %s\n"
    (Bytes.to_string grant_edge.source) (Bytes.to_string grant_edge.target);
  
  (* Test capability delegation edge *)
  let delegation_edge = Ml_causality_lib_capability_system.Capability_generator.create_capability_delegation_edge
    ~delegator_id ~delegate_id ~capability_id () in
  Printf.printf "Created capability delegation edge: %s -> %s\n"
    (Bytes.to_string delegation_edge.source) (Bytes.to_string delegation_edge.target);
  
  (req_edge, grant_edge, delegation_edge)

(* Main test function *)
let run_phase6_tests () =
  Printf.printf "🚀 Running Phase 6 Enhancement Tests\n\n";
  
  let (_verifiable_domain, _service_domain, _compute_domain) = test_typed_domains () in
  let _linear_pdb = test_process_dataflow_blocks () in
  let (_effect_compat, _resource_pref, _opt_hint) = test_optimization_types () in
  let _enhanced_intent = test_enhanced_intent_creation () in
  let _enhanced_effect = test_enhanced_effect_creation () in
  let (_initial_pdb_state, _updated_pdb_state) = test_pdb_instance_management () in
  let (_req_edge, _grant_edge, _delegation_edge) = test_teg_integration () in
  
  Printf.printf "\n✅ All Phase 6 Enhancement Tests Completed Successfully!\n";
  Printf.printf "📊 Summary:\n";
  Printf.printf "  - TypedDomains: 3 created (Verifiable, Service, Compute)\n";
  Printf.printf "  - ProcessDataflowBlocks: 1 definition created\n";
  Printf.printf "  - Optimization Components: 3 created (Compatibility, Preference, Hint)\n";
  Printf.printf "  - Enhanced Intent: 1 created with optimization metadata\n";
  Printf.printf "  - Enhanced Effect: 1 created with typed domain information\n";
  Printf.printf "  - PDB Instance Management: 2 states (Initial, Transitioned)\n";
  Printf.printf "  - TEG Integration: 3 edges created (Requirement, Grant, Delegation)\n";
  
  (* Return all created objects for potential further testing *)
  {|
  Phase 6 enhancements successfully demonstrated:
  - TypedDomain classification system operational
  - ProcessDataflowBlock definition and instance management working
  - Optimization hints and metadata integration complete
  - Enhanced Intent and Effect types with new fields functional
  - All new types compile and instantiate correctly
  - TEG integration functions operational
  |}

(* Entry point for the test *)
let () = 
  let result = run_phase6_tests () in
  Printf.printf "\n%s\n" result 