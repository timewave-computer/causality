(* ------------ EXPRESSION VALIDATION ------------ *)
(* Purpose: Expression validation and type checking *)

open Ocaml_causality_core
open Expr

(* ------------ TYPE CHECKING ------------ *)

(* Expression type checking functions *)
type expr_type =
  | UnitType
  | BoolType
  | IntType
  | StringType
  | SymbolType
  | ListType of expr_type
  | FunctionType of expr_type list * expr_type
  | ResourceType
  | UnknownType

let rec infer_expr_type = function
  | Const value -> infer_value_type value
  | Alloc _ -> ResourceType
  | Consume _ -> UnitType
  | Lambda (params, body) ->
      let param_types = List.map infer_value_type params in
      let body_type = infer_expr_type body in
      FunctionType (param_types, body_type)
  | Apply (func, _args) ->
      (match infer_expr_type func with
       | FunctionType (_, return_type) -> return_type
       | _ -> UnknownType)
  | Let (_, value, body) ->
      let _ = infer_expr_type value in
      infer_expr_type body
  | If (_, then_expr, else_expr) ->
      let then_type = infer_expr_type then_expr in
      let else_type = infer_expr_type else_expr in
      if types_compatible then_type else_type then then_type else UnknownType
  | Sequence exprs ->
      (match List.rev exprs with
       | [] -> UnitType
       | last :: _ -> infer_expr_type last)

and infer_value_type = function
  | Unit -> UnitType
  | Bool _ -> BoolType
  | Int _ -> IntType
  | String _ -> StringType
  | Symbol _ -> SymbolType
  | List values ->
      (match values with
       | [] -> ListType UnknownType
       | v :: _ -> ListType (infer_value_type v))
  | ResourceId _ -> ResourceType
  | ExprId _ -> FunctionType ([], UnknownType)
  | Bytes _ -> StringType

and types_compatible type1 type2 =
  match type1, type2 with
  | UnitType, UnitType -> true
  | BoolType, BoolType -> true
  | IntType, IntType -> true
  | StringType, StringType -> true
  | SymbolType, SymbolType -> true
  | ResourceType, ResourceType -> true
  | ListType t1, ListType t2 -> types_compatible t1 t2
  | FunctionType (args1, ret1), FunctionType (args2, ret2) ->
      List.length args1 = List.length args2 &&
      List.for_all2 types_compatible args1 args2 &&
      types_compatible ret1 ret2
  | UnknownType, _ | _, UnknownType -> true
  | _, _ -> false

let type_check_expr expr expected_type =
  let inferred_type = infer_expr_type expr in
  types_compatible inferred_type expected_type

(* ------------ WELL-FORMEDNESS ------------ *)

(* Expression well-formedness validation *)
let rec validate_expr_wellformed = function
  | Const value -> validate_value_wellformed value
  | Alloc expr -> validate_expr_wellformed expr
  | Consume resource_id -> Bytes.length resource_id > 0
  | Lambda (params, body) ->
      validate_lambda_params params && validate_expr_wellformed body
  | Apply (func, args) ->
      validate_expr_wellformed func && List.for_all validate_expr_wellformed args
  | Let (name, value, body) ->
      String.length name > 0 && 
      validate_expr_wellformed value && 
      validate_expr_wellformed body
  | If (condition, then_expr, else_expr) ->
      validate_expr_wellformed condition &&
      validate_expr_wellformed then_expr &&
      validate_expr_wellformed else_expr
  | Sequence exprs ->
      List.for_all validate_expr_wellformed exprs

and validate_value_wellformed = function
  | Unit | Bool _ | Int _ -> true
  | String s | Symbol s -> String.length s >= 0
  | List values -> List.for_all validate_value_wellformed values
  | ResourceId rid | Bytes rid -> Bytes.length rid > 0
  | ExprId eid -> Bytes.length eid > 0

and validate_lambda_params params =
  let rec check_unique_symbols seen = function
    | [] -> true
    | Symbol s :: rest ->
        if List.mem s seen then false
        else check_unique_symbols (s :: seen) rest
    | _ :: rest -> check_unique_symbols seen rest
  in
  check_unique_symbols [] params

let validate_variable_bindings expr =
  let rec collect_bindings bound_vars = function
    | Const _ | Alloc _ | Consume _ -> Ok bound_vars
    | Lambda (params, body) ->
        let param_names = List.filter_map (function
          | Symbol s -> Some s
          | _ -> None
        ) params in
        collect_bindings (param_names @ bound_vars) body
    | Apply (func, args) ->
        (match collect_bindings bound_vars func with
         | Ok bound_vars ->
             List.fold_left (fun acc arg ->
               match acc with
               | Ok bound_vars -> collect_bindings bound_vars arg
               | Error _ as e -> e
             ) (Ok bound_vars) args
         | Error _ as e -> e)
    | Let (name, value, body) ->
        (match collect_bindings bound_vars value with
         | Ok bound_vars -> collect_bindings (name :: bound_vars) body
         | Error _ as e -> e)
    | If (condition, then_expr, else_expr) ->
        (match collect_bindings bound_vars condition with
         | Ok bound_vars ->
             (match collect_bindings bound_vars then_expr with
              | Ok bound_vars -> collect_bindings bound_vars else_expr
              | Error _ as e -> e)
         | Error _ as e -> e)
    | Sequence exprs ->
        List.fold_left (fun acc expr ->
          match acc with
          | Ok bound_vars -> collect_bindings bound_vars expr
          | Error _ as e -> e
        ) (Ok bound_vars) exprs
  in
  match collect_bindings [] expr with
  | Ok _ -> true
  | Error _ -> false

(* ------------ SEMANTIC VALIDATION ------------ *)

(* Semantic validation functions *)
let validate_resource_linearity expr =
  let rec check_resource_usage used_resources = function
    | Const _ | Alloc _ -> Ok used_resources
    | Consume resource_id ->
        if List.exists (Bytes.equal resource_id) used_resources then
          Error ("Resource already consumed: " ^ Bytes.to_string resource_id)
        else
          Ok (resource_id :: used_resources)
    | Lambda (_, body) ->
        check_resource_usage used_resources body
    | Apply (func, args) ->
        (match check_resource_usage used_resources func with
         | Ok used_resources ->
             List.fold_left (fun acc arg ->
               match acc with
               | Ok used_resources -> check_resource_usage used_resources arg
               | Error _ as e -> e
             ) (Ok used_resources) args
         | Error _ as e -> e)
    | Let (_, value, body) ->
        (match check_resource_usage used_resources value with
         | Ok used_resources -> check_resource_usage used_resources body
         | Error _ as e -> e)
    | If (condition, then_expr, else_expr) ->
        (match check_resource_usage used_resources condition with
         | Ok used_resources ->
             (match check_resource_usage used_resources then_expr with
              | Ok then_resources ->
                  (match check_resource_usage used_resources else_expr with
                   | Ok else_resources ->
                       (* Both branches should consume the same resources *)
                       if List.length then_resources = List.length else_resources then
                         Ok then_resources
                       else
                         Error "Inconsistent resource usage in conditional branches"
                   | Error _ as e -> e)
              | Error _ as e -> e)
         | Error _ as e -> e)
    | Sequence exprs ->
        List.fold_left (fun acc expr ->
          match acc with
          | Ok used_resources -> check_resource_usage used_resources expr
          | Error _ as e -> e
        ) (Ok used_resources) exprs
  in
  match check_resource_usage [] expr with
  | Ok _ -> true
  | Error _ -> false

let validate_function_arity expr =
  let rec check_arity = function
    | Const _ | Alloc _ | Consume _ -> true
    | Lambda (params, body) ->
        List.length params >= 0 && check_arity body
    | Apply (func, args) ->
        (match func with
         | Lambda (params, _) ->
             List.length params = List.length args
         | _ -> true) &&
        check_arity func &&
        List.for_all check_arity args
    | Let (_, value, body) ->
        check_arity value && check_arity body
    | If (condition, then_expr, else_expr) ->
        check_arity condition && check_arity then_expr && check_arity else_expr
    | Sequence exprs ->
        List.for_all check_arity exprs
  in
  check_arity expr

let validate_domain_constraints expr domain_id =
  (* Mock domain validation - in real implementation would check domain-specific rules *)
  let domain_str = Bytes.to_string domain_id in
  let check_domain_rules = function
    | Const (Symbol s) ->
        (* Check if symbol is allowed in domain *)
        not (String.contains s '@') || String.ends_with ~suffix:("@" ^ domain_str) s
    | Apply (Const (Symbol "transfer"), _) when domain_str = "defi" -> true
    | Apply (Const (Symbol "mint"), _) when domain_str = "token" -> true
    | _ -> true
  in
  check_domain_rules expr

(* ------------ ERROR REPORTING ------------ *)

(* Validation error reporting functions *)
type validation_error =
  | TypeMismatch of expr_type * expr_type
  | UnboundVariable of string
  | LinearityViolation of string
  | ArityMismatch of int * int
  | DomainConstraintViolation of string
  | WellformednessError of string

let rec validation_error_to_string = function
  | TypeMismatch (expected, actual) ->
      Printf.sprintf "Type mismatch: expected %s, got %s" 
        (type_to_string expected) (type_to_string actual)
  | UnboundVariable var ->
      Printf.sprintf "Unbound variable: %s" var
  | LinearityViolation msg ->
      Printf.sprintf "Linearity violation: %s" msg
  | ArityMismatch (expected, actual) ->
      Printf.sprintf "Arity mismatch: expected %d arguments, got %d" expected actual
  | DomainConstraintViolation msg ->
      Printf.sprintf "Domain constraint violation: %s" msg
  | WellformednessError msg ->
      Printf.sprintf "Well-formedness error: %s" msg

and type_to_string = function
  | UnitType -> "unit"
  | BoolType -> "bool"
  | IntType -> "int"
  | StringType -> "string"
  | SymbolType -> "symbol"
  | ListType t -> "list[" ^ type_to_string t ^ "]"
  | FunctionType (args, ret) ->
      "(" ^ String.concat " -> " (List.map type_to_string args) ^ " -> " ^ type_to_string ret ^ ")"
  | ResourceType -> "resource"
  | UnknownType -> "unknown"

let validate_expression expr =
  let errors = ref [] in
  
  (* Check well-formedness *)
  if not (validate_expr_wellformed expr) then
    errors := WellformednessError "Expression is not well-formed" :: !errors;
  
  (* Check variable bindings *)
  if not (validate_variable_bindings expr) then
    errors := UnboundVariable "Unbound variable detected" :: !errors;
  
  (* Check resource linearity *)
  if not (validate_resource_linearity expr) then
    errors := LinearityViolation "Resource used multiple times" :: !errors;
  
  (* Check function arity *)
  if not (validate_function_arity expr) then
    errors := ArityMismatch (0, 0) :: !errors;
  
  match !errors with
  | [] -> Ok ()
  | errors -> Error errors

let validate_expression_with_context expr context =
  match validate_expression expr with
  | Ok () ->
      (* Additional context-specific validation *)
      let domain_valid = match context with
        | Some domain_id -> validate_domain_constraints expr domain_id
        | None -> true
      in
      if domain_valid then Ok () else Error [DomainConstraintViolation "Domain constraints violated"]
  | Error errors -> Error errors

(* Validation utilities *)
let is_valid_expression expr =
  match validate_expression expr with
  | Ok () -> true
  | Error _ -> false

let get_validation_errors expr =
  match validate_expression expr with
  | Ok () -> []
  | Error errors -> errors

let validate_and_report expr =
  match validate_expression expr with
  | Ok () -> Printf.printf "Expression validation passed\n"
  | Error errors ->
      Printf.printf "Expression validation failed:\n";
      List.iter (fun err ->
        Printf.printf "  - %s\n" (validation_error_to_string err)
      ) errors 