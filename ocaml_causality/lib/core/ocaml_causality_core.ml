(* ------------ CAUSALITY CORE ------------ *)
(* Purpose: Unified interface for all core Causality types *)

(* ========================================= *)
(* BASIC TYPES AND IDENTIFIERS *)
(* ========================================= *)

(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's [u8; N] for IDs. *)
type bytes = Bytes.t

(** Represents a string. Corresponds to Rust's Str or String. *)
type str_t = string

(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to Rust's Timestamp. *)
type timestamp = int64

(* Unique identifiers *)
type resource_id = bytes
type expr_id = bytes
type value_expr_id = bytes
type entity_id = bytes
type domain_id = bytes
type handler_id = bytes
type effect_id = bytes
type edge_id = bytes
type node_id = bytes

(* ========================================= *)
(* RESULT TYPES *)
(* ========================================= *)

(** Result type for operations that can fail *)
type ('a, 'e) result = 
  | Ok of 'a
  | Error of 'e

(** Standard error type for Causality operations *)
type causality_error =
  | LinearityViolation of str_t
  | InvalidResource of resource_id
  | InvalidExpression of expr_id
  | FFIError of str_t
  | SerializationError of str_t
  | DomainError of str_t

(* ========================================= *)
(* LISP VALUE TYPES *)
(* ========================================= *)

(** LispValue type corresponding to Rust's LispValue for FFI *)
type lisp_value =
  | Unit
  | Bool of bool
  | Int of int64
  | String of str_t
  | Symbol of str_t
  | List of lisp_value list
  | ResourceId of resource_id
  | ExprId of expr_id
  | Bytes of bytes

(* ========================================= *)
(* DOMAIN TYPES *)
(* ========================================= *)

(** TypedDomain classification for execution environments *)
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

(* ========================================= *)
(* RESOURCE TYPES *)
(* ========================================= *)

(** Resource flow specification *)
type resource_flow = {
  resource_type: str_t;  
  quantity: int64;       
  domain_id: domain_id;  
}

(** Nullifier representing proof that a resource has been consumed *)
type nullifier = {
  resource_id: resource_id;
  nullifier_hash: bytes;
}

(** Represents a quantifiable asset or capability *)
type resource = {
  id: resource_id; 
  name: str_t; 
  domain_id: domain_id; 
  resource_type: str_t; 
  quantity: int64;
  timestamp: timestamp; 
}

(** Resource pattern for matching resources *)
type resource_pattern = {
  resource_type: str_t;
  domain_id: domain_id option;
}

(* ========================================= *)
(* CORE CAUSALITY TYPES *)
(* ========================================= *)

(** Represents a desired outcome or goal in the system *)
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

(** Represents a computational effect in the causality system *)
type effect = {
  id: effect_id; 
  name: str_t; 
  domain_id: domain_id; 
  effect_type: str_t; 
  inputs: resource_flow list;
  outputs: resource_flow list;
  expression: expr_id option; 
  timestamp: timestamp; 
  hint: expr_id option;
}

(** Represents logic for processing effects or intents *)
type handler = {
  id: handler_id; 
  name: str_t; 
  domain_id: domain_id; 
  handles_type: str_t; 
  priority: int; 
  expression: expr_id option;
  timestamp: timestamp;
  hint: expr_id option;
}

(** Represents a collection of effects and intents *)
type transaction = {
  id: entity_id; 
  name: str_t; 
  domain_id: domain_id; 
  effects: effect_id list; 
  intents: entity_id list; 
  inputs: resource_flow list; 
  outputs: resource_flow list; 
  timestamp: timestamp; 
}

(* Re-export all core modules *)
module Patterns = struct
  include Patterns
end
