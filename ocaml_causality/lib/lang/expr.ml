(* ------------ EXPRESSION TYPES ------------ *)
(* Purpose: Main expression types and operations *)

(* Import types from core module *)
open Ocaml_causality_core

type expr =
  | EAtom of Ast.atom
  | EConst of Value.value_expr
  | EVar of str_t
  | EApply of expr * expr list
  | ECombinator of Ast.atomic_combinator 

(* ------------ EXPRESSION OPERATIONS ------------ *)
(* Purpose: Expression operations and transformations *)

open Ocaml_causality_core
open Value

(* ------------ EXPRESSION AST ------------ *)

(** Expression AST type corresponding to Rust's Expr *)
type expr_ast =
  | Const of lisp_value
  | Alloc of expr_ast
  | Consume of resource_id
  | Lambda of lisp_value list * expr_ast
  | Apply of expr_ast * expr_ast list
  | Let of string * expr_ast * expr_ast
  | If of expr_ast * expr_ast * expr_ast
  | Sequence of expr_ast list

(* ------------ EXPR MODULE ------------ *)

module Expr = struct
  type t = expr_ast

  (* AST constructors *)
  let const value = Const value
  let alloc expr = Alloc expr
  let consume resource_id = Consume resource_id
  let lambda params body = Lambda (params, body)
  let apply func args = Apply (func, args)
  let let_binding name value body = Let (name, value, body)
  let if_then_else cond then_expr else_expr = If (cond, then_expr, else_expr)
  let sequence exprs = Sequence exprs

  (* Convenience constructors *)
  let const_int i = Const (LispValue.int i)
  let const_string s = Const (LispValue.string s)
  let const_bool b = Const (LispValue.bool b)
  let const_unit = Const (LispValue.unit)

  (* Expression analysis *)
  let rec free_variables = function
    | Const _ -> []
    | Alloc expr -> free_variables expr
    | Consume _ -> []
    | Lambda (params, body) ->
        let param_names = List.map LispValue.as_symbol params in
        List.filter (fun v -> not (List.mem v param_names)) (free_variables body)
    | Apply (func, args) ->
        List.concat (free_variables func :: List.map free_variables args)
    | Let (name, value, body) ->
        let value_vars = free_variables value in
        let body_vars = List.filter ((<>) name) (free_variables body) in
        value_vars @ body_vars
    | If (cond, then_expr, else_expr) ->
        List.concat [free_variables cond; free_variables then_expr; free_variables else_expr]
    | Sequence exprs ->
        List.concat (List.map free_variables exprs)

  (* Expression utilities *)
  let rec to_string = function
    | Const value -> LispValue.to_string_debug value
    | Alloc expr -> "(alloc " ^ to_string expr ^ ")"
    | Consume rid -> "(consume #<resource:" ^ Bytes.to_string rid ^ ">)"
    | Lambda (params, body) ->
        let param_strs = List.map LispValue.to_string_debug params in
        "(lambda (" ^ String.concat " " param_strs ^ ") " ^ to_string body ^ ")"
    | Apply (func, args) ->
        let arg_strs = List.map to_string args in
        "(" ^ to_string func ^ " " ^ String.concat " " arg_strs ^ ")"
    | Let (name, value, body) ->
        "(let ((" ^ name ^ " " ^ to_string value ^ ")) " ^ to_string body ^ ")"
    | If (cond, then_expr, else_expr) ->
        "(if " ^ to_string cond ^ " " ^ to_string then_expr ^ " " ^ to_string else_expr ^ ")"
    | Sequence exprs ->
        "(begin " ^ String.concat " " (List.map to_string exprs) ^ ")"

  (* Compilation to expr_id - these would interface with Rust FFI *)
  let compile_and_register_expr (expr: t) : (expr_id, causality_error) result =
    (* Use FFI wrapper to compile expression *)
    let expr_str = to_string expr in
    match Ocaml_causality_interop.Ffi.safe_compile_expr expr_str with
    | Ok (Some expr_id) -> Ok expr_id
    | Ok None -> Error (FFIError "Expression compilation returned null")
    | Error err -> Error err

  (* Predefined expression lookup *)
  let get_predefined_expr_id (name: string) : expr_id option =
    (* Fallback to hardcoded predefined expressions *)
    match name with
    | "issue_ticket_logic" -> Some (Bytes.of_string "issue_ticket_expr_id")
    | "transfer_ticket_logic" -> Some (Bytes.of_string "transfer_ticket_expr_id")
    | _ -> None

  (* Expression evaluation context *)
  type eval_context = {
    bindings: (string * lisp_value) list;
    resources: resource_id list;
  }

  let empty_context = {
    bindings = [];
    resources = [];
  }

  let bind_value name value ctx = 
    { ctx with bindings = (name, value) :: ctx.bindings }

  let lookup_binding name ctx =
    List.assoc_opt name ctx.bindings

  (* Basic expression evaluation (for local testing - real evaluation would be in Rust) *)
  let rec eval_expr ctx = function
    | Const value -> Ok value
    | Alloc _ -> Error (FFIError "Alloc requires FFI call to Rust")
    | Consume _ -> Error (FFIError "Consume requires FFI call to Rust") 
    | Lambda (_params, _body) -> 
        (* Return a closure representation *)
        Ok (LispValue.list [LispValue.symbol "closure"; LispValue.list _params])
    | Apply (func, _args) ->
        (match eval_expr ctx func with
         | Ok _func_val -> Error (FFIError "Apply requires FFI call to Rust")
         | Error e -> Error e)
    | Let (name, value, body) ->
        (match eval_expr ctx value with
         | Ok val_result -> 
             let new_ctx = bind_value name val_result ctx in
             eval_expr new_ctx body
         | Error e -> Error e)
    | If (cond, then_expr, else_expr) ->
        (match eval_expr ctx cond with
         | Ok (Bool true) -> eval_expr ctx then_expr
         | Ok (Bool false) -> eval_expr ctx else_expr
         | Ok _ -> Error (FFIError "Condition must be boolean")
         | Error e -> Error e)
    | Sequence exprs ->
        let rec eval_sequence acc = function
          | [] -> Ok LispValue.unit
          | [last] -> eval_expr ctx last
          | hd :: tl ->
              (match eval_expr ctx hd with
               | Ok _ -> eval_sequence acc tl
               | Error e -> Error e)
        in
        eval_sequence LispValue.unit exprs
end 