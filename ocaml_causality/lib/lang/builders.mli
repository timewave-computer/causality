(*
 * Builders Module Interface
 *
 * This module provides builder functions for constructing expressions
 * in a more convenient way than directly using the AST constructors.
 * These functions form the foundation of the DSL for writing expressions.
 *)

open Ast

(* ------------ ATOMIC VALUE BUILDERS ------------ *)

(** Create a symbol reference *)
val sym : string -> expr

(** Create a string literal *)
val str_lit : string -> expr

(** Create an integer literal *)
val int_lit : int64 -> expr

(** Create a float literal *)
val float_lit : float -> expr

(** Create a boolean literal *)
val bool_lit : bool -> expr

(** Create a keyword literal (e.g. :foo) *)
val keyword_lit : string -> expr

(** Create a nil/null constant expression *)
val nil_lit : expr (* Represents EConst Types.VNil *)

(* ------------ LIST BUILDERS ------------ *)

(** Create a list expression *)
val list : expr list -> expr

(** Create a cons expression *)
val cons : expr -> expr -> expr

(** Create a car (first) expression *)
val car : expr -> expr

(** Create a cdr (rest) expression *)
val cdr : expr -> expr

(** Create an nth expression *)
val nth : expr -> expr -> expr

(** Create a length expression *)
val length : expr -> expr

(* ------------ CONTROL FLOW BUILDERS ------------ *)

(** Create an if expression *)
val if_ : expr -> expr -> expr -> expr

(** Create a let expression *)
val let_ : (string * expr) list -> expr list -> expr

(** Create a let* expression *)
val let_star : (string * expr) list -> expr list -> expr

(* ------------ LOGICAL OPERATORS ------------ *)

(** Create an and expression *)
val and_ : expr list -> expr

(** Create an or expression *)
val or_ : expr list -> expr

(** Create a not expression *)
val not_ : expr -> expr

(* ------------ COMPARISON OPERATORS ------------ *)

(** Create an equality comparison *)
val eq : expr -> expr -> expr

(** Create a greater than comparison *)
val gt : expr -> expr -> expr

(** Create a less than comparison *)
val lt : expr -> expr -> expr

(** Create a greater than or equal comparison *)
val gte : expr -> expr -> expr

(** Create a less than or equal comparison *)
val lte : expr -> expr -> expr

(* ------------ ARITHMETIC OPERATORS ------------ *)

(** Create an addition expression *)
val add : expr -> expr -> expr

(** Create a subtraction expression *)
val sub : expr -> expr -> expr

(** Create a multiplication expression *)
val mul : expr -> expr -> expr

(** Create a division expression *)
val div : expr -> expr -> expr

(* ------------ MAP OPERATIONS ------------ *)

(** Create a map expression *)
val make_map : (string * expr) list -> expr

(** Create a map get expression *)
val map_get : expr -> string -> expr

(** Create a map has key expression *)
val map_has_key : expr -> string -> expr

(* ------------ FUNCTION DEFINITIONS ------------ *)

(** Create a lambda expression *)
val lambda : string list -> expr -> expr

(** Create a function definition *)
val defun : string -> string list -> expr -> expr

(** Create a function application *)
val apply : expr -> expr list -> expr

(** Create a define expression (for global definitions) *)
val define : string -> expr -> expr

(** Create a quote expression *)
val quote : expr -> expr

(** Create a dynamic expression for future evaluation *)
val dynamic : int -> expr -> expr

(* ------------ FIELD ACCESS ------------ *)

(** Create a field access expression *)
val get_field : expr -> expr -> expr

(** Create a context value access expression *)
val get_context_value : string -> expr

(** Create a completed expression (checks effect completion) *)
val completed : expr -> expr

(* ------------ CORE VALUE BUILDERS (for EConst) ------------ *)

(** Create a Core Types.VNil value *)
val vnil : Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VBool value *)
val vbool : bool -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VString value *)
val vstr : string -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VInt value *)
val vint : int64 -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VList value *)
val vlist : Ocaml_causality_core.Types.value_expr list -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VMap value from a list of (string * Core Types.value_expr) *)
val vmap : (string * Ocaml_causality_core.Types.value_expr) list -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VStruct value from a list of (string * Core Types.value_expr) *)
val vstruct : (string * Ocaml_causality_core.Types.value_expr) list -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VRef (value reference) *)
val vref_value : Ocaml_causality_core.entity_id -> Ocaml_causality_core.Types.value_expr

(** Create a Core Types.VRef (expression reference) *)
val vref_expr : Ocaml_causality_core.entity_id -> Ocaml_causality_core.Types.value_expr 