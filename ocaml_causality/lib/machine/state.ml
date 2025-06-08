(** Layer 0: Register Machine State

    This module implements the complete machine state including registers,
    resource heap, program counter, and linearity enforcement. *)

open Value
open Instruction

type register_id = RegisterId.t
(** Type aliases for convenience *)

type register_value = Value.register_value
type machine_value = Value.machine_value

(** {1 Resource Management} *)

(** Resource heap for linear resource management *)
module ResourceHeap = struct
  type t = (bytes, machine_value * bool) Hashtbl.t
  (* resource_id -> (value, consumed) *)

  (** Create empty resource heap *)
  let create () : t = Hashtbl.create 256

  (** Allocate a new resource *)
  let alloc (heap : t) (value : machine_value) : t * bytes =
    (* Generate a simple resource ID - in reality this would be content-addressed *)
    let resource_id = Bytes.create 32 in
    (* Fill with some deterministic content for now *)
    for i = 0 to 31 do
      Bytes.set_uint8 resource_id i (Random.int 256)
    done;
    Hashtbl.add heap resource_id (value, false);
    (heap, resource_id)

  (** Consume a resource (linear resource management) *)
  let consume (heap : t) (resource_id : bytes) : (t, string) result =
    match Hashtbl.find_opt heap resource_id with
    | None -> Error "Resource not found"
    | Some (_, true) -> Error "Resource already consumed"
    | Some (value, false) ->
        Hashtbl.replace heap resource_id (value, true);
        Ok heap

  (** Check if resource is consumed *)
  let is_consumed (heap : t) (resource_id : bytes) : bool =
    match Hashtbl.find_opt heap resource_id with
    | None -> true (* Not found = consumed *)
    | Some (_, consumed) -> consumed

  (** Get resource value (if not consumed) *)
  let get_value (heap : t) (resource_id : bytes) : machine_value option =
    match Hashtbl.find_opt heap resource_id with
    | None -> None
    | Some (_, true) -> None (* Consumed *)
    | Some (value, false) -> Some value
end

(** Nullifier set for preventing double-spending *)
module NullifierSet = struct
  type t = (bytes, unit) Hashtbl.t

  (** Create empty nullifier set *)
  let create () : t = Hashtbl.create 256

  (** Add nullifier (fails if already exists) *)
  let add (nullifiers : t) (nullifier_id : bytes) : (t, string) result =
    if Hashtbl.mem nullifiers nullifier_id then
      Error "Nullifier already exists (double-spending detected)"
    else (
      Hashtbl.add nullifiers nullifier_id ();
      Ok nullifiers)

  (** Check if nullifier exists *)
  let contains (nullifiers : t) (nullifier_id : bytes) : bool =
    Hashtbl.mem nullifiers nullifier_id
end

(** {1 Register File} *)

(** Register file implementation *)
module RegisterFile = struct
  type t = (register_id, register_value) Hashtbl.t

  (** Create empty register file *)
  let create () : t = Hashtbl.create 64

  (** Get register value *)
  let get (registers : t) (reg_id : register_id) : register_value option =
    Hashtbl.find_opt registers reg_id

  (** Set register value *)
  let set (registers : t) (reg_id : register_id) (value : register_value) : t =
    Hashtbl.replace registers reg_id value;
    registers

  (** Check if register exists *)
  let exists (registers : t) (reg_id : register_id) : bool =
    Hashtbl.mem registers reg_id

  (** Mark register as consumed *)
  let consume (registers : t) (reg_id : register_id) : (t, string) result =
    match get registers reg_id with
    | None -> Error "Register not found"
    | Some reg_val when reg_val.metadata.consumed ->
        Error "Register already consumed"
    | Some reg_val ->
        let consumed_val = RegisterValue.consume reg_val in
        Ok (set registers reg_id consumed_val)
end

(** {1 Machine State} *)

type machine_state = {
    registers : RegisterFile.t
  ; heap : ResourceHeap.t
  ; nullifiers : NullifierSet.t
  ; pc : int (* Program counter *)
  ; call_stack : int list (* Return addresses *)
  ; program : instruction array
  ; halted : bool
}
(** Complete machine state *)

(** Machine state operations *)
module MachineState = struct
  (** Create initial machine state *)
  let create (program : instruction array) : machine_state =
    {
      registers = RegisterFile.create ()
    ; heap = ResourceHeap.create ()
    ; nullifiers = NullifierSet.create ()
    ; pc = 0
    ; call_stack = []
    ; program
    ; halted = false
    }

  (** Get register value *)
  let get_register (state : machine_state) (reg_id : register_id) :
      register_value option =
    RegisterFile.get state.registers reg_id

  (** Set register value *)
  let set_register (state : machine_state) (reg_id : register_id)
      (value : register_value) : machine_state =
    let new_registers = RegisterFile.set state.registers reg_id value in
    { state with registers = new_registers }

  (** Allocate resource *)
  let alloc_resource (state : machine_state) (value : machine_value) :
      machine_state * bytes =
    let new_heap, resource_id = ResourceHeap.alloc state.heap value in
    ({ state with heap = new_heap }, resource_id)

  (** Consume resource *)
  let consume_resource (state : machine_state) (resource_id : bytes) :
      (machine_state * machine_value, string) result =
    match ResourceHeap.get_value state.heap resource_id with
    | None -> Error "Resource not found or already consumed"
    | Some value -> (
        match ResourceHeap.consume state.heap resource_id with
        | Error msg -> Error msg
        | Ok new_heap -> Ok ({ state with heap = new_heap }, value))

  (** Advance program counter *)
  let advance_pc (state : machine_state) : machine_state =
    { state with pc = state.pc + 1 }

  (** Jump to address *)
  let jump_to (state : machine_state) (address : int) : machine_state =
    { state with pc = address }

  (** Push return address *)
  let push_call (state : machine_state) (return_addr : int) : machine_state =
    { state with call_stack = return_addr :: state.call_stack }

  (** Pop return address *)
  let pop_call (state : machine_state) : (machine_state * int, string) result =
    match state.call_stack with
    | [] -> Error "Call stack underflow"
    | addr :: rest -> Ok ({ state with call_stack = rest }, addr)

  (** Halt the machine *)
  let halt (state : machine_state) : machine_state =
    { state with halted = true }

  (** Check if machine is halted *)
  let is_halted (state : machine_state) : bool = state.halted

  (** Get current instruction *)
  let current_instruction (state : machine_state) : instruction option =
    if state.pc >= 0 && state.pc < Array.length state.program then
      Some state.program.(state.pc)
    else None
end
