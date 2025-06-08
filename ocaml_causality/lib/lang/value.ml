(* ------------ VALUE EXPRESSIONS ------------ *)
(* Purpose: Value expression types and LispValue operations *)

(* Import types from core module *)
open Ocaml_causality_core

type value_expr_ref_target = VERValue of value_expr_id | VERExpr of expr_id

type value_expr =
  | VNil
  | VBool of bool
  | VString of str_t
  | VInt of int64
  | VList of value_expr list
  | VRef of value_expr_ref_target

(* ------------ LISP VALUE MODULE ------------ *)

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
    | ResourceId rid -> "#<resource:" ^ Bytes.to_string rid ^ ">"
    | ExprId eid -> "#<expr:" ^ Bytes.to_string eid ^ ">"
    | Bytes b -> "#<bytes:" ^ Bytes.to_string b ^ ">"

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

  (* Extractors (unsafe - will raise exceptions for wrong types) *)
  let as_bool = function Bool b -> b | _ -> failwith "Not a boolean"
  let as_int = function Int i -> i | _ -> failwith "Not an integer"
  let as_string = function String s -> s | _ -> failwith "Not a string"
  let as_symbol = function Symbol s -> s | _ -> failwith "Not a symbol"
  let as_list = function List l -> l | _ -> failwith "Not a list"

  let as_resource_id = function
    | ResourceId rid -> rid
    | _ -> failwith "Not a resource ID"

  let as_expr_id = function
    | ExprId eid -> eid
    | _ -> failwith "Not an expression ID"

  let as_bytes = function Bytes b -> b | _ -> failwith "Not bytes"

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
  let length = function
    | List l -> Int64.of_int (List.length l)
    | _ -> failwith "Not a list"

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

  (* Equality *)
  let rec equal a b =
    match (a, b) with
    | Unit, Unit -> true
    | Bool a, Bool b -> a = b
    | Int a, Int b -> a = b
    | String a, String b -> a = b
    | Symbol a, Symbol b -> a = b
    | List a, List b -> (
        try List.for_all2 equal a b with Invalid_argument _ -> false)
    | ResourceId a, ResourceId b -> Bytes.equal a b
    | ExprId a, ExprId b -> Bytes.equal a b
    | Bytes a, Bytes b -> Bytes.equal a b
    | _, _ -> false
end
