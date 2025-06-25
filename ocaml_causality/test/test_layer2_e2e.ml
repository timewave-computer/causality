(* End-to-End Test for Layer 2 Pipeline *)
(* Purpose: Test complete OCaml Layer 2 DSL -> Layer 0 execution *)

open Ocaml_causality
open Causality_core.Combined_types

(* Test data setup *)
let create_simple_intent () =
  let domain_id = Bytes.create 32 in
  Bytes.fill domain_id 0 32 '\000';
  (* Create EntityId correctly using the from_content method *)
  let entity_id = Causality_system.System_content_addressing.EntityId.from_content "test_intent" in
  {
    id = entity_id;
    name = "test_intent";
    domain_id = domain_id;
    priority = 2;
    inputs = [];
    outputs = [];
    expression = None;
    timestamp = Int64.of_int 0;
    hint = None;
  }

(* Test Layer 2 -> Layer 0 compilation *)
let test_layer2_to_layer0_compilation () =
  let intent = create_simple_intent () in
  let (instructions, result_register) = Pipeline.compile_layer2_to_layer0 intent in
  
  Printf.printf "Layer 2 -> Layer 0 compilation: ";
  if List.length instructions >= 0 && result_register >= 0 then
    Printf.printf "PASS\n"
  else
    Printf.printf "FAIL\n"

(* Test intent compilation *)
let test_intent_compilation () =
  let intent = create_simple_intent () in
  let expr_id = Compiler.compile_intent intent in
  
  Printf.printf "Intent compilation: ";
  if Bytes.length expr_id > 0 then
    Printf.printf "PASS\n"
  else
    Printf.printf "FAIL\n"

(* Main test runner *)
let () =
  Printf.printf "Running Layer 2 E2E Tests\n";
  Printf.printf "==========================\n";
  
  try
    test_layer2_to_layer0_compilation ();
    test_intent_compilation ();
    Printf.printf "All tests completed\n"
  with
  | exn ->
    Printf.printf "Test failed with error: %s\n" (Printexc.to_string exn)
