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

(** ProcessDataflowBlock definition structure *)
and process_dataflow_definition = {
  definition_id: expr_id;
  name: str_t;
  input_schema: (str_t, str_t) BatMap.t;  (* field_name -> type_name *)
  output_schema: (str_t, str_t) BatMap.t;
  state_schema: (str_t, str_t) BatMap.t;
  nodes: pdb_node list;
  edges: pdb_edge list;
  default_typed_domain: typed_domain;
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
  Optimization Types
-----------------------------------------------------------------------------*)

(** Effect compatibility specification for optimization *)
and effect_compatibility = {
  effect_type: str_t;
  source_typed_domain: typed_domain;
  target_typed_domain: typed_domain;
  compatibility_score: float;
  transfer_overhead: int64;
}

(** Resource preference specification for optimization *)
and resource_preference = {
  resource_type: str_t;
  preferred_typed_domain: typed_domain;
  preference_weight: float;
  cost_multiplier: float;
}

(** Optimization hint for strategy selection *)
and optimization_hint = {
  strategy_preference: str_t option;
  cost_weight: float;
  time_weight: float;
  quality_weight: float;
  typed_domain_constraints: typed_domain list;
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
  (* Phase 6 optimization enhancements *)
  optimization_hint: expr_id option;
  compatibility_metadata: effect_compatibility list;
  resource_preferences: resource_preference list;
  target_typed_domain: typed_domain option;
  process_dataflow_hint: process_dataflow_initiation_hint option;
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
  resources: resource_flow list; 
  nullifiers: resource_flow list; 
  scoped_by: handler_id; 
  intent_id: expr_id option;
  (* Phase 6 optimization enhancements *)
  source_typed_domain: typed_domain;
  target_typed_domain: typed_domain;
  originating_dataflow_instance: entity_id option;
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