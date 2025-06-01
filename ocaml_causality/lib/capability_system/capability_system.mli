(*
 * Capability System Module Interface
 *
 * Defines types and operations for managing capabilities within the Causality system.
 * Capabilities control access to resources and operations.
 *)

open Ocaml_causality_core
open Ocaml_causality_core.Types

(*---------------------------------------------------------------------------
 * Capability Types
 *---------------------------------------------------------------------------*)

(** Capability definition *)
type capability_def = {
  capability_id: entity_id;           (** Unique capability identifier *)
  name: string;                       (** Human-readable name *)
  description: string;                (** Capability description *)
  required_permissions: string list;  (** Prerequisites (e.g., other capability IDs or specific permission tags) *)
  domain_id: domain_id;               (** Domain scope *)
  metadata: value_expr;               (** Additional metadata (using Core Types.value_expr) *)
}

(** Capability grant *)
type capability_grant = {
  grant_id: entity_id;                (** Unique grant identifier *)
  capability_id: entity_id;           (** Reference to capability_def *)
  grantee_id: entity_id;              (** Actor receiving capability *)
  grantor_id: entity_id;              (** Actor granting capability *)
  conditions: value_expr;             (** Grant conditions (using Core Types.value_expr) *)
  expires_at: timestamp option;       (** Optional expiration timestamp *)
  domain_id: domain_id;               (** Domain scope *)
}

(** Capability state (potentially for SMT storage or tracking) *)
type capability_state = {
  state_id: entity_id;                (** Content-addressed state ID *)
  active_grants: entity_id list;      (** Active grant IDs *)
  revoked_grants: entity_id list;     (** Revoked grant IDs *)
  delegation_chains: value_expr;      (** Delegation relationships (using Core Types.value_expr) *)
  domain_id: domain_id;               (** Domain scope *)
}

(*---------------------------------------------------------------------------
 * Content Addressing Utilities (Potentially internal to .ml)
 *---------------------------------------------------------------------------*)

(** Convert capability definition to a serializable value expression *)
val capability_def_to_value_expr : capability_def -> value_expr

(** Convert capability grant to a serializable value expression *)
val capability_grant_to_value_expr : capability_grant -> value_expr

(** Convert capability state to a serializable value expression *)
val capability_state_to_value_expr : capability_state -> value_expr

(*---------------------------------------------------------------------------
 * SMT-Integrated Operations
 *---------------------------------------------------------------------------*)

(** Generate SMT storage key for a capability definition *)
val capability_def_smt_key : capability_id:entity_id -> domain_id:domain_id -> string

(** Generate SMT storage key for a capability grant *)
val capability_grant_smt_key : grant_id:entity_id -> domain_id:domain_id -> string

(** Generate SMT storage key for a capability state *)
val capability_state_smt_key : state_id:entity_id -> domain_id:domain_id -> string (* Or perhaps grantee_id *)

(** Create a capability definition and its resource representation.
    Returns the definition, the corresponding resource, and its content_id. *)
val create_capability_definition : 
  name:string -> 
  description:string -> 
  required_permissions:string list -> 
  domain_id:domain_id -> 
  ?metadata:value_expr -> 
  unit -> 
  (capability_def * resource * entity_id)

(** Create a capability grant and a potential effect representing the grant operation.
    Returns the grant, the corresponding effect, and its content_id. *)
val create_capability_grant :
  capability_id:entity_id ->
  grantee_id:entity_id ->
  grantor_id:entity_id ->
  domain_id:domain_id ->
  ?conditions:value_expr ->
  ?expires_at:timestamp ->
  unit ->
  (capability_grant * effect * entity_id)

(*---------------------------------------------------------------------------
 * TEG (Temporal Effect Graph) Integration Types and Operations
 *---------------------------------------------------------------------------*)

(** Placeholder for a Temporal Effect Graph (TEG) edge. 
    Actual definition would depend on the graph representation used. *)
type teg_edge = {
  source_node_id: entity_id;
  target_node_id: entity_id;
  edge_type: string; (* e.g., "requires_capability", "grants_capability" *)
  condition_expr_id: entity_id option; (* Optional expression ID for conditional edge *)
  metadata: value_expr; (* Additional edge metadata *)
}

(** Create and register a TEG edge for a capability requirement.
    Returns the created edge or its ID. *)
val create_capability_requirement_edge : 
  effect_id:entity_id -> (* ID of the effect requiring the capability *)
  capability_id:entity_id -> 
  ?condition_expr_id:entity_id -> 
  unit -> 
  teg_edge

(** Create and register a TEG edge for a capability grant.
    Returns the created edge or its ID. *)
val create_capability_grant_edge :
  grantor_id:entity_id ->
  grantee_id:entity_id ->
  capability_id:entity_id ->
  ?condition_expr_id:entity_id ->
  unit ->
  teg_edge

(** Create and register a TEG edge for capability delegation.
    Returns the created edge or its ID. *)
val create_capability_delegation_edge :
  delegator_id:entity_id ->
  delegatee_id:entity_id ->
  capability_id:entity_id ->
  ?condition_expr_id:entity_id ->
  unit ->
  teg_edge

(*---------------------------------------------------------------------------
 * Core Capability Operations
 *---------------------------------------------------------------------------*)

(** Check if a grantee has a specific capability in a domain. *)
val has_capability :
  capability_id:entity_id ->
  grantee_id:entity_id ->
  domain_id:domain_id ->
  unit ->
  bool

(** Grant a capability to a grantee.
    Returns the created grant and its ID. *)
val grant_capability :
  capability_id:entity_id ->
  grantee_id:entity_id ->
  grantor_id:entity_id ->
  domain_id:domain_id ->
  ?conditions:value_expr ->
  ?expires_at:timestamp -> (* Added from capability_grant type *)
  unit ->
  (capability_grant * entity_id)

(** Revoke a capability grant or all grants for a capability from a grantee. *)
val revoke_capability :
  grant_id:entity_id -> (* Specific grant to revoke *)
  revoker_id:entity_id -> (* Actor performing the revocation *)
  domain_id:domain_id ->
  unit ->
  unit (* Raises exception on failure *)

(** Verify capability with zero-knowledge proof (placeholder). *)
val verify_capability_zk :
  capability_id:entity_id ->
  grantee_id:entity_id ->
  domain_id:domain_id ->
  smt_root:string -> (* Assuming SMT root hash is a string for now *)
  proof:bytes -> (* ZK proof bytes *)
  unit ->
  bool

(*---------------------------------------------------------------------------
 * Enhanced System Functions (Potentially SMT specific implementations)
 *---------------------------------------------------------------------------*)

(** Check if a grantee has a specific capability using SMT backend (if different from has_capability). *)
val has_capability_smt :
  capability_id:entity_id ->
  grantee_id:entity_id ->
  domain_id:domain_id ->
  smt_root:string -> (* Explicit SMT root for verification against a specific state *)
  unit ->
  bool

(** Grant a capability with TEG integration (if different from grant_capability).
    Returns the grant, its ID, and perhaps the TEG edge ID or related info. *)
val grant_capability_teg :
  capability_id:entity_id ->
  grantee_id:entity_id ->
  grantor_id:entity_id ->
  domain_id:domain_id ->
  ?conditions:value_expr ->
  ?expires_at:timestamp ->
  unit ->
  (capability_grant * entity_id * teg_edge) (* Assuming it creates and returns the TEG edge *)

(** Generate complete capability system configuration (e.g., for initialization or export). *)
val generate_capability_system_config : unit -> value_expr

(* Further interface definitions will go here, e.g.:
   - val create_capability_definition : ... -> capability_def
   - val grant_capability : ... -> capability_grant
   - val has_capability : ... -> bool 
   - etc.
*) 