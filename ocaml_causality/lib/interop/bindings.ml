(* ------------ CAUSALITY BINDINGS ------------ *)
(* Purpose: High-level OCaml API for Causality framework integration *)

open Ocaml_causality_core

(* ========================================= *)
(* LISP VALUE API *)
(* ========================================= *)

module LispValue = struct
  type t = lisp_value

  (* Constructors *)
  let unit = Unit
  let bool b = Bool b
  let int i = Int i
  let string s = String s
  let symbol s = Symbol s
  let list l = List l
  let resource_id rid = ResourceId rid
  let expr_id eid = ExprId eid
  let bytes b = Bytes b

  (* Utility functions *)
  let rec to_string_debug = function
    | Unit -> "()"
    | Bool true -> "true"
    | Bool false -> "false"
    | Int i -> Int64.to_string i
    | String s -> "\"" ^ s ^ "\""
    | Symbol s -> s
    | List [] -> "()"
    | List l -> "(" ^ String.concat " " (List.map to_string_debug l) ^ ")"
    | ResourceId rid -> "#<resource:" ^ (Bytes.to_string rid) ^ ">"
    | ExprId eid -> "#<expr:" ^ (Bytes.to_string eid) ^ ">"
    | Bytes b -> "#<bytes:" ^ (Bytes.to_string b) ^ ">"

  (* Type predicates *)
  let is_unit = function Unit -> true | _ -> false
  let is_bool = function Bool _ -> true | _ -> false
  let is_int = function Int _ -> true | _ -> false
  let is_string = function String _ -> true | _ -> false
  let is_symbol = function Symbol _ -> true | _ -> false
  let is_list = function List _ -> true | _ -> false
  let is_resource_id = function ResourceId _ -> true | _ -> false
  let is_expr_id = function ExprId _ -> true | _ -> false
  let is_bytes = function Bytes _ -> true | _ -> false

  (* Safe extractors *)
  let try_as_bool = function Bool b -> Some b | _ -> None
  let try_as_int = function Int i -> Some i | _ -> None
  let try_as_string = function String s -> Some s | _ -> None
  let try_as_symbol = function Symbol s -> Some s | _ -> None
  let try_as_list = function List l -> Some l | _ -> None
  let try_as_resource_id = function ResourceId rid -> Some rid | _ -> None
  let try_as_expr_id = function ExprId eid -> Some eid | _ -> None
  let try_as_bytes = function Bytes b -> Some b | _ -> None

  (* List operations *)
  let cons head tail =
    match tail with
    | List l -> List (head :: l)
    | _ -> failwith "tail must be a list"

  let car = function
    | List (h :: _) -> h
    | List [] -> Unit
    | _ -> failwith "Not a list"

  let cdr = function
    | List (_ :: t) -> List t
    | List [] -> List []
    | _ -> failwith "Not a list"
end

(* ========================================= *)
(* EXPRESSION API *)
(* ========================================= *)

module Expr = struct
  type expr_ast =
    | Const of lisp_value
    | Alloc of expr_ast
    | Consume of resource_id
    | Lambda of lisp_value list * expr_ast
    | Apply of expr_ast * expr_ast list
    | Let of string * expr_ast * expr_ast
    | If of expr_ast * expr_ast * expr_ast
    | Sequence of expr_ast list

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

  (* Compilation to expr_id - would interface with Rust FFI *)
  let compile_and_register_expr (expr: t) : (expr_id, causality_error) result =
    let expr_str = to_string expr in
    let expr_bytes = Bytes.of_string expr_str in
    Ok expr_bytes

  (* Predefined expression lookup *)
  let get_predefined_expr_id (name: string) : expr_id option =
    match name with
    | "issue_ticket_logic" -> Some (Bytes.of_string "issue_ticket_expr_id")
    | "transfer_ticket_logic" -> Some (Bytes.of_string "transfer_ticket_expr_id")
    | _ -> None
end

(* ========================================= *)
(* INTENT API *)
(* ========================================= *)

module Intent = struct
  type t = {
    mutable name: str_t;
    mutable domain_id: str_t;
    mutable input_resources: resource_id list;
    mutable parameters: lisp_value list;
    mutable lisp_logic: expr_id option;
    mutable priority: int;
    mutable outputs: resource_flow list;
  }

  let create ~name ~domain_id = {
    name;
    domain_id;
    input_resources = [];
    parameters = [];
    lisp_logic = None;
    priority = 0;
    outputs = [];
  }

  let add_input_resource intent resource_id =
    intent.input_resources <- resource_id :: intent.input_resources

  let add_parameter intent param =
    intent.parameters <- param :: intent.parameters

  let set_lisp_logic intent expr_id =
    intent.lisp_logic <- Some expr_id

  let set_priority intent priority =
    intent.priority <- priority

  let add_output intent output =
    intent.outputs <- output :: intent.outputs

  let submit intent : (unit, causality_error) result =
    try
      Printf.printf "Submitting intent: %s to domain %s\n" 
        intent.name intent.domain_id;
      Ok ()
    with
    | exn -> Error (FFIError ("Failed to submit intent: " ^ Printexc.to_string exn))
end

(* ========================================= *)
(* SYSTEM API *)
(* ========================================= *)

module System = struct
  let get_last_produced_resource_id () : resource_id =
    Bytes.of_string "last_resource_id"

  let get_resource_by_id (_id: resource_id) : (resource option, causality_error) result =
    Ok None

  let get_domain_info (_domain_id: domain_id) : (typed_domain option, causality_error) result =
    Ok None

  let get_system_metrics () : (str_t, causality_error) result =
    Ok "System metrics placeholder"
end 