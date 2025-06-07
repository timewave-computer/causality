(* ------------ DSL BUILDER FUNCTIONS ------------ *)
(* Purpose: DSL builder functions for constructing expressions *)

open Ocaml_causality_core
open Expr
open Value

(* ------------ BASIC CONSTRUCTORS ------------ *)

(* Basic expression constructors *)
let let_ name value body = Expr.let_binding name value body

let if_ condition then_expr else_expr = Expr.if_then_else condition then_expr else_expr

let apply_ func args = Expr.apply func args

(* ------------ CONTROL FLOW BUILDERS ------------ *)

(* Control flow DSL functions *)
let when_ condition action = 
  Expr.if_then_else condition action Expr.const_unit

let unless_ condition action =
  Expr.if_then_else condition Expr.const_unit action

let cond cases default =
  List.fold_right (fun (condition, action) acc ->
    Expr.if_then_else condition action acc
  ) cases default

(* ------------ RESOURCE BUILDERS ------------ *)

(* Resource construction DSL functions *)
let allocate_resource value = Expr.alloc value

let consume_resource resource_id = Expr.consume resource_id

let with_resource resource_expr body_fn =
  Expr.let_binding "resource" (Expr.alloc resource_expr) 
    (body_fn (Expr.const (LispValue.symbol "resource")))

(* ------------ EXPRESSION BUILDERS ------------ *)

(* Expression construction functions *)
let make_lambda params body = Expr.lambda params body

let make_application func args = Expr.apply func args

let make_sequence exprs = Expr.sequence exprs

let make_let_binding name value body = Expr.let_binding name value body

(* Convenience builders *)
let int_expr i = Expr.const_int i
let string_expr s = Expr.const_string s
let bool_expr b = Expr.const_bool b
let unit_expr = Expr.const_unit
let symbol_expr s = Expr.const (LispValue.symbol s)

(* ------------ VALIDATION HELPERS ------------ *)

(* Builder validation functions *)
let validate_lambda_params params =
  let rec check_duplicates = function
    | [] -> true
    | Symbol s :: rest ->
        not (List.exists (function Symbol s' -> s = s' | _ -> false) rest) &&
        check_duplicates rest
    | _ :: rest -> check_duplicates rest
  in
  check_duplicates params

let validate_expression expr =
  try
    let _ = Expr.to_string expr in
    true
  with
  | _ -> false

let validate_resource_usage expr =
  (* Simple validation - check that resources are properly consumed *)
  let free_vars = Expr.free_variables expr in
  not (List.exists (fun var -> String.contains var '#') free_vars)

(* Builder combinators *)
let (>>=) expr f = 
  Expr.let_binding "temp" expr (f (Expr.const (LispValue.symbol "temp")))

let (>>|) expr f =
  Expr.apply f [expr]

let pipe exprs =
  match exprs with
  | [] -> Expr.const_unit
  | first :: rest ->
      List.fold_left (fun acc expr ->
        Expr.apply expr [acc]
      ) first rest 