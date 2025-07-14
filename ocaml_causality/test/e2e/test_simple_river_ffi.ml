(* Simple River FFI Test - Verify basic functionality *)

open Printf

(* Import FFI bindings *)
open Ocaml_causality

let test_basic_ffi () =
  printf "=== Basic FFI Test ===\n";
  
  (* Test registry stats *)
  let (engines, envs) = Compiler.Simulation_ffi.get_registry_stats () in
  printf "Initial registry state - Engines: %Ld, Environments: %Ld\n" engines envs;
  
  (* Create a simulation engine *)
  let engine = Compiler.Simulation_ffi.create_engine () in
  printf "Created engine with handle: %d\n" engine;
  
  (* Test registry stats after creation *)
  let (engines2, envs2) = Compiler.Simulation_ffi.get_registry_stats () in
  printf "After creation - Engines: %Ld, Environments: %Ld\n" engines2 envs2;
  
  (* Test basic instruction execution *)
  printf "Testing instruction execution...\n";
  begin match Compiler.Simulation_ffi.simulate_instructions engine 5 with
  | Ok result ->
    printf "Instruction execution result: %s\n" result;
  | Error err ->
    printf "Instruction execution failed: %s\n" err;
  end;
  
  (* Test Lisp compilation and execution *)
  printf "Testing Lisp compilation...\n";
  let simple_lisp = "(+ 1 2)" in
  begin match Compiler.Simulation_ffi.simulate_lisp_code engine simple_lisp with
  | Ok result ->
    printf "Lisp execution result: %s\n" result;
  | Error err ->
    printf "Lisp execution failed: %s\n" err;
  end;
  
  (* Get engine stats *)
  let (steps, gas, effects) = Compiler.Simulation_ffi.get_engine_stats engine in
  printf "Engine stats - Steps: %d, Gas: %Ld, Effects: %d\n" steps gas effects;
  
  (* Test snapshot creation *)
  printf "Testing snapshot creation...\n";
  let snapshot_id = Compiler.Simulation_ffi.snapshot_engine engine "test_snapshot" in
  printf "Created snapshot: %s\n" snapshot_id;
  
  (* Test engine reset *)
  printf "Testing engine reset...\n";
  let reset_success = Compiler.Simulation_ffi.reset_engine engine in
  printf "Engine reset: %s\n" (if reset_success then "success" else "failed");
  
  (* Cleanup engine *)
  let cleanup_success = Compiler.Simulation_ffi.cleanup_engine engine in
  printf "Engine cleanup: %s\n" (if cleanup_success then "success" else "failed");
  
  (* Final registry stats *)
  let (engines3, envs3) = Compiler.Simulation_ffi.get_registry_stats () in
  printf "Final registry state - Engines: %Ld, Environments: %Ld\n" engines3 envs3;
  
  printf "=== Basic FFI Test Complete ===\n";
  true

let () =
  let success = test_basic_ffi () in
  exit (if success then 0 else 1) 