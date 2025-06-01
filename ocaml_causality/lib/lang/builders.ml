(*
 * Builders Module
 *
 * This module provides builder functions for constructing expressions
 * in a more convenient way than directly using the AST constructors.
 * These functions form the foundation of the DSL for writing expressions.
 *)

open Ast
open Batteries (* For BatMap *)

(* ------------ ATOMIC VALUE BUILDERS ------------ *)

(** Create a symbol reference *)
let sym name = EAtom (Symbol name)

(** Create a string literal *)
let str_lit s = EAtom (String s)

(** Create an integer literal *)
let int_lit n = EAtom (Integer n)

(** Create a float literal *)
let float_lit f = EAtom (Float f)

(** Create a boolean literal *)
let bool_lit b = EAtom (Boolean b)

(** Create a keyword literal (e.g. :foo) *)
let keyword_lit k_name = EAtom (String (Printf.sprintf ":%s" k_name))

(** Create a nil/null constant expression *)
let nil_lit = EConst (Ocaml_causality_core.Types.VNil)

(* ------------ LIST BUILDERS ------------ *)

(** Create a list expression *)
let list exprs = EApply (ECombinator List, exprs)

(** Create a cons expression *)
let cons head tail = EApply (ECombinator Cons, [head; tail])

(** Create a car (first) expression *)
let car list_expr = EApply (ECombinator Car, [list_expr])

(** Create a cdr (rest) expression *)
let cdr list_expr = EApply (ECombinator Cdr, [list_expr])

(** Create an nth expression *)
let nth index_expr list_expr = EApply (ECombinator Nth, [index_expr; list_expr])

(** Create a length expression *)
let length list_expr = EApply (ECombinator Length, [list_expr])

(* ------------ CONTROL FLOW BUILDERS ------------ *)

(** Create an if expression *)
let if_ cond then_expr else_expr = 
  EApply (ECombinator If, [cond; then_expr; else_expr])

(** Create a let expression *)
let let_ bindings body =
  EApply (ECombinator Let, [
    list (List.map (fun (name, value) -> 
      list [sym name; value]
    ) bindings);
    list body
  ])

(** Create a let* expression *)
let let_star bindings body =
  EApply (ECombinator LetStar, [
    list (List.map (fun (name, value) -> 
      list [sym name; value]
    ) bindings);
    list body
  ])

(* ------------ LOGICAL OPERATORS ------------ *)

(** Create an and expression *)
let and_ exprs = EApply (ECombinator And, exprs)

(** Create an or expression *)
let or_ exprs = EApply (ECombinator Or, exprs)

(** Create a not expression *)
let not_ expr = EApply (ECombinator Not, [expr])

(* ------------ COMPARISON OPERATORS ------------ *)

(** Create an equality comparison *)
let eq a b = EApply (ECombinator Eq, [a; b])

(** Create a greater than comparison *)
let gt a b = EApply (ECombinator Gt, [a; b])

(** Create a less than comparison *)
let lt a b = EApply (ECombinator Lt, [a; b])

(** Create a greater than or equal comparison *)
let gte a b = EApply (ECombinator Gte, [a; b])

(** Create a less than or equal comparison *)
let lte a b = EApply (ECombinator Lte, [a; b])

(* ------------ ARITHMETIC OPERATORS ------------ *)

(** Create an addition expression *)
let add a b = EApply (ECombinator Add, [a; b])

(** Create a subtraction expression *)
let sub a b = EApply (ECombinator Sub, [a; b])

(** Create a multiplication expression *)
let mul a b = EApply (ECombinator Mul, [a; b])

(** Create a division expression *)
let div a b = EApply (ECombinator Div, [a; b])

(* ------------ MAP OPERATIONS ------------ *)

(** Create a map expression *)
let make_map entries =
  EApply (ECombinator MakeMap, [
    list (List.map (fun (key, value) -> 
      list [str_lit key; value]
    ) entries)
  ])

(** Create a map get expression *)
let map_get map_expr key = 
  EApply (ECombinator MapGet, [map_expr; str_lit key])

(** Create a map has key expression *)
let map_has_key map_expr key = 
  EApply (ECombinator MapHasKey, [map_expr; str_lit key])

(* ------------ FUNCTION DEFINITIONS ------------ *)

(** Create a lambda expression *)
let lambda params body = ELambda (params, body)

(** Create a function definition *)
let defun name params body =
  EApply (ECombinator Define, [
    sym name;
    ELambda (params, body)
  ])

(** Create a function application *)
let apply func args = EApply (func, args)

(** Create a define expression (for global definitions) *)
let define name value_expr = EApply (ECombinator Define, [sym name; value_expr])

(** Create a quote expression *)
let quote data_expr = EApply (ECombinator Quote, [data_expr])

(** Create a dynamic expression for future evaluation *)
let dynamic steps expr_val = EDynamic (steps, expr_val)

(* ------------ FIELD ACCESS ------------ *)

(** Create a field access expression *)
let get_field obj field = 
  EApply (ECombinator GetField, [obj; field])

(** Create a context value access expression *)
let get_context_value key = 
  EApply (ECombinator GetContextValue, [str_lit key])

(** Create a completed expression (checks effect completion) *)
let completed effect_ref_expr = EApply (ECombinator Completed, [effect_ref_expr])

(* ------------ CORE VALUE BUILDERS (for EConst) ------------ *)

(** Create a Core Types.VNil value *)
let vnil = Ocaml_causality_core.Types.VNil

(** Create a Core Types.VBool value *)
let vbool b = Ocaml_causality_core.Types.VBool b

(** Create a Core Types.VString value *)
let vstr s = Ocaml_causality_core.Types.VString s

(** Create a Core Types.VInt value *)
let vint i = Ocaml_causality_core.Types.VInt i

(** Create a Core Types.VList value *)
let vlist (items: Ocaml_causality_core.Types.value_expr list) = Ocaml_causality_core.Types.VList items

(** Create a Core Types.VMap value from a list of (string * Core Types.value_expr) *)
let vmap entries =
  (* Temporary stub - VMap expects BatMap but we don't have Batteries *)
  (* This would need proper BatMap implementation *)
  let _ = entries in (* suppress unused warning *)
  failwith "VMap creation requires Batteries.BatMap - not implemented"

(** Create a Core Types.VStruct value from a list of (string * Core Types.value_expr) *)
let vstruct entries = 
  (* Temporary stub - VStruct expects BatMap but we don't have Batteries *)
  let _ = entries in (* suppress unused warning *)
  failwith "VStruct creation requires Batteries.BatMap - not implemented"

(** Create a Core Types.VRef (value reference) *)
let vref_value entity_id = 
  Ocaml_causality_core.Types.VRef (Ocaml_causality_core.Types.VERValue entity_id)

(** Create a Core Types.VRef (expression reference) *)
let vref_expr entity_id = 
  Ocaml_causality_core.Types.VRef (Ocaml_causality_core.Types.VERExpr entity_id) 