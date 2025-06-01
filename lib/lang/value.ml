(* ------------ VALUE EXPRESSIONS ------------ *)
(* Purpose: Value expression types and operations *)

(* Import types from core module *)
open Ocaml_causality_core

type value_expr_ref_target =
  | VERValue of value_expr_id
  | VERExpr of expr_id

type value_expr =
  | VNil
  | VBool of bool
  | VString of str_t
  | VInt of int64
  | VList of value_expr list
  | VRef of value_expr_ref_target 