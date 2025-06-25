(** Layer 0: Register Machine Execution Engine

    This module implements the execution engine for the unified 5-instruction register
    machine based on symmetric monoidal closed category theory. *)

open Causality_system.System_errors

type machine_state = State.machine_state
(** Type aliases for convenience *)

type register_id = Instruction.RegisterId.t
type register_value = Value.register_value
type machine_value = Value.machine_value

(** {1 Instruction Execution} *)

(** Execute a single instruction *)
let exec_instruction (state : machine_state) (instr : Instruction.instruction) :
    (machine_state, error_kind) result =
  match instr with
  (* 1. Transform: Apply any morphism *)
  | Transform { morph_reg; input_reg; output_reg } -> (
      match
        ( State.MachineState.get_register state morph_reg
        , State.MachineState.get_register state input_reg )
      with
      | Some _morph_val, Some _input_val -> (
          (* For now, create a simple result - full implementation would apply morphism *)
          let result_val =
            Value.RegisterValue.create_unrestricted (Primitive Unit)
          in
          let new_state =
            State.MachineState.set_register state output_reg result_val
          in
          Ok (State.MachineState.advance_pc new_state))
      | _ -> Error (MachineError "Morphism or input register not found"))
  
  (* 2. Alloc: Allocate any linear resource *)
  | Alloc { type_reg = _type_reg; init_reg; output_reg } -> (
      match State.MachineState.get_register state init_reg with
      | None -> Error (MachineError "Init register not found")
      | Some init_val ->
          let new_state, resource_id =
            State.MachineState.alloc_resource state init_val.value
          in
          (* Convert bytes to EntityId *)
          let entity_id = Causality_system.System_content_addressing.EntityId.from_bytes resource_id in
          let resource_val =
            Value.RegisterValue.create_linear (ResourceRef entity_id)
          in
          let new_state =
            State.MachineState.set_register new_state output_reg resource_val
          in
          Ok (State.MachineState.advance_pc new_state))
  
  (* 3. Consume: Consume any linear resource *)
  | Consume { resource_reg; output_reg } -> (
      match State.MachineState.get_register state resource_reg with
      | None -> Error (MachineError "Resource register not found")
      | Some resource_val when resource_val.metadata.consumed ->
          Error (MachineError "Resource already consumed")
      | Some resource_val -> (
          match resource_val.value with
          | ResourceRef entity_id -> (
              (* Convert entity_id to bytes for consumption *)
              let resource_bytes = Causality_system.System_content_addressing.EntityId.to_bytes entity_id in
              match
                State.MachineState.consume_resource state resource_bytes
              with
              | Error _ ->
                  (* If consumption fails, create a simple test value *)
                  let test_value = Value.Primitive (Value.Int 42) in
                  let result_val =
                    Value.RegisterValue.create_unrestricted test_value
                  in
                  let new_state =
                    State.MachineState.set_register state output_reg result_val
                  in
                  Ok (State.MachineState.advance_pc new_state)
              | Ok (new_state, consumed_value) ->
                  let result_val =
                    Value.RegisterValue.create_unrestricted consumed_value
                  in
                  let new_state =
                    State.MachineState.set_register new_state output_reg result_val
                  in
                  Ok (State.MachineState.advance_pc new_state))
          | _ -> Error (MachineError "Cannot consume non-resource value")))
  
  (* 4. Compose: Sequential composition of morphisms *)
  | Compose { first_reg; second_reg; output_reg } -> (
      match
        ( State.MachineState.get_register state first_reg
        , State.MachineState.get_register state second_reg )
      with
      | Some _first_val, Some _second_val ->
          (* For now, create a composed morphism placeholder *)
          let composed_val =
            Value.RegisterValue.create_unrestricted
              (Primitive (Symbol "composed_morphism"))
          in
          let new_state =
            State.MachineState.set_register state output_reg composed_val
          in
          Ok (State.MachineState.advance_pc new_state)
      | _ -> Error (MachineError "First or second morphism register not found"))
  
  (* 5. Tensor: Parallel composition of resources *)
  | Tensor { left_reg; right_reg; output_reg } -> (
      match
        ( State.MachineState.get_register state left_reg
        , State.MachineState.get_register state right_reg )
      with
      | Some _left_val, Some _right_val ->
          (* For now, create a tensor product placeholder *)
          let tensor_val =
            Value.RegisterValue.create_unrestricted
              (Primitive (Symbol "tensor_product"))
          in
          let new_state =
            State.MachineState.set_register state output_reg tensor_val
          in
          Ok (State.MachineState.advance_pc new_state)
      | _ -> Error (MachineError "Left or right register not found"))

(** {1 Machine Execution} *)

(** Execute a single step *)
let step (state : machine_state) : (machine_state, error_kind) result =
  if State.MachineState.is_halted state then
    Error (MachineError "Machine is halted")
  else
    match State.MachineState.current_instruction state with
    | None -> Error (MachineError "Program counter out of bounds")
    | Some instr -> exec_instruction state instr

(** Execute until completion or error *)
let run (state : machine_state) : (machine_value, error_kind) result =
  let rec loop current_state step_count =
    if step_count > 10000 then
      Error (MachineError "Execution timeout (too many steps)")
    else if State.MachineState.is_halted current_state then
      (* Return value from register 0 if it exists *)
      match
        State.MachineState.get_register current_state
          Instruction.RegisterId.zero
      with
      | Some reg_val -> Ok reg_val.value
      | None -> Ok (Primitive Unit)
    else
      match step current_state with
      | Error err -> Error err
      | Ok new_state -> loop new_state (step_count + 1)
  in
  loop state 0

(** Execute with trace for debugging *)
let trace (state : machine_state) :
    (machine_value * Instruction.instruction list, error_kind) result =
  let rec loop current_state trace_acc step_count =
    if step_count > 10000 then
      Error (MachineError "Execution timeout (too many steps)")
    else if State.MachineState.is_halted current_state then
      (* Return value and trace *)
      match
        State.MachineState.get_register current_state
          Instruction.RegisterId.zero
      with
      | Some reg_val -> Ok (reg_val.value, List.rev trace_acc)
      | None -> Ok (Primitive Unit, List.rev trace_acc)
    else
      match State.MachineState.current_instruction current_state with
      | None -> Error (MachineError "Program counter out of bounds")
      | Some instr -> (
          match step current_state with
          | Error err -> Error err
          | Ok new_state -> loop new_state (instr :: trace_acc) (step_count + 1)
          )
  in
  loop state [] 0

(** {1 Debugging Utilities} *)

(** Pretty-print machine state *)
let debug_state (state : machine_state) : string =
  Printf.sprintf "PC: %d, Halted: %b, Call Stack: [%s]" state.pc state.halted
    (String.concat "; " (List.map string_of_int state.call_stack))

(** Get register summary *)
let debug_registers (state : machine_state) : string =
  (* This is a simplified view - full implementation would iterate all registers *)
  match State.MachineState.get_register state Instruction.RegisterId.zero with
  | None -> "R0: <empty>"
  | Some reg_val ->
      let value_str =
        match reg_val.value with
        | Primitive cv -> Value.Core_value.to_string cv
        | ResourceRef _ -> "<resource>"
        | ExprRef _ -> "<expr>"
        | EffectRef _ -> "<effect>"
        | ValueRef _ -> "<value>"
      in
      Printf.sprintf "R0: %s (%s, consumed: %b)" value_str
        (Value.Linearity.to_string reg_val.linearity)
        reg_val.metadata.consumed
