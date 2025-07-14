(* ------------ E2E TEST: ACCOUNT FACTORY CREATION ------------ *)
(* Purpose: Test OCaml program creating Valence account factory account *)

module Valence_ffi = Test_valence_ffi_mock
module Async_runtime = Test_async_runtime_mock  
module Gas_estimation = Test_gas_estimation_mock

open Valence_ffi
open Async_runtime
open Gas_estimation

(* Test configuration *)
let test_owner = "test_user_123"
let test_permissions = ["read"; "write"; "execute"]

(* Test account factory creation *)
let test_create_account_factory () =
  Printf.printf "=== Testing Account Factory Creation ===\n";
  
  (* Create account factory configuration *)
  let config = make_account_factory_config ~owner:test_owner ~permissions:test_permissions () in
  Printf.printf "Created config for owner: %s\n" config.owner;
  
  (* Test gas estimation *)
  let gas_estimate = Gas_estimation.AccountFactory.estimate_create_account_gas ~permissions_count:(List.length test_permissions) () in
  Printf.printf "Estimated gas: %d, cost: %d\n" gas_estimate.gas_limit gas_estimate.total_cost;
  
  (* Create account factory *)
  match safe_create_account_factory config with
  | Ok account -> 
      Printf.printf "✓ Account factory created successfully\n";
      Printf.printf "Account handle type: %s\n" (Obj.tag (Obj.repr account) |> string_of_int);
      Ok account
  | Error msg -> 
      Printf.printf "✗ Failed to create account factory: %s\n" msg;
      Error msg

(* Test async account factory creation *)
let test_async_account_factory_creation () =
  Printf.printf "\n=== Testing Async Account Factory Creation ===\n";
  
  (* Create runtime *)
  let runtime = create_runtime () in
  Printf.printf "Created async runtime\n";
  
  (* Create account factory asynchronously *)
  match Async_runtime.AccountFactory.create_account_async runtime test_owner test_permissions with
  | Ok (account, handle) ->
      Printf.printf "✓ Async account factory created successfully\n";
      Printf.printf "Account: %s, Transaction ID: %s\n" account handle.tx_id;
      Printf.printf "Status: %s\n" (match handle.status with
        | Confirmed -> "Confirmed"
        | Pending -> "Pending"
        | Failed msg -> "Failed: " ^ msg
        | Timeout -> "Timeout");
      Ok (account, handle)
  | Error msg ->
      Printf.printf "✗ Failed to create async account factory: %s\n" msg;
      Error msg

(* Test account factory validation *)
let test_account_factory_validation () =
  Printf.printf "\n=== Testing Account Factory Validation ===\n";
  
  (* Test valid owner *)
  let valid_owner = validate_account_owner test_owner in
  Printf.printf "Owner validation (%s): %b\n" test_owner valid_owner;
  
  (* Test invalid owner *)
  let invalid_owner = validate_account_owner "" in
  Printf.printf "Empty owner validation: %b\n" invalid_owner;
  
  (* Test account validity *)
  let account_valid = is_account_valid "test_account" in
  Printf.printf "Account validity check: %b\n" account_valid;
  
  Ok ()

(* Test gas estimation for account factory *)
let test_gas_estimation () =
  Printf.printf "\n=== Testing Gas Estimation ===\n";
  
  (* Test different operation gas estimates *)
  let create_estimate = estimate_gas CreateAccount in
  Printf.printf "Create account gas: %d, price: %d, cost: %d\n" 
    create_estimate.gas_limit create_estimate.gas_price create_estimate.total_cost;
  
  let approve_estimate = estimate_gas ApproveLibrary in
  Printf.printf "Approve library gas: %d, price: %d, cost: %d\n"
    approve_estimate.gas_limit approve_estimate.gas_price approve_estimate.total_cost;
  
  let swap_estimate = estimate_gas (SubmitTransaction "swap") in
  Printf.printf "Swap transaction gas: %d, price: %d, cost: %d\n"
    swap_estimate.gas_limit swap_estimate.gas_price swap_estimate.total_cost;
  
  (* Test gas price oracle *)
  let recommended_price = Gas_estimation.GasPriceOracle.get_recommended_gas_price () in
  let fast_price = Gas_estimation.GasPriceOracle.get_fast_gas_price () in
  let economy_price = Gas_estimation.GasPriceOracle.get_economy_gas_price () in
  Printf.printf "Gas prices - Recommended: %d, Fast: %d, Economy: %d\n"
    recommended_price fast_price economy_price;
  
  Ok ()

(* Test interface generation *)
let test_interface_generation () =
  Printf.printf "\n=== Testing Interface Generation ===\n";
  
  let config = make_account_factory_config ~owner:test_owner ~permissions:test_permissions () in
  
  match generate_account_interface config with
  | Ok interface_code ->
      Printf.printf "✓ Interface generated successfully\n";
      Printf.printf "Interface preview (first 200 chars):\n%s...\n" 
        (String.sub interface_code 0 (min 200 (String.length interface_code)));
      Ok interface_code
  | Error msg ->
      Printf.printf "✗ Failed to generate interface: %s\n" msg;
      Error msg

(* Test deployment script generation *)
let test_deployment_script_generation () =
  Printf.printf "\n=== Testing Deployment Script Generation ===\n";
  
  let configs = [
    make_account_factory_config ~owner:"user1" ();
    make_account_factory_config ~owner:"user2" ();
    make_account_factory_config ~owner:"user3" ();
  ] in
  
  match generate_deployment_script configs with
  | Ok script ->
      Printf.printf "✓ Deployment script generated successfully\n";
      Printf.printf "Script preview (first 300 chars):\n%s...\n"
        (String.sub script 0 (min 300 (String.length script)));
      Ok script
  | Error msg ->
      Printf.printf "✗ Failed to generate deployment script: %s\n" msg;
      Error msg

(* Test runtime statistics *)
let test_runtime_statistics () =
  Printf.printf "\n=== Testing Runtime Statistics ===\n";
  
  let runtime = create_runtime () in
  
  (* Perform some operations *)
  let _ = Async_runtime.AccountFactory.create_account_async runtime "stats_test_user" ["read"; "write"] in
  let _ = submit_transaction_async runtime "test_account" "test_operation" "test_data" in
  
  let stats = get_runtime_stats runtime in
  Printf.printf "Runtime Statistics:\n";
  Printf.printf "  Total transactions: %d\n" stats.total_transactions;
  Printf.printf "  Successful transactions: %d\n" stats.successful_transactions;
  Printf.printf "  Failed transactions: %d\n" stats.failed_transactions;
  Printf.printf "  Active transactions: %d\n" stats.active_transactions;
  Printf.printf "  Average completion time: %d seconds\n" stats.average_completion_time;
  
  Ok stats

(* Simplified test runner that returns proper types *)
let run_all_tests () =
  Printf.printf "Starting Account Factory Creation E2E Tests\n";
  Printf.printf "============================================\n";
  
  let tests = [
    ("Account Factory Creation", fun () -> 
      match test_create_account_factory () with
      | Ok _ -> Ok ()
      | Error msg -> Error msg);
    ("Async Account Factory Creation", fun () ->
      match test_async_account_factory_creation () with
      | Ok _ -> Ok ()
      | Error msg -> Error msg);
    ("Account Factory Validation", test_account_factory_validation);
    ("Gas Estimation", test_gas_estimation);
    ("Interface Generation", fun () ->
      match test_interface_generation () with
      | Ok _ -> Ok ()
      | Error msg -> Error msg);
    ("Deployment Script Generation", fun () ->
      match test_deployment_script_generation () with
      | Ok _ -> Ok ()
      | Error msg -> Error msg);
    ("Runtime Statistics", fun () ->
      match test_runtime_statistics () with
      | Ok _ -> Ok ()
      | Error msg -> Error msg);
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
  
  Printf.printf "\n============================================\n";
  Printf.printf "Test Results: %d/%d tests passed\n" passed total;
  
  if passed = total then
    Printf.printf "🎉 All tests passed!\n"
  else
    Printf.printf "❌ Some tests failed\n";
  
  results

(* Entry point *)
let () = 
  let _ = run_all_tests () in
  () 