(*
 * JSON Conversion Module Interface
 *
 * This module provides functionality for converting Causality types to and from
 * JSON representations. It enables interoperability with web APIs, JavaScript
 * environments, and other systems that use JSON as a data exchange format.
 *)

(* TODO: Add Yojson dependency or create custom JSON implementation *)
(* open Yojson.Safe *)

(* Temporary JSON type definition *)
type json = 
  | Null
  | Bool of bool
  | Int of int
  | String of string
  | Assoc of (string * json) list
  | List of json list
  | Intlit of string

open Ocaml_causality_core
open Ocaml_causality_lang

(** JSON conversion error *)
type json_error =
  | InvalidJson of string          (** Invalid JSON format *)
  | MissingField of string         (** Required field is missing *)
  | TypeMismatch of string         (** Type doesn't match expected type *)
  | UnsupportedType of string      (** Type not supported for JSON conversion *)
  | InvalidValue of string         (** Value doesn't meet constraints *)

(** Result type for JSON conversion operations *)
type 'a result = ('a, json_error) Result.t

(** Convert an entity ID to JSON *)
val entity_id_to_json : entity_id -> json

(** Convert JSON to an entity ID *)
val entity_id_of_json : json -> entity_id result

(** Convert a timestamp to JSON *)
val timestamp_to_json : timestamp -> json

(** Convert JSON to a timestamp *)
val timestamp_of_json : json -> timestamp result

(** Convert a domain ID to JSON *)
val domain_id_to_json : domain_id -> json

(** Convert JSON to a domain ID *)
val domain_id_of_json : json -> domain_id result

(** Convert an AST atom to JSON *)
val atom_to_json : Ast.atom -> json

(** Convert JSON to an AST atom *)
val atom_of_json : json -> Ast.atom result

(** Convert an AST value expression to JSON *)
val value_expr_to_json : Ast.value_expr -> json

(** Convert JSON to an AST value expression *)
val value_expr_of_json : json -> Ast.value_expr result

(** Convert an AST expression to JSON *)
val expr_to_json : Ast.expr -> json

(** Convert JSON to an AST expression *)
val expr_of_json : json -> Ast.expr result

(** Convert a resource to JSON *)
val resource_to_json : resource -> json

(** Convert JSON to a resource *)
val resource_of_json : json -> resource result

(** Convert an effect instance to JSON *)
val effect_to_json : Ocaml_causality_effects.Effects.effect_instance -> json

(** Convert JSON to an effect instance *)
val effect_of_json : json -> Ocaml_causality_effects.Effects.effect_instance result

(** Convert any Causality value to JSON *)
val to_json : 'a -> string -> json

(** Convert JSON to a specific Causality type *)
val of_json : json -> string -> 'a result

(** Convert a value to a JSON string *)
val to_string : 'a -> string -> string

(** Parse a JSON string to a specific Causality type *)
val parse_string : string -> string -> 'a result 