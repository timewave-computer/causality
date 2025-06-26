(** Tests for the Layer 0 register machine *)

open Causality_machine

let test_basic_execution () =
  (* Create a simple program using the 5-instruction system *)
  let program = [|
    (* Allocate a resource with value 42 *)
    Instruction.Alloc {
      type_reg = Instruction.RegisterId.create 0l;
      init_reg = Instruction.RegisterId.create 1l;
      output_reg = Instruction.RegisterId.create 2l;
    };
    (* Transform the resource *)
    Instruction.Transform {
      morph_reg = Instruction.RegisterId.create 3l;
      input_reg = Instruction.RegisterId.create 2l;
      output_reg = Instruction.RegisterId.create 4l;
    };
    (* Consume the resource *)
    Instruction.Consume {
      resource_reg = Instruction.RegisterId.create 4l;
      output_reg = Instruction.RegisterId.create 5l;
    };
  |] in
  
  (* Create initial machine state *)
  let state = State.MachineState.create program in
  
  (* Set up initial registers *)
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 0l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Symbol "int"))) in
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 1l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Int 42))) in
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 3l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Symbol "identity"))) in
  
  (* Execute the program *)
  match Reduction.run state with
  | Ok _result -> print_endline "Test passed: Basic execution successful"
  | Error err -> 
      Printf.printf "Test failed: %s\n" (Causality_system.System_errors.string_of_error_kind err)

let test_compose_tensor () =
  (* Test composition and tensor operations *)
  let program = [|
    (* Create two morphisms *)
    Instruction.Alloc {
      type_reg = Instruction.RegisterId.create 0l;
      init_reg = Instruction.RegisterId.create 1l;
      output_reg = Instruction.RegisterId.create 2l;
    };
    Instruction.Alloc {
      type_reg = Instruction.RegisterId.create 0l;
      init_reg = Instruction.RegisterId.create 3l;
      output_reg = Instruction.RegisterId.create 4l;
    };
    (* Compose them *)
    Instruction.Compose {
      first_reg = Instruction.RegisterId.create 2l;
      second_reg = Instruction.RegisterId.create 4l;
      output_reg = Instruction.RegisterId.create 5l;
    };
    (* Create two resources *)
    Instruction.Alloc {
      type_reg = Instruction.RegisterId.create 6l;
      init_reg = Instruction.RegisterId.create 7l;
      output_reg = Instruction.RegisterId.create 8l;
    };
    Instruction.Alloc {
      type_reg = Instruction.RegisterId.create 6l;
      init_reg = Instruction.RegisterId.create 9l;
      output_reg = Instruction.RegisterId.create 10l;
    };
    (* Tensor them *)
    Instruction.Tensor {
      left_reg = Instruction.RegisterId.create 8l;
      right_reg = Instruction.RegisterId.create 10l;
      output_reg = Instruction.RegisterId.create 11l;
    };
  |] in
  
  let state = State.MachineState.create program in
  
  (* Set up initial registers for morphism types and values *)
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 0l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Symbol "morphism"))) in
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 1l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Symbol "add_one"))) in
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 3l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Symbol "multiply_two"))) in
  
  (* Set up initial registers for resource types and values *)
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 6l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Symbol "int"))) in
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 7l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Int 10))) in
  let state = State.MachineState.set_register state (Instruction.RegisterId.create 9l)
    (Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Int 20))) in
  
  (* Execute the program *)
  match Reduction.run state with
  | Ok _result -> print_endline "Test passed: Compose and tensor operations successful"
  | Error err -> 
      Printf.printf "Test failed: %s\n" (Causality_system.System_errors.string_of_error_kind err)

let () =
  print_endline "Running Layer 0 machine tests...";
  test_basic_execution ();
  test_compose_tensor ();
  print_endline "Machine tests completed."
