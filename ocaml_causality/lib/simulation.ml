(* OCaml FFI for Causality Simulation *)

open Ctypes
open Foreign

(* Type definitions for opaque pointers *)
let simulation_state : unit ptr typ = ptr void

(* FFI function bindings *)
let causality_load_bytecode =
  foreign "causality_load_bytecode" (ptr char @-> size_t @-> returning simulation_state)

let causality_free_simulation_state =
  foreign "causality_free_simulation_state" (simulation_state @-> returning void)

let causality_run_simulation_step =
  foreign "causality_run_simulation_step" (simulation_state @-> returning void) (* Simplified for now *)

let causality_get_simulation_result =
  foreign "causality_get_simulation_result" (simulation_state @-> returning (ptr void)) (* Simplified *)

(* OCaml wrapper functions *)
let load_simulation bytecode =
  let bytecode_ptr = Ctypes.string_to_char_ptr bytecode in
  let len = Unsigned.Size_t.of_int (String.length bytecode) in
  causality_load_bytecode bytecode_ptr len

let free_simulation = causality_free_simulation_state

let run_step = causality_run_simulation_step

let get_result = causality_get_simulation_result 