(* Purpose: S-expression serialization interface for OCaml types *)

open Types

(* Helper functions *)
val sexp_of_bytes : bytes -> Sexplib0.Sexp.t
val bytes_from_sexp : Sexplib0.Sexp.t -> bytes

(* Core expression serialization *)
val atom_to_sexp : atom -> Sexplib0.Sexp.t
val value_expr_to_sexp : value_expr -> Sexplib0.Sexp.t
val expr_to_sexp : expr -> Sexplib0.Sexp.t
val atomic_combinator_to_sexp : atomic_combinator -> Sexplib0.Sexp.t

(* Core expression deserialization *)
val atom_from_sexp : Sexplib0.Sexp.t -> atom
val value_expr_from_sexp : Sexplib0.Sexp.t -> value_expr
val expr_from_sexp : Sexplib0.Sexp.t -> expr
val atomic_combinator_from_sexp : Sexplib0.Sexp.t -> atomic_combinator

(* Core type serialization *)
val resource_flow_to_sexp : resource_flow -> Sexplib0.Sexp.t
val resource_flow_from_sexp : Sexplib0.Sexp.t -> resource_flow

val resource_to_sexp : resource -> Sexplib0.Sexp.t
val resource_from_sexp : Sexplib0.Sexp.t -> resource

val intent_to_sexp : intent -> Sexplib0.Sexp.t
val intent_from_sexp : Sexplib0.Sexp.t -> intent

val effect_to_sexp : effect -> Sexplib0.Sexp.t
val effect_from_sexp : Sexplib0.Sexp.t -> effect

val handler_to_sexp : handler -> Sexplib0.Sexp.t
val handler_from_sexp : Sexplib0.Sexp.t -> handler

val transaction_to_sexp : transaction -> Sexplib0.Sexp.t
val transaction_from_sexp : Sexplib0.Sexp.t -> transaction

(* String conversion functions - serialization only *)
val value_expr_to_string : value_expr -> string
val expr_to_string : expr -> string
val resource_flow_to_string : resource_flow -> string
val resource_to_string : resource -> string
val intent_to_string : intent -> string
val effect_to_string : effect -> string
val handler_to_string : handler -> string
val transaction_to_string : transaction -> string 