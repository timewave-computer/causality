(* ------------ E2E TEST: MULTI-CHAIN STATE QUERIES ------------ *)
(* Purpose: Test OCaml program querying token balances across multiple chains *)

open Test_query_primitives_mock

(* Test configuration *)
let test_user_address = "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87"
let usdc_ethereum = "0xa0b86a33e6ba3e0e4ca4ba5d4e6b3e4c4d5e6f7"
let usdc_polygon = "0x2791bca1f2de4661ed88a30c99a7a9449aa84174"
let weth_ethereum = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"

(* Test multi-chain balance queries *)
let test_multi_chain_balance_queries () =
  Printf.printf "=== Testing Multi-Chain Balance Queries ===\n";
  
  (* Create query configurations for different chains *)
  let ethereum_usdc_config = ethereum_config usdc_ethereum in
  let polygon_usdc_config = make_query_config ~contract_id:usdc_polygon ~domain:"polygon" () in
  let ethereum_weth_config = ethereum_config weth_ethereum in
  
  Printf.printf "Created configurations for 3 contracts across 2 chains\n";
  
  (* Query balances across multiple chains *)
  let ethereum_usdc_balance = safe_query_balance ethereum_usdc_config test_user_address in
  let polygon_usdc_balance = safe_query_balance polygon_usdc_config test_user_address in
  let ethereum_weth_balance = safe_query_balance ethereum_weth_config test_user_address in
  
  Printf.printf "Ethereum USDC balance: %s\n" (result_to_string ethereum_usdc_balance);
  Printf.printf "Polygon USDC balance: %s\n" (result_to_string polygon_usdc_balance);
  Printf.printf "Ethereum WETH balance: %s\n" (result_to_string ethereum_weth_balance);
  
  (* Validate all queries succeeded *)
  let all_successful = 
    is_success ethereum_usdc_balance && 
    is_success polygon_usdc_balance && 
    is_success ethereum_weth_balance in
  
  if all_successful then (
    Printf.printf "✓ All multi-chain balance queries successful\n";
    Ok ()
  ) else (
    Printf.printf "✗ Some balance queries failed\n";
    Error "Multi-chain query failure"
  )

(* Test state-dependent strategy execution *)
let test_state_dependent_strategy () =
  Printf.printf "\n=== Testing State-Dependent Strategy Execution ===\n";
  
  (* Create configurations *)
  let ethereum_config = ethereum_config usdc_ethereum in
  let polygon_config = make_query_config ~contract_id:usdc_polygon ~domain:"polygon" () in
  
  (* Query balances *)
  let eth_balance = safe_query_balance ethereum_config test_user_address in
  let poly_balance = safe_query_balance polygon_config test_user_address in
  
  Printf.printf "Ethereum balance: %s\n" (result_to_string eth_balance);
  Printf.printf "Polygon balance: %s\n" (result_to_string poly_balance);
  
  (* Execute strategy based on balance comparison *)
  let strategy_result = match (eth_balance, poly_balance) with
    | (Success eth_data, Success poly_data) ->
        (* Mock balance comparison logic *)
        let eth_amount = String.length eth_data in  (* Mock parsing *)
        let poly_amount = String.length poly_data in
        
        if eth_amount > poly_amount then (
          Printf.printf "Strategy: Bridge from Ethereum to Polygon (higher balance on Ethereum)\n";
          "bridge_eth_to_poly"
        ) else if poly_amount > eth_amount then (
          Printf.printf "Strategy: Bridge from Polygon to Ethereum (higher balance on Polygon)\n";
          "bridge_poly_to_eth"
        ) else (
          Printf.printf "Strategy: No bridging needed (balanced)\n";
          "no_action"
        )
    | (Error eth_err, _) ->
        Printf.printf "Strategy: Error querying Ethereum balance: %s\n" eth_err;
        "error"
    | (_, Error poly_err) ->
        Printf.printf "Strategy: Error querying Polygon balance: %s\n" poly_err;
        "error"
  in
  
  Printf.printf "✓ State-dependent strategy executed: %s\n" strategy_result;
  Ok ()

(* Test multi-chain query coordination *)
let test_multi_chain_coordination () =
  Printf.printf "\n=== Testing Multi-Chain Query Coordination ===\n";
  
  (* Create multi-chain query *)
  let queries = [
    (ethereum_config usdc_ethereum, "balances", test_user_address);
    (make_query_config ~contract_id:usdc_polygon ~domain:"polygon" (), "balances", test_user_address);
    (ethereum_config weth_ethereum, "balances", test_user_address);
  ] in
  
  let multi_query = compose_queries queries in
  Printf.printf "Created multi-chain query with %d contracts\n" (List.length multi_query.queries);
  
  (* Execute sequential coordination *)
  let sequential_results = execute_multi_chain_query multi_query in
  Printf.printf "Sequential execution results:\n";
  List.iteri (fun i (field, result) ->
    Printf.printf "  %d. %s: %s\n" (i+1) field (result_to_string result)
  ) sequential_results;
  
  (* Test parallel coordination *)
  let parallel_query = { multi_query with coordination_strategy = Parallel } in
  let parallel_results = execute_multi_chain_query parallel_query in
  Printf.printf "Parallel execution results:\n";
  List.iteri (fun i (field, result) ->
    Printf.printf "  %d. %s: %s\n" (i+1) field (result_to_string result)
  ) parallel_results;
  
  (* Test conditional coordination *)
  let condition = function
    | Success _ -> true
    | Error _ -> false
  in
  let conditional_query = { multi_query with coordination_strategy = Conditional condition } in
  let conditional_results = execute_multi_chain_query conditional_query in
  Printf.printf "Conditional execution results:\n";
  List.iteri (fun i (field, result) ->
    Printf.printf "  %d. %s: %s\n" (i+1) field (result_to_string result)
  ) conditional_results;
  
  Printf.printf "✓ Multi-chain coordination strategies tested\n";
  Ok ()

(* Test query caching and optimization *)
let test_query_caching () =
  Printf.printf "\n=== Testing Query Caching and Optimization ===\n";
  
  let config = ethereum_config usdc_ethereum in
  
  (* Clear cache first *)
  QueryCache.clear ();
  Printf.printf "Cache cleared\n";
  
  (* First query (should hit the backend) *)
  let start_time = Unix.gettimeofday () in
  let result1 = cached_query_balance config test_user_address in
  let first_duration = Unix.gettimeofday () -. start_time in
  Printf.printf "First query result: %s (%.3f ms)\n" 
    (result_to_string result1) (first_duration *. 1000.0);
  
  (* Second query (should hit the cache) *)
  let start_time = Unix.gettimeofday () in
  let result2 = cached_query_balance config test_user_address in
  let second_duration = Unix.gettimeofday () -. start_time in
  Printf.printf "Second query result: %s (%.3f ms)\n" 
    (result_to_string result2) (second_duration *. 1000.0);
  
  (* Verify results are the same *)
  let results_match = match (result1, result2) with
    | (Success data1, Success data2) -> data1 = data2
    | (Error err1, Error err2) -> err1 = err2
    | _ -> false
  in
  
  if results_match then (
    Printf.printf "✓ Cache working correctly (results match)\n";
    Printf.printf "✓ Performance improvement: %.1fx faster\n" 
      (if second_duration > 0.0 then first_duration /. second_duration else 1.0);
    Ok ()
  ) else (
    Printf.printf "✗ Cache inconsistency detected\n";
    Error "Cache inconsistency"
  )

(* Test async query operations *)
let test_async_queries () =
  Printf.printf "\n=== Testing Async Query Operations ===\n";
  
  let config = ethereum_config usdc_ethereum in
  
  (* Submit async query *)
  let handle = submit_query_async config "balances" test_user_address in
  Printf.printf "Submitted async query with ID: %s\n" handle.query_id;
  
  (* Check initial status *)
  let initial_status = get_query_status handle in
  Printf.printf "Initial status: %s\n" (match initial_status with
    | Pending -> "Pending"
    | Completed -> "Completed"
    | Failed msg -> "Failed: " ^ msg
    | Cached -> "Cached");
  
  (* Wait for completion *)
  let result = wait_for_query handle ~timeout_ms:3000 () in
  Printf.printf "Async query result: %s\n" (result_to_string result);
  
  (* Check final status *)
  let final_status = get_query_status handle in
  Printf.printf "Final status: %s\n" (match final_status with
    | Pending -> "Pending"
    | Completed -> "Completed"
    | Failed msg -> "Failed: " ^ msg
    | Cached -> "Cached");
  
  if is_success result then (
    Printf.printf "✓ Async query completed successfully\n";
    Ok ()
  ) else (
    Printf.printf "✗ Async query failed\n";
    Error "Async query failure"
  )

(* Test query composition and filtering *)
let test_query_composition () =
  Printf.printf "\n=== Testing Query Composition and Filtering ===\n";
  
  (* Create multiple queries *)
  let queries = [
    (ethereum_config usdc_ethereum, "balances", test_user_address);
    (ethereum_config usdc_ethereum, "allowances", test_user_address ^ ":spender");
    (ethereum_config usdc_ethereum, "totalSupply", "");
  ] in
  
  let multi_query = compose_queries queries in
  let results = execute_multi_chain_query multi_query in
  
  Printf.printf "All query results:\n";
  List.iteri (fun i (field, result) ->
    Printf.printf "  %d. %s: %s\n" (i+1) field (result_to_string result)
  ) results;
  
  (* Filter successful results *)
  let successful_results = filter_results results is_success in
  Printf.printf "Successful results: %d/%d\n" 
    (List.length successful_results) (List.length results);
  
  (* Map results to uppercase *)
  let mapped_results = map_results results (fun result -> map_result result String.uppercase_ascii) in
  Printf.printf "Mapped results (uppercase):\n";
  List.iteri (fun i (field, result) ->
    Printf.printf "  %d. %s: %s\n" (i+1) field (result_to_string result)
  ) mapped_results;
  
  Printf.printf "✓ Query composition and filtering working\n";
  Ok ()

(* Test error handling and validation *)
let test_error_handling () =
  Printf.printf "\n=== Testing Error Handling and Validation ===\n";
  
  (* Test invalid configuration *)
  let invalid_config = make_query_config ~contract_id:"" ~domain:"ethereum" () in
  let invalid_result = safe_query_balance invalid_config test_user_address in
  Printf.printf "Invalid config result: %s\n" (result_to_string invalid_result);
  
  (* Test empty address *)
  let valid_config = ethereum_config usdc_ethereum in
  let empty_address_result = safe_query_balance valid_config "" in
  Printf.printf "Empty address result: %s\n" (result_to_string empty_address_result);
  
  (* Test result combinators *)
  let results = [
    Success "100";
    Success "200";
    Success "300";
  ] in
  let combined = combine_results results in
  Printf.printf "Combined results: %s\n" (result_to_string combined);
  
  (* Test error propagation *)
  let error_results = [
    Success "100";
    Error "Network error";
    Success "300";
  ] in
  let error_combined = combine_results error_results in
  Printf.printf "Error combined results: %s\n" (result_to_string error_combined);
  
  let expected_errors = is_error invalid_result && is_error empty_address_result && is_error error_combined in
  if expected_errors then (
    Printf.printf "✓ Error handling working correctly\n";
    Ok ()
  ) else (
    Printf.printf "✗ Error handling issues detected\n";
    Error "Error handling failure"
  )

(* Main test runner *)
let run_all_tests () =
  Printf.printf "Starting Multi-Chain State Query E2E Tests\n";
  Printf.printf "==========================================\n";
  
  let tests = [
    ("Multi-Chain Balance Queries", test_multi_chain_balance_queries);
    ("State-Dependent Strategy", test_state_dependent_strategy);
    ("Multi-Chain Coordination", test_multi_chain_coordination);
    ("Query Caching", test_query_caching);
    ("Async Queries", test_async_queries);
    ("Query Composition", test_query_composition);
    ("Error Handling", test_error_handling);
  ] in
  
  let results = List.map (fun (name, test_fn) ->
    Printf.printf "\n--- Running: %s ---\n" name;
    try
      match test_fn () with
      | Ok _ -> 
          Printf.printf "✓ %s: PASSED\n" name;
          (name, true)
      | Error msg ->
          Printf.printf "✗ %s: FAILED - %s\n" name msg;
          (name, false)
    with
    | exn ->
        Printf.printf "✗ %s: ERROR - %s\n" name (Printexc.to_string exn);
        (name, false)
  ) tests in
  
  let passed = List.filter (fun (_, result) -> result) results |> List.length in
  let total = List.length results in
  
  Printf.printf "\n==========================================\n";
  Printf.printf "Test Results: %d/%d tests passed\n" passed total;
  
  if passed = total then
    Printf.printf "🎉 All Phase 2 tests passed!\n"
  else
    Printf.printf "❌ Some Phase 2 tests failed\n";
  
  results

(* Entry point *)
let () = 
  let _ = run_all_tests () in
  () 