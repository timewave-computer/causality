(* Layer 2 Compilation Tests *)
(* Purpose: Test each compilation phase individually *)

open Ocaml_causality
open Causality_core.Combined_types

(* Test helper function to create test intent *)
let create_test_intent name =
  let domain_id = Bytes.create 32 in
  Bytes.fill domain_id 0 32 '\000';
  let entity_id = Causality_system.System_content_addressing.EntityId.from_content name in
  {
    id = entity_id;
    name = name;
    domain_id = domain_id;
    priority = 2;
    inputs = [];
    outputs = [];
    expression = None;
    timestamp = Int64.of_int 0;
    hint = None;
  }

(* Test FFI type conversion *)
let test_ffi_type_conversion () =
  let intent = create_test_intent "ffi_test" in
  let expr_id = Compiler.compile_intent intent in
  
  Printf.printf "FFI type conversion: ";
  if Bytes.length expr_id > 0 then
    Printf.printf "PASS\n"
  else
    Printf.printf "FAIL\n"

(* Test Layer 2 -> Layer 1 compilation *)
let test_layer2_to_layer1 () =
  let intent = create_test_intent "layer1_test" in
  let expr_id = Compiler.compile_intent intent in
  
  Printf.printf "Layer 2 -> Layer 1 compilation: ";
  if Bytes.length expr_id = 32 then
    Printf.printf "PASS\n"
  else
    Printf.printf "FAIL\n"

(* Main test runner *)
let () =
  Printf.printf "Running Layer 2 Compilation Tests\n";
  Printf.printf "==================================\n";
  
  try
    test_ffi_type_conversion ();
    test_layer2_to_layer1 ();
    Printf.printf "All tests completed\n"
  with
  | exn ->
    Printf.printf "Test failed with error: %s\n" (Printexc.to_string exn)
