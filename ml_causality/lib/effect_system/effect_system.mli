(** OCaml Effect System Implementation

    This module provides functionality for translating OCaml algebraic effects to
    TEL (Temporal Effect Language) graph representations. It maps the OCaml effect 
    model to TEL graph components as described in graph.md.
*)

open Ml_causality_lib_types.Types

(** {1 Effect Type Definitions} *)

(** Represents configuration for a user-defined OCaml effect type *)
type effect_type_config = {
  effect_name: string;
  payload_validator: (value_expr -> bool) option;
  default_handler_id: handler_id option;
  ssz_hash: string;                        (** Content-addressed SSZ hash *)
}

(** Registry for effect types that can be used in the system *)
val register_effect_type : 
  effect_name:string -> 
  ?payload_validator:(value_expr -> bool) -> 
  ?default_handler_id:handler_id -> 
  unit -> effect_type_config

(** Get an effect type configuration by name *)
val get_effect_type : string -> effect_type_config option

(** {1 Effect Instance Creation} *)

(** Define a specific effect instance with parameters *)
(* TODO: Commenting out until tel_effect_resource type is defined
val create_effect : 
  effect_type:string -> 
  params:value_expr -> 
  ?static_validation_logic:string -> 
  ?dynamic_logic:string ->
  unit -> tel_effect_resource
*)

(** {1 Effect Handler Management} *)

(** Definition of a handler for OCaml effects *)
type handler_definition = {
  handler_id: handler_id;
  handler_name: string;
  handles_effects: string list;
  config: value_expr;
  static_validator: (value_expr -> bool) option;
  dynamic_logic_ref: string;
  ssz_hash: string;                        (** Content-addressed SSZ hash *)
}

(** Register a handler for one or more effect types *)
val register_handler : 
  handler_id:handler_id ->
  handler_name:string ->
  handles_effects:string list ->
  config:value_expr ->
  ?static_validator:(value_expr -> bool) ->
  dynamic_logic_ref:string ->
  unit -> handler_definition

(** Get a handler definition by ID *)
val get_handler : handler_id -> handler_definition option

(** {1 TEL Graph Construction} *)

(* TODO: All TEL graph functions commented out until types are defined
(** Create edges connecting an effect to its appropriate handler(s) *)
val connect_effect_to_handlers : 
  effect_id:effect_id -> 
  effect_type:string -> 
  unit -> tel_edge list

(** Link an effect to all of its registered handlers *)
val link_effect_to_handlers : 
  effect_id:effect_id -> 
  effect_type:string -> 
  unit -> tel_edge list

(** Build a complete TEL graph from registered effects and handlers *)
val build_tel_graph : unit -> tel_graph

(** {1 Effect Translation Utilities} *)

(** Configuration for OCaml effect to TEL translation *)
type effect_translation_config = {
  auto_create_handlers: bool;        (** Automatically create handler nodes for effects *)
  include_validation_edges: bool;    (** Include static validation edges in the graph *)
}

(** Default translation configuration *)
val default_translation_config : effect_translation_config

(** Create a TEL graph from OCaml effect performances in code *)
val translate_ocaml_effects : 
  ?config:effect_translation_config -> 
  unit -> tel_graph

(** {1 PPX Integration} *)

(** Content-addressed effect node for the unified SSZ/SMT/TEG system *)
type extracted_effect_node = {
  ssz_root: string;                    (** Content-addressed identity *)
  effect_type: string;                 (** OCaml effect type name *)
  parameters: value_expr;              (** SSZ-serializable parameters *)
  source_location: string option;      (** Source file:line for debugging *)
  dependencies: string list;           (** SSZ roots of dependency effects *)
}

(** Result of extracting effects from OCaml code *)
type effect_extraction_result = {
  effect_nodes: extracted_effect_node list;  (** Content-addressed effect nodes *)
  temporal_edges: tel_edge list;             (** TEG edges between effects *)
  smt_updates: (string * value_expr) list;   (** SMT key-value pairs to store *)
}

(** Extract effect performances from OCaml code using unified SSZ/SMT/TEG approach *)
val extract_effect_performs : 
  ocaml_code:string -> 
  effect_extraction_result
*)

(** {1 Handling Continuations} *)

(** Validates if the continuation is used exactly once in a handler's logic code *)
val validate_continuation_usage : dynamic_logic_code:string -> bool

(** Enforces the linear use of continuations in a handler's logic *)
val enforce_continuation_linearity : handler_id:handler_id -> bool

(** {1 Type-Driven Translation} *)

(** Simple representation of an OCaml effect type *)
type ocaml_effect_type = {
  effect_name: string;
  parameter_type: string;
  return_type: string;
}

(** Extract type information from OCaml effect definitions *)
val extract_effect_type : ocaml_code:string -> ocaml_effect_type option

(** Generate a default value_expr based on a type string *)
val generate_value_expr_from_type : type_str:string -> value_expr

(** Create an effect with type information *)
val create_effect_with_type_info : 
  effect_type:ocaml_effect_type -> 
  params_opt:value_expr option -> 
  value_expr 