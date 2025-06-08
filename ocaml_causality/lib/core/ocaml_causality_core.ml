(* ------------ CAUSALITY CORE ------------ *)
(* Purpose: Unified interface for all core Causality types *)

(* ========================================= *)
(* BASIC TYPES AND IDENTIFIERS *)
(* ========================================= *)

type bytes = Bytes.t
(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's
    [u8; N] for IDs. *)

type str_t = string
(** Represents a string. Corresponds to Rust's Str or String. *)

type timestamp = int64
(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to
    Rust's Timestamp. *)

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
type ('a, 'e) result = Ok of 'a | Error of 'e

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
        domain_id : domain_id
      ; zk_constraints : bool
      ; deterministic_only : bool
    }
  | ServiceDomain of {
        domain_id : domain_id
      ; external_apis : str_t list
      ; non_deterministic_allowed : bool
    }
  | ComputeDomain of {
        domain_id : domain_id
      ; compute_intensive : bool
      ; parallel_execution : bool
    }

type domain_compatibility = {
    source_domain : typed_domain
  ; target_domain : typed_domain
  ; transfer_cost : int64
  ; compatibility_score : float
}
(** Domain compatibility specification for cross-domain operations *)

(* ========================================= *)
(* RESOURCE TYPES *)
(* ========================================= *)

type resource_flow = {
    resource_type : str_t
  ; quantity : int64
  ; domain_id : domain_id
}
(** Resource flow specification *)

type nullifier = { resource_id : resource_id; nullifier_hash : bytes }
(** Nullifier representing proof that a resource has been consumed *)

type resource = {
    id : resource_id
  ; name : str_t
  ; domain_id : domain_id
  ; resource_type : str_t
  ; quantity : int64
  ; timestamp : timestamp
}
(** Represents a quantifiable asset or capability *)

type resource_pattern = { resource_type : str_t; domain_id : domain_id option }
(** Resource pattern for matching resources *)

(* ========================================= *)
(* CORE CAUSALITY TYPES *)
(* ========================================= *)

type intent = {
    id : entity_id
  ; name : str_t
  ; domain_id : domain_id
  ; priority : int
  ; inputs : resource_flow list
  ; outputs : resource_flow list
  ; expression : expr_id option
  ; timestamp : timestamp
  ; hint : expr_id option
}
(** Represents a desired outcome or goal in the system *)

type effect = {
    id : effect_id
  ; name : str_t
  ; domain_id : domain_id
  ; effect_type : str_t
  ; inputs : resource_flow list
  ; outputs : resource_flow list
  ; expression : expr_id option
  ; timestamp : timestamp
  ; hint : expr_id option
}
(** Represents a computational effect in the causality system *)

type handler = {
    id : handler_id
  ; name : str_t
  ; domain_id : domain_id
  ; handles_type : str_t
  ; priority : int
  ; expression : expr_id option
  ; timestamp : timestamp
  ; hint : expr_id option
}
(** Represents logic for processing effects or intents *)

type transaction = {
    id : entity_id
  ; name : str_t
  ; domain_id : domain_id
  ; effects : effect_id list
  ; intents : entity_id list
  ; inputs : resource_flow list
  ; outputs : resource_flow list
  ; timestamp : timestamp
}
(** Represents a collection of effects and intents *)

(* Re-export all core modules *)
module Patterns = struct
  include Patterns
end
