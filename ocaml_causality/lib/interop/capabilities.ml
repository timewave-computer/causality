(* ------------ CAPABILITY SYSTEM ------------ *)
(* Purpose: Capability system and authorization *)

open Ocaml_causality_core

(* ------------ CAPABILITY DEFINITIONS ------------ *)

(* Capability types and definitions *)
type capability_type =
  | ReadCapability
  | WriteCapability
  | ExecuteCapability
  | AdminCapability
  | TransferCapability

type capability = {
    id : entity_id
  ; capability_type : capability_type
  ; resource_pattern : string
  ; domain_id : domain_id
  ; owner : string
  ; expires_at : timestamp option
  ; created_at : timestamp
}

type authorization_context = {
    user_id : string
  ; domain_id : domain_id
  ; requested_operation : string
  ; target_resource : string
}

(* ------------ AUTHORIZATION ------------ *)

(* Authorization and permission checking functions *)
let check_capability capability context =
  let type_matches =
    match (capability.capability_type, context.requested_operation) with
    | ReadCapability, "read" -> true
    | WriteCapability, "write" -> true
    | ExecuteCapability, "execute" -> true
    | AdminCapability, _ -> true
    | TransferCapability, "transfer" -> true
    | _, _ -> false
  in

  let domain_matches = Bytes.equal capability.domain_id context.domain_id in

  let resource_matches =
    let pattern = capability.resource_pattern in
    if pattern = "*" then true else String.equal pattern context.target_resource
  in

  let owner_matches = String.equal capability.owner context.user_id in

  let not_expired =
    match capability.expires_at with
    | None -> true
    | Some expiry -> expiry > 1640995200L (* Current time mock *)
  in

  type_matches && domain_matches && resource_matches && owner_matches
  && not_expired

let authorize_operation context capabilities =
  List.exists (fun cap -> check_capability cap context) capabilities

let get_user_capabilities user_id domain_id capabilities =
  List.filter
    (fun cap ->
      String.equal cap.owner user_id && Bytes.equal cap.domain_id domain_id)
    capabilities

(* ------------ CAPABILITY MANAGEMENT ------------ *)

(* Capability creation and revocation functions *)
let create_capability capability_type resource_pattern domain_id owner
    expires_at =
  let cap_id = Bytes.create 32 in
  for i = 0 to 31 do
    Bytes.set_uint8 cap_id i (Random.int 256)
  done;
  {
    id = cap_id
  ; capability_type
  ; resource_pattern
  ; domain_id
  ; owner
  ; expires_at
  ; created_at = 1640995200L (* Fixed timestamp *)
  }

let revoke_capability capability_id capabilities =
  List.filter (fun cap -> not (Bytes.equal cap.id capability_id)) capabilities

let extend_capability_expiry capability new_expiry =
  { capability with expires_at = Some new_expiry }

let transfer_capability capability new_owner =
  { capability with owner = new_owner }

(* ------------ UTILITIES ------------ *)

(* Capability utilities and validation functions *)
let validate_capability capability =
  let validations =
    [
      (String.length capability.owner > 0, "Owner must not be empty")
    ; ( String.length capability.resource_pattern > 0
      , "Resource pattern must not be empty" )
    ; (capability.created_at > 0L, "Created timestamp must be positive")
    ]
  in

  let rec check_validations = function
    | [] -> Ok ()
    | (true, _) :: rest -> check_validations rest
    | (false, msg) :: _ -> Error (FFIError msg)
  in
  check_validations validations

let capability_to_string capability =
  let type_str =
    match capability.capability_type with
    | ReadCapability -> "read"
    | WriteCapability -> "write"
    | ExecuteCapability -> "execute"
    | AdminCapability -> "admin"
    | TransferCapability -> "transfer"
  in
  Printf.sprintf "Capability(%s, %s, %s, %s)" type_str
    capability.resource_pattern capability.owner
    (Bytes.to_string capability.domain_id)

let is_capability_expired capability =
  match capability.expires_at with
  | None -> false
  | Some expiry -> expiry <= 1640995200L (* Current time mock *)

let get_capability_permissions capability =
  match capability.capability_type with
  | ReadCapability -> [ "read" ]
  | WriteCapability -> [ "write" ]
  | ExecuteCapability -> [ "execute" ]
  | AdminCapability -> [ "read"; "write"; "execute"; "admin" ]
  | TransferCapability -> [ "transfer" ]

(* Capability registry *)
module CapabilityRegistry = struct
  type t = {
      mutable capabilities : capability list
    ; mutable revoked_capabilities : entity_id list
  }

  let create () = { capabilities = []; revoked_capabilities = [] }

  let register_capability registry capability =
    match validate_capability capability with
    | Ok () ->
        registry.capabilities <- capability :: registry.capabilities;
        Ok capability.id
    | Error e -> Error e

  let revoke_capability registry capability_id =
    registry.capabilities <-
      List.filter
        (fun cap -> not (Bytes.equal cap.id capability_id))
        registry.capabilities;
    registry.revoked_capabilities <-
      capability_id :: registry.revoked_capabilities

  let lookup_capability registry capability_id =
    List.find_opt
      (fun cap -> Bytes.equal cap.id capability_id)
      registry.capabilities

  let list_user_capabilities registry user_id domain_id =
    get_user_capabilities user_id domain_id registry.capabilities

  let check_authorization registry context =
    authorize_operation context registry.capabilities
end

(* Default capability registry *)
let default_capability_registry = CapabilityRegistry.create ()
