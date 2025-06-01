(*
 * Rust Bridge Module Interface
 *
 * This module provides FFI bindings and interoperability functions
 * for communication with Rust components.
 *)

(** Rust FFI types *)

open Ocaml_causality_lang
open Ocaml_causality_effects

(** FFI error type *)
type ffi_error =
  | EncodingError of string    (** Error encoding data for FFI *)
  | DecodingError of string    (** Error decoding data from FFI *)
  | RustError of string        (** Error from Rust code *)
  | UnsupportedType of string  (** Type not supported for FFI *)

(** Result type for FFI operations *)
type 'a result = ('a, ffi_error) Result.t

(** Initialize the bridge *)
val initialize : unit -> unit

(** Finalize the bridge *)
val finalize : unit -> unit

(** Evaluate an expression using the Rust runtime *)
val evaluate_in_rust : Ast.expr -> Ast.value_expr result

(** Execute an effect in the Rust runtime *)
val execute_effect_in_rust : Effects.effect_instance -> Ast.value_expr result

(** Call a Rust function from OCaml *)
val call_rust_function : string -> Ast.value_expr list -> Ast.value_expr result 