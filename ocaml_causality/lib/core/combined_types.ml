(* ------------ CORE CAUSALITY TYPES ------------ *)
(* Purpose: All core types in one module for clean dependencies *)

(* ------------ BASIC TYPE ALIASES ------------ *)

(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's [u8; N] for IDs. *)
type bytes = Bytes.t

(** Represents a string. Corresponds to Rust's Str or String. *)
type str_t = string

(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to Rust's Timestamp. *)
type timestamp = int64

(* ------------ IDENTIFIER TYPES ------------ *)

(** Unique identifier for an expression. *)
type expr_id = bytes

(** Unique identifier for a value expression. *)
type value_expr_id = bytes

(** Generic unique identifier for an entity. *)
type entity_id = bytes

(** Unique identifier for a domain. *)
type domain_id = bytes

(** Unique identifier for a handler. *)
type handler_id = bytes

(** Unique identifier for an edge. *)
type edge_id = bytes

(** Unique identifier for a node. *)
type node_id = bytes

(* ------------ DOMAIN TYPES ------------ *)

(** TypedDomain classification for execution environments. *)
type typed_domain =
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
type domain_compatibility = {
  source_domain: typed_domain;
  target_domain: typed_domain;
  transfer_cost: int64;
  compatibility_score: float;
}

(* ------------ RESOURCE TYPES ------------ *)

(** Resource flow specification. *)
type resource_flow = {
  resource_type: str_t;  
  quantity: int64;       
  domain_id: domain_id;  
}

(** Nullifier representing proof that a resource has been consumed. *)
type nullifier = {
  resource_id: entity_id;
  nullifier_hash: bytes;
}

(** Represents a quantifiable asset or capability. *)
type resource = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  resource_type: str_t; 
  quantity: int64;
  timestamp: timestamp; 
}

(* ------------ CORE CAUSALITY TYPES ------------ *)

(** Represents a desired outcome or goal in the system. *)
type intent = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  priority: int;
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp;
  hint: expr_id option;
}

(** Represents a computational effect in the causality system. *)
type effect = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effect_type: str_t; 
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp; 
  hint: expr_id option;
}

(** Represents logic for processing effects or intents. *)
type handler = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  handles_type: str_t; 
  priority: int; 
  expression: expr_id option;
  timestamp: timestamp;
  hint: expr_id option;
}

(** Represents a collection of effects and intents. *)
type transaction = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effects: entity_id list; 
  intents: entity_id list; 
  inputs: resource_flow list; 
  outputs: resource_flow list; 
  timestamp: timestamp; 
}

(* ------------ PATTERN TYPES ------------ *)

(** Resource pattern for matching resources. *)
type resource_pattern = {
  resource_type: str_t;
  domain_id: domain_id option;
} 