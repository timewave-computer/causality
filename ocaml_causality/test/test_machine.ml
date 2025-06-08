(* Tests for the register machine execution engine *)

open Causality_machine
open Causality_system.System_errors

(* Helper to create a simple program *)
let simple_program () =
  [|
     (* Load unit value into register 1 *)
     Instruction.Move
       {
         src = Instruction.RegisterId.create 0l
       ; dst = Instruction.RegisterId.create 1l
       }
   ; (* Return the value *)
     Instruction.Return { result_reg = Some (Instruction.RegisterId.create 1l) }
  |]

(* Test basic machine creation and execution *)
let test_machine_creation () =
  let program = simple_program () in
  let state = State.MachineState.create program in

  (* Check initial state *)
  assert (not (State.MachineState.is_halted state));
  assert (state.pc = 0);

  (* Set up register 0 with a unit value *)
  let unit_val =
    Value.RegisterValue.create_unrestricted (Value.Primitive Value.Unit)
  in
  let _state_with_reg =
    State.MachineState.set_register state
      (Instruction.RegisterId.create 0l)
      unit_val
  in

  Printf.printf "✓ Machine creation test passed\n"

(* Test single step execution *)
let test_single_step () =
  let program = simple_program () in
  let state = State.MachineState.create program in

  (* Set up register 0 with a unit value *)
  let unit_val =
    Value.RegisterValue.create_unrestricted (Value.Primitive Value.Unit)
  in
  let state =
    State.MachineState.set_register state
      (Instruction.RegisterId.create 0l)
      unit_val
  in

  (* Execute one step (Move instruction) *)
  match Reduction.step state with
  | Error err ->
      Printf.printf "✗ Single step test failed: %s\n" (string_of_error_kind err);
      assert false
  | Ok new_state -> (
      (* Check that PC advanced *)
      assert (new_state.pc = 1);

      (* Check that register 1 now has the unit value *)
      match
        State.MachineState.get_register new_state
          (Instruction.RegisterId.create 1l)
      with
      | None ->
          Printf.printf "✗ Single step test failed: register 1 not found\n";
          assert false
      | Some reg_val ->
          assert (reg_val.value = Value.Primitive Value.Unit);
          Printf.printf "✓ Single step execution test passed\n")

(* Test constraint evaluation *)
let test_constraint_evaluation () =
  let program = simple_program () in
  let state = State.MachineState.create program in

  (* Set up registers with values *)
  let int_val1 =
    Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Int 5))
  in
  let int_val2 =
    Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Int 10))
  in
  let state =
    State.MachineState.set_register state
      (Instruction.RegisterId.create 1l)
      int_val1
  in
  let state =
    State.MachineState.set_register state
      (Instruction.RegisterId.create 2l)
      int_val2
  in

  (* Test various constraints *)
  let constraint_true = Instruction.True in
  let constraint_false = Instruction.False in
  let constraint_less =
    Instruction.LessThan
      (Instruction.RegisterId.create 1l, Instruction.RegisterId.create 2l)
  in
  let constraint_greater =
    Instruction.GreaterThan
      (Instruction.RegisterId.create 1l, Instruction.RegisterId.create 2l)
  in

  assert (Reduction.eval_constraint state constraint_true = true);
  assert (Reduction.eval_constraint state constraint_false = false);
  assert (Reduction.eval_constraint state constraint_less = true);
  (* 5 < 10 *)
  assert (Reduction.eval_constraint state constraint_greater = false);

  (* 5 > 10 is false *)
  Printf.printf "✓ Constraint evaluation test passed\n"

(* Test effect execution with pre/post conditions *)
let test_effect_execution () =
  let program =
    [|
       Instruction.Perform
         {
           effect =
             {
               tag = "test_effect"
             ; pre = Instruction.True
             ; post = Instruction.True
             ; hints = []
             }
         ; out_reg = Instruction.RegisterId.create 1l
         }
    |]
  in
  let state = State.MachineState.create program in

  match Reduction.step state with
  | Error err ->
      Printf.printf "✗ Effect execution test failed: %s\n"
        (string_of_error_kind err);
      assert false
  | Ok new_state -> (
      (* Check that output register was set *)
      match
        State.MachineState.get_register new_state
          (Instruction.RegisterId.create 1l)
      with
      | None ->
          Printf.printf
            "✗ Effect execution test failed: output register not found\n";
          assert false
      | Some reg_val ->
          assert (reg_val.value = Value.Primitive Value.Unit);
          Printf.printf "✓ Effect execution test passed\n")

(* Test resource allocation and consumption *)
let test_resource_allocation () =
  let program =
    [|
       (* Allocate a resource *)
       Instruction.Alloc
         {
           type_reg = Instruction.RegisterId.create 0l
         ; val_reg = Instruction.RegisterId.create 1l
         ; out_reg = Instruction.RegisterId.create 2l
         }
     ; (* Consume the resource *)
       Instruction.Consume
         {
           resource_reg = Instruction.RegisterId.create 2l
         ; out_reg = Instruction.RegisterId.create 3l
         }
    |]
  in
  let state = State.MachineState.create program in

  (* Set up input registers *)
  let type_val =
    Value.RegisterValue.create_unrestricted
      (Value.Primitive (Value.Symbol "TestType"))
  in
  let val_val =
    Value.RegisterValue.create_unrestricted (Value.Primitive (Value.Int 42))
  in
  let state =
    State.MachineState.set_register state
      (Instruction.RegisterId.create 0l)
      type_val
  in
  let state =
    State.MachineState.set_register state
      (Instruction.RegisterId.create 1l)
      val_val
  in

  (* Execute allocation *)
  match Reduction.step state with
  | Error err ->
      Printf.printf "✗ Resource allocation test failed at alloc: %s\n"
        (string_of_error_kind err);
      assert false
  | Ok state_after_alloc -> (
      (* Check that resource reference was created *)
      match
        State.MachineState.get_register state_after_alloc
          (Instruction.RegisterId.create 2l)
      with
      | None ->
          Printf.printf
            "✗ Resource allocation test failed: resource not created\n";
          assert false
      | Some reg_val -> (
          match reg_val.value with
          | Value.ResourceRef _ -> (
              (* Now execute consumption *)
              match Reduction.step state_after_alloc with
              | Error err ->
                  Printf.printf
                    "✗ Resource allocation test failed at consume: %s\n"
                    (string_of_error_kind err);
                  assert false
              | Ok state_after_consume -> (
                  (* Check that consumption result was created *)
                  match
                    State.MachineState.get_register state_after_consume
                      (Instruction.RegisterId.create 3l)
                  with
                  | None ->
                      Printf.printf
                        "✗ Resource allocation test failed: consumption result \
                         not found\n";
                      assert false
                  | Some _result_val ->
                      Printf.printf
                        "✓ Resource allocation and consumption test passed\n"))
          | _ ->
              Printf.printf
                "✗ Resource allocation test failed: not a resource reference\n";
              assert false))

(* Test debugging utilities *)
let test_debugging () =
  let program = simple_program () in
  let state = State.MachineState.create program in

  let debug_output = Reduction.debug_state state in
  assert (String.contains debug_output '0');

  (* Should contain PC = 0 *)
  let reg_output = Reduction.debug_registers state in
  assert (String.contains reg_output 'R');

  (* Should contain register info *)
  Printf.printf "✓ Debugging utilities test passed\n"

(* Run all tests *)
let () =
  Printf.printf "Running register machine tests...\n\n";
  test_machine_creation ();
  test_single_step ();
  test_constraint_evaluation ();
  test_effect_execution ();
  test_resource_allocation ();
  test_debugging ();
  Printf.printf "\n✅ All register machine tests passed!\n"
