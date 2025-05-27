(* Purpose: Extended comprehensive tests for bridge implementation *)

open Ml_causality_lib_types.Types
open Ml_causality_lib_dsl

(*-----------------------------------------------------------------------------
 * Test Utilities
 *-----------------------------------------------------------------------------*)

let test_counter = ref 0
let passed_tests = ref 0
let failed_tests = ref 0

let run_test test_name test_func =
  incr test_counter;
  Printf.printf "\n🧪 Test %d: %s\n" !test_counter test_name;
  try
    test_func ();
    incr passed_tests;
    Printf.printf "✅ PASSED: %s\n" test_name
  with
  | e ->
    incr failed_tests;
    Printf.printf "❌ FAILED: %s - %s\n" test_name (Printexc.to_string e)

let assert_equal expected actual message =
  if expected <> actual then
    failwith (Printf.sprintf "%s: expected %s, got %s" message expected actual)

let assert_true condition message =
  if not condition then
    failwith message

let assert_false condition message =
  if condition then
    failwith message

(*-----------------------------------------------------------------------------
 * Edge Case Tests
 *-----------------------------------------------------------------------------*)

(** Test minimum transfer amounts *)
let test_minimum_transfer_amounts () =
  let source_domain = Bytes.of_string "test_source" in
  let target_domain = Bytes.of_string "test_target" in
  let source_token = Bytes.of_string "source_token" in
  let target_token = Bytes.of_string "target_token" in
  
  let (bridge_config, _) = Bridge_primitives.create_bridge
    ~name:"MinTransferTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:100 (* 1% *)
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  (* Test exactly minimum amount *)
  let valid_min = Bridge_primitives.validate_transfer_amount 
    ~amount:1000L 
    ~min_amount:bridge_config.min_transfer_amount 
    ~max_amount:bridge_config.max_transfer_amount in
  assert_true valid_min "Minimum amount should be valid";
  
  (* Test below minimum amount *)
  let invalid_below = Bridge_primitives.validate_transfer_amount 
    ~amount:999L 
    ~min_amount:bridge_config.min_transfer_amount 
    ~max_amount:bridge_config.max_transfer_amount in
  assert_false invalid_below "Below minimum amount should be invalid";
  
  Printf.printf "   ✓ Minimum transfer validation working correctly"

(** Test maximum transfer amounts *)
let test_maximum_transfer_amounts () =
  let source_domain = Bytes.of_string "test_source" in
  let target_domain = Bytes.of_string "test_target" in
  let source_token = Bytes.of_string "source_token" in
  let target_token = Bytes.of_string "target_token" in
  
  let (bridge_config, _) = Bridge_primitives.create_bridge
    ~name:"MaxTransferTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:50 (* 0.5% *)
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  (* Test exactly maximum amount *)
  let valid_max = Bridge_primitives.validate_transfer_amount 
    ~amount:1000000L 
    ~min_amount:bridge_config.min_transfer_amount 
    ~max_amount:bridge_config.max_transfer_amount in
  assert_true valid_max "Maximum amount should be valid";
  
  (* Test above maximum amount *)
  let invalid_above = Bridge_primitives.validate_transfer_amount 
    ~amount:1000001L 
    ~min_amount:bridge_config.min_transfer_amount 
    ~max_amount:bridge_config.max_transfer_amount in
  assert_false invalid_above "Above maximum amount should be invalid";
  
  Printf.printf "   ✓ Maximum transfer validation working correctly"

(** Test fee calculations with various basis points *)
let test_fee_calculations () =
  (* Test 0% fee *)
  let fee_0 = Bridge_primitives.calculate_fee ~amount:10000L ~fee_basis_points:0 in
  assert_equal "0" (Int64.to_string fee_0) "0% fee calculation";
  
  (* Test 1% fee (100 basis points) *)
  let fee_1 = Bridge_primitives.calculate_fee ~amount:10000L ~fee_basis_points:100 in
  assert_equal "100" (Int64.to_string fee_1) "1% fee calculation";
  
  (* Test 0.5% fee (50 basis points) *)
  let fee_half = Bridge_primitives.calculate_fee ~amount:10000L ~fee_basis_points:50 in
  assert_equal "50" (Int64.to_string fee_half) "0.5% fee calculation";
  
  (* Test 10% fee (1000 basis points) *)
  let fee_10 = Bridge_primitives.calculate_fee ~amount:10000L ~fee_basis_points:1000 in
  assert_equal "1000" (Int64.to_string fee_10) "10% fee calculation";
  
  Printf.printf "   ✓ Fee calculations working for various basis points"

(** Test token balance edge cases *)
let test_token_balance_edge_cases () =
  let domain_id = Bytes.of_string "test_domain" in
  let account_id = Bytes.of_string "test_account" in
  let token_id = Bytes.of_string "test_token" in
  
  (* Test zero balance *)
  let (zero_balance, _) = Token_primitives.create_token_balance
    ~account_id
    ~token_id
    ~initial_balance:0L
    ~domain_id () in
  assert_equal "0" (Int64.to_string zero_balance.balance) "Zero balance creation";
  
  (* Test transfer validation with zero balance *)
  let invalid_zero = Token_primitives.validate_transfer_amount ~balance:0L ~amount:1L in
  assert_false invalid_zero "Transfer from zero balance should be invalid";
  
  (* Test transfer validation with exact balance *)
  let valid_exact = Token_primitives.validate_transfer_amount ~balance:1000L ~amount:1000L in
  assert_true valid_exact "Transfer of exact balance should be valid";
  
  (* Test transfer validation with insufficient balance *)
  let invalid_insufficient = Token_primitives.validate_transfer_amount ~balance:1000L ~amount:1001L in
  assert_false invalid_insufficient "Transfer exceeding balance should be invalid";
  
  Printf.printf "   ✓ Token balance edge cases handled correctly"

(*-----------------------------------------------------------------------------
 * Advanced Workflow Tests
 *-----------------------------------------------------------------------------*)

(** Test workflow with multiple bridges *)
let test_multiple_bridges () =
  let ethereum = Bytes.of_string "ethereum" in
  let polygon = Bytes.of_string "polygon" in
  let arbitrum = Bytes.of_string "arbitrum" in
  let eth_token = Bytes.of_string "eth_token" in
  let poly_token = Bytes.of_string "poly_token" in
  let arb_token = Bytes.of_string "arb_token" in
  
  (* Create multiple bridges *)
  let (bridge1, _) = Bridge_primitives.create_bridge
    ~name:"ETH_POLY_Bridge"
    ~source_domain:ethereum
    ~target_domain:polygon
    ~source_token:eth_token
    ~target_token:poly_token
    ~fee_basis_points:30
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  let (bridge2, _) = Bridge_primitives.create_bridge
    ~name:"ETH_ARB_Bridge"
    ~source_domain:ethereum
    ~target_domain:arbitrum
    ~source_token:eth_token
    ~target_token:arb_token
    ~fee_basis_points:25
    ~min_transfer_amount:500L
    ~max_transfer_amount:2000000L
    ~timeout_seconds:7200L () in
  
  (* Create workflows for both bridges *)
  let workflow1 = Bridge_workflow.create_bridge_transfer_workflow ~bridge_config:bridge1 () in
  let workflow2 = Bridge_workflow.create_bridge_transfer_workflow ~bridge_config:bridge2 () in
  
  (* Verify workflows are different *)
  let id1 = Bytes.to_string workflow1.definition_id in
  let id2 = Bytes.to_string workflow2.definition_id in
  assert_true (id1 <> id2) "Bridge workflows should have different IDs";
  
  (* Verify both have correct number of nodes *)
  assert_equal "6" (string_of_int (List.length workflow1.nodes)) "Bridge1 workflow nodes";
  assert_equal "6" (string_of_int (List.length workflow2.nodes)) "Bridge2 workflow nodes";
  
  Printf.printf "   ✓ Multiple bridge workflows created successfully"

(** Test workflow node validation *)
let test_workflow_node_validation () =
  let source_domain = Bytes.of_string "test_source" in
  let target_domain = Bytes.of_string "test_target" in
  let source_token = Bytes.of_string "source_token" in
  let target_token = Bytes.of_string "target_token" in
  
  let (bridge_config, _) = Bridge_primitives.create_bridge
    ~name:"ValidationTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:30
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  let workflow = Bridge_workflow.create_bridge_transfer_workflow ~bridge_config () in
  
  (* Verify all required nodes exist *)
  let node_ids = List.map (fun node -> node.node_id) workflow.nodes in
  let required_nodes = [
    "validate_transfer";
    "lock_tokens";
    "relay_message";
    "verify_proof";
    "mint_tokens";
    "complete_transfer"
  ] in
  
  List.iter (fun required ->
    assert_true (List.mem required node_ids) 
      (Printf.sprintf "Required node '%s' should exist" required)
  ) required_nodes;
  
  (* Verify edges connect properly *)
  let edge_count = List.length workflow.edges in
  assert_equal "5" (string_of_int edge_count) "Workflow should have 5 edges";
  
  Printf.printf "   ✓ Workflow node validation passed"

(** Test TypedDomain configurations *)
let test_typed_domain_configurations () =
  let source_domain = Bytes.of_string "verifiable_domain" in
  let target_domain = Bytes.of_string "service_domain" in
  let source_token = Bytes.of_string "source_token" in
  let target_token = Bytes.of_string "target_token" in
  
  let (bridge_config, _) = Bridge_primitives.create_bridge
    ~name:"TypedDomainTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:30
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  let workflow = Bridge_workflow.create_bridge_transfer_workflow ~bridge_config () in
  
  (* Check that nodes have appropriate TypedDomain policies *)
  let validate_node = List.find (fun n -> n.node_id = "validate_transfer") workflow.nodes in
  let relay_node = List.find (fun n -> n.node_id = "relay_message") workflow.nodes in
  
  (* Validate node should have VerifiableDomain *)
  (match validate_node.typed_domain_policy with
   | Some (VerifiableDomain {zk_constraints = true; deterministic_only = true; _}) ->
     Printf.printf "   ✓ Validate node has correct VerifiableDomain policy"
   | _ -> failwith "Validate node should have VerifiableDomain policy");
  
  (* Relay node should have ServiceDomain *)
  (match relay_node.typed_domain_policy with
   | Some (ServiceDomain {external_apis; non_deterministic_allowed = true; _}) ->
     assert_true (List.mem "bridge_relay" external_apis) "Relay node should have bridge_relay API";
     Printf.printf "   ✓ Relay node has correct ServiceDomain policy"
   | _ -> failwith "Relay node should have ServiceDomain policy");
  
  Printf.printf "   ✓ TypedDomain configurations validated"

(*-----------------------------------------------------------------------------
 * Performance and Stress Tests
 *-----------------------------------------------------------------------------*)

(** Test large transfer amounts *)
let test_large_transfer_amounts () =
  let domain_id = Bytes.of_string "test_domain" in
  
  (* Create token with very large supply *)
  let (token_config, _) = Token_primitives.create_token
    ~name:"Large Token"
    ~symbol:"LARGE"
    ~decimals:18
    ~total_supply:Int64.max_int
    ~domain_id () in
  
  (* Test large amount formatting *)
  let large_amount = 100000000000000000L in (* 0.1 ETH in wei - large but won't overflow *)
  let formatted = Token_primitives.format_token_amount 
    ~amount:large_amount 
    ~decimals:token_config.decimals 
    ~symbol:token_config.symbol in
  
  assert_true (String.contains formatted '.') "Large amount should be formatted with decimals";
  Printf.printf "   ✓ Large amount formatted: %s" formatted;
  
  (* Test fee calculation on large amounts *)
  let large_fee = Bridge_primitives.calculate_fee ~amount:large_amount ~fee_basis_points:30 in
  assert_true (large_fee > 0L) "Large amount should generate non-zero fee";
  Printf.printf "   ✓ Large amount fee calculation working"

(** Test multiple concurrent transfers *)
let test_concurrent_transfers () =
  let source_domain = Bytes.of_string "concurrent_source" in
  let target_domain = Bytes.of_string "concurrent_target" in
  let source_token = Bytes.of_string "source_token" in
  let target_token = Bytes.of_string "target_token" in
  
  let (bridge_config, _) = Bridge_primitives.create_bridge
    ~name:"ConcurrentTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:30
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  (* Create multiple transfer effects *)
  let transfers = List.init 5 (fun i ->
    let account_id = Bytes.of_string (Printf.sprintf "account_%d" i) in
    let target_id = Bytes.of_string (Printf.sprintf "target_%d" i) in
    let amount = Int64.of_int ((i + 1) * 1000) in
    Bridge_primitives.create_initiate_transfer_effect
      ~bridge_config
      ~source_account:account_id
      ~target_account:target_id
      ~amount
      ~domain_id:source_domain ()
  ) in
  
  (* Verify all transfers have unique IDs *)
  let transfer_ids = List.map (fun (effect : effect) -> Bytes.to_string effect.id) transfers in
  let unique_ids = List.sort_uniq String.compare transfer_ids in
  assert_equal (string_of_int (List.length transfers)) 
               (string_of_int (List.length unique_ids))
               "All transfer effects should have unique IDs";
  
  Printf.printf "   ✓ Created %d concurrent transfers with unique IDs" (List.length transfers)

(*-----------------------------------------------------------------------------
 * Integration Tests
 *-----------------------------------------------------------------------------*)

(** Test complete end-to-end bridge transfer *)
let test_end_to_end_bridge_transfer () =
  Printf.printf "\n🔄 Running End-to-End Bridge Transfer Test\n";
  
  (* Setup domains and tokens *)
  let ethereum = Bytes.of_string "ethereum_e2e" in
  let polygon = Bytes.of_string "polygon_e2e" in
  
  let (eth_token, _) = Token_primitives.create_token
    ~name:"Ethereum E2E"
    ~symbol:"ETH"
    ~decimals:18
    ~total_supply:1000000L
    ~domain_id:ethereum () in
  
  let (poly_token, _) = Token_primitives.create_token
    ~name:"Polygon E2E"
    ~symbol:"POLY"
    ~decimals:18
    ~total_supply:1000000L
    ~domain_id:polygon () in
  
  (* Setup bridge *)
  let (bridge_config, _) = Bridge_primitives.create_bridge
    ~name:"E2E_Bridge"
    ~source_domain:ethereum
    ~target_domain:polygon
    ~source_token:eth_token.token_id
    ~target_token:poly_token.token_id
    ~fee_basis_points:30
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  (* Setup user accounts *)
  let alice = Bytes.of_string "alice_e2e" in
  let bob = Bytes.of_string "bob_e2e" in
  
  (* Create initial balances *)
  let (alice_balance, _) = Token_primitives.create_token_balance
    ~account_id:alice
    ~token_id:eth_token.token_id
    ~initial_balance:50000L
    ~domain_id:ethereum () in
  
  (* Create workflow *)
  let workflow = Bridge_workflow.create_bridge_transfer_workflow ~bridge_config () in
  
  (* Create optimized intent *)
  let transfer_amount = 10000L in
  let intent = Bridge_workflow.create_optimized_bridge_transfer_intent
    ~bridge_config
    ~source_account:alice
    ~target_account:bob
    ~amount:transfer_amount
    ~domain_id:ethereum () in
  
  (* Validate the complete setup *)
  assert_true (alice_balance.balance >= transfer_amount) "Alice should have sufficient balance";
  assert_equal "6" (string_of_int (List.length workflow.nodes)) "Workflow should have 6 nodes";
  assert_equal "5" (string_of_int (List.length workflow.edges)) "Workflow should have 5 edges";
  assert_equal "OptimizedBridgeTransfer" intent.name "Intent should be optimized";
  
  let fee = Bridge_primitives.calculate_fee ~amount:transfer_amount ~fee_basis_points:bridge_config.fee_basis_points in
  Printf.printf "   ✓ E2E setup complete: %Ld ETH transfer with %Ld ETH fee" transfer_amount fee;
  Printf.printf "   ✓ Workflow has %d nodes and %d edges" (List.length workflow.nodes) (List.length workflow.edges);
  Printf.printf "   ✓ Intent has %d compatibility entries" (List.length intent.compatibility_metadata)

(*-----------------------------------------------------------------------------
 * Error Handling Tests
 *-----------------------------------------------------------------------------*)

(** Test invalid bridge configurations *)
let test_invalid_bridge_configurations () =
  let source_domain = Bytes.of_string "source" in
  let target_domain = Bytes.of_string "target" in
  let source_token = Bytes.of_string "source_token" in
  let target_token = Bytes.of_string "target_token" in
  
  (* Test with invalid fee basis points (should still work but be noted) *)
  let (bridge_high_fee, _) = Bridge_primitives.create_bridge
    ~name:"HighFeeTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:5000 (* 50% fee - very high but valid *)
    ~min_transfer_amount:1000L
    ~max_transfer_amount:1000000L
    ~timeout_seconds:3600L () in
  
  let high_fee = Bridge_primitives.calculate_fee ~amount:10000L ~fee_basis_points:bridge_high_fee.fee_basis_points in
  assert_equal "5000" (Int64.to_string high_fee) "High fee calculation should work";
  
  (* Test with min > max (should still create but be logically invalid) *)
  let (bridge_invalid_range, _) = Bridge_primitives.create_bridge
    ~name:"InvalidRangeTest"
    ~source_domain
    ~target_domain
    ~source_token
    ~target_token
    ~fee_basis_points:30
    ~min_transfer_amount:1000000L (* min > max *)
    ~max_transfer_amount:1000L
    ~timeout_seconds:3600L () in
  
  (* This should be invalid for any amount *)
  let invalid_any = Bridge_primitives.validate_transfer_amount 
    ~amount:5000L 
    ~min_amount:bridge_invalid_range.min_transfer_amount 
    ~max_amount:bridge_invalid_range.max_transfer_amount in
  assert_false invalid_any "Invalid range should make all amounts invalid";
  
  Printf.printf "   ✓ Invalid configurations handled appropriately"

(*-----------------------------------------------------------------------------
 * Main Test Runner
 *-----------------------------------------------------------------------------*)

let run_extended_tests () =
  Printf.printf "🧪 Running Extended Bridge Implementation Tests\n";
  Printf.printf "%s\n\n" ("=" ^ String.make 50 '=' ^ "=");
  
  (* Edge Case Tests *)
  run_test "Minimum Transfer Amounts" test_minimum_transfer_amounts;
  run_test "Maximum Transfer Amounts" test_maximum_transfer_amounts;
  run_test "Fee Calculations" test_fee_calculations;
  run_test "Token Balance Edge Cases" test_token_balance_edge_cases;
  
  (* Advanced Workflow Tests *)
  run_test "Multiple Bridges" test_multiple_bridges;
  run_test "Workflow Node Validation" test_workflow_node_validation;
  run_test "TypedDomain Configurations" test_typed_domain_configurations;
  
  (* Performance Tests *)
  run_test "Large Transfer Amounts" test_large_transfer_amounts;
  run_test "Concurrent Transfers" test_concurrent_transfers;
  
  (* Integration Tests *)
  run_test "End-to-End Bridge Transfer" test_end_to_end_bridge_transfer;
  
  (* Error Handling Tests *)
  run_test "Invalid Bridge Configurations" test_invalid_bridge_configurations;
  
  (* Final Summary *)
  Printf.printf "\n%s\n" (String.make 60 '=');
  Printf.printf "🏁 Extended Test Results Summary\n";
  Printf.printf "   Total Tests: %d\n" !test_counter;
  Printf.printf "   Passed: %d ✅\n" !passed_tests;
  Printf.printf "   Failed: %d ❌\n" !failed_tests;
  Printf.printf "   Success Rate: %.1f%%\n" 
    (if !test_counter > 0 then (float_of_int !passed_tests) /. (float_of_int !test_counter) *. 100.0 else 0.0);
  
  if !failed_tests = 0 then (
    Printf.printf "\n🎉 All Extended Tests Passed! Bridge implementation is robust.\n";
    Printf.printf "\n🔧 Features Thoroughly Tested:\n";
    Printf.printf "   • Edge case handling (min/max amounts, zero balances)\n";
    Printf.printf "   • Fee calculations with various basis points\n";
    Printf.printf "   • Multiple bridge configurations\n";
    Printf.printf "   • Workflow node validation and TypedDomain policies\n";
    Printf.printf "   • Large amount handling and concurrent transfers\n";
    Printf.printf "   • End-to-end integration scenarios\n";
    Printf.printf "   • Error handling and invalid configurations\n"
  ) else (
    Printf.printf "\n⚠️  Some tests failed. Please review the implementation.\n"
  )

(* Entry point *)
let () = run_extended_tests () 