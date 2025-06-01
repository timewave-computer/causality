(*
 * Rust Bridge Module
 *
 * This module provides Foreign Function Interface (FFI) integration with the
 * Rust implementation of Causality. It enables calling Rust functions from OCaml
 * and passing data between the two languages.
 *)

open Ocaml_causality_core
open Ocaml_causality_lang
open Ocaml_causality_serialization
open Ctypes
open Foreign

(* ------------ BRIDGING TYPES ------------ *)

(** Rust string representation *)
type rust_string

(** Rust result representation *)
type 'a rust_result

(** Rust entity ID representation *)
type rust_entity_id

(** Rust expression representation *)
type rust_expr

(** Rust value expression representation *)
type rust_value_expr

(** FFI error type *)
type ffi_error =
  | EncodingError of string    (* Error encoding data for FFI *)
  | DecodingError of string    (* Error decoding data from FFI *)
  | RustError of string        (* Error from Rust code *)
  | UnsupportedType of string  (* Type not supported for FFI *)

(** Result type for FFI operations *)
type 'a result = ('a, ffi_error) Result.t

(* ------------ FFI BINDINGS ------------ *)

(* Note: These are placeholder bindings. The actual implementations would
   use the Ctypes library to define the FFI interface. *)

(** Initialize the Rust runtime *)
let init_rust_runtime = 
  foreign "causality_init_runtime" (void @-> returning int)

(** Shutdown the Rust runtime *)
let shutdown_rust_runtime = 
  foreign "causality_shutdown_runtime" (void @-> returning void)

(** Create a string in Rust *)
let create_rust_string = 
  foreign "causality_create_string" (string @-> returning (ptr rust_string))

(** Free a Rust string *)
let free_rust_string = 
  foreign "causality_free_string" (ptr rust_string @-> returning void)

(** Get string content from Rust *)
let rust_string_content = 
  foreign "causality_string_content" (ptr rust_string @-> returning string)

(** Evaluate an expression in Rust *)
let evaluate_expr_in_rust = 
  foreign "causality_evaluate_expr" 
    (ptr rust_expr @-> returning (ptr rust_value_expr))

(** Convert a Rust expression to OCaml *)
let rust_expr_to_ocaml = 
  foreign "causality_expr_to_ocaml" 
    (ptr rust_expr @-> returning (ptr void))

(** Convert an OCaml expression to Rust *)
let ocaml_expr_to_rust = 
  foreign "causality_ocaml_to_expr" 
    (ptr void @-> returning (ptr rust_expr))

(* ------------ MEMORY MANAGEMENT ------------ *)

(** Container for Rust allocated memory that needs to be freed *)
type rust_allocated = {
  ptr: unit ptr;
  free_fn: unit ptr -> unit;
}

(** List of currently allocated Rust memory *)
let allocated_memory : rust_allocated list ref = ref []

(** Register memory for later cleanup *)
let register_allocation ptr free_fn =
  allocated_memory := { ptr; free_fn } :: !allocated_memory;
  ptr

(** Free a specific allocation *)
let free_allocation ptr =
  let matching, remaining = List.partition 
    (fun alloc -> ptr = alloc.ptr) 
    !allocated_memory in
  List.iter (fun alloc -> alloc.free_fn ptr) matching;
  allocated_memory := remaining

(** Free all allocated Rust memory *)
let free_all_allocations () =
  List.iter (fun alloc -> alloc.free_fn alloc.ptr) !allocated_memory;
  allocated_memory := []

(* ------------ INITIALIZATION ------------ *)

(** Initialize the bridge *)
let initialize () =
  let init_result = init_rust_runtime () in
  if init_result <> 0 then
    failwith (Printf.sprintf "Failed to initialize Rust runtime: error code %d" init_result)

(** Finalize the bridge *)
let finalize () =
  free_all_allocations ();
  shutdown_rust_runtime ()

(* Register finalization with at_exit *)
let () = Stdlib.at_exit finalize

(* ------------ TYPE CONVERSION ------------ *)

(** Convert an entity ID to Rust *)
let entity_id_to_rust (id: entity_id) : rust_entity_id ptr result =
  try
    let rust_str = create_rust_string id in
    let ptr = register_allocation (to_voidp rust_str) (fun p -> 
      free_rust_string (from_voidp rust_string p)
    ) in
    Ok (from_voidp rust_entity_id ptr)
  with 
  | exn -> Error (EncodingError (Printexc.to_string exn))

(** Convert a Rust entity ID to OCaml *)
let entity_id_from_rust (ptr: rust_entity_id ptr) : entity_id result =
  try
    let str_ptr = to_voidp ptr |> from_voidp rust_string in
    let str = rust_string_content str_ptr in
    Ok str
  with
  | exn -> Error (DecodingError (Printexc.to_string exn))

(** Convert an AST expression to Rust *)
let expr_to_rust (expr: Ast.expr) : rust_expr ptr result =
  try
    (* For MVP: Serialize to bytes using SSZ, then pass to Rust *)
    let serialized = Ssz.encode expr in
    let binary_ptr = to_voidp (ocaml_caml_bytes_start serialized) in
    let rust_expr_ptr = ocaml_expr_to_rust binary_ptr in
    let ptr = register_allocation (to_voidp rust_expr_ptr) (fun p ->
      (* Would call a Rust function to free this memory *)
      ()
    ) in
    Ok (from_voidp rust_expr ptr)
  with
  | exn -> Error (EncodingError (Printexc.to_string exn))

(** Convert a Rust expression to OCaml *)
let expr_from_rust (ptr: rust_expr ptr) : Ast.expr result =
  try
    let ocaml_ptr = rust_expr_to_ocaml ptr in
    (* For MVP: this would extract serialized bytes and deserialize *)
    match Ssz.decode (Bytes.empty) with  (* Placeholder *)
    | Ok expr -> Ok expr
    | Error e -> Error (DecodingError "Failed to decode expression from Rust")
  with
  | exn -> Error (DecodingError (Printexc.to_string exn))

(* ------------ PUBLIC API ------------ *)

(** Evaluate an expression using the Rust runtime *)
let evaluate_in_rust (expr: Ast.expr) : Ast.value_expr result =
  match expr_to_rust expr with
  | Ok rust_expr ->
      (try
        let result_ptr = evaluate_expr_in_rust rust_expr in
        (* Convert result back to OCaml *)
        (* For MVP, this is a placeholder *)
        Ok (Ast.VUnit)
      with
      | exn -> Error (RustError (Printexc.to_string exn)))
  | Error e -> Error e

(** Execute an effect in the Rust runtime *)
let execute_effect_in_rust (effect: Ocaml_causality_effects.Effects.effect_instance)
    : Ast.value_expr result =
  (* For MVP, this is a placeholder *)
  Error (UnsupportedType "Effect execution in Rust not implemented yet")

(** Call a Rust function from OCaml *)
let call_rust_function (name: string) (args: Ast.value_expr list) : Ast.value_expr result =
  (* For MVP, this is a placeholder *)
  Error (UnsupportedType (Printf.sprintf "Rust function call not implemented: %s" name))
