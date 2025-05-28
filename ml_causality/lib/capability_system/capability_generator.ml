(** 
 * Capability System Generator - REFACTORED for Homoiconic SMT AST Integration
 *
 * This module generates content-addressed capability nodes and integrates with:
 * - SSZ-based content addressing for capability identity
 * - SMT storage for verifiable capability state
 * - TEG integration for temporal capability relationships
 * - Homoiconic AST system for capability manipulation as data
 *)

open Ml_causality_lib_types.Types

(*---------------------------------------------------------------------------
 * Native OCaml Utilities (No DSL Dependencies)
 *---------------------------------------------------------------------------*)

(** Helper to convert value_expr to canonical S-expression string for hashing *)
let rec value_expr_to_s_expression (ve: value_expr) : string =
  match ve with
  | VNil -> "nil"
  | VBool b -> string_of_bool b
  | VString s -> Printf.sprintf "\"%s\"" (String.escaped s)
  | VInt i -> Int64.to_string i
  | VList items -> 
    let item_strs = List.map value_expr_to_s_expression items in
    Printf.sprintf "(%s)" (String.concat " " item_strs)
  | VMap m ->
    let entries = BatMap.bindings m in
    let sorted_entries = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) entries in
    let entry_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression v)
    ) sorted_entries in
    Printf.sprintf "(map (%s))" (String.concat " " entry_strs)
  | VStruct s_map -> 
    let fields = BatMap.bindings s_map in
    let sorted_fields = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) fields in
    let field_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression v)
    ) sorted_fields in
    Printf.sprintf "(struct (%s))" (String.concat " " field_strs)
  | VRef (VERValue id) -> Printf.sprintf "(ref:value %s)" (Bytes.to_string id)
  | VRef (VERExpr id) -> Printf.sprintf "(ref:expr %s)" (Bytes.to_string id)
  | VLambda { params; body_expr_id; captured_env } ->
    let param_str = String.concat " " params in
    let env_entries = BatMap.bindings captured_env in
    let sorted_env = List.sort (fun (k1, _) (k2, _) -> String.compare k1 k2) env_entries in
    let env_strs = List.map (fun (k, v) -> 
      Printf.sprintf "(%s %s)" (String.escaped k) (value_expr_to_s_expression v)
    ) sorted_env in
    Printf.sprintf "(lambda (%s) %s (env %s))" param_str (Bytes.to_string body_expr_id) (String.concat " " env_strs)

(** Use digestif to create cryptographic hashes for content addressing *)
let value_expr_to_id (ve: value_expr) : bytes =
  let s_expr = value_expr_to_s_expression ve in
  let hash_hex = Digestif.SHA256.to_hex (Digestif.SHA256.digest_string s_expr) in
  Bytes.of_string hash_hex

(** Create unique deterministic ID from components *)
let _generate_id (components: string list) : string =
  let sorted_components = List.filter (fun s -> s <> "") components |> List.sort String.compare in
  let concatenated = String.concat ":" sorted_components in
  Digestif.SHA256.to_hex (Digestif.SHA256.digest_string concatenated)

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
  capability_id: string;             (* Reference to capability *)
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
 * Content Addressing Utilities for Capabilities
 *---------------------------------------------------------------------------*)

(** Convert capability definition to content-addressed value *)
let capability_def_to_value_expr (cap_def: capability_def) : value_expr =
  VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "capability_definition");
    ("capability_id", VString cap_def.capability_id);
    ("name", VString cap_def.name);
    ("description", VString cap_def.description);
    ("required_permissions", VList (List.map (fun p -> VString p) cap_def.required_permissions));
    ("domain_id", VString cap_def.domain);
    ("metadata", cap_def.metadata);
  ]))

(** Convert capability grant to content-addressed value *)
let capability_grant_to_value_expr (grant: capability_grant) : value_expr =
  VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "capability_grant");
    ("grant_id", VString grant.grant_id);
    ("capability_id", VString grant.capability_id);
    ("grantee", VString grant.grantee);
    ("grantor", VString grant.grantor);
    ("conditions", grant.conditions);
    ("expires_at", match grant.expires_at with 
      | Some exp -> VString exp 
      | None -> VNil);
    ("domain_id", VString grant.domain);
  ]))

(** Convert capability state to content-addressed value *)
let capability_state_to_value_expr (state: capability_state) : value_expr =
  VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "capability_state");
    ("state_id", VString state.state_id);
    ("active_grants", VList (List.map (fun id -> VString id) state.active_grants));
    ("revoked_grants", VList (List.map (fun id -> VString id) state.revoked_grants));
    ("delegation_chains", state.delegation_chains);
    ("domain_id", VString state.domain);
  ]))

(*---------------------------------------------------------------------------
 * SMT-Integrated Capability Operations
 *---------------------------------------------------------------------------*)

(** Generate SMT storage key for capability definition *)
let capability_def_smt_key (cap_id: string) (domain: string) : string =
  _generate_id ["capability_def"; domain; cap_id]

(** Generate SMT storage key for capability grant *)
let capability_grant_smt_key (grant_id: string) (domain: string) : string =
  _generate_id ["capability_grant"; domain; grant_id]

(** Generate SMT storage key for capability state *)
let capability_state_smt_key (grantee: string) (domain: string) : string =
  _generate_id ["capability_state"; domain; grantee]

(** Create content-addressed capability definition resource *)
let create_capability_definition ~name ~description ~required_permissions ~domain ?(metadata=VNil) () =
  let cap_id = _generate_id ["cap"; domain; name] in
  let cap_def = {
    capability_id = cap_id;
    name;
    description;
    required_permissions;
    domain;
    metadata;
  } in
  let value_expr = capability_def_to_value_expr cap_def in
  let content_id = value_expr_to_id value_expr in
  
  (* Create as a resource in the TEG *)
  let cap_resource = {
    id = content_id;
    name = name;
    domain_id = Bytes.of_string domain;
    resource_type = "capability_definition";
    quantity = 1L;
    timestamp = 0L;
  } in
  (cap_def, cap_resource, content_id)

(** Create content-addressed capability grant *)
let create_capability_grant ~capability_id ~grantee ~grantor ~domain ?(conditions=VNil) ?expires_at () =
  let grant_id = _generate_id ["grant"; domain; grantee; capability_id; grantor] in
  let grant = {
    grant_id;
    capability_id;
    grantee;
    grantor;
    conditions;
    expires_at;
    domain;
  } in
  let value_expr = capability_grant_to_value_expr grant in
  let content_id = value_expr_to_id value_expr in
  
  (* Create as effect resource in TEG *)
  let grant_effect = {
    id = content_id;
    name = "GrantCapability";
    domain_id = Bytes.of_string domain;
    effect_type = "capability_grant";
    inputs = [];
    outputs = [];
    expression = None;
    timestamp = 0L;
    hint = None;  (* Soft preferences for optimization *)
  } in
  (grant, grant_effect, content_id)

(*---------------------------------------------------------------------------
 * Core Capability Operations (OCaml Functions)
 *---------------------------------------------------------------------------*)

(** Check if a grantee has a specific capability in a domain *)
let has_capability ~capability_id ~grantee ~domain () =
  let state_key = capability_state_smt_key grantee domain in
  let grant_key = capability_grant_smt_key capability_id domain in
  (* This would interface with actual SMT storage in a real implementation *)
  (state_key, grant_key, "has_capability_result")

(** Grant a capability to a grantee *)
let grant_capability ~capability_id ~grantee ~grantor ~domain ?(conditions=VNil) () =
  let grant_id = _generate_id ["grant"; domain; grantee; capability_id; grantor] in
  let grant_record = {
    grant_id;
    capability_id;
    grantee;
    grantor;
    conditions;
    expires_at = None;
    domain;
  } in
  let grant_value = capability_grant_to_value_expr grant_record in
  let content_id = value_expr_to_id grant_value in
  (grant_record, content_id)

(** Revoke a capability from a grantee *)
let revoke_capability ~capability_id ~grantee ~domain () =
  let revoke_key = _generate_id ["revoke"; domain; grantee; capability_id] in
  (revoke_key, "revoked")

(** Verify capability with zero-knowledge proof *)
let verify_capability_zk ~capability_id ~grantee ~domain ~smt_root ~proof () =
  let grant_key = capability_grant_smt_key capability_id domain in
  let _ = grantee in (* Used in real implementation *)
  (* This would verify the ZK proof in a real implementation *)
  (grant_key, smt_root, proof, "verification_result")

(*---------------------------------------------------------------------------
 * Enhanced Capability System Functions
 *---------------------------------------------------------------------------*)

(** Check if a grantee has a specific capability using SMT backend *)
let has_capability_smt ~capability_id ~grantee ~domain () =
  let state_key = capability_state_smt_key grantee domain in
  let grant_key = capability_grant_smt_key capability_id domain in
  (* In a real implementation, this would query the SMT storage *)
  let query_result = Printf.sprintf "smt_query(%s,%s)" state_key grant_key in
  if query_result <> "nil" then
    "active_grant_found"
  else
    "no_grant_found"

(** Grant a capability using TEG integration *)
let grant_capability_teg ~capability_id ~grantee ~grantor ~domain ~conditions () =
  let grant_resource = create_capability_grant ~capability_id ~grantee ~grantor ~domain ~conditions () in
  (* In a real implementation, this would update the SMT storage *)
  let smt_update = Printf.sprintf "smt_updated_for_%s" capability_id in
  (grant_resource, smt_update)

(*---------------------------------------------------------------------------
 * TEG Integration Functions
 *---------------------------------------------------------------------------*)

(** Create TEG edge for capability requirement *)
let create_capability_requirement_edge ~effect_id ~capability_id ?condition () =
  let edge_id = Bytes.of_string (_generate_id ["req_edge"; effect_id; capability_id]) in
  let source_node = Bytes.of_string effect_id in
  let target_node = Bytes.of_string capability_id in
  let metadata = match condition with
    | Some cond_id -> Some (VRef (VERExpr cond_id))
    | None -> None
  in
  {
    id = edge_id;
    source = source_node;
    target = target_node;
    kind = DependsOn target_node;
    metadata;
  }

(** Create TEG edge for capability grant *)
let create_capability_grant_edge ~grantor_id ~grantee_id ~capability_id ?condition () =
  let edge_id = Bytes.of_string (_generate_id ["grant_edge"; grantor_id; grantee_id; capability_id]) in
  let source_node = Bytes.of_string grantor_id in
  let target_node = Bytes.of_string grantee_id in
  let metadata = match condition with
    | Some cond_id -> Some (VRef (VERExpr cond_id))
    | None -> Some (VStruct (BatMap.of_enum (BatList.enum [
        ("capability_id", VString capability_id);
        ("grant_type", VString "direct");
      ])))
  in
  {
    id = edge_id;
    source = source_node;
    target = target_node;
    kind = ControlFlow;
    metadata;
  }

(** Create TEG edge for capability delegation *)
let create_capability_delegation_edge ~delegator_id ~delegate_id ~capability_id ?condition () =
  let edge_id = Bytes.of_string (_generate_id ["delegation_edge"; delegator_id; delegate_id; capability_id]) in
  let source_node = Bytes.of_string delegator_id in
  let target_node = Bytes.of_string delegate_id in
  let metadata = match condition with
    | Some cond_id -> Some (VRef (VERExpr cond_id))
    | None -> Some (VStruct (BatMap.of_enum (BatList.enum [
        ("capability_id", VString capability_id);
        ("delegation_type", VString "transitive");
      ])))
  in
  {
    id = edge_id;
    source = source_node;
    target = target_node;
    kind = ControlFlow;
    metadata;
  }

(** Generate complete capability system configuration *)
let generate_capability_system_config () =
  let config = VStruct (BatMap.of_enum (BatList.enum [
    ("type", VString "capability_system_config");
    ("smt_backend", VString "enabled");
    ("teg_integration", VBool true);
    ("zk_verification", VBool true);
    ("content_addressing", VBool true);
  ])) in
  config


