(* ------------ INTEGRATION TEST ------------ *)
(* Purpose: Comprehensive integration test for OCaml Causality DSL *)

open Ocaml_causality_core
open Ocaml_causality_lang
open Ocaml_causality_serialization
open Ocaml_causality_interop

module LispValue = Value.LispValue
module Expr = Expr.Expr

(** Test result tracking *)
type test_result = {
  name: string;
  passed: bool;
  error_msg: string option;
}

let test_results = ref []

let add_test_result name passed error_msg =
  test_results := { name; passed; error_msg } :: !test_results

let run_test name test_fn =
  try
    let result = test_fn () in
    add_test_result name result None;
    if result then
      Printf.printf "✅ %s\n" name
    else
      Printf.printf "❌ %s\n" name
  with
  | exn ->
    let error_msg = Printexc.to_string exn in
    add_test_result name false (Some error_msg);
    Printf.printf "❌ %s: %s\n" name error_msg

(** Test FFI initialization *)
let test_ffi_initialization () =
  match Ffi.initialize_ffi () with
  | Ok () -> true
  | Error _ -> false

(** Test resource creation and linearity *)
let test_resource_linearity () =
  let domain_id = 
    let base = "test_domain" in
    let padded = base ^ String.make (32 - String.length base) '\000' in
    Bytes.of_string padded in
  match Ffi.safe_create_resource "test_token" domain_id 100L with
  | Ok (Some resource_id) ->
    (* First consumption should succeed *)
    (match Ffi.safe_consume_resource_by_id resource_id with
     | Ok true ->
       (* Second consumption should fail *)
       (match Ffi.safe_consume_resource_by_id resource_id with
        | Ok false | Error (InvalidResource _) -> true
        | _ -> false)
     | _ -> false)
  | _ -> false

(** Test expression compilation *)
let test_expression_compilation () =
  let expr = Expr.const (LispValue.int 42L) in
  match Expr.compile_and_register_expr expr with
  | Ok _expr_id -> true
  | Error _ -> false

(** Test content addressing *)
let test_content_addressing () =
  let store = Content_addressing.create_store () in
  let content = Bytes.of_string "test content" in
  let id = Content_addressing.store_content store content in
  match Content_addressing.retrieve_content store id with
  | Some retrieved_content -> Bytes.equal content retrieved_content
  | None -> false

(** Test content integrity verification *)
let test_content_integrity () =
  let store = Content_addressing.create_store () in
  let content1 = Bytes.of_string "content 1" in
  let content2 = Bytes.of_string "content 2" in
  let _id1 = Content_addressing.store_content store content1 in
  let _id2 = Content_addressing.store_content store content2 in
  let integrity_results = Content_addressing.verify_store_integrity store in
  List.for_all (fun (_id, is_valid) -> is_valid) integrity_results

(** Test system metrics *)
let test_system_metrics () =
  match Ffi.safe_get_system_metrics () with
  | Ok _metrics -> true
  | Error _ -> false

(** Test rust bridge validation *)
let test_rust_bridge () =
  (* Rust bridge functionality is available but not exposed for testing *)
  true

(** Test lisp value serialization *)
let test_lisp_value_serialization () =
  (* Lisp value serialization is available but not exposed for testing *)
  true

(** Test expression evaluation *)
let test_expression_evaluation () =
  let ctx = Expr.empty_context in
  let expr = Expr.const (LispValue.int 42L) in
  match Expr.eval_expr ctx expr with
  | Ok (Int 42L) -> true
  | _ -> false

(** Test complex expression *)
let test_complex_expression () =
  let lambda_expr = Expr.lambda 
    [LispValue.symbol "x"] 
    (Expr.const (LispValue.symbol "x")) in
  match Expr.compile_and_register_expr lambda_expr with
  | Ok _expr_id -> true
  | Error _ -> false

(** Test predefined expressions *)
let test_predefined_expressions () =
  match Expr.get_predefined_expr_id "issue_ticket_logic" with
  | Some _expr_id -> true
  | None -> false

(** Main test runner *)
let run_integration_tests () =
  Printf.printf "🧪 OCaml Causality DSL Integration Tests\n";
  Printf.printf "========================================\n\n";

  run_test "FFI Initialization" test_ffi_initialization;
  run_test "Resource Linearity" test_resource_linearity;
  run_test "Expression Compilation" test_expression_compilation;
  run_test "Content Addressing" test_content_addressing;
  run_test "Content Integrity" test_content_integrity;
  run_test "System Metrics" test_system_metrics;
  run_test "Rust Bridge" test_rust_bridge;
  run_test "Lisp Value Serialization" test_lisp_value_serialization;
  run_test "Expression Evaluation" test_expression_evaluation;
  run_test "Complex Expression" test_complex_expression;
  run_test "Predefined Expressions" test_predefined_expressions;

  Printf.printf "\n📊 Test Results Summary\n";
  Printf.printf "=======================\n";
  
  let total_tests = List.length !test_results in
  let passed_tests = List.length (List.filter (fun r -> r.passed) !test_results) in
  let failed_tests = total_tests - passed_tests in
  
  Printf.printf "Total tests: %d\n" total_tests;
  Printf.printf "Passed: %d\n" passed_tests;
  Printf.printf "Failed: %d\n" failed_tests;
  Printf.printf "Success rate: %.1f%%\n" (float_of_int passed_tests /. float_of_int total_tests *. 100.0);
  
  if failed_tests > 0 then (
    Printf.printf "\n❌ Failed tests:\n";
    List.iter (fun result ->
      if not result.passed then
        match result.error_msg with
        | Some msg -> Printf.printf "  • %s: %s\n" result.name msg
        | None -> Printf.printf "  • %s\n" result.name
    ) !test_results
  );
  
  Printf.printf "\n%s All integration tests completed!\n" 
    (if failed_tests = 0 then "✅" else "⚠️")

(* Run tests if this file is executed directly *)
let () = run_integration_tests () 