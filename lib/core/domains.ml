(* ------------ DOMAIN TYPES AND LOGIC ------------ *)
(* Purpose: Domain types, domain logic, and compatibility *)

(* Import identifiers from the same module *)
include Identifiers

(* ------------ TYPE DEFINITIONS ------------ *)

(** TypedDomain classification for execution environments. 
    Corresponds to Rust's TypedDomain enum. *)
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

(* ------------ DOMAIN COMPATIBILITY ------------ *)

(* TODO: Add domain compatibility checking functions *)

(* ------------ DOMAIN OPERATIONS ------------ *)

(* TODO: Add domain creation and validation functions *)

(* ------------ DOMAIN UTILITIES ------------ *)

(* TODO: Add utility functions for domain handling *) 