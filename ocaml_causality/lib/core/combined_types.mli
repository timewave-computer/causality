(** Core Causality Types Interface
    
    This module provides all essential type definitions for the Causality system,
    including identifiers, domains, resources, intents, effects, handlers, and transactions.
*)

(** {1 Basic Type Aliases} *)

type bytes = Bytes.t
(** Represents a byte array, typically a 32-byte hash. Corresponds to Rust's [u8; N] for IDs. *)

type str_t = string
(** Represents a string. Corresponds to Rust's Str or String. *)

type timestamp = int64
(** Represents a timestamp, typically nanoseconds since epoch. Corresponds to Rust's Timestamp. *)

(** {1 Identifier Types} *)

type expr_id = bytes
(** Unique identifier for an expression. *)

type value_expr_id = bytes
(** Unique identifier for a value expression. *)

type entity_id = bytes
(** Generic unique identifier for an entity. *)

type domain_id = bytes
(** Unique identifier for a domain. *)

type handler_id = bytes
(** Unique identifier for a handler. *)

type edge_id = bytes
(** Unique identifier for an edge. *)

type node_id = bytes
(** Unique identifier for a node. *)

(** {1 Domain Types} *)

(** TypedDomain classification for execution environments. *)
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
  ; compatibility_score : Fixed_point.t
}
(** Domain compatibility specification for cross-domain operations *)

(** {1 Resource Types} *)

type resource_flow = {
    resource_type : str_t
  ; quantity : int64
  ; domain_id : domain_id
}
(** Resource flow specification. *)

type nullifier = { 
    resource_id : Causality_system.System_content_addressing.EntityId.t
  ; nullifier_hash : bytes 
}
(** Nullifier representing proof that a resource has been consumed. *)

type resource = {
    id : Causality_system.System_content_addressing.EntityId.t
  ; name : str_t
  ; domain_id : domain_id
  ; resource_type : str_t
  ; quantity : int64
  ; timestamp : timestamp
}
(** Represents a quantifiable asset or capability. *)

(** {1 Core Causality Types} *)

type intent = {
    id : Causality_system.System_content_addressing.EntityId.t
  ; name : str_t
  ; domain_id : domain_id
  ; priority : int
  ; inputs : resource_flow list
  ; outputs : resource_flow list
  ; expression : expr_id option
  ; timestamp : timestamp
  ; hint : expr_id option
}
(** Represents a desired outcome or goal in the system. *)

type effect = {
    id : Causality_system.System_content_addressing.EntityId.t
  ; name : str_t
  ; domain_id : domain_id
  ; effect_type : str_t
  ; inputs : resource_flow list
  ; outputs : resource_flow list
  ; expression : expr_id option
  ; timestamp : timestamp
  ; hint : expr_id option
}
(** Represents a computational effect in the causality system. *)

type handler = {
    id : Causality_system.System_content_addressing.EntityId.t
  ; name : str_t
  ; domain_id : domain_id
  ; handles_type : str_t
  ; priority : int
  ; expression : expr_id option
  ; timestamp : timestamp
  ; hint : expr_id option
}
(** Represents logic for processing effects or intents. *)

type transaction = {
    id : Causality_system.System_content_addressing.EntityId.t
  ; name : str_t
  ; domain_id : domain_id
  ; effects : Causality_system.System_content_addressing.EntityId.t list
  ; intents : Causality_system.System_content_addressing.EntityId.t list
  ; inputs : resource_flow list
  ; outputs : resource_flow list
  ; timestamp : timestamp
}
(** Represents a collection of effects and intents. *)

(** {1 Pattern Types} *)

type resource_pattern = { 
    resource_type : str_t
  ; domain_id : domain_id option 
}
(** Resource pattern for matching resources. *)
