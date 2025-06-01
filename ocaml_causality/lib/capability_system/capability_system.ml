(* Purpose: Implementation of the Capability System module. *)

open Ocaml_causality_core
open Ocaml_causality_core.Types
open Ocaml_causality_serialization.Content_addressing (* For content_id generation, etc. *)
open Batteries (* For BatMap *)
(* May need Smt module from serialization as well later *)

(* Type definitions are in capability_system.mli and are thus available here. *)

(* In-memory storage (placeholders, to be replaced or augmented with SMT) *)
let capability_definitions : (entity_id, capability_def) Hashtbl.t = Hashtbl.create 16
let capability_grants : (entity_id, capability_grant) Hashtbl.t = Hashtbl.create 32
(* let capability_states : (entity_id, capability_state) Hashtbl.t = Hashtbl.create 16 (* If state is managed directly here *) *)

(* Placeholder for TEG edges if managed globally here *)
(* let teg_edges : (entity_id, Capability_system_t.teg_edge) Hashtbl.t = Hashtbl.create 32 *)

(*** Content Addressing Utilities (Implementation) ***)

(* Note: Actual SSZ encoding for these types would be needed for true content addressing. 
   For now, these might convert to a specific value_expr representation that is then hashed. 
   The .mli implies they convert to value_expr, so we'll follow that. *)

let capability_def_to_value_expr (def: capability_def) : value_expr =
  (* This requires a defined way to represent the def as a value_expr. 
     Using a VStruct for now. The exact fields should match what's needed for hashing/storage. *)
  VStruct (BatMap.String.of_list [
    ("capability_id", VString (Bytes.to_string def.capability_id)); (* Assuming entity_id is bytes *)
    ("name", VString def.name);
    ("description", VString def.description);
    ("required_permissions", VList (List.map (fun p -> VString p) def.required_permissions));
    ("domain_id", VString (Bytes.to_string def.domain_id)); (* Assuming domain_id is bytes *)
    ("metadata", def.metadata);
  ])

let capability_grant_to_value_expr (grant: capability_grant) : value_expr =
  VStruct (BatMap.String.of_list [
    ("grant_id", VString (Bytes.to_string grant.grant_id));
    ("capability_id", VString (Bytes.to_string grant.capability_id));
    ("grantee_id", VString (Bytes.to_string grant.grantee_id));
    ("grantor_id", VString (Bytes.to_string grant.grantor_id));
    ("conditions", grant.conditions);
    ("expires_at", match grant.expires_at with Some ts -> VInt ts | None -> VNil);
    ("domain_id", VString (Bytes.to_string grant.domain_id));
  ])

let capability_state_to_value_expr (state: capability_state) : value_expr =
  VStruct (BatMap.String.of_list [
    ("state_id", VString (Bytes.to_string state.state_id));
    ("active_grants", VList (List.map (fun id -> VString (Bytes.to_string id)) state.active_grants));
    ("revoked_grants", VList (List.map (fun id -> VString (Bytes.to_string id)) state.revoked_grants));
    ("delegation_chains", state.delegation_chains);
    ("domain_id", VString (Bytes.to_string state.domain_id));
  ])

(*** SMT-Integrated Operations (Implementation) ***)

let capability_def_smt_key ~(capability_id:entity_id) ~(domain_id:domain_id) : string =
  Printf.sprintf "capability_def:%s:%s" (Bytes.to_string capability_id) (Bytes.to_string domain_id)

let capability_grant_smt_key ~(grant_id:entity_id) ~(domain_id:domain_id) : string =
  Printf.sprintf "capability_grant:%s:%s" (Bytes.to_string grant_id) (Bytes.to_string domain_id)

let capability_state_smt_key ~(state_id:entity_id) ~(domain_id:domain_id) : string =
  Printf.sprintf "capability_state:%s:%s" (Bytes.to_string state_id) (Bytes.to_string domain_id)


let create_capability_definition 
  ~name 
  ~description 
  ~required_permissions 
  ~domain_id 
  ?metadata 
  () =
  let meta = match metadata with Some m -> m | None -> VNil in
  let cap_id_str = Printf.sprintf "%s:%s" name (Bytes.to_string domain_id) in (* Simplistic ID generation for now *)
  let cap_id = hash_bytes (Bytes.of_string cap_id_str) (* Use actual content addressing based on relevant fields *) in 
  let def = {
    capability_id = cap_id;
    name;
    description;
    required_permissions;
    domain_id;
    metadata = meta;
  } in
  Hashtbl.replace capability_definitions cap_id def;
  
  (* Create a resource representation of this definition *)
  (* The exact structure of this resource needs to be defined. 
     For now, let's assume its content_id is the capability_id itself or derived from it. *)
  let res_content_id = def.capability_id in 
  let res : resource = {
    resource_id = res_content_id; 
    resource_name = Printf.sprintf "capability_definition_%s" name;
    resource_domain_id = domain_id;
    resource_type = "Causality/CapabilityDefinition";
    resource_quantity = 1L;
    resource_timestamp = Int64.of_float (Unix.time ()); (* Placeholder timestamp *)
    resource_metadata = Some (capability_def_to_value_expr def); (* Embed full def as metadata *)
    resource_content = None; (* Or some serialized form of key aspects *)
  } in
  (def, res, res_content_id)


let create_capability_grant 
  ~capability_id 
  ~grantee_id 
  ~grantor_id 
  ~domain_id 
  ?conditions 
  ?expires_at 
  () =
  let cond = match conditions with Some c -> c | None -> VNil in
  let grant_id_str = Printf.sprintf "%s:%s:%s" (Bytes.to_string capability_id) (Bytes.to_string grantee_id) (Bytes.to_string grantor_id) in
  let grant_id = hash_bytes (Bytes.of_string grant_id_str) in 
  let grant : capability_grant = {
    grant_id;
    capability_id;
    grantee_id;
    grantor_id;
    conditions = cond;
    expires_at;
    domain_id;
  } in 
  Hashtbl.replace capability_grants grant_id grant;

  (* Create an effect representation of this grant operation *)
  let effect_id_str = Printf.sprintf "grant_effect:%s" (Bytes.to_string grant_id) in
  let effect_id = hash_bytes (Bytes.of_string effect_id_str) in 
  let effect_payload = capability_grant_to_value_expr grant in (* Or a more specific payload *)
  let eff : effect = {
    effect_id;
    effect_name = "System/GrantCapability";
    effect_domain_id = domain_id;
    effect_type_tag = "capability.grant"; (* Example tag *)
    effect_payload;
    effect_status = Pending; (* Assuming status type from Core.Types *) 
    effect_dependencies = [];
    effect_outputs = []; (* Or VRef to the grant_id *)
    effect_timestamp = Int64.of_float (Unix.time ());
    effect_metadata = None;
    effect_context = [];
    effect_expressions = [];
    effect_intent_id = VNil; (* Placeholder - needs proper intent link if used *) 
  } in
  (grant, eff, grant_id)

(* TODO: Implement TEG Integration functions here *)
let create_capability_requirement_edge
  ~effect_id
  ~capability_id
  ?condition_expr_id
  () =
  (* Placeholder: In a real system, this would interact with a TEG component. *)
  let edge_id_str = Printf.sprintf "req_edge:%s:%s" (Bytes.to_string effect_id) (Bytes.to_string capability_id) in
  let edge_id = hash_bytes (Bytes.of_string edge_id_str) in (* Assuming hash_bytes is available *)
  {
    source_node_id = effect_id;
    target_node_id = capability_id;
    edge_type = "requires_capability";
    condition_expr_id;
    metadata = VNil; (* Placeholder metadata *)
  }

let create_capability_grant_edge
  ~grantor_id
  ~grantee_id
  ~capability_id
  ?condition_expr_id
  () =
  let edge_id_str = Printf.sprintf "grant_edge:%s:%s:%s" (Bytes.to_string grantor_id) (Bytes.to_string grantee_id) (Bytes.to_string capability_id) in
  let edge_id = hash_bytes (Bytes.of_string edge_id_str) in
   {
    source_node_id = grantor_id; (* Or a node representing the grant action *)
    target_node_id = grantee_id; (* Or a node representing the grantee possessing the capability *)
    edge_type = "grants_capability";
    condition_expr_id;
    metadata = VStruct (BatMap.String.of_list [("capability_id", VString (Bytes.to_string capability_id))]);
  }

let create_capability_delegation_edge
  ~delegator_id
  ~delegatee_id
  ~capability_id
  ?condition_expr_id
  () =
  let edge_id_str = Printf.sprintf "deleg_edge:%s:%s:%s" (Bytes.to_string delegator_id) (Bytes.to_string delegatee_id) (Bytes.to_string capability_id) in
  let edge_id = hash_bytes (Bytes.of_string edge_id_str) in
  {
    source_node_id = delegator_id;
    target_node_id = delegatee_id;
    edge_type = "delegates_capability";
    condition_expr_id;
    metadata = VStruct (BatMap.String.of_list [("capability_id", VString (Bytes.to_string capability_id))]);
  }

(* TODO: Implement Core Capability Operations (has_capability, grant_capability, revoke_capability, etc.) here *)
let has_capability
  ~capability_id
  ~grantee_id
  ~domain_id
  () =
  (* Placeholder: Check in-memory store. SMT version would query SMT. *)
  Hashtbl.fold (fun _grant_id grant acc ->
    acc || (
      grant.capability_id = capability_id &&
      grant.grantee_id = grantee_id &&
      grant.domain_id = domain_id &&
      (match grant.expires_at with None -> true | Some ts -> Int64.to_float ts > Unix.time())
      (* TODO: Add condition checking if grant.conditions is not VNil *)
    )
  ) capability_grants false

let grant_capability
  ~capability_id
  ~grantee_id
  ~grantor_id
  ~domain_id
  ?conditions
  ?expires_at
  () =
  (* This largely duplicates create_capability_grant but returns (grant, grant_id) *)
  (* In a real system, create_capability_grant might be internal, and this would be the public API *)
  let cond = match conditions with Some c -> c | None -> VNil in
  let grant_id_str = Printf.sprintf "%s:%s:%s" (Bytes.to_string capability_id) (Bytes.to_string grantee_id) (Bytes.to_string grantor_id) in
  let grant_id = hash_bytes (Bytes.of_string grant_id_str) in
  let grant : capability_grant = {
    grant_id;
    capability_id;
    grantee_id;
    grantor_id;
    conditions = cond;
    expires_at;
    domain_id;
  } in
  Hashtbl.replace capability_grants grant_id grant;
  (grant, grant_id)

let revoke_capability
  ~grant_id
  ~revoker_id (* TODO: Use revoker_id to check permissions for revocation *)
  ~domain_id (* TODO: Use domain_id to ensure correct scope *)
  () =
  (* Placeholder: Simple removal. A real system would handle SMT updates, event logging, etc. *)
  if Hashtbl.mem capability_grants grant_id then
    let grant = Hashtbl.find capability_grants grant_id in
    if grant.domain_id = domain_id then
        (* Basic permission check: only grantor or a superuser (not implemented) can revoke *)
        if grant.grantor_id = revoker_id then
            Hashtbl.remove capability_grants grant_id
        else
            failwith (Printf.sprintf "Revoker %s does not have permission to revoke grant %s" (Bytes.to_string revoker_id) (Bytes.to_string grant_id))
    else
        failwith (Printf.sprintf "Grant %s not found in domain %s" (Bytes.to_string grant_id) (Bytes.to_string domain_id))
  else
    failwith (Printf.sprintf "Grant %s not found" (Bytes.to_string grant_id))
  (* TODO: Add to revoked_grants list in capability_state if that's being used *)

let verify_capability_zk
  ~capability_id
  ~grantee_id
  ~domain_id
  ~smt_root
  ~proof
  () =
  (* Placeholder: ZK verification is complex and external. *)
  Printf.printf "Verifying capability %s for %s in %s against SMT root %s with proof (len %d)
"
    (Bytes.to_string capability_id) (Bytes.to_string grantee_id) (Bytes.to_string domain_id) smt_root (Bytes.length proof);
  failwith "ZK verification not yet implemented"

(* TODO: Implement Enhanced System Functions here *)
let has_capability_smt
  ~capability_id
  ~grantee_id
  ~domain_id
  ~smt_root
  () =
  (* Placeholder: This would involve querying an SMT with the given root. *)
  Printf.printf "Checking SMT for capability %s for %s in %s with SMT root %s
"
    (Bytes.to_string capability_id) (Bytes.to_string grantee_id) (Bytes.to_string domain_id) smt_root;
  failwith "SMT capability check not yet implemented"

let grant_capability_teg
  ~capability_id
  ~grantee_id
  ~grantor_id
  ~domain_id
  ?conditions
  ?expires_at
  () =
  let (grant, grant_id) = grant_capability ~capability_id ~grantee_id ~grantor_id ~domain_id ?conditions ?expires_at () in
  let teg_edge = create_capability_grant_edge ~grantor_id ~grantee_id ~capability_id ?condition_expr_id:(None) () in
  (* In a real system, condition_expr_id for TEG edge might be derived from grant.conditions *)
  (grant, grant_id, teg_edge)

(* TODO: Implement generate_capability_system_config here *)
let generate_capability_system_config () =
  (* Placeholder: Serialize current in-memory state to a value_expr.
     A real system might read from a canonical configuration source or SMT. *)
  let defs_list = Hashtbl.fold (fun _ def acc -> (capability_def_to_value_expr def) :: acc) capability_definitions [] in
  let grants_list = Hashtbl.fold (fun _ grant acc -> (capability_grant_to_value_expr grant) :: acc) capability_grants [] in
  VStruct (BatMap.String.of_list [
    ("capability_definitions", VList defs_list);
    ("capability_grants", VList grants_list);
    (* Add other system state components as needed, e.g., TEG state, SMT roots *)
  ]) 