(*
 * S-expression Serialization Module
 *
 * This module provides S-expression serialization for the core types in the
 * Causality system, enabling interoperability with Rust and human-readable
 * representations of data structures.
 *)

open Ocaml_causality_core
open Ocaml_causality_lang.Ast

(* ------------ HELPER FUNCTIONS ------------ *)

(** Convert bytes to hex string for S-expression representation *)
let sexp_of_bytes (b : bytes) : Sexplib0.Sexp.t = 
  let hex_chars = "0123456789abcdef" in
  let len = Bytes.length b in
  let hex = Bytes.create (len * 2) in
  for i = 0 to len - 1 do
    let byte = Bytes.get_uint8 b i in
    Bytes.set hex (i * 2) hex_chars.[byte lsr 4];
    Bytes.set hex (i * 2 + 1) hex_chars.[byte land 15]
  done;
  Sexplib0.Sexp.Atom (Bytes.to_string hex)

(** Convert string to S-expression atom *)
let sexp_of_string (s : string) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.Atom s

(** Convert int64 to S-expression *)
let sexp_of_int64 (i : int64) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.Atom (Int64.to_string i)

(** Convert bool to S-expression *)
let sexp_of_bool (b : bool) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.Atom (string_of_bool b)

(* ------------ AST ATOM SERIALIZATION ------------ *)

(** Convert AST atom to S-expression *)
let atom_to_sexp (a : atom) : Sexplib0.Sexp.t =
  match a with
  | Integer n -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "Integer"; sexp_of_int64 n]
  | String s -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "String"; sexp_of_string s]
  | Float f -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "Float"; Sexplib0.Sexp.Atom (string_of_float f)]
  | Boolean b -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "Boolean"; sexp_of_bool b]
  | Symbol s -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "Symbol"; sexp_of_string s]

(* ------------ VALUE EXPRESSION SERIALIZATION ------------ *)

(** Convert value expression to S-expression *)
let rec value_expr_to_sexp (ve : value_expr) : Sexplib0.Sexp.t =
  match ve with
  | VUnit -> Sexplib0.Sexp.Atom "VUnit"
  | VAtom a -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VAtom"; atom_to_sexp a]
  | VList l -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VList"; Sexplib0.Sexp.List (List.map value_expr_to_sexp l)]
  | VMap m ->
      let bindings = List.map (fun (k, v) -> 
        Sexplib0.Sexp.List [sexp_of_string k; value_expr_to_sexp v]
      ) m in
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VMap"; Sexplib0.Sexp.List bindings]
  | VClosure _ -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VClosure"; Sexplib0.Sexp.Atom "<closure>"]
  | VNative _ -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VNative"; Sexplib0.Sexp.Atom "<native>"]

(** Convert atomic combinator to S-expression *)
let atomic_combinator_to_sexp (c : atomic_combinator) : Sexplib0.Sexp.t =
  let name = match c with
    | S -> "S" | K -> "K" | I -> "I" | C -> "C"
    | If -> "If" | Let -> "Let" | LetStar -> "LetStar"
    | And -> "And" | Or -> "Or" | Not -> "Not"
    | Eq -> "Eq" | Gt -> "Gt" | Lt -> "Lt" | Gte -> "Gte" | Lte -> "Lte"
    | Add -> "Add" | Sub -> "Sub" | Mul -> "Mul" | Div -> "Div"
    | GetContextValue -> "GetContextValue" | GetField -> "GetField" | Completed -> "Completed"
    | List -> "List" | Nth -> "Nth" | Length -> "Length" | Cons -> "Cons" | Car -> "Car" | Cdr -> "Cdr"
    | MakeMap -> "MakeMap" | MapGet -> "MapGet" | MapHasKey -> "MapHasKey"
    | Define -> "Define" | Defun -> "Defun" | Quote -> "Quote"
  in
  Sexplib0.Sexp.Atom name

(** Convert expression to S-expression *)
let rec expr_to_sexp (e : expr) : Sexplib0.Sexp.t =
  match e with
  | EAtom a -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EAtom"; atom_to_sexp a]
  | EConst v -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EConst"; value_expr_to_sexp v]
  | EVar s -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EVar"; sexp_of_string s]
  | ELambda (params, body) ->
      Sexplib0.Sexp.List [
        Sexplib0.Sexp.Atom "ELambda";
        Sexplib0.Sexp.List (List.map sexp_of_string params);
        expr_to_sexp body
      ]
  | EApply (func, args) ->
      Sexplib0.Sexp.List [
        Sexplib0.Sexp.Atom "EApply";
        expr_to_sexp func;
        Sexplib0.Sexp.List (List.map expr_to_sexp args)
      ]
  | ECombinator c ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ECombinator"; atomic_combinator_to_sexp c]
  | EDynamic (idx, e) ->
      Sexplib0.Sexp.List [
        Sexplib0.Sexp.Atom "EDynamic";
        Sexplib0.Sexp.Atom (string_of_int idx);
        expr_to_sexp e
      ]

(* ------------ CORE TYPE SERIALIZATION ------------ *)

(** Convert resource flow to S-expression *)
let resource_flow_to_sexp (rf : Types.resource_flow) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [
    Sexplib0.Sexp.Atom "ResourceFlow";
    Sexplib0.Sexp.List [
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "resource_type"; sexp_of_string rf.flow_resource_type];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "quantity"; sexp_of_int64 rf.flow_quantity];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "domain_id"; sexp_of_bytes rf.flow_domain_id];
    ]
  ]

(** Convert resource to S-expression *)
let resource_to_sexp (r : Types.resource) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [
    Sexplib0.Sexp.Atom "Resource";
    Sexplib0.Sexp.List [
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "id"; sexp_of_bytes r.resource_id];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "name"; sexp_of_string r.resource_name];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "domain_id"; sexp_of_bytes r.resource_domain_id];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "resource_type"; sexp_of_string r.resource_type];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "quantity"; sexp_of_int64 r.resource_quantity];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "timestamp"; sexp_of_int64 r.resource_timestamp];
    ]
  ]

(** Convert intent to S-expression *)
let intent_to_sexp (i : Types.intent) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [
    Sexplib0.Sexp.Atom "Intent";
    Sexplib0.Sexp.List [
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "id"; sexp_of_bytes i.intent_id];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "name"; sexp_of_string i.intent_name];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "domain_id"; sexp_of_bytes i.intent_domain_id];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "priority"; Sexplib0.Sexp.Atom (string_of_int i.intent_priority)];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "inputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp i.intent_inputs)];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "outputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp i.intent_outputs)];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "timestamp"; sexp_of_int64 i.intent_timestamp];
    ]
  ]

(** Convert effect to S-expression *)
let effect_to_sexp (e : Types.effect) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [
    Sexplib0.Sexp.Atom "Effect";
    Sexplib0.Sexp.List [
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "id"; sexp_of_bytes e.effect_id];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "name"; sexp_of_string e.effect_name];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "domain_id"; sexp_of_bytes e.effect_domain_id];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "effect_type"; sexp_of_string e.effect_type];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "inputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp e.effect_inputs)];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "outputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp e.effect_outputs)];
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "timestamp"; sexp_of_int64 e.effect_timestamp];
    ]
  ]

(* ------------ CANONICAL S-EXPRESSION GENERATION ------------ *)

(** Convert value expression to canonical S-expression string for hashing *)
let rec value_expr_to_canonical_string (ve : value_expr) : string =
  match ve with
  | VUnit -> "unit"
  | VAtom (Symbol s) -> s
  | VAtom (String s) -> Printf.sprintf "\"%s\"" (String.escaped s)
  | VAtom (Integer i) -> Int64.to_string i
  | VAtom (Float f) -> string_of_float f
  | VAtom (Boolean b) -> string_of_bool b
  | VList items -> 
      let item_strs = List.map value_expr_to_canonical_string items in
      Printf.sprintf "(%s)" (String.concat " " item_strs)
  | VMap entries ->
      let sorted_entries = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) entries in
      let entry_strs = List.map (fun (k, v) -> 
        Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_canonical_string v)
      ) sorted_entries in
      Printf.sprintf "(map %s)" (String.concat " " entry_strs)
  | VClosure _ -> "<closure>"
  | VNative _ -> "<native>"

(** Convert expression to canonical S-expression string *)
let rec expr_to_canonical_string (e : expr) : string =
  match e with
  | EAtom (Symbol s) -> s
  | EAtom (String s) -> Printf.sprintf "\"%s\"" (String.escaped s)
  | EAtom (Integer i) -> Int64.to_string i
  | EAtom (Float f) -> string_of_float f
  | EAtom (Boolean b) -> string_of_bool b
  | EVar name -> name
  | EConst v -> value_expr_to_canonical_string v
  | ELambda (params, body) ->
      Printf.sprintf "(lambda (%s) %s)" 
        (String.concat " " params) 
        (expr_to_canonical_string body)
  | EApply (func, args) ->
      Printf.sprintf "(%s %s)" 
        (expr_to_canonical_string func)
        (String.concat " " (List.map expr_to_canonical_string args))
  | ECombinator c ->
      let name = match c with
        | S -> "S" | K -> "K" | I -> "I" | C -> "C"
        | If -> "if" | Let -> "let" | LetStar -> "let*"
        | And -> "and" | Or -> "or" | Not -> "not"
        | Eq -> "=" | Gt -> ">" | Lt -> "<" | Gte -> ">=" | Lte -> "<="
        | Add -> "+" | Sub -> "-" | Mul -> "*" | Div -> "/"
        | GetContextValue -> "get-context-value" | GetField -> "get-field" | Completed -> "completed"
        | List -> "list" | Nth -> "nth" | Length -> "length" | Cons -> "cons" | Car -> "car" | Cdr -> "cdr"
        | MakeMap -> "make-map" | MapGet -> "map-get" | MapHasKey -> "map-has-key"
        | Define -> "define" | Defun -> "defun" | Quote -> "quote"
      in name
  | EDynamic (idx, e) ->
      Printf.sprintf "(dynamic %d %s)" idx (expr_to_canonical_string e)

(* ------------ UTILITY FUNCTIONS ------------ *)

(** Convert S-expression to string *)
let sexp_to_string (s : Sexplib0.Sexp.t) : string =
  Sexplib0.Sexp.to_string_hum s

(** Pretty print S-expression *)
let pp_sexp fmt (s : Sexplib0.Sexp.t) =
  Format.pp_print_string fmt (sexp_to_string s)