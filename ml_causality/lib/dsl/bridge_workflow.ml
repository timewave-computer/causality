(* Purpose: Complete bridge workflow using ProcessDataflowBlock and intent-based optimization *)

open Ml_causality_lib_types.Types

(*-----------------------------------------------------------------------------
 * Helper Functions (avoiding circular dependency)
 *-----------------------------------------------------------------------------*)

(** Create optimization hint directly *)
let create_optimization_hint_direct ~strategy_preference ~cost_weight ~time_weight ~quality_weight ~typed_domain_constraints () =
  {
    strategy_preference = Some strategy_preference;
    cost_weight;
    time_weight;
    quality_weight;
    typed_domain_constraints;
  }

(** Create enhanced intent directly *)
let create_enhanced_intent_direct ~name ~domain_id ~priority ~inputs ~outputs ~optimization_hint ~compatibility_metadata ~resource_preferences ~target_typed_domain ~process_dataflow_hint () =
  let intent_id = Bytes.of_string (Printf.sprintf "intent_%s_%d" name priority) in
  {
    id = intent_id;
    name;
    domain_id;
    priority;
    inputs;
    outputs;
    expression = None;
    timestamp = 0L;
    optimization_hint;
    compatibility_metadata;
    resource_preferences;
    target_typed_domain;
    process_dataflow_hint;
  }

(** Create PDB instance state directly *)
let create_pdb_instance_state_direct ~definition_id ~initial_node_id ~initial_state () =
  let instance_id = Bytes.of_string (Printf.sprintf "instance_%s_%Ld" 
    (Bytes.to_string definition_id) (Int64.of_float (Unix.time ()))) in
  let current_time = Int64.of_float (Unix.time ()) in
  {
    instance_id;
    definition_id;
    current_node_id = initial_node_id;
    state_values = initial_state;
    created_timestamp = current_time;
    last_updated = current_time;
  }

(** Transition PDB instance directly *)
let transition_pdb_instance_direct ~instance_state ~target_node_id ~new_state () =
  let current_time = Int64.of_float (Unix.time ()) in
  {
    instance_state with
    current_node_id = target_node_id;
    state_values = new_state;
    last_updated = current_time;
  }

(*-----------------------------------------------------------------------------
 * Bridge Workflow ProcessDataflowBlock Definition
 *-----------------------------------------------------------------------------*)

(** Create the complete bridge transfer workflow as a ProcessDataflowBlock *)
let create_bridge_transfer_workflow ~(bridge_config : Bridge_primitives.bridge_config) () =
  let workflow_id = Bytes.of_string (Printf.sprintf "bridge_workflow_%s" bridge_config.name) in
  
  (* Define the workflow nodes *)
  let validate_node = {
    node_id = "validate_transfer";
    node_type = "validation";
    typed_domain_policy = Some (VerifiableDomain {
      domain_id = bridge_config.source_domain;
      zk_constraints = true;
      deterministic_only = true;
    });
    action_template = Some (Bytes.of_string "validate_transfer_action");
    gating_conditions = [Bytes.of_string "sufficient_balance"; Bytes.of_string "valid_amount_range"];
  } in

  let lock_tokens_node = {
    node_id = "lock_tokens";
    node_type = "token_operation";
    typed_domain_policy = Some (VerifiableDomain {
      domain_id = bridge_config.source_domain;
      zk_constraints = true;
      deterministic_only = true;
    });
    action_template = Some (Bytes.of_string "lock_tokens_action");
    gating_conditions = [Bytes.of_string "validation_passed"];
  } in

  let relay_message_node = {
    node_id = "relay_message";
    node_type = "cross_domain_message";
    typed_domain_policy = Some (ServiceDomain {
      domain_id = Bytes.of_string "messaging_layer";
      external_apis = ["bridge_relay"; "proof_verification"];
      non_deterministic_allowed = true;
    });
    action_template = Some (Bytes.of_string "relay_proof_action");
    gating_conditions = [Bytes.of_string "tokens_locked"];
  } in

  let verify_proof_node = {
    node_id = "verify_proof";
    node_type = "verification";
    typed_domain_policy = Some (VerifiableDomain {
      domain_id = bridge_config.target_domain;
      zk_constraints = true;
      deterministic_only = true;
    });
    action_template = Some (Bytes.of_string "verify_proof_action");
    gating_conditions = [Bytes.of_string "proof_received"];
  } in

  let mint_tokens_node = {
    node_id = "mint_tokens";
    node_type = "token_operation";
    typed_domain_policy = Some (VerifiableDomain {
      domain_id = bridge_config.target_domain;
      zk_constraints = true;
      deterministic_only = true;
    });
    action_template = Some (Bytes.of_string "mint_tokens_action");
    gating_conditions = [Bytes.of_string "proof_verified"];
  } in

  let complete_transfer_node = {
    node_id = "complete_transfer";
    node_type = "finalization";
    typed_domain_policy = Some (VerifiableDomain {
      domain_id = bridge_config.target_domain;
      zk_constraints = true;
      deterministic_only = true;
    });
    action_template = Some (Bytes.of_string "complete_transfer_action");
    gating_conditions = [Bytes.of_string "tokens_minted"];
  } in

  (* Define workflow edges *)
  let edges = [
    {
      from_node = validate_node.node_id;
      to_node = lock_tokens_node.node_id;
      condition = Some (Bytes.of_string "validation_success");
      transition_type = "success";
    };
    {
      from_node = lock_tokens_node.node_id;
      to_node = relay_message_node.node_id;
      condition = Some (Bytes.of_string "lock_success");
      transition_type = "success";
    };
    {
      from_node = relay_message_node.node_id;
      to_node = verify_proof_node.node_id;
      condition = Some (Bytes.of_string "relay_success");
      transition_type = "cross_domain";
    };
    {
      from_node = verify_proof_node.node_id;
      to_node = mint_tokens_node.node_id;
      condition = Some (Bytes.of_string "verification_success");
      transition_type = "success";
    };
    {
      from_node = mint_tokens_node.node_id;
      to_node = complete_transfer_node.node_id;
      condition = Some (Bytes.of_string "mint_success");
      transition_type = "success";
    };
  ] in

  (* Define input/output schemas *)
  let input_schema = BatMap.of_enum (BatList.enum [
    ("source_account", "bytes");
    ("target_account", "bytes");
    ("amount", "int64");
    ("bridge_config", "bridge_config");
  ]) in

  let output_schema = BatMap.of_enum (BatList.enum [
    ("transfer_id", "bytes");
    ("status", "transfer_status");
    ("final_amount", "int64");
    ("fee_paid", "int64");
  ]) in

  let state_schema = BatMap.of_enum (BatList.enum [
    ("current_node", "bytes");
    ("transfer_metadata", "bridge_transfer_metadata");
    ("locked_amount", "int64");
    ("proof_data", "bytes");
  ]) in

  (* Create the ProcessDataflowBlock definition *)
  {
    definition_id = workflow_id;
    name = Printf.sprintf "BridgeTransferWorkflow_%s" bridge_config.name;
    input_schema;
    output_schema;
    state_schema;
    nodes = [validate_node; lock_tokens_node; relay_message_node; verify_proof_node; mint_tokens_node; complete_transfer_node];
    edges;
    default_typed_domain = VerifiableDomain {
      domain_id = bridge_config.source_domain;
      zk_constraints = true;
      deterministic_only = true;
    };
  }

(*-----------------------------------------------------------------------------
 * Intent-Based Bridge Transfer with Optimization
 *-----------------------------------------------------------------------------*)

(** Create an optimized bridge transfer intent *)
let create_optimized_bridge_transfer_intent ~(bridge_config : Bridge_primitives.bridge_config) ~source_account ~target_account ~amount ~domain_id () =

  (* Create effect compatibility metadata *)
  let compatibility_metadata = [
    {
      effect_type = "token_lock";
      source_typed_domain = VerifiableDomain {
        domain_id = bridge_config.source_domain;
        zk_constraints = true;
        deterministic_only = true;
      };
      target_typed_domain = VerifiableDomain {
        domain_id = bridge_config.source_domain;
        zk_constraints = true;
        deterministic_only = true;
      };
      compatibility_score = 1.0;
      transfer_overhead = 0L;
    };
    {
      effect_type = "cross_domain_message";
      source_typed_domain = VerifiableDomain {
        domain_id = bridge_config.source_domain;
        zk_constraints = true;
        deterministic_only = true;
      };
      target_typed_domain = ServiceDomain {
        domain_id = Bytes.of_string "messaging_layer";
        external_apis = ["bridge_relay"];
        non_deterministic_allowed = true;
      };
      compatibility_score = 0.8;
      transfer_overhead = 100L; (* Some overhead for cross-domain messaging *)
    };
    {
      effect_type = "token_mint";
      source_typed_domain = ServiceDomain {
        domain_id = Bytes.of_string "messaging_layer";
        external_apis = ["bridge_relay"];
        non_deterministic_allowed = true;
      };
      target_typed_domain = VerifiableDomain {
        domain_id = bridge_config.target_domain;
        zk_constraints = true;
        deterministic_only = true;
      };
      compatibility_score = 0.9;
      transfer_overhead = 50L;
    };
  ] in

  (* Create resource preferences *)
  let resource_preferences = [
    {
      resource_type = "token_balance";
      preferred_typed_domain = VerifiableDomain {
        domain_id = bridge_config.source_domain;
        zk_constraints = true;
        deterministic_only = true;
      };
      preference_weight = 1.0;
      cost_multiplier = 1.0;
    };
    {
      resource_type = "bridge_transfer";
      preferred_typed_domain = VerifiableDomain {
        domain_id = bridge_config.source_domain;
        zk_constraints = true;
        deterministic_only = true;
      };
      preference_weight = 0.8;
      cost_multiplier = 1.2;
    };
  ] in

  (* Create ProcessDataflowBlock initiation hint *)
  let workflow = create_bridge_transfer_workflow ~bridge_config () in
  let dataflow_hint = {
    df_def_id = workflow.definition_id;
    initial_params = VStruct (BatMap.of_enum (BatList.enum [
      ("source_account", VString (Bytes.to_string source_account));
      ("target_account", VString (Bytes.to_string target_account));
      ("amount", VInt amount);
      ("bridge_config", VString (Bytes.to_string bridge_config.bridge_id));
    ]));
    target_typed_domain = Some (VerifiableDomain {
      domain_id = bridge_config.target_domain;
      zk_constraints = true;
      deterministic_only = true;
    });
  } in

  (* Calculate fee for input amount *)
  let fee = Int64.div (Int64.mul amount (Int64.of_int bridge_config.fee_basis_points)) 10000L in

  (* Create the optimized intent *)
  create_enhanced_intent_direct
    ~name:"OptimizedBridgeTransfer"
    ~domain_id
    ~priority:5
    ~inputs:[{
      resource_type = "token_balance";
      quantity = Int64.add amount fee;
      domain_id = bridge_config.source_domain;
    }]
    ~outputs:[{
      resource_type = "token_balance";
      quantity = amount;
      domain_id = bridge_config.target_domain;
    }]
    ~optimization_hint:None
    ~compatibility_metadata
    ~resource_preferences
    ~target_typed_domain:(Some (VerifiableDomain {
      domain_id = bridge_config.target_domain;
      zk_constraints = true;
      deterministic_only = true;
    }))
    ~process_dataflow_hint:(Some dataflow_hint)
    ()

(*-----------------------------------------------------------------------------
 * Complete Bridge Transfer Example
 *-----------------------------------------------------------------------------*)

(** Execute a complete bridge transfer workflow *)
let execute_bridge_transfer_example () =
  Printf.printf "🌉 Executing Complete Bridge Transfer Example\n\n";

  (* Step 1: Create domains *)
  let ethereum_domain = Bytes.of_string "ethereum_domain" in
  let polygon_domain = Bytes.of_string "polygon_domain" in
  
  Printf.printf "📍 Created domains: Ethereum and Polygon\n";

  (* Step 2: Create tokens *)
  let (eth_token_config, eth_token_resource) = Token_primitives.create_token
    ~name:"Ethereum Token"
    ~symbol:"ETH"
    ~decimals:18
    ~total_supply:1000000L
    ~domain_id:ethereum_domain () in

  let (poly_token_config, poly_token_resource) = Token_primitives.create_token
    ~name:"Polygon Token"
    ~symbol:"POLY"
    ~decimals:18
    ~total_supply:1000000L
    ~domain_id:polygon_domain () in

  Printf.printf "🪙 Created tokens: ETH on Ethereum, POLY on Polygon\n";

  (* Step 3: Create bridge *)
  let (bridge_config, bridge_resource) = Bridge_primitives.create_bridge
    ~name:"ETH_POLY_Bridge"
    ~source_domain:ethereum_domain
    ~target_domain:polygon_domain
    ~source_token:eth_token_config.token_id
    ~target_token:poly_token_config.token_id
    ~fee_basis_points:30 (* 0.3% fee *)
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in

  Printf.printf "🌉 Created bridge: %s\n" bridge_config.name;

  (* Step 4: Create user accounts *)
  let alice_account = Bytes.of_string "alice_account" in
  let bob_account = Bytes.of_string "bob_account" in

  (* Step 5: Create initial token balances *)
  let (alice_eth_balance, _) = Token_primitives.create_token_balance
    ~account_id:alice_account
    ~token_id:eth_token_config.token_id
    ~initial_balance:10000L
    ~domain_id:ethereum_domain () in

  Printf.printf "💰 Alice has %Ld ETH on Ethereum\n" alice_eth_balance.balance;

  (* Step 6: Create the bridge workflow *)
  let workflow = create_bridge_transfer_workflow ~bridge_config () in
  Printf.printf "⚙️  Created bridge workflow with %d nodes\n" (List.length workflow.nodes);

  (* Step 7: Create optimized transfer intent *)
  let transfer_amount = 5000L in
  let optimized_intent = create_optimized_bridge_transfer_intent
    ~bridge_config
    ~source_account:alice_account
    ~target_account:bob_account
    ~amount:transfer_amount
    ~domain_id:ethereum_domain () in

  Printf.printf "🎯 Created optimized intent for %Ld ETH transfer\n" transfer_amount;

  (* Step 8: Create workflow instance *)
  let instance_state = create_pdb_instance_state_direct
    ~definition_id:workflow.definition_id
    ~initial_node_id:(List.hd workflow.nodes).node_id
    ~initial_state:(VStruct (BatMap.of_enum (BatList.enum [
      ("transfer_amount", VInt transfer_amount);
      ("source_account", VString (Bytes.to_string alice_account));
      ("target_account", VString (Bytes.to_string bob_account));
      ("status", VString "Initiated");
    ]))) () in

  Printf.printf "🔄 Created workflow instance: %s\n" (Bytes.to_string instance_state.instance_id);

  (* Step 9: Simulate workflow execution *)
  Printf.printf "\n📋 Workflow Execution Steps:\n";
  List.iteri (fun i node ->
    Printf.printf "  %d. %s (%s)\n" (i+1) 
      (match node.action_template with 
       | Some action_id -> Bytes.to_string action_id
       | None -> "no_action")
      node.node_type
  ) workflow.nodes;

  Printf.printf "\n✅ Bridge transfer workflow example completed!\n";
  Printf.printf "📊 Summary:\n";
  Printf.printf "  - Bridge: %s\n" bridge_config.name;
  Printf.printf "  - Transfer: %Ld ETH → %Ld POLY\n" transfer_amount transfer_amount;
  let fee = Int64.div (Int64.mul transfer_amount (Int64.of_int bridge_config.fee_basis_points)) 10000L in
  Printf.printf "  - Fee: %Ld ETH\n" fee;
  Printf.printf "  - Workflow nodes: %d\n" (List.length workflow.nodes);
  Printf.printf "  - Optimization hints: %d\n" (List.length optimized_intent.compatibility_metadata);

  (* Return the complete workflow setup *)
  (workflow, optimized_intent, instance_state) 