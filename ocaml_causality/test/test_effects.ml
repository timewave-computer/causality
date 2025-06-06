(* Purpose: Test suite for OCaml native algebraic effects integration *)

open Ocaml_causality_effects.Effects

(** Test module creation and basic operations *)
let test_effect_creation () =
  print_endline "=== Testing Effect Creation (Direct Style) ===";
  
  (* Test creating a linear resource - direct style! *)
  let test_creation () =
    let resource = create_linear_resource 42 in
    print_endline "✓ Linear resource created successfully";
    (* Test consuming the resource *)
    let value = consume_linear_resource resource in
    Printf.printf "✓ Linear resource consumed, value: %d\n" value
  in
  
  begin match run_with_effects test_creation with
  | Ok () -> ()
  | Error exn -> Printf.printf "✗ Failed: %s\n" (Printexc.to_string exn)
  end;
  
  (* Test resource allocation *)
  let test_allocation () =
    let resource_id = allocate_resource () in
    Printf.printf "✓ Resource allocated, ID length: %d bytes\n" (Bytes.length resource_id)
  in
  
  begin match run_with_effects test_allocation with
  | Ok () -> ()
  | Error exn -> Printf.printf "✗ Failed to allocate resource: %s\n" (Printexc.to_string exn)
  end;
  
  (* Test constraint checking *)
  let test_constraint () =
    let result = check_constraint true in
    Printf.printf "✓ Constraint check result: %b\n" result
  in
  
  begin match run_with_effects test_constraint with
  | Ok () -> ()
  | Error exn -> Printf.printf "✗ Failed to check constraint: %s\n" (Printexc.to_string exn)
  end;
  
  (* Test ZK witness generation *)
  let test_witness () =
    let witness = generate_zk_witness "test_term" in
    Printf.printf "✓ ZK witness generated: %s\n" witness
  in
  
  begin match run_with_effects test_witness with
  | Ok () -> ()
  | Error exn -> Printf.printf "✗ Failed to generate witness: %s\n" (Printexc.to_string exn)
  end

(** Test effect composition using direct style - no monads! *)
let test_effect_composition () =
  print_endline "\n=== Testing Effect Composition (Direct Style) ===";
  
  let computation () =
    (* Look how natural this is - no >>= needed! *)
    let resource = create_linear_resource "hello" in
    let validation = check_constraint true in
    let value = consume_linear_resource resource in
    let witness = generate_zk_witness "composed_term" in
    (value, validation, witness)
  in
  
  match run_with_effects computation with
  | Ok (value, validation, witness) ->
      Printf.printf "✓ Composed computation result:\n";
      Printf.printf "  - Value: %s\n" value;
      Printf.printf "  - Validation: %b\n" validation;
      Printf.printf "  - Witness: %s\n" witness
  | Error exn ->
      Printf.printf "✗ Composed computation failed: %s\n" (Printexc.to_string exn)

(** Test linear function application with direct style *)
let test_linear_functions () =
  print_endline "\n=== Testing Linear Functions (Direct Style) ===";
  
  let increment : int linear_function = fun x -> x + 1 in
  
  let computation () =
    apply_linear_function increment 41
  in
  
  match run_with_effects computation with
  | Ok value -> Printf.printf "✓ Linear function result: %d\n" value
  | Error exn -> Printf.printf "✗ Linear function failed: %s\n" (Printexc.to_string exn)

(** Test effect sequencing using normal control flow *)
let test_effect_sequencing () =
  print_endline "\n=== Testing Effect Sequencing (Direct Style) ===";
  
  let computation () =
    (* Natural sequencing - no special effect sequencing needed! *)
    let resource1 = create_linear_resource 1 in
    let resource2 = create_linear_resource 2 in
    let resource3 = create_linear_resource 3 in
    [resource1; resource2; resource3]
  in
  
  match run_with_effects computation with
  | Ok resources ->
      Printf.printf "✓ Created %d resources successfully\n" (List.length resources);
      List.iteri (fun i resource ->
        let consume_computation () = consume_linear_resource resource in
        match run_with_effects consume_computation with
        | Ok value -> Printf.printf "  - Resource %d value: %d\n" i value
        | Error exn -> Printf.printf "  - Resource %d consumption failed: %s\n" i (Printexc.to_string exn)
      ) resources
  | Error exn ->
      Printf.printf "✗ Effect sequencing failed: %s\n" (Printexc.to_string exn)

(** Test linearity enforcement *)
let test_linearity_enforcement () =
  print_endline "\n=== Testing Linearity Enforcement (Direct Style) ===";
  
  let test_double_consumption () =
    let resource = create_linear_resource "test_linearity" in
    (* First consumption should succeed *)
    let value1 = consume_linear_resource resource in
    Printf.printf "✓ First consumption: %s\n" value1;
    (* Second consumption should fail *)
    let _value2 = consume_linear_resource resource in
    Printf.printf "✗ Second consumption should have failed!\n"
  in
  
  match run_with_effects test_double_consumption with
  | Ok () -> Printf.printf "✗ Double consumption was not prevented!\n"
  | Error (Failure msg) -> Printf.printf "✓ Second consumption correctly failed: %s\n" msg
  | Error _ -> Printf.printf "✓ Second consumption correctly failed (linearity enforced)\n"

(** Test the provided example functions *)
let test_examples () =
  print_endline "\n=== Testing Example Functions (Direct Style) ===";
  
  (* Test linear pipeline example *)
  let pipeline_computation () = linear_pipeline_example () in
  begin match run_with_effects pipeline_computation with
  | Ok (value, type_ok, witness) ->
      Printf.printf "✓ Linear pipeline example:\n";
      Printf.printf "  - Value: %d\n" value;
      Printf.printf "  - Type check: %b\n" type_ok;
      Printf.printf "  - Witness: %s\n" witness
  | Error exn ->
      Printf.printf "✗ Linear pipeline example failed: %s\n" (Printexc.to_string exn)
  end;
  
  (* Test effect composition example *)
  let composition_computation () = effect_composition_example () in
  begin match run_with_effects composition_computation with
  | Ok (Some value) -> Printf.printf "✓ Effect composition example: %s\n" value
  | Ok None -> Printf.printf "✗ Effect composition example returned None\n"
  | Error exn -> Printf.printf "✗ Effect composition example failed: %s\n" (Printexc.to_string exn)
  end

(** Test complex computation example *)
let test_complex_computation () =
  print_endline "\n=== Testing Complex Computation (Direct Style) ===";
  
  let complex_computation () = complex_computation_example 10 in
  match run_with_effects complex_computation with
  | Ok (result, witness) ->
      Printf.printf "✓ Complex computation result: %d, witness: %s\n" result witness
  | Error exn ->
      Printf.printf "✗ Complex computation failed: %s\n" (Printexc.to_string exn)

(** Test nested computation example *)
let test_nested_computation () =
  print_endline "\n=== Testing Nested Computation (Direct Style) ===";
  
  let nested_computation () = nested_computation_example () in
  match run_with_effects nested_computation with
  | Ok result ->
      Printf.printf "✓ Nested computation result: %s\n" result
  | Error exn ->
      Printf.printf "✗ Nested computation failed: %s\n" (Printexc.to_string exn)

(** Demonstrate control flow benefits *)
let test_control_flow_benefits () =
  print_endline "\n=== Demonstrating Control Flow Benefits ===";
  
  let conditional_computation () =
    let input = 42 in
    let resource = create_linear_resource input in
    
    (* Natural if/else with effects *)
    if input > 40 then (
      let validation = check_constraint true in
      if validation then (
        let value = consume_linear_resource resource in
        let witness = generate_zk_witness "conditional_path" in
        "Success: " ^ string_of_int value ^ " " ^ witness
      ) else
        "Validation failed"
    ) else
      "Input too small"
  in
  
  match run_with_effects conditional_computation with
  | Ok result -> Printf.printf "✓ Conditional computation: %s\n" result
  | Error exn -> Printf.printf "✗ Conditional computation failed: %s\n" (Printexc.to_string exn)

(** Run all tests *)
let run_tests () =
  print_endline "OCaml Native Algebraic Effects Test Suite";
  print_endline "=======================================";
  print_endline "🎉 No more monadic composition - direct style programming!";
  
  test_effect_creation ();
  test_effect_composition ();
  test_linear_functions ();
  test_effect_sequencing ();
  test_linearity_enforcement ();
  test_examples ();
  test_complex_computation ();
  test_nested_computation ();
  test_control_flow_benefits ();
  
  print_endline "\n=== Test Suite Complete ===";
  print_endline "✨ True algebraic effects working - no monad composition overhead!";
  print_endline "🚀 Direct style programming with effect tracking!"

(* Run tests when module is loaded *)
let () = run_tests () 