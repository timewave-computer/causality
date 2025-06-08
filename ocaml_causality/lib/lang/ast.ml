(* ------------ ABSTRACT SYNTAX TREE ------------ *)
(* Purpose: Core AST types for expressions and values *)

(* ------------ EXPRESSION AST TYPES ------------ *)
(* Purpose: Abstract syntax tree types for expressions *)

(* Import types from core module *)
open Ocaml_causality_core

type atomic_combinator =
  | List
  | MakeMap
  | GetField
  | Length
  | Eq
  | Lt
  | Gt
  | Add
  | Sub
  | Mul
  | Div
  | And
  | Or
  | Not
  | If
  | Let
  | Define
  | Quote
  | Cons
  | Car
  | Cdr

type atom = AInt of int64 | AString of str_t | ABoolean of bool | ANil

(* ------------ VALUE TYPES ------------ *)

(* Value expression types *)
type value_expr =
  | VConst of lisp_value
  | VAtom of atom
  | VCombinator of atomic_combinator
  | VList of value_expr list
  | VSymbol of str_t
  | VQuote of value_expr

(* ------------ ATOMIC TYPES ------------ *)

(* Extended atomic combinator operations *)
type combinator_arity = int

let combinator_arity = function
  | List -> 0 (* Variable arity *)
  | MakeMap -> 0 (* Variable arity *)
  | GetField -> 2
  | Length -> 1
  | Eq | Lt | Gt -> 2
  | Add | Sub | Mul | Div -> 2
  | And | Or -> 2
  | Not -> 1
  | If -> 3
  | Let -> 3
  | Define -> 2
  | Quote -> 1
  | Cons -> 2
  | Car | Cdr -> 1

let combinator_to_string = function
  | List -> "list"
  | MakeMap -> "make-map"
  | GetField -> "get-field"
  | Length -> "length"
  | Eq -> "eq"
  | Lt -> "lt"
  | Gt -> "gt"
  | Add -> "add"
  | Sub -> "sub"
  | Mul -> "mul"
  | Div -> "div"
  | And -> "and"
  | Or -> "or"
  | Not -> "not"
  | If -> "if"
  | Let -> "let"
  | Define -> "define"
  | Quote -> "quote"
  | Cons -> "cons"
  | Car -> "car"
  | Cdr -> "cdr"

(* ------------ AST UTILITIES ------------ *)

(* AST traversal and manipulation functions *)
let rec map_value_expr f = function
  | VConst v -> f (VConst v)
  | VAtom a -> f (VAtom a)
  | VCombinator c -> f (VCombinator c)
  | VList exprs -> f (VList (List.map (map_value_expr f) exprs))
  | VSymbol s -> f (VSymbol s)
  | VQuote expr -> f (VQuote (map_value_expr f expr))

let rec fold_value_expr f acc = function
  | VConst v -> f acc (VConst v)
  | VAtom a -> f acc (VAtom a)
  | VCombinator c -> f acc (VCombinator c)
  | VList exprs ->
      let acc' = f acc (VList []) in
      List.fold_left (fold_value_expr f) acc' exprs
  | VSymbol s -> f acc (VSymbol s)
  | VQuote expr ->
      let acc' = f acc (VQuote (VConst Unit)) in
      fold_value_expr f acc' expr

let rec collect_symbols = function
  | VConst _ | VAtom _ | VCombinator _ -> []
  | VSymbol s -> [ s ]
  | VList exprs -> List.concat_map collect_symbols exprs
  | VQuote expr -> collect_symbols expr

let rec collect_constants = function
  | VConst v -> [ v ]
  | VAtom (AInt i) -> [ Int i ]
  | VAtom (AString s) -> [ String s ]
  | VAtom (ABoolean b) -> [ Bool b ]
  | VAtom ANil -> [ Unit ]
  | VCombinator _ | VSymbol _ -> []
  | VList exprs -> List.concat_map collect_constants exprs
  | VQuote expr -> collect_constants expr

let rec substitute_symbol target replacement = function
  | VConst v -> VConst v
  | VAtom a -> VAtom a
  | VCombinator c -> VCombinator c
  | VSymbol s when s = target -> replacement
  | VSymbol s -> VSymbol s
  | VList exprs -> VList (List.map (substitute_symbol target replacement) exprs)
  | VQuote expr -> VQuote (substitute_symbol target replacement expr)

let rec count_nodes = function
  | VConst _ | VAtom _ | VCombinator _ | VSymbol _ -> 1
  | VList exprs -> 1 + List.fold_left ( + ) 0 (List.map count_nodes exprs)
  | VQuote expr -> 1 + count_nodes expr

let rec depth = function
  | VConst _ | VAtom _ | VCombinator _ | VSymbol _ -> 1
  | VList exprs -> (
      match exprs with
      | [] -> 1
      | _ -> 1 + List.fold_left max 0 (List.map depth exprs))
  | VQuote expr -> 1 + depth expr

(* AST validation *)
let rec is_well_formed = function
  | VConst _ | VAtom _ | VCombinator _ | VSymbol _ -> true
  | VList exprs -> List.for_all is_well_formed exprs
  | VQuote expr -> is_well_formed expr

let validate_combinator_usage = function
  | VList (VCombinator c :: args) ->
      let expected_arity = combinator_arity c in
      expected_arity = 0 || List.length args = expected_arity
  | _ -> true

(* ------------ TYPE CHECKING ------------ *)

(* Basic type checking functions *)
type ast_type =
  | TUnit
  | TBool
  | TInt
  | TString
  | TSymbol
  | TList of ast_type
  | TCombinator
  | TUnknown

let rec infer_value_expr_type = function
  | VConst Unit -> TUnit
  | VConst (Bool _) -> TBool
  | VConst (Int _) -> TInt
  | VConst (String _) -> TString
  | VConst (Symbol _) -> TSymbol
  | VConst (List values) -> (
      match values with
      | [] -> TList TUnknown
      | v :: _ -> TList (infer_lisp_value_type v))
  | VConst _ -> TUnknown
  | VAtom (AInt _) -> TInt
  | VAtom (AString _) -> TString
  | VAtom (ABoolean _) -> TBool
  | VAtom ANil -> TUnit
  | VCombinator _ -> TCombinator
  | VSymbol _ -> TSymbol
  | VList exprs -> (
      match exprs with
      | [] -> TList TUnknown
      | expr :: _ -> TList (infer_value_expr_type expr))
  | VQuote _ -> TSymbol

and infer_lisp_value_type = function
  | Unit -> TUnit
  | Bool _ -> TBool
  | Int _ -> TInt
  | String _ -> TString
  | Symbol _ -> TSymbol
  | List values -> (
      match values with
      | [] -> TList TUnknown
      | v :: _ -> TList (infer_lisp_value_type v))
  | ResourceId _ | ExprId _ | Bytes _ -> TUnknown

let rec type_compatible t1 t2 =
  match (t1, t2) with
  | TUnit, TUnit -> true
  | TBool, TBool -> true
  | TInt, TInt -> true
  | TString, TString -> true
  | TSymbol, TSymbol -> true
  | TCombinator, TCombinator -> true
  | TList t1', TList t2' -> type_compatible t1' t2'
  | TUnknown, _ | _, TUnknown -> true
  | _, _ -> false

let type_check_value_expr expr expected_type =
  let inferred_type = infer_value_expr_type expr in
  type_compatible inferred_type expected_type

(* Type checking for combinator applications *)
let type_check_combinator_application combinator args =
  match (combinator, args) with
  | Add, [ arg1; arg2 ]
  | Sub, [ arg1; arg2 ]
  | Mul, [ arg1; arg2 ]
  | Div, [ arg1; arg2 ] ->
      type_check_value_expr arg1 TInt && type_check_value_expr arg2 TInt
  | Eq, [ arg1; arg2 ] | Lt, [ arg1; arg2 ] | Gt, [ arg1; arg2 ] ->
      let t1 = infer_value_expr_type arg1 in
      let t2 = infer_value_expr_type arg2 in
      type_compatible t1 t2
  | And, [ arg1; arg2 ] | Or, [ arg1; arg2 ] ->
      type_check_value_expr arg1 TBool && type_check_value_expr arg2 TBool
  | Not, [ arg ] -> type_check_value_expr arg TBool
  | Length, [ arg ] -> (
      match infer_value_expr_type arg with
      | TList _ | TString -> true
      | _ -> false)
  | Car, [ arg ] | Cdr, [ arg ] -> (
      match infer_value_expr_type arg with TList _ -> true | _ -> false)
  | Cons, [ _; _ ] -> true (* Cons can work with any types *)
  | If, [ condition; then_expr; else_expr ] ->
      type_check_value_expr condition TBool
      &&
      let t1 = infer_value_expr_type then_expr in
      let t2 = infer_value_expr_type else_expr in
      type_compatible t1 t2
  | Quote, [ _ ] -> true (* Quote can work with any expression *)
  | List, _ -> true (* List can contain any types *)
  | _, _ -> false (* Arity mismatch or unsupported combinator *)

(* Conversion functions *)
let atom_to_lisp_value = function
  | AInt i -> Int i
  | AString s -> String s
  | ABoolean b -> Bool b
  | ANil -> Unit

let lisp_value_to_atom = function
  | Int i -> Some (AInt i)
  | String s -> Some (AString s)
  | Bool b -> Some (ABoolean b)
  | Unit -> Some ANil
  | _ -> None

let rec value_expr_to_lisp_value = function
  | VConst v -> v
  | VAtom a -> atom_to_lisp_value a
  | VCombinator c -> Symbol (combinator_to_string c)
  | VSymbol s -> Symbol s
  | VList exprs -> List (List.map value_expr_to_lisp_value exprs)
  | VQuote expr -> List [ Symbol "quote"; value_expr_to_lisp_value expr ]

(* Pretty printing *)
let rec value_expr_to_string = function
  | VConst Unit -> "()"
  | VConst (Bool true) -> "true"
  | VConst (Bool false) -> "false"
  | VConst (Int i) -> Int64.to_string i
  | VConst (String s) -> "\"" ^ s ^ "\""
  | VConst (Symbol s) -> s
  | VConst (List values) ->
      "("
      ^ String.concat " "
          (List.map
             (fun v ->
               match lisp_value_to_atom v with
               | Some a -> atom_to_string a
               | None -> "?")
             values)
      ^ ")"
  | VConst _ -> "?"
  | VAtom a -> atom_to_string a
  | VCombinator c -> combinator_to_string c
  | VSymbol s -> s
  | VList exprs ->
      "(" ^ String.concat " " (List.map value_expr_to_string exprs) ^ ")"
  | VQuote expr -> "'" ^ value_expr_to_string expr

and atom_to_string = function
  | AInt i -> Int64.to_string i
  | AString s -> "\"" ^ s ^ "\""
  | ABoolean true -> "true"
  | ABoolean false -> "false"
  | ANil -> "nil"

(* AST construction helpers *)
let make_int i = VAtom (AInt i)
let make_string s = VAtom (AString s)
let make_bool b = VAtom (ABoolean b)
let make_nil = VAtom ANil
let make_symbol s = VSymbol s
let make_list exprs = VList exprs
let make_quote expr = VQuote expr
let make_combinator_call combinator args = VList (VCombinator combinator :: args)

(* Common AST patterns *)
let make_if condition then_expr else_expr =
  make_combinator_call If [ condition; then_expr; else_expr ]

let make_let var value body =
  make_combinator_call Let [ VSymbol var; value; body ]

let make_arithmetic_op op left right = make_combinator_call op [ left; right ]
let make_comparison op left right = make_combinator_call op [ left; right ]
