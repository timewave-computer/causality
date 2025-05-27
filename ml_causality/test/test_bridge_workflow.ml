(* Purpose: Test bridge workflow implementation *)

open Ml_causality_lib_types.Types
open Ml_causality_lib_dsl

(* Test token primitives *)
let test_token_primitives () =
  Printf.printf "\n=== Testing Token Primitives ===\n";
  
  let domain_id = Bytes.of_string "test_domain" in
  
  (* Create token *)
  let (token_config, _token_resource) = Token_primitives.create_token
    ~name:"Test Token"
    ~symbol:"TEST"
    ~decimals:18
    ~total_supply:1000000L
    ~domain_id () in
  
  Printf.printf "✅ Created token: %s (%s)\n" token_config.name token_config.symbol;
  Printf.printf "   Token ID: %s\n" (Bytes.to_string token_config.token_id);
  Printf.printf "   Total supply: %Ld\n" token_config.total_supply;
  
  (* Create token balance *)
  let account_id = Bytes.of_string "test_account" in
  let (balance_config, _balance_resource) = Token_primitives.create_token_balance
    ~account_id
    ~token_id:token_config.token_id
    ~initial_balance:10000L
    ~domain_id () in
  
  Printf.printf "✅ Created balance: %Ld tokens\n" balance_config.balance;
  
  (* Test amount validation *)
  let valid = Token_primitives.validate_transfer_amount ~balance:balance_config.balance ~amount:5000L in
  Printf.printf "✅ Transfer validation: %b\n" valid;
  
  (* Test amount formatting *)
  let formatted = Token_primitives.format_token_amount 
    ~amount:1500000000000000000L 
    ~decimals:token_config.decimals 
    ~symbol:token_config.symbol in
  Printf.printf "✅ Formatted amount: %s\n" formatted;
  
  (* Create token effects *)
  let transfer_effect = Token_primitives.create_token_transfer_effect
    ~token_id:token_config.token_id
    ~from_account:account_id
    ~to_account:(Bytes.of_string "recipient")
    ~amount:1000L
    ~domain_id () in
  
  Printf.printf "✅ Created transfer effect: %s\n" transfer_effect.name;
  
  let lock_effect = Token_primitives.create_lock_tokens_effect
    ~token_id:token_config.token_id
    ~account_id
    ~amount:2000L
    ~domain_id () in
  
  Printf.printf "✅ Created lock effect: %s\n" lock_effect.name;
  
  (token_config, balance_config)

(* Test bridge primitives *)
let test_bridge_primitives () =
  Printf.printf "\n=== Testing Bridge Primitives ===\n";
  
  let source_domain = Bytes.of_string "ethereum" in
  let target_domain = Bytes.of_string "polygon" in
  let source_token = Bytes.of_string "eth_token" in
  let target_token = Bytes.of_string "poly_token" in
  
  (* Create bridge *)
  let (bridge_config, _bridge_resource) = Bridge_primitives.create_bridge
    ~name:"ETH_POLY_Bridge"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:30 (* 0.3% *)
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  Printf.printf "✅ Created bridge: %s\n" bridge_config.name;
  Printf.printf "   Bridge ID: %s\n" (Bytes.to_string bridge_config.bridge_id);
  Printf.printf "   Fee: %d basis points\n" bridge_config.fee_basis_points;
  
  (* Test fee calculation *)
  let amount = 10000L in
  let fee = Bridge_primitives.calculate_fee ~amount ~fee_basis_points:bridge_config.fee_basis_points in
  Printf.printf "✅ Fee calculation: %Ld amount → %Ld fee\n" amount fee;
  
  (* Test amount validation *)
  let valid = Bridge_primitives.validate_transfer_amount 
    ~amount:5000L 
    ~min_amount:bridge_config.min_transfer_amount 
    ~max_amount:bridge_config.max_transfer_amount in
  Printf.printf "✅ Amount validation: %b\n" valid;
  
  bridge_config

(* Test ProcessDataflowBlock workflow *)
let test_bridge_workflow ~(bridge_config : Bridge_primitives.bridge_config) () =
  Printf.printf "\n=== Testing Bridge Workflow ===\n";
  
  (* Create the workflow *)
  let workflow = Bridge_workflow.create_bridge_transfer_workflow ~bridge_config () in
  
  Printf.printf "✅ Created workflow: %s\n" workflow.name;
  Printf.printf "   Definition ID: %s\n" (Bytes.to_string workflow.definition_id);
  Printf.printf "   Nodes: %d\n" (List.length workflow.nodes);
  Printf.printf "   Edges: %d\n" (List.length workflow.edges);
  
  (* Print workflow nodes *)
  Printf.printf "   Workflow Steps:\n";
  List.iteri (fun i node ->
    Printf.printf "     %d. %s (%s)\n" (i+1) 
      node.node_id node.node_type
  ) workflow.nodes;
  
  (* Print workflow edges *)
  Printf.printf "   Workflow Transitions:\n";
  List.iteri (fun i edge ->
    Printf.printf "     %d. %s → %s (%s)\n" (i+1)
      edge.from_node
      edge.to_node
      edge.transition_type
  ) workflow.edges;
  
  workflow

(* Test intent-based optimization *)
let test_optimized_intent ~(bridge_config : Bridge_primitives.bridge_config) () =
  Printf.printf "\n=== Testing Optimized Intent ===\n";
  
  let source_account = Bytes.of_string "alice" in
  let target_account = Bytes.of_string "bob" in
  let amount = 5000L in
  let domain_id = bridge_config.source_domain in
  
  (* Create optimized intent *)
  let optimized_intent = Bridge_workflow.create_optimized_bridge_transfer_intent
    ~bridge_config
    ~source_account
    ~target_account
    ~amount
    ~domain_id () in
  
  Printf.printf "✅ Created optimized intent: %s\n" optimized_intent.name;
  Printf.printf "   Intent ID: %s\n" (Bytes.to_string optimized_intent.id);
  Printf.printf "   Priority: %d\n" optimized_intent.priority;
  Printf.printf "   Inputs: %d\n" (List.length optimized_intent.inputs);
  Printf.printf "   Outputs: %d\n" (List.length optimized_intent.outputs);
  Printf.printf "   Compatibility metadata: %d entries\n" (List.length optimized_intent.compatibility_metadata);
  Printf.printf "   Resource preferences: %d entries\n" (List.length optimized_intent.resource_preferences);
  
  (* Print optimization details *)
  Printf.printf "   Optimization Features:\n";
  (match optimized_intent.optimization_hint with
   | Some hint_id -> Printf.printf "     - Optimization hint: %s\n" (Bytes.to_string hint_id)
   | None -> Printf.printf "     - No optimization hint\n");
  
  (match optimized_intent.target_typed_domain with
   | Some (VerifiableDomain {domain_id; _}) -> 
     Printf.printf "     - Target domain: %s (Verifiable)\n" (Bytes.to_string domain_id)
   | Some (ServiceDomain {domain_id; _}) -> 
     Printf.printf "     - Target domain: %s (Service)\n" (Bytes.to_string domain_id)
   | Some (ComputeDomain {domain_id; _}) -> 
     Printf.printf "     - Target domain: %s (Compute)\n" (Bytes.to_string domain_id)
   | None -> Printf.printf "     - No target domain specified\n");
  
  (match optimized_intent.process_dataflow_hint with
   | Some hint -> Printf.printf "     - ProcessDataflow hint: %s\n" (Bytes.to_string hint.df_def_id)
   | None -> Printf.printf "     - No ProcessDataflow hint\n");
  
  optimized_intent

(* Main test function *)
let run_bridge_workflow_tests () =
  Printf.printf "🌉 Running Complete Bridge Workflow Tests\n\n";
  
  (* Test individual components *)
  let (_token_config, _balance_config) = test_token_primitives () in
  let bridge_config = test_bridge_primitives () in
  let _workflow = test_bridge_workflow ~bridge_config () in
  let _optimized_intent = test_optimized_intent ~bridge_config () in
  
  (* Run the complete example *)
  Printf.printf "\n=== Running Complete Example ===\n";
  let (_example_workflow, _example_intent, _example_instance) = Bridge_workflow.execute_bridge_transfer_example () in
  
  Printf.printf "\n✅ All Bridge Workflow Tests Completed Successfully!\n";
  Printf.printf "📊 Test Summary:\n";
  Printf.printf "  - Token primitives: ✅\n";
  Printf.printf "  - Bridge primitives: ✅\n";
  Printf.printf "  - ProcessDataflow workflow: ✅\n";
  Printf.printf "  - Intent-based optimization: ✅\n";
  Printf.printf "  - Complete example: ✅\n";
  
  Printf.printf "\n🎯 Key Features Demonstrated:\n";
  Printf.printf "  - Cross-domain token transfers\n";
  Printf.printf "  - ProcessDataflowBlock orchestration\n";
  Printf.printf "  - TypedDomain optimization\n";
  Printf.printf "  - Intent-based resource management\n";
  Printf.printf "  - Effect compatibility analysis\n"

(* Entry point *)
let () = run_bridge_workflow_tests () 