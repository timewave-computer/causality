(*
 * Validation Module
 *
 * This module provides functions for validating expressions,
 * including type checking, structure validation, and semantic analysis.
 *)

open Ast

(* ------------ EXPRESSION TYPES ------------ *)

(** Simple type system for expression validation *)
type expr_type =
  | TUnit                     (** Unit/void type *)
  | TInt                      (** Integer type *)
  | TFloat                    (** Float type *)
  | TString                   (** String type *)
  | TBool                     (** Boolean type *)
  | TList of expr_type        (** List type *)
  | TMap of expr_type         (** Map type *)
  | TFunction of expr_type list * expr_type  (** Function type *)
  | TAny                      (** Any type (for polymorphic functions) *)
  | TResource                 (** Resource type *)
  | TDomain                   (** Domain type *)
  | TUnknown                  (** Unknown/error type *)

(* ------------ TYPE INFERENCE ------------ *)

(** Infer the type of an atom *)
let infer_atom_type = function
  | Symbol _ -> TUnknown  (* Can't determine type without context *)
  | String _ -> TString
  | Integer _ -> TInt
  | Float _ -> TFloat
  | Boolean _ -> TBool

(** Infer the type of a value expression *)
let rec infer_value_type = function
  | VUnit -> TUnit
  | VAtom a -> infer_atom_type a
  | VList items ->
      (* Try to infer element type from first item, default to TAny *)
      (match items with
       | [] -> TList TAny
       | first :: _ -> TList (infer_value_type first))
  | VMap _ -> TMap TAny  (* Generic map type *)
  | VClosure _ -> TFunction ([], TAny)  (* Generic function type *)
  | VNative _ -> TFunction ([], TAny)   (* Generic function type *)

(** Infer the expected return type of a combinator *)
let infer_combinator_type = function
  | Add | Sub | Mul | Div -> TInt
  | Eq | Gt | Lt | Gte | Lte | And | Or | Not | MapHasKey -> TBool
  | If -> TAny  (* Depends on branches *)
  | Let | LetStar -> TAny  (* Depends on body *)
  | List -> TList TAny
  | MakeMap -> TMap TAny
  | MapGet -> TAny  (* Depends on map values *)
  | Length -> TInt
  | Car -> TAny  (* Depends on list element type *)
  | Cdr -> TList TAny
  | Cons -> TList TAny
  | Define | Defun -> TUnit
  | Quote -> TAny
  | GetContextValue | GetField -> TAny
  | Completed -> TBool
  | S | K | I | C -> TAny  (* Core combinators can have any type *)
  | Nth -> TAny  (* Depends on list element type *)

(* ------------ VALIDATION FUNCTIONS ------------ *)

(** Check if a value has the expected type *)
let rec type_matches value expected_type =
  let actual_type = infer_value_type value in
  match expected_type, actual_type with
  | TAny, _ -> true  (* Any type matches with anything *)
  | _, TAny -> true  (* Any type matches with anything *)
  | TUnit, TUnit -> true
  | TInt, TInt -> true
  | TFloat, TFloat -> true
  | TString, TString -> true
  | TBool, TBool -> true
  | TList t1, TList t2 -> type_matches_list t1 t2
  | TMap t1, TMap t2 -> type_matches_map t1 t2
  | TFunction (params1, ret1), TFunction (params2, ret2) ->
      type_matches_function params1 ret1 params2 ret2
  | TResource, TResource -> true
  | TDomain, TDomain -> true
  | _, _ -> false

(** Check if list element types match *)
and type_matches_list t1 t2 =
  match t1, t2 with
  | TAny, _ -> true
  | _, TAny -> true
  | _, _ -> t1 = t2

(** Check if map value types match *)
and type_matches_map t1 t2 =
  match t1, t2 with
  | TAny, _ -> true
  | _, TAny -> true
  | _, _ -> t1 = t2

(** Check if function types match *)
and type_matches_function params1 ret1 params2 ret2 =
  (* For simplicity, just check return types *)
  type_matches_list ret1 ret2

(** Validate the arguments for a combinator *)
let validate_combinator_args comb args =
  let expected_count = Combinators.expected_args comb in
  if expected_count >= 0 && List.length args != expected_count then
    Error (Printf.sprintf "Combinator %s expects %d arguments, got %d"
             (Combinators.combinator_to_string comb) expected_count (List.length args))
  else
    Ok ()

(** Validate a complete expression for well-formedness *)
let rec validate_expr expr =
  match expr with
  | EAtom _ -> Ok ()  (* Atoms are always valid *)
  | EConst _ -> Ok ()  (* Constants are always valid *)
  | EVar _ -> Ok ()  (* Variables need environment to validate *)
  | ELambda (params, body) ->
      (* Check for duplicate parameter names *)
      let param_set = List.sort_uniq String.compare params in
      if List.length param_set <> List.length params then
        Error "Lambda has duplicate parameter names"
      else
        validate_expr body
  | EApply (func, args) ->
      (* Validate function and all arguments *)
      let func_result = validate_expr func in
      let args_results = List.map validate_expr args in
      let errors = List.filter_map
                     (function Error e -> Some e | Ok () -> None)
                     (func_result :: args_results) in
      if List.length errors > 0 then
        Error (String.concat "; " errors)
      else
        Ok ()
  | ECombinator comb ->
      (* Combinators by themselves are valid *)
      Ok ()
  | EDynamic (_, expr) ->
      (* Validate the inner expression *)
      validate_expr expr

(** Check if an expression is well-typed for a given expected type *)
let rec type_check expr expected_type =
  match expr with
  | EAtom atom ->
      let atom_type = infer_atom_type atom in
      if atom_type = expected_type || expected_type = TAny then
        Ok ()
      else
        Error (Printf.sprintf "Expected %s, got atom of type %s"
                 (string_of_type expected_type) (string_of_type atom_type))
  | EConst value ->
      let value_type = infer_value_type value in
      if type_matches value expected_type then
        Ok ()
      else
        Error (Printf.sprintf "Expected %s, got constant of type %s"
                 (string_of_type expected_type) (string_of_type value_type))
  | EVar _ ->
      Ok ()  (* Can't type check variables without environment *)
  | ELambda _ ->
      if expected_type = TFunction ([], TAny) || expected_type = TAny then
        Ok ()
      else
        Error (Printf.sprintf "Expected %s, got function"
                 (string_of_type expected_type))
  | EApply (ECombinator comb, args) ->
      let comb_type = infer_combinator_type comb in
      if comb_type = expected_type || expected_type = TAny then
        Ok ()
      else
        Error (Printf.sprintf "Combinator %s returns %s, expected %s"
                 (Combinators.combinator_to_string comb)
                 (string_of_type comb_type) (string_of_type expected_type))
  | EApply _ ->
      Ok ()  (* Can't fully type check applications without evaluation *)
  | ECombinator _ ->
      Ok ()  (* Bare combinators need application to type check *)
  | EDynamic _ ->
      Ok ()  (* Can't type check dynamic expressions without evaluation *)

(** Convert type to string representation *)
and string_of_type = function
  | TUnit -> "unit"
  | TInt -> "int"
  | TFloat -> "float"
  | TString -> "string"
  | TBool -> "bool"
  | TList t -> Printf.sprintf "list(%s)" (string_of_type t)
  | TMap t -> Printf.sprintf "map(%s)" (string_of_type t)
  | TFunction (_, ret) -> Printf.sprintf "function -> %s" (string_of_type ret)
  | TAny -> "any"
  | TResource -> "resource"
  | TDomain -> "domain"
  | TUnknown -> "unknown" 