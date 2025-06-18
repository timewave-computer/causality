(* ------------ VALENCE ACCOUNT FACTORY PRIMITIVES ------------ *)
(* Purpose: OCaml DSL primitives for Valence account factory operations *)

open Expr
open Value

(* ------------ ACCOUNT FACTORY PRIMITIVES ------------ *)

(* Create account factory account *)
let create_account_factory =
  Expr.lambda
    [
      LispValue.symbol "owner"
    ; LispValue.symbol "permissions"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_owner"))
           [ Expr.const (LispValue.symbol "owner") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "create_valence_account"))
           [
             Expr.const (LispValue.symbol "factory")
           ; Expr.const (LispValue.symbol "owner")
           ; Expr.const (LispValue.symbol "permissions")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_account_created"))
           [ Expr.const (LispValue.symbol "owner") ]
       ])

(* Approve library for account factory *)
let approve_library =
  Expr.lambda
    [
      LispValue.symbol "account"
    ; LispValue.symbol "library"
    ; LispValue.symbol "permissions"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_account_owner"))
           [ Expr.const (LispValue.symbol "account") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "validate_library"))
           [ Expr.const (LispValue.symbol "library") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "set_library_approval"))
           [
             Expr.const (LispValue.symbol "account")
           ; Expr.const (LispValue.symbol "library")
           ; Expr.const (LispValue.symbol "permissions")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_library_approved"))
           [
             Expr.const (LispValue.symbol "account")
           ; Expr.const (LispValue.symbol "library")
           ]
       ])

(* Submit transaction to account factory *)
let submit_transaction =
  Expr.lambda
    [
      LispValue.symbol "account"
    ; LispValue.symbol "operation"
    ; LispValue.symbol "data"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_transaction"))
           [
             Expr.const (LispValue.symbol "account")
           ; Expr.const (LispValue.symbol "operation")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "execute_transaction"))
           [
             Expr.const (LispValue.symbol "account")
           ; Expr.const (LispValue.symbol "operation")
           ; Expr.const (LispValue.symbol "data")
           ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "emit_transaction_submitted"))
           [
             Expr.const (LispValue.symbol "account")
           ; Expr.const (LispValue.symbol "operation")
           ]
       ])

(* Get account factory status *)
let get_account_status =
  Expr.lambda
    [ LispValue.symbol "account" ]
    (Expr.apply
       (Expr.const (LispValue.symbol "query_account_status"))
       [ Expr.const (LispValue.symbol "account") ])

(* List approved libraries for account *)
let list_approved_libraries =
  Expr.lambda
    [ LispValue.symbol "account" ]
    (Expr.apply
       (Expr.const (LispValue.symbol "query_approved_libraries"))
       [ Expr.const (LispValue.symbol "account") ])

(* Get transaction history *)
let get_transaction_history =
  Expr.lambda
    [ LispValue.symbol "account"; LispValue.symbol "limit" ]
    (Expr.apply
       (Expr.const (LispValue.symbol "query_transaction_history"))
       [
         Expr.const (LispValue.symbol "account")
       ; Expr.const (LispValue.symbol "limit")
       ])

(* ------------ CONVENIENCE FUNCTIONS ------------ *)

(* Create account factory with default permissions *)
let create_default_account_factory =
  Expr.lambda
    [ LispValue.symbol "owner" ]
    (Expr.apply
       create_account_factory
       [
         Expr.const (LispValue.symbol "owner")
       ; Expr.const (LispValue.list [
           LispValue.symbol "read"
         ; LispValue.symbol "write"
         ; LispValue.symbol "execute"
         ])
       ])

(* Approve library with default permissions *)
let approve_library_default =
  Expr.lambda
    [ LispValue.symbol "account"; LispValue.symbol "library" ]
    (Expr.apply
       approve_library
       [
         Expr.const (LispValue.symbol "account")
       ; Expr.const (LispValue.symbol "library")
       ; Expr.const (LispValue.list [
           LispValue.symbol "read"
         ; LispValue.symbol "execute"
         ])
       ])

(* Submit simple transaction *)
let submit_simple_transaction =
  Expr.lambda
    [ LispValue.symbol "account"; LispValue.symbol "operation" ]
    (Expr.apply
       submit_transaction
       [
         Expr.const (LispValue.symbol "account")
       ; Expr.const (LispValue.symbol "operation")
       ; Expr.const (LispValue.symbol "unit")
       ])

(* ------------ VALIDATION HELPERS ------------ *)

(* Validate account factory configuration *)
let validate_account_factory_config =
  Expr.lambda
    [
      LispValue.symbol "owner"
    ; LispValue.symbol "permissions"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_owner_format"))
           [ Expr.const (LispValue.symbol "owner") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "validate_permissions_list"))
           [ Expr.const (LispValue.symbol "permissions") ]
       ])

(* Validate library approval *)
let validate_library_approval =
  Expr.lambda
    [
      LispValue.symbol "account"
    ; LispValue.symbol "library"
    ]
    (Expr.sequence
       [
         Expr.apply
           (Expr.const (LispValue.symbol "validate_account_exists"))
           [ Expr.const (LispValue.symbol "account") ]
       ; Expr.apply
           (Expr.const (LispValue.symbol "validate_library_exists"))
           [ Expr.const (LispValue.symbol "library") ]
       ])

(* ------------ PRIMITIVE REGISTRY ------------ *)

(* Valence primitive registry *)
module ValencePrimitiveRegistry = struct
  type t = (string * Expr.t) list ref

  let create () = ref []

  let register registry name primitive =
    registry := (name, primitive) :: !registry

  let lookup registry name = List.assoc_opt name !registry
  let list_primitives registry = List.map fst !registry
end

(* Default Valence primitive registry *)
let default_valence_registry = ValencePrimitiveRegistry.create ()

let () =
  let open ValencePrimitiveRegistry in
  register default_valence_registry "create_account_factory" create_account_factory;
  register default_valence_registry "approve_library" approve_library;
  register default_valence_registry "submit_transaction" submit_transaction;
  register default_valence_registry "get_account_status" get_account_status;
  register default_valence_registry "list_approved_libraries" list_approved_libraries;
  register default_valence_registry "get_transaction_history" get_transaction_history;
  register default_valence_registry "create_default_account_factory" create_default_account_factory;
  register default_valence_registry "approve_library_default" approve_library_default;
  register default_valence_registry "submit_simple_transaction" submit_simple_transaction;
  register default_valence_registry "validate_account_factory_config" validate_account_factory_config;
  register default_valence_registry "validate_library_approval" validate_library_approval

(* ------------ DSL BUILDER FUNCTIONS ------------ *)

(* High-level DSL functions for account factory operations *)
let account_factory ~owner ?(permissions=[]) () =
  Expr.apply create_account_factory [
    Expr.const (LispValue.symbol owner);
    Expr.const (LispValue.list (List.map LispValue.symbol permissions))
  ]

let library_approval ~account ~library ?(permissions=[]) () =
  Expr.apply approve_library [
    Expr.const (LispValue.symbol account);
    Expr.const (LispValue.symbol library);
    Expr.const (LispValue.list (List.map LispValue.symbol permissions))
  ]

let transaction_submission ~account ~operation ?(data="unit") () =
  Expr.apply submit_transaction [
    Expr.const (LispValue.symbol account);
    Expr.const (LispValue.symbol operation);
    Expr.const (LispValue.symbol data)
  ]

(* ------------ EFFECT TRACKING ------------ *)

(* Track effects for account factory operations *)
let track_account_factory_effects expr =
  let effects = ref [] in
  let rec extract_effects = function
    | Apply (Const (Symbol name), _) when 
        String.starts_with ~prefix:"create_" name ||
        String.starts_with ~prefix:"approve_" name ||
        String.starts_with ~prefix:"submit_" name ->
        effects := name :: !effects
    | Apply (func, args) ->
        extract_effects func;
        List.iter extract_effects args
    | Sequence exprs ->
        List.iter extract_effects exprs
    | Let (_, value, body) ->
        extract_effects value;
        extract_effects body
    | If (cond, then_expr, else_expr) ->
        extract_effects cond;
        extract_effects then_expr;
        extract_effects else_expr
    | Lambda (_, body) ->
        extract_effects body
    | _ -> ()
  in
  extract_effects expr;
  List.rev !effects 