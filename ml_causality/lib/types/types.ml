(* Purpose: OCaml type definitions corresponding to causality_types crate. *)

(* Basic type aliases for Rust correspondence *)

(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's [u8; N] for IDs. *)
type bytes = Bytes.t 

(** Represents a string. Corresponds to Rust's Str or String. *)
type str_t = string 

(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to Rust's Timestamp. *)
type timestamp = int64 

(** Unique identifier for an expression. Corresponds to Rust's ExprId. *)
type expr_id = bytes 

(** Unique identifier for a value expression. (Note: In Rust, ExprId often serves for content-addressed values too) *)
type value_expr_id = bytes 

(** Generic unique identifier for an entity. Corresponds to Rust's EntityId. *)
type entity_id = bytes 

(** Unique identifier for a domain. Corresponds to Rust's DomainId. *)
type domain_id = bytes 

(** Unique identifier for a handler. Corresponds to Rust's HandlerId. *)
type handler_id = bytes 

(** Unique identifier for an edge. Corresponds to Rust's EdgeId. *)
type edge_id = bytes

(** Unique identifier for a node. Corresponds to Rust's NodeId. *)
type node_id = bytes

(*-----------------------------------------------------------------------------
  Expression AST Types (Corresponds to causality_types::expr)
-----------------------------------------------------------------------------*)

(* Corresponds to causality_types::expr::ast::AtomicCombinator *)
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
  | Defun
  | Quote
  | S
  | K
  | I
  | C
  | Gte
  | Lte
  | GetContextValue
  | Completed
  | Nth
  | Cons
  | Car
  | Cdr
  | MapGet
  | MapHasKey

(* Corresponds to causality_types::expr::ast::Atom *)
type atom =
  | AInt of int64
  | AString of str_t 
  | ABoolean of bool
  | ANil

(* Corresponds to Rust's ValueExprRef enum, defining the target of a VRef *)
and value_expr_ref_target =
  | VERValue of value_expr_id  (* Reference to a ValueExpr *)
  | VERExpr of expr_id         (* Reference to a quoted Expr *)

(* Corresponds to causality_types::expr::value::ValueExpr *)
and value_expr =  
  | VNil 
  | VBool of bool 
  | VString of str_t 
  | VInt of int64 
  | VList of value_expr list 
  | VMap of (str_t, value_expr) BatMap.t 
  | VStruct of (str_t, value_expr) BatMap.t 
  | VRef of value_expr_ref_target 
  | VLambda of {
      params: str_t list;
      body_expr_id: expr_id;
      captured_env: (str_t, value_expr) BatMap.t;
    }

(* Corresponds to causality_types::expr::ast::Expr - mutually recursive with value_expr *)
and expr = 
  | EAtom of atom
  | EConst of value_expr 
  | EVar of str_t 
  | ELambda of str_t list * expr 
  | EApply of expr * expr list 
  | ECombinator of atomic_combinator
  | EDynamic of int * expr

(* Module for ordered string maps, if BatMap is not available/desired, use Map.Make(String) *)
(* Ensure BatMap or an equivalent is available in the project dependencies *)
(* module StringMap = BatMap.Make(String) *)

(*-----------------------------------------------------------------------------
  TypedDomain Types
-----------------------------------------------------------------------------*)

(** TypedDomain classification for execution environments. 
    Corresponds to Rust's TypedDomain enum. *)
and typed_domain =
  | VerifiableDomain of {
      domain_id: domain_id;
      zk_constraints: bool;
      deterministic_only: bool;
    }
  | ServiceDomain of {
      domain_id: domain_id;
      external_apis: str_t list;
      non_deterministic_allowed: bool;
    }
  | ComputeDomain of {
      domain_id: domain_id;
      compute_intensive: bool;
      parallel_execution: bool;
    }

(** Domain compatibility specification for cross-domain operations *)
and domain_compatibility = {
  source_domain: typed_domain;
  target_domain: typed_domain;
  transfer_cost: int64;
  compatibility_score: float;
}

(*-----------------------------------------------------------------------------
  ProcessDataflowBlock Types
-----------------------------------------------------------------------------*)

(** ProcessDataflowBlock node definition *)
and pdb_node = {
  node_id: str_t;
  node_type: str_t;
  typed_domain_policy: typed_domain option;
  action_template: expr_id option;
  gating_conditions: expr_id list;
}

(** ProcessDataflowBlock edge definition *)
and pdb_edge = {
  from_node: str_t;
  to_node: str_t;
  condition: expr_id option;
  transition_type: str_t;
}

(** Type schema representation matching Rust's TypeExpr *)
and type_schema = 
  | Unit
  | Bool
  | Integer
  | Number
  | String
  | List of type_schema
  | Optional of type_schema
  | Map of type_schema * type_schema
  | Record of (str_t * type_schema) list
  | Union of type_schema list
  | Any

(** Auto-schema generator trait interface *)
type 'a schema_generator = {
  generate_schema: unit -> type_schema;
  schema_name: string;
}

(** ProcessDataflowBlock definition structure with automatic schema generation *)
and process_dataflow_definition = {
  definition_id: expr_id;
  name: str_t;
  (* Removed manual schema fields - now generated automatically *)
  nodes: pdb_node list;
  edges: pdb_edge list;
  default_typed_domain: typed_domain;
  (* Type phantom data - would be used for generic schema generation *)
  input_schema_gen: type_schema option;
  output_schema_gen: type_schema option;
  state_schema_gen: type_schema option;
}

(** Typed ProcessDataflow definition with automatic schema generation *)
type ('input, 'output, 'state) typed_process_dataflow = {
  definition: process_dataflow_definition;
  input_generator: 'input schema_generator;
  output_generator: 'output schema_generator;
  state_generator: 'state schema_generator;
}

(** ProcessDataflowBlock instance state *)
and process_dataflow_instance_state = {
  instance_id: entity_id;
  definition_id: expr_id;
  current_node_id: str_t;
  state_values: value_expr;
  created_timestamp: timestamp;
  last_updated: timestamp;
}

(** ProcessDataflowBlock reference types *)
and process_dataflow_reference =
  | DefinitionId of expr_id
  | InstanceId of entity_id

(** ProcessDataflowBlock initiation hint for intents *)
and process_dataflow_initiation_hint = {
  df_def_id: expr_id;
  initial_params: value_expr;
  target_typed_domain: typed_domain option;
}

(*-----------------------------------------------------------------------------
  Core Causality Types (Corresponds to causality_types::core)
-----------------------------------------------------------------------------*)

(** Resource flow specification. Defines how resources are consumed or produced. Corresponds to Rust's `ResourceFlow`. *)
and resource_flow = {
  resource_type: str_t;  
  quantity: int64;       
  domain_id: domain_id;  
}

(** Resource pattern for matching resources. Corresponds to Rust's `ResourcePattern`. *)
and resource_pattern = {
  resource_type: str_t;
  domain_id: domain_id option;
  constraints: (str_t, str_t) BatMap.t;
}

(** Nullifier representing proof that a resource has been consumed. Corresponds to Rust's `Nullifier`. *)
and nullifier = {
  resource_id: entity_id;
  nullifier_hash: bytes;
}

(** Represents a quantifiable asset or capability. Corresponds to Rust's `Resource`. *)
and resource = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  resource_type: str_t; 
  quantity: int64;
  timestamp: timestamp; 
}

(** Represents a desired outcome or goal in the system. Corresponds to Rust's `Intent`. *)
and intent = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  priority: int;
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp;
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(** Represents a computational effect in the causality system. Corresponds to Rust's `Effect`. *)
and effect = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effect_type: str_t; 
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp; 
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(** Represents logic for processing effects or intents. Corresponds to Rust's `Handler`. *)
and handler = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  handles_type: str_t; 
  priority: int; 
  expression: expr_id option;
  timestamp: timestamp;
  hint: expr_id option;  (* Soft preferences for optimization *)
}

(** Represents a collection of effects and intents, forming an atomic unit of change. Corresponds to Rust's `Transaction`. *)
and transaction = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effects: entity_id list; 
  intents: entity_id list; 
  inputs: resource_flow list; 
  outputs: resource_flow list; 
  timestamp: timestamp; 
}

(*-----------------------------------------------------------------------------
  TEL Edge Types (Corresponds to causality_types::tel::graph)
-----------------------------------------------------------------------------*)

(** Resource reference for edge kinds *)
type resource_ref = {
  resource_id: entity_id;
  resource_type: str_t;
}

(** Edge kind defining the relationship type in TEL graph *)
type edge_kind =
  | ControlFlow
  | Next of node_id
  | DependsOn of node_id
  | Consumes of resource_ref
  | Produces of resource_ref
  | Applies of handler_id
  | ScopedBy of handler_id
  | Override of handler_id

(** TEL Edge structure *)
type tel_edge = {
  id: edge_id;
  source: node_id;
  target: node_id;
  kind: edge_kind;
  metadata: value_expr option;
}

(*-----------------------------------------------------------------------------
  Schema Generation Module (for automatic ProcessDataflow schemas)
-----------------------------------------------------------------------------*)

(** Schema generation utilities *)
module SchemaGen = struct
  
  (** Generate schema for basic types *)
  let string_schema : string schema_generator = {
    generate_schema = (fun () -> String);
    schema_name = "string";
  } [@@warning "-32"]
  
  let int_schema : int schema_generator = {
    generate_schema = (fun () -> Integer);
    schema_name = "int";
  } [@@warning "-32"]
  
  let bool_schema : bool schema_generator = {
    generate_schema = (fun () -> Bool);
    schema_name = "bool";
  } [@@warning "-32"]
  
  let unit_schema : unit schema_generator = {
    generate_schema = (fun () -> Unit);
    schema_name = "unit";
  } [@@warning "-32"]
  
  (** Generate schema for lists *)
  let list_schema (inner : 'a schema_generator) : 'a list schema_generator = {
    generate_schema = (fun () -> List (inner.generate_schema ()));
    schema_name = "list_" ^ inner.schema_name;
  } [@@warning "-32"]
  
  (** Generate schema for options *)
  let option_schema (inner : 'a schema_generator) : 'a option schema_generator = {
    generate_schema = (fun () -> Optional (inner.generate_schema ()));
    schema_name = "option_" ^ inner.schema_name;
  } [@@warning "-32"]
  
  (** Generate schema for maps *)
  let map_schema (key : 'k schema_generator) (value : 'v schema_generator) : ('k, 'v) BatMap.t schema_generator = {
    generate_schema = (fun () -> Map (key.generate_schema (), value.generate_schema ()));
    schema_name = "map_" ^ key.schema_name ^ "_" ^ value.schema_name;
  } [@@warning "-32"]
  
  (** Generate schema for records - manual definition required *)
  let record_schema (fields : (string * type_schema) list) (name : string) = {
    generate_schema = (fun () -> Record fields);
    schema_name = name;
  } [@@warning "-32"]
  
end

(** Create a typed ProcessDataflow with automatic schema generation *)
let create_typed_dataflow 
    (definition_id : expr_id)
    (name : str_t)
    (input_gen : 'input schema_generator)
    (output_gen : 'output schema_generator)
    (state_gen : 'state schema_generator) : ('input, 'output, 'state) typed_process_dataflow =
  let definition = {
    definition_id;
    name;
    nodes = [];
    edges = [];
    default_typed_domain = VerifiableDomain { domain_id = Bytes.of_string "default"; zk_constraints = true; deterministic_only = true };
    input_schema_gen = Some (input_gen.generate_schema ());
    output_schema_gen = Some (output_gen.generate_schema ());
    state_schema_gen = Some (state_gen.generate_schema ());
  } in
  {
    definition;
    input_generator = input_gen;
    output_generator = output_gen;
    state_generator = state_gen;
  } [@@warning "-32"]

(** Get auto-generated schemas from typed dataflow *)
let get_schemas (dataflow : ('i, 'o, 's) typed_process_dataflow) = 
  (dataflow.input_generator.generate_schema (),
   dataflow.output_generator.generate_schema (),
   dataflow.state_generator.generate_schema ()) [@@warning "-32"]