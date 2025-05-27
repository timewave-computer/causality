(* Purpose: S-expression serialization for OCaml types
   This module provides S-expression serialization for the core types in the
   Causality system, enabling interoperability with Rust as specified
   in the ml_work/serialization.md document. *)

open Types

(* Helper functions for creating canonical S-expressions *)

(* Helper for bytes, using hex encoding for better readability. *)
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

let atom_to_sexp (a : atom) : Sexplib0.Sexp.t =
  match a with
  | AInt n -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "AInt"; Sexplib0.Sexp.Atom (Int64.to_string n)]
  | AString s -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "AString"; Sexplib0.Sexp.Atom s]
  | ABoolean b -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ABoolean"; Sexplib0.Sexp.Atom (string_of_bool b)]
  | ANil -> Sexplib0.Sexp.Atom "ANil"

let rec value_expr_to_sexp (ve : value_expr) : Sexplib0.Sexp.t =
  match ve with
  | VNil -> Sexplib0.Sexp.Atom "VNil"
  | VBool b -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VBool"; Sexplib0.Sexp.Atom (string_of_bool b)]
  | VString s -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VString"; Sexplib0.Sexp.Atom s]
  | VInt n -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VInt"; Sexplib0.Sexp.Atom (Int64.to_string n)]
  | VList l -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VList"; Sexplib0.Sexp.List (List.map value_expr_to_sexp l)]
  | VMap m ->
    let bindings = BatMap.bindings m in
    let sexp_bindings = List.map (fun (k, v_exp) -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; value_expr_to_sexp v_exp]) bindings in
    Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VMap"; Sexplib0.Sexp.List sexp_bindings]
  | VStruct s_map ->
    let fields = BatMap.bindings s_map in
    let sexp_fields = List.map (fun (k, v_exp) -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; value_expr_to_sexp v_exp]) fields in
    Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VStruct"; Sexplib0.Sexp.List sexp_fields]
  | VRef target ->
    (match target with
    | VERValue id -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VRef"; Sexplib0.Sexp.Atom "VERValue"; sexp_of_bytes id]
    | VERExpr id -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VRef"; Sexplib0.Sexp.Atom "VERExpr"; sexp_of_bytes id])
  | VLambda { params; body_expr_id; captured_env } ->
    let env_bindings = BatMap.bindings captured_env in
    let sexp_env = List.map (fun (k, v_exp) -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; value_expr_to_sexp v_exp]) env_bindings in
    Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VLambda";
          Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":params"; Sexplib0.Sexp.List (List.map (fun p -> Sexplib0.Sexp.Atom p) params)];
          Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":body-expr-id"; sexp_of_bytes body_expr_id];
          Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":captured-env"; Sexplib0.Sexp.List sexp_env]]

let rec expr_to_sexp (e : expr) : Sexplib0.Sexp.t =
  match e with
  | EAtom atom -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EAtom"; atom_to_sexp atom]
  | EConst value_expr -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EConst"; value_expr_to_sexp value_expr]
  | EVar name_str -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EVar"; Sexplib0.Sexp.Atom name_str]
  | ELambda (params, body) ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ELambda";
            Sexplib0.Sexp.List (List.map (fun p -> Sexplib0.Sexp.Atom p) params);
            expr_to_sexp body]
  | EApply (func_expr, args_expr_list) ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EApply";
            expr_to_sexp func_expr;
            Sexplib0.Sexp.List (List.map expr_to_sexp args_expr_list)]
  | ECombinator op -> Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ECombinator"; atomic_combinator_to_sexp op]
  | EDynamic (step_bound, expr) ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EDynamic"; 
            Sexplib0.Sexp.Atom (string_of_int step_bound);
            expr_to_sexp expr]

and atomic_combinator_to_sexp (ac : atomic_combinator) : Sexplib0.Sexp.t =
  match ac with
  | List -> Sexplib0.Sexp.Atom "list"
  | MakeMap -> Sexplib0.Sexp.Atom "make-map"
  | GetField -> Sexplib0.Sexp.Atom "get-field"
  | Length -> Sexplib0.Sexp.Atom "length"
  | Eq -> Sexplib0.Sexp.Atom "eq"
  | Lt -> Sexplib0.Sexp.Atom "lt"
  | Gt -> Sexplib0.Sexp.Atom "gt"
  | Add -> Sexplib0.Sexp.Atom "add"
  | Sub -> Sexplib0.Sexp.Atom "sub"
  | Mul -> Sexplib0.Sexp.Atom "mul"
  | Div -> Sexplib0.Sexp.Atom "div"
  | And -> Sexplib0.Sexp.Atom "and"
  | Or -> Sexplib0.Sexp.Atom "or"
  | Not -> Sexplib0.Sexp.Atom "not"
  | If -> Sexplib0.Sexp.Atom "if"
  | Let -> Sexplib0.Sexp.Atom "let"
  | Define -> Sexplib0.Sexp.Atom "define"
  | Defun -> Sexplib0.Sexp.Atom "defun"
  | Quote -> Sexplib0.Sexp.Atom "quote"
  | S -> Sexplib0.Sexp.Atom "s"
  | K -> Sexplib0.Sexp.Atom "k"
  | I -> Sexplib0.Sexp.Atom "i"
  | C -> Sexplib0.Sexp.Atom "c"
  | Gte -> Sexplib0.Sexp.Atom "gte"
  | Lte -> Sexplib0.Sexp.Atom "lte"
  | GetContextValue -> Sexplib0.Sexp.Atom "get-context-value"
  | Completed -> Sexplib0.Sexp.Atom "completed"
  | Nth -> Sexplib0.Sexp.Atom "nth"
  | Cons -> Sexplib0.Sexp.Atom "cons"
  | Car -> Sexplib0.Sexp.Atom "car"
  | Cdr -> Sexplib0.Sexp.Atom "cdr"
  | MapGet -> Sexplib0.Sexp.Atom "map-get"
  | MapHasKey -> Sexplib0.Sexp.Atom "map-has-key"

(* Helper functions for deserialization *)

let bytes_from_sexp = function
  | Sexplib0.Sexp.Atom s -> 
      (* Parse hex string back to bytes *)
      let len = String.length s in
      if len mod 2 <> 0 then failwith "Invalid hex string length (must be even)";
      let result = Bytes.create (len / 2) in
      for i = 0 to (len / 2) - 1 do
        let hex_byte = String.sub s (i * 2) 2 in
        let byte_val = int_of_string ("0x" ^ hex_byte) in
        Bytes.set_uint8 result i byte_val
      done;
      result
  | sexp -> failwith ("Invalid bytes representation: " ^ Sexplib0.Sexp.to_string sexp)

let expect_atom = function
  | Sexplib0.Sexp.Atom a -> a
  | sexp -> failwith ("Expected atom: " ^ Sexplib0.Sexp.to_string sexp)

let expect_list = function
  | Sexplib0.Sexp.List l -> l
  | sexp -> failwith ("Expected list: " ^ Sexplib0.Sexp.to_string sexp)

let expect_tag tag = function
  | Sexplib0.Sexp.List (Sexplib0.Sexp.Atom t :: rest) when t = tag -> Sexplib0.Sexp.List rest
  | sexp -> failwith ("Expected list with tag " ^ tag ^ ": " ^ Sexplib0.Sexp.to_string sexp)

let require_field key fields =
  let rec find = function
    | [] -> failwith ("Required field missing: " ^ key)
    | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; v] :: rest when k = key -> v
    | _ :: rest -> find rest
  in
  find fields

let option_to_sexp f = function
  | None -> Sexplib0.Sexp.Atom "nil"
  | Some x -> f x

let option_from_sexp f_val_from_sexp = function
  | Sexplib0.Sexp.Atom "nil" -> None
  | sexp -> Some (f_val_from_sexp sexp)

(* Deserialization functions *)

let rec value_expr_from_sexp = function
  | Sexplib0.Sexp.Atom "VNil" -> VNil
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VBool"; Sexplib0.Sexp.Atom b] -> VBool (bool_of_string b)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VString"; Sexplib0.Sexp.Atom s] -> VString s
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VInt"; Sexplib0.Sexp.Atom n] -> VInt (Int64.of_string n)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VList"; Sexplib0.Sexp.List l] -> VList (List.map value_expr_from_sexp l)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VMap"; Sexplib0.Sexp.List bindings] ->
    let map_bindings = List.map (function
      | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; v] -> (k, value_expr_from_sexp v)
      | sexp -> failwith ("Invalid VMap binding: " ^ Sexplib0.Sexp.to_string sexp)
    ) bindings in
    VMap (BatMap.of_enum (BatList.enum map_bindings))
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VStruct"; Sexplib0.Sexp.List fields] ->
    let struct_fields = List.map (function
      | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; v] -> (k, value_expr_from_sexp v)
      | sexp -> failwith ("Invalid VStruct field: " ^ Sexplib0.Sexp.to_string sexp)
    ) fields in
    VStruct (BatMap.of_enum (BatList.enum struct_fields))
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VRef"; Sexplib0.Sexp.Atom "VERValue"; id_sexp] ->
    VRef (VERValue (bytes_from_sexp id_sexp))
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VRef"; Sexplib0.Sexp.Atom "VERExpr"; id_sexp] ->
    VRef (VERExpr (bytes_from_sexp id_sexp))
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VLambda"; Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":params"; Sexplib0.Sexp.List params]; Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":body-expr-id"; body_id]; Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":captured-env"; Sexplib0.Sexp.List env]] ->
    let param_list = List.map expect_atom params in
    let body_expr_id = bytes_from_sexp body_id in
    let env_bindings = List.map (function
      | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom k; v] -> (k, value_expr_from_sexp v)
      | sexp -> failwith ("Invalid VLambda env binding: " ^ Sexplib0.Sexp.to_string sexp)
    ) env in
    VLambda { params = param_list; body_expr_id; captured_env = BatMap.of_enum (BatList.enum env_bindings) }
  | sexp -> failwith ("Invalid ValueExpr: " ^ Sexplib0.Sexp.to_string sexp)

and expr_from_sexp = function
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EAtom"; atom_sexp] -> EAtom (atom_from_sexp atom_sexp)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EConst"; value_sexp] -> EConst (value_expr_from_sexp value_sexp)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EVar"; Sexplib0.Sexp.Atom name] -> EVar name
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ELambda"; Sexplib0.Sexp.List params; body_sexp] ->
    let param_list = List.map expect_atom params in
    ELambda (param_list, expr_from_sexp body_sexp)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EApply"; func_sexp; Sexplib0.Sexp.List args] ->
    EApply (expr_from_sexp func_sexp, List.map expr_from_sexp args)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ECombinator"; comb_sexp] -> ECombinator (atomic_combinator_from_sexp comb_sexp)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "EDynamic"; Sexplib0.Sexp.Atom step_str; expr_sexp] ->
    EDynamic (int_of_string step_str, expr_from_sexp expr_sexp)
  | sexp -> failwith ("Invalid Expr: " ^ Sexplib0.Sexp.to_string sexp)

and atom_from_sexp = function
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "AInt"; Sexplib0.Sexp.Atom n] -> AInt (Int64.of_string n)
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "AString"; Sexplib0.Sexp.Atom s] -> AString s
  | Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ABoolean"; Sexplib0.Sexp.Atom b] -> ABoolean (bool_of_string b)
  | Sexplib0.Sexp.Atom "ANil" -> ANil
  | sexp -> failwith ("Invalid Atom: " ^ Sexplib0.Sexp.to_string sexp)

and atomic_combinator_from_sexp = function
  | Sexplib0.Sexp.Atom "list" -> List
  | Sexplib0.Sexp.Atom "make-map" -> MakeMap
  | Sexplib0.Sexp.Atom "get-field" -> GetField
  | Sexplib0.Sexp.Atom "length" -> Length
  | Sexplib0.Sexp.Atom "eq" -> Eq
  | Sexplib0.Sexp.Atom "lt" -> Lt
  | Sexplib0.Sexp.Atom "gt" -> Gt
  | Sexplib0.Sexp.Atom "add" -> Add
  | Sexplib0.Sexp.Atom "sub" -> Sub
  | Sexplib0.Sexp.Atom "mul" -> Mul
  | Sexplib0.Sexp.Atom "div" -> Div
  | Sexplib0.Sexp.Atom "and" -> And
  | Sexplib0.Sexp.Atom "or" -> Or
  | Sexplib0.Sexp.Atom "not" -> Not
  | Sexplib0.Sexp.Atom "if" -> If
  | Sexplib0.Sexp.Atom "let" -> Let
  | Sexplib0.Sexp.Atom "define" -> Define
  | Sexplib0.Sexp.Atom "defun" -> Defun
  | Sexplib0.Sexp.Atom "quote" -> Quote
  | Sexplib0.Sexp.Atom "s" -> S
  | Sexplib0.Sexp.Atom "k" -> K
  | Sexplib0.Sexp.Atom "i" -> I
  | Sexplib0.Sexp.Atom "c" -> C
  | Sexplib0.Sexp.Atom "gte" -> Gte
  | Sexplib0.Sexp.Atom "lte" -> Lte
  | Sexplib0.Sexp.Atom "get-context-value" -> GetContextValue
  | Sexplib0.Sexp.Atom "completed" -> Completed
  | Sexplib0.Sexp.Atom "nth" -> Nth
  | Sexplib0.Sexp.Atom "cons" -> Cons
  | Sexplib0.Sexp.Atom "car" -> Car
  | Sexplib0.Sexp.Atom "cdr" -> Cdr
  | Sexplib0.Sexp.Atom "map-get" -> MapGet
  | Sexplib0.Sexp.Atom "map-has-key" -> MapHasKey
  | sexp -> failwith ("Invalid AtomicCombinator: " ^ Sexplib0.Sexp.to_string sexp)

(* Core Type Serialization *)

let resource_flow_to_sexp (rf : resource_flow) : Sexplib0.Sexp.t =
  match rf with
  | { resource_type; quantity; domain_id; _ } ->
    Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "resource-flow";
          Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":resource-type"; Sexplib0.Sexp.Atom resource_type];
          Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":quantity"; Sexplib0.Sexp.Atom (Int64.to_string quantity)];
          Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
         ]

let resource_flow_from_sexp (sexp : Sexplib0.Sexp.t) : resource_flow =
  let tagged = expect_tag "resource-flow" sexp in
  let fields = expect_list tagged in
  let resource_type = expect_atom (require_field ":resource-type" fields) in
  let quantity_str = expect_atom (require_field ":quantity" fields) in 
  let domain_id_sexp = require_field ":domain-id" fields in
  { resource_type;
    quantity = Int64.of_string quantity_str;
    domain_id = bytes_from_sexp domain_id_sexp;
  }

let resource_to_sexp (r : resource) : Sexplib0.Sexp.t =
  let { id; name; domain_id; resource_type; quantity; timestamp } = r in
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "resource";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":id"; sexp_of_bytes id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":name"; Sexplib0.Sexp.Atom name];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":resource-type"; Sexplib0.Sexp.Atom resource_type];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":quantity"; Sexplib0.Sexp.Atom (Int64.to_string quantity)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":timestamp"; Sexplib0.Sexp.Atom (Int64.to_string timestamp)];
       ]

let resource_from_sexp (sexp : Sexplib0.Sexp.t) : resource =
  let tagged = expect_tag "resource" sexp in
  let fields = expect_list tagged in
  let id_sexp = require_field ":id" fields in
  let name = expect_atom (require_field ":name" fields) in
  let domain_id_sexp = require_field ":domain-id" fields in
  let resource_type = expect_atom (require_field ":resource-type" fields) in
  let quantity_str = expect_atom (require_field ":quantity" fields) in
  let timestamp_str = expect_atom (require_field ":timestamp" fields) in
  { id = bytes_from_sexp id_sexp;
    name;
    domain_id = bytes_from_sexp domain_id_sexp;
    resource_type;
    quantity = Int64.of_string quantity_str;
    timestamp = Int64.of_string timestamp_str;
  }

(* Phase 6 Enhancement: TypedDomain serialization *)
let typed_domain_to_sexp (td : typed_domain) : Sexplib0.Sexp.t =
  match td with
  | VerifiableDomain { domain_id; zk_constraints; deterministic_only } ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "VerifiableDomain";
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":zk-constraints"; Sexplib0.Sexp.Atom (string_of_bool zk_constraints)];
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":deterministic-only"; Sexplib0.Sexp.Atom (string_of_bool deterministic_only)];
           ]
  | ServiceDomain { domain_id; external_apis; non_deterministic_allowed } ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ServiceDomain";
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":external-apis"; Sexplib0.Sexp.List (List.map (fun api -> Sexplib0.Sexp.Atom api) external_apis)];
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":non-deterministic-allowed"; Sexplib0.Sexp.Atom (string_of_bool non_deterministic_allowed)];
           ]
  | ComputeDomain { domain_id; compute_intensive; parallel_execution } ->
      Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "ComputeDomain";
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":compute-intensive"; Sexplib0.Sexp.Atom (string_of_bool compute_intensive)];
            Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":parallel-execution"; Sexplib0.Sexp.Atom (string_of_bool parallel_execution)];
           ]

let typed_domain_from_sexp (sexp : Sexplib0.Sexp.t) : typed_domain =
  match sexp with
  | Sexplib0.Sexp.List (Sexplib0.Sexp.Atom "VerifiableDomain" :: fields) ->
      let domain_id_sexp = require_field ":domain-id" fields in
      let zk_constraints_str = expect_atom (require_field ":zk-constraints" fields) in
      let deterministic_only_str = expect_atom (require_field ":deterministic-only" fields) in
      VerifiableDomain {
        domain_id = bytes_from_sexp domain_id_sexp;
        zk_constraints = bool_of_string zk_constraints_str;
        deterministic_only = bool_of_string deterministic_only_str;
      }
  | Sexplib0.Sexp.List (Sexplib0.Sexp.Atom "ServiceDomain" :: fields) ->
      let domain_id_sexp = require_field ":domain-id" fields in
      let external_apis_sexp = require_field ":external-apis" fields in
      let non_deterministic_allowed_str = expect_atom (require_field ":non-deterministic-allowed" fields) in
      ServiceDomain {
        domain_id = bytes_from_sexp domain_id_sexp;
        external_apis = List.map expect_atom (expect_list external_apis_sexp);
        non_deterministic_allowed = bool_of_string non_deterministic_allowed_str;
      }
  | Sexplib0.Sexp.List (Sexplib0.Sexp.Atom "ComputeDomain" :: fields) ->
      let domain_id_sexp = require_field ":domain-id" fields in
      let compute_intensive_str = expect_atom (require_field ":compute-intensive" fields) in
      let parallel_execution_str = expect_atom (require_field ":parallel-execution" fields) in
      ComputeDomain {
        domain_id = bytes_from_sexp domain_id_sexp;
        compute_intensive = bool_of_string compute_intensive_str;
        parallel_execution = bool_of_string parallel_execution_str;
      }
  | _ -> failwith ("Invalid TypedDomain: " ^ Sexplib0.Sexp.to_string sexp)

(* Phase 6 Enhancement: Effect compatibility serialization *)
let effect_compatibility_to_sexp (ec : effect_compatibility) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "effect-compatibility";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":effect-type"; Sexplib0.Sexp.Atom ec.effect_type];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":source-typed-domain"; typed_domain_to_sexp ec.source_typed_domain];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":target-typed-domain"; typed_domain_to_sexp ec.target_typed_domain];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":compatibility-score"; Sexplib0.Sexp.Atom (string_of_float ec.compatibility_score)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":transfer-overhead"; Sexplib0.Sexp.Atom (Int64.to_string ec.transfer_overhead)];
       ]

let effect_compatibility_from_sexp (sexp : Sexplib0.Sexp.t) : effect_compatibility =
  let tagged = expect_tag "effect-compatibility" sexp in
  let fields = expect_list tagged in
  let effect_type = expect_atom (require_field ":effect-type" fields) in
  let source_typed_domain_sexp = require_field ":source-typed-domain" fields in
  let target_typed_domain_sexp = require_field ":target-typed-domain" fields in
  let compatibility_score_str = expect_atom (require_field ":compatibility-score" fields) in
  let transfer_overhead_str = expect_atom (require_field ":transfer-overhead" fields) in
  {
    effect_type;
    source_typed_domain = typed_domain_from_sexp source_typed_domain_sexp;
    target_typed_domain = typed_domain_from_sexp target_typed_domain_sexp;
    compatibility_score = float_of_string compatibility_score_str;
    transfer_overhead = Int64.of_string transfer_overhead_str;
  }

(* Phase 6 Enhancement: Resource preference serialization *)
let resource_preference_to_sexp (rp : resource_preference) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "resource-preference";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":resource-type"; Sexplib0.Sexp.Atom rp.resource_type];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":preferred-typed-domain"; typed_domain_to_sexp rp.preferred_typed_domain];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":preference-weight"; Sexplib0.Sexp.Atom (string_of_float rp.preference_weight)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":cost-multiplier"; Sexplib0.Sexp.Atom (string_of_float rp.cost_multiplier)];
       ]

let resource_preference_from_sexp (sexp : Sexplib0.Sexp.t) : resource_preference =
  let tagged = expect_tag "resource-preference" sexp in
  let fields = expect_list tagged in
  let resource_type = expect_atom (require_field ":resource-type" fields) in
  let preferred_typed_domain_sexp = require_field ":preferred-typed-domain" fields in
  let preference_weight_str = expect_atom (require_field ":preference-weight" fields) in
  let cost_multiplier_str = expect_atom (require_field ":cost-multiplier" fields) in
  {
    resource_type;
    preferred_typed_domain = typed_domain_from_sexp preferred_typed_domain_sexp;
    preference_weight = float_of_string preference_weight_str;
    cost_multiplier = float_of_string cost_multiplier_str;
  }

(* Phase 6 Enhancement: ProcessDataflowBlock initiation hint serialization *)
let process_dataflow_initiation_hint_to_sexp (hint : process_dataflow_initiation_hint) : Sexplib0.Sexp.t =
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "process-dataflow-initiation-hint";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":df-def-id"; sexp_of_bytes hint.df_def_id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":initial-params"; value_expr_to_sexp hint.initial_params];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":target-typed-domain"; option_to_sexp typed_domain_to_sexp hint.target_typed_domain];
       ]

let process_dataflow_initiation_hint_from_sexp (sexp : Sexplib0.Sexp.t) : process_dataflow_initiation_hint =
  let tagged = expect_tag "process-dataflow-initiation-hint" sexp in
  let fields = expect_list tagged in
  let df_def_id_sexp = require_field ":df-def-id" fields in
  let initial_params_sexp = require_field ":initial-params" fields in
  let target_typed_domain_sexp = require_field ":target-typed-domain" fields in
  {
    df_def_id = bytes_from_sexp df_def_id_sexp;
    initial_params = value_expr_from_sexp initial_params_sexp;
    target_typed_domain = option_from_sexp typed_domain_from_sexp target_typed_domain_sexp;
  }

(* Fixed version using explicit field access *)
let intent_to_sexp (intent_record : intent) =
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "intent";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":id"; sexp_of_bytes intent_record.id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":name"; Sexplib0.Sexp.Atom intent_record.name];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes intent_record.domain_id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":priority"; Sexplib0.Sexp.Atom (string_of_int intent_record.priority)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":inputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp intent_record.inputs)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":outputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp intent_record.outputs)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":expression"; option_to_sexp sexp_of_bytes intent_record.expression];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":timestamp"; Sexplib0.Sexp.Atom (Int64.to_string intent_record.timestamp)];
        (* Phase 6 optimization enhancements *)
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":optimization-hint"; option_to_sexp sexp_of_bytes intent_record.optimization_hint];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":compatibility-metadata"; Sexplib0.Sexp.List (List.map effect_compatibility_to_sexp intent_record.compatibility_metadata)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":resource-preferences"; Sexplib0.Sexp.List (List.map resource_preference_to_sexp intent_record.resource_preferences)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":target-typed-domain"; option_to_sexp typed_domain_to_sexp intent_record.target_typed_domain];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":process-dataflow-hint"; option_to_sexp process_dataflow_initiation_hint_to_sexp intent_record.process_dataflow_hint];
       ]

let intent_from_sexp sexp =
  let tagged = expect_tag "intent" sexp in
  let fields = expect_list tagged in
  let id_sexp = require_field ":id" fields in
  let name = expect_atom (require_field ":name" fields) in
  let domain_id_sexp = require_field ":domain-id" fields in
  let priority_str = expect_atom (require_field ":priority" fields) in
  let inputs_sexp = require_field ":inputs" fields in
  let outputs_sexp = require_field ":outputs" fields in
  let expression_sexp = require_field ":expression" fields in
  let timestamp_str = expect_atom (require_field ":timestamp" fields) in
  (* Phase 6 optimization enhancements - use optional field lookup *)
  let optimization_hint_sexp = try Some (require_field ":optimization-hint" fields) with _ -> None in
  let compatibility_metadata_sexp = try Some (require_field ":compatibility-metadata" fields) with _ -> None in
  let resource_preferences_sexp = try Some (require_field ":resource-preferences" fields) with _ -> None in
  let target_typed_domain_sexp = try Some (require_field ":target-typed-domain" fields) with _ -> None in
  let process_dataflow_hint_sexp = try Some (require_field ":process-dataflow-hint" fields) with _ -> None in
  { id = bytes_from_sexp id_sexp;
    name;
    domain_id = bytes_from_sexp domain_id_sexp;
    priority = int_of_string priority_str;
    inputs = List.map resource_flow_from_sexp (expect_list inputs_sexp);
    outputs = List.map resource_flow_from_sexp (expect_list outputs_sexp);
    expression = option_from_sexp bytes_from_sexp expression_sexp;
    timestamp = Int64.of_string timestamp_str;
    (* Phase 6 optimization enhancements *)
    optimization_hint = (match optimization_hint_sexp with 
      | Some sexp -> option_from_sexp bytes_from_sexp sexp 
      | None -> None);
    compatibility_metadata = (match compatibility_metadata_sexp with 
      | Some sexp -> List.map effect_compatibility_from_sexp (expect_list sexp) 
      | None -> []);
    resource_preferences = (match resource_preferences_sexp with 
      | Some sexp -> List.map resource_preference_from_sexp (expect_list sexp) 
      | None -> []);
    target_typed_domain = (match target_typed_domain_sexp with 
      | Some sexp -> option_from_sexp typed_domain_from_sexp sexp 
      | None -> None);
    process_dataflow_hint = (match process_dataflow_hint_sexp with 
      | Some sexp -> option_from_sexp process_dataflow_initiation_hint_from_sexp sexp 
      | None -> None);
  }

let effect_to_sexp (effect_record : effect) =
  let { id; name; domain_id; effect_type; inputs; outputs; expression; timestamp; resources; nullifiers; scoped_by; intent_id; source_typed_domain; target_typed_domain; originating_dataflow_instance } = effect_record in
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "effect";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":id"; sexp_of_bytes id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":name"; Sexplib0.Sexp.Atom name];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":effect-type"; Sexplib0.Sexp.Atom effect_type];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":inputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp inputs)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":outputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp outputs)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":expression"; option_to_sexp sexp_of_bytes expression];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":timestamp"; Sexplib0.Sexp.Atom (Int64.to_string timestamp)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":resources"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp resources)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":nullifiers"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp nullifiers)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":scoped-by"; sexp_of_bytes scoped_by];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":intent-id"; option_to_sexp sexp_of_bytes intent_id];
        (* Phase 6 optimization enhancements *)
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":source-typed-domain"; typed_domain_to_sexp source_typed_domain];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":target-typed-domain"; typed_domain_to_sexp target_typed_domain];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":originating-dataflow-instance"; option_to_sexp sexp_of_bytes originating_dataflow_instance];
       ]

let effect_from_sexp sexp =
  let tagged = expect_tag "effect" sexp in
  let fields = expect_list tagged in
  let id_sexp = require_field ":id" fields in
  let name = expect_atom (require_field ":name" fields) in
  let domain_id_sexp = require_field ":domain-id" fields in
  let effect_type = expect_atom (require_field ":effect-type" fields) in
  let inputs_sexp = require_field ":inputs" fields in
  let outputs_sexp = require_field ":outputs" fields in
  let expression_sexp = require_field ":expression" fields in
  let timestamp_str = expect_atom (require_field ":timestamp" fields) in
  let resources_sexp = require_field ":resources" fields in
  let nullifiers_sexp = require_field ":nullifiers" fields in
  let scoped_by_sexp = require_field ":scoped-by" fields in
  let intent_id_sexp = require_field ":intent-id" fields in
  (* Phase 6 optimization enhancements - use optional field lookup *)
  let source_typed_domain_sexp = try Some (require_field ":source-typed-domain" fields) with _ -> None in
  let target_typed_domain_sexp = try Some (require_field ":target-typed-domain" fields) with _ -> None in
  let originating_dataflow_instance_sexp = try Some (require_field ":originating-dataflow-instance" fields) with _ -> None in
  { id = bytes_from_sexp id_sexp;
    name;
    domain_id = bytes_from_sexp domain_id_sexp;
    effect_type;
    inputs = List.map resource_flow_from_sexp (expect_list inputs_sexp);
    outputs = List.map resource_flow_from_sexp (expect_list outputs_sexp);
    expression = option_from_sexp bytes_from_sexp expression_sexp;
    timestamp = Int64.of_string timestamp_str;
    resources = List.map resource_flow_from_sexp (expect_list resources_sexp);
    nullifiers = List.map resource_flow_from_sexp (expect_list nullifiers_sexp);
    scoped_by = bytes_from_sexp scoped_by_sexp;
    intent_id = option_from_sexp bytes_from_sexp intent_id_sexp;
    (* Phase 6 optimization enhancements *)
    source_typed_domain = (match source_typed_domain_sexp with 
      | Some sexp -> typed_domain_from_sexp sexp 
      | None -> VerifiableDomain { domain_id = Bytes.of_string "default"; zk_constraints = true; deterministic_only = true });
    target_typed_domain = (match target_typed_domain_sexp with 
      | Some sexp -> typed_domain_from_sexp sexp 
      | None -> VerifiableDomain { domain_id = Bytes.of_string "default"; zk_constraints = true; deterministic_only = true });
    originating_dataflow_instance = (match originating_dataflow_instance_sexp with 
      | Some sexp -> option_from_sexp bytes_from_sexp sexp 
      | None -> None);
  }

let handler_to_sexp { id; name; domain_id; handles_type; priority; expression; timestamp } =
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "handler";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":id"; sexp_of_bytes id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":name"; Sexplib0.Sexp.Atom name];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":handles-type"; Sexplib0.Sexp.Atom handles_type];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":priority"; Sexplib0.Sexp.Atom (string_of_int priority)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":expression"; option_to_sexp sexp_of_bytes expression];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":timestamp"; Sexplib0.Sexp.Atom (Int64.to_string timestamp)];
       ]

let handler_from_sexp sexp =
  let tagged = expect_tag "handler" sexp in
  let fields = expect_list tagged in
  let id_sexp = require_field ":id" fields in
  let name = expect_atom (require_field ":name" fields) in
  let domain_id_sexp = require_field ":domain-id" fields in
  let handles_type = expect_atom (require_field ":handles-type" fields) in
  let priority_str = expect_atom (require_field ":priority" fields) in
  let expression_sexp = require_field ":expression" fields in
  let timestamp_str = expect_atom (require_field ":timestamp" fields) in
  { id = bytes_from_sexp id_sexp;
    name;
    domain_id = bytes_from_sexp domain_id_sexp;
    handles_type;
    priority = int_of_string priority_str;
    expression = option_from_sexp bytes_from_sexp expression_sexp;
    timestamp = Int64.of_string timestamp_str;
  }

let transaction_to_sexp (transaction_record : transaction) =
  let { id; name; domain_id; effects; intents; inputs; outputs; timestamp } = transaction_record in
  Sexplib0.Sexp.List [Sexplib0.Sexp.Atom "transaction";
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":id"; sexp_of_bytes id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":name"; Sexplib0.Sexp.Atom name];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":domain-id"; sexp_of_bytes domain_id];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":effects"; Sexplib0.Sexp.List (List.map sexp_of_bytes effects)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":intents"; Sexplib0.Sexp.List (List.map sexp_of_bytes intents)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":inputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp inputs)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":outputs"; Sexplib0.Sexp.List (List.map resource_flow_to_sexp outputs)];
        Sexplib0.Sexp.List [Sexplib0.Sexp.Atom ":timestamp"; Sexplib0.Sexp.Atom (Int64.to_string timestamp)];
       ]

let transaction_from_sexp sexp =
  let tagged = expect_tag "transaction" sexp in
  let fields = expect_list tagged in
  let id_sexp = require_field ":id" fields in
  let name = expect_atom (require_field ":name" fields) in
  let domain_id_sexp = require_field ":domain-id" fields in
  let effects_sexp = require_field ":effects" fields in
  let intents_sexp = require_field ":intents" fields in
  let inputs_sexp = require_field ":inputs" fields in
  let outputs_sexp = require_field ":outputs" fields in
  let timestamp_str = expect_atom (require_field ":timestamp" fields) in
  { id = bytes_from_sexp id_sexp;
    name;
    domain_id = bytes_from_sexp domain_id_sexp;
    effects = List.map bytes_from_sexp (expect_list effects_sexp);
    intents = List.map bytes_from_sexp (expect_list intents_sexp);
    inputs = List.map resource_flow_from_sexp (expect_list inputs_sexp);
    outputs = List.map resource_flow_from_sexp (expect_list outputs_sexp);
    timestamp = Int64.of_string timestamp_str;
  }

(* String Conversion - Serialization Only *)

let value_expr_to_string (v : value_expr) : string = Sexplib0.Sexp.to_string_hum (value_expr_to_sexp v)

let expr_to_string (e : expr) : string = Sexplib0.Sexp.to_string_hum (expr_to_sexp e)

let resource_flow_to_string (rf : resource_flow) : string = Sexplib0.Sexp.to_string_hum (resource_flow_to_sexp rf)

let resource_to_string (r : resource) : string = Sexplib0.Sexp.to_string_hum (resource_to_sexp r)

let intent_to_string (i : intent) : string = Sexplib0.Sexp.to_string_hum (intent_to_sexp i)

let effect_to_string (e : effect) : string = Sexplib0.Sexp.to_string_hum (effect_to_sexp e)

let handler_to_string (h : handler) : string = Sexplib0.Sexp.to_string_hum (handler_to_sexp h)

let transaction_to_string (t : transaction) : string = Sexplib0.Sexp.to_string_hum (transaction_to_sexp t)