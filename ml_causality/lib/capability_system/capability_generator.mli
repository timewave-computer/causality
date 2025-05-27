(** capability_generator.mli - Interface for Homoiconic SMT AST Capability System *)

(* Purpose: Interface for the capability system generator enhanced with content addressing, 
   SMT storage, TEG integration, and homoiconic AST manipulation. *)

open Ml_causality_lib_types.Types

(*---------------------------------------------------------------------------
 * Content-Addressed Capability Types
 *---------------------------------------------------------------------------*)

(** Content-addressed capability definition *)
type capability_def = {
  capability_id: string;              (* Unique capability identifier *)
  name: string;                       (* Human-readable name *)
  description: string;                (* Capability description *)
  required_permissions: string list;  (* Prerequisites *)
  domain: string;                     (* Domain scope *)
  metadata: value_expr;               (* Additional metadata *)
}

(** Content-addressed capability grant *)
type capability_grant = {
  grant_id: string;                   (* Unique grant identifier *)
  capability_id: string;              (* Reference to capability *)
  grantee: string;                    (* Actor receiving capability *)
  grantor: string;                    (* Actor granting capability *)
  conditions: value_expr;             (* Grant conditions *)
  expires_at: string option;          (* Optional expiration *)
  domain: string;                     (* Domain scope *)
}

(** SMT-stored capability state *)
type capability_state = {
  state_id: string;                   (* Content-addressed state ID *)
  active_grants: string list;         (* Active grant IDs *)
  revoked_grants: string list;        (* Revoked grant IDs *)
  delegation_chains: value_expr;      (* Delegation relationships *)
  domain: string;                     (* Domain scope *)
}

(*---------------------------------------------------------------------------
 * Content Addressing Utilities
 *---------------------------------------------------------------------------*)

(** Convert capability definition to content-addressed value *)
val capability_def_to_value_expr : capability_def -> value_expr

(** Convert capability grant to content-addressed value *)
val capability_grant_to_value_expr : capability_grant -> value_expr

(** Convert capability state to content-addressed value *)
val capability_state_to_value_expr : capability_state -> value_expr

(*---------------------------------------------------------------------------
 * SMT-Integrated Operations
 *---------------------------------------------------------------------------*)

(** Generate SMT storage key for capability definition *)
val capability_def_smt_key : string -> string -> string

(** Generate SMT storage key for capability grant *)
val capability_grant_smt_key : string -> string -> string

(** Generate SMT storage key for capability state *)
val capability_state_smt_key : string -> string -> string

(** Create content-addressed capability definition resource *)
val create_capability_definition : 
  name:string -> 
  description:string -> 
  required_permissions:string list -> 
  domain:string -> 
  ?metadata:value_expr -> 
  unit -> 
  (capability_def * resource * bytes)

(** Create content-addressed capability grant *)
val create_capability_grant :
  capability_id:string ->
  grantee:string ->
  grantor:string ->
  domain:string ->
  ?conditions:value_expr ->
  ?expires_at:string ->
  unit ->
  (capability_grant * effect * bytes)

(*---------------------------------------------------------------------------
 * TEG Integration
 *---------------------------------------------------------------------------*)

(** Create TEG edge for capability requirement *)
val create_capability_requirement_edge : 
  effect_id:string -> 
  capability_id:string -> 
  ?condition:expr_id -> 
  unit -> 
  tel_edge

(** Create TEG edge for capability grant *)
val create_capability_grant_edge :
  grantor_id:string ->
  grantee_id:string ->
  capability_id:string ->
  ?condition:expr_id ->
  unit ->
  tel_edge

(** Create TEG edge for capability delegation *)
val create_capability_delegation_edge :
  delegator_id:string ->
  delegate_id:string ->
  capability_id:string ->
  ?condition:expr_id ->
  unit ->
  tel_edge

(*---------------------------------------------------------------------------
 * Core Capability Operations (OCaml Functions)
 *---------------------------------------------------------------------------*)

(** Check if a grantee has a specific capability in a domain *)
val has_capability :
  capability_id:string ->
  grantee:string ->
  domain:string ->
  unit ->
  (string * string * string)

(** Grant a capability to a grantee *)
val grant_capability :
  capability_id:string ->
  grantee:string ->
  grantor:string ->
  domain:string ->
  ?conditions:value_expr ->
  unit ->
  (capability_grant * bytes)

(** Revoke a capability from a grantee *)
val revoke_capability :
  capability_id:string ->
  grantee:string ->
  domain:string ->
  unit ->
  (string * string)

(** Verify capability with zero-knowledge proof *)
val verify_capability_zk :
  capability_id:string ->
  grantee:string ->
  domain:string ->
  smt_root:string ->
  proof:string ->
  unit ->
  (string * string * string * string)

(*---------------------------------------------------------------------------
 * Enhanced System Functions
 *---------------------------------------------------------------------------*)

(** Check if a grantee has a specific capability using SMT backend *)
val has_capability_smt :
  capability_id:string ->
  grantee:string ->
  domain:string ->
  unit ->
  string

(** Grant a capability using TEG integration *)
val grant_capability_teg :
  capability_id:string ->
  grantee:string ->
  grantor:string ->
  domain:string ->
  conditions:value_expr ->
  unit ->
  ((capability_grant * effect * bytes) * string)

(** Generate complete capability system configuration *)
val generate_capability_system_config : unit -> value_expr

 