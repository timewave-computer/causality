(* Purpose: Bridge primitives for cross-domain operations *)

open Ocaml_causality_core

(** Bridge configuration for cross-domain transfers *)
type bridge_config = {
  bridge_id: Identifiers.entity_id;
  name: string;
  source_domain: Identifiers.domain_id;
  target_domain: Identifiers.domain_id;
  fee_basis_points: int;
  min_transfer_amount: int64;
  max_transfer_amount: int64;
  supported_tokens: string list;
  verification_type: string;
}

(** Bridge state tracking *)
type bridge_state = {
  state_id: Identifiers.entity_id;
  bridge_config_id: Identifiers.entity_id;
  locked_amounts: (string * int64) list; (* token_type, amount *)
  pending_transfers: Identifiers.entity_id list;
  total_volume: int64;
  last_updated: int64;
}

(** Bridge transfer record *)
type bridge_transfer = {
  transfer_id: Identifiers.entity_id;
  bridge_config_id: Identifiers.entity_id;
  source_account: Identifiers.entity_id;
  target_account: Identifiers.entity_id;
  token_type: string;
  amount: int64;
  fee: int64;
  status: string;
  source_tx_hash: string option;
  target_tx_hash: string option;
  created_at: int64;
  completed_at: int64 option;
}

(** Create a bridge configuration *)
let create_bridge_config ~name ~source_domain ~target_domain ~fee_basis_points ~min_transfer_amount ~max_transfer_amount ~supported_tokens ~verification_type () =
  let bridge_id = Printf.sprintf "bridge_%s" name |> Bytes.of_string in
  {
    bridge_id;
    name;
    source_domain;
    target_domain;
    fee_basis_points;
    min_transfer_amount;
    max_transfer_amount;
    supported_tokens;
    verification_type;
  }

(** Create bridge state *)
let create_bridge_state ~bridge_config_id () =
  let state_id = Printf.sprintf "state_%s" (Bytes.to_string bridge_config_id) |> Bytes.of_string in
  {
    state_id;
    bridge_config_id;
    locked_amounts = [];
    pending_transfers = [];
    total_volume = 0L;
    last_updated = Int64.of_float (Unix.time ());
  }

(** Create bridge transfer record *)
let create_bridge_transfer ~bridge_config_id ~source_account ~target_account ~token_type ~amount () =
  let transfer_id = Printf.sprintf "transfer_%s_%s_%Ld" 
    (Bytes.to_string source_account) (Bytes.to_string target_account) amount |> Bytes.of_string in
  {
    transfer_id;
    bridge_config_id;
    source_account;
    target_account;
    token_type;
    amount;
    fee = 0L; (* Will be calculated separately *)
    status = "pending";
    source_tx_hash = None;
    target_tx_hash = None;
    created_at = Int64.of_float (Unix.time ());
    completed_at = None;
  }

(** Validate bridge transfer parameters *)
let validate_bridge_transfer (config: bridge_config) (transfer: bridge_transfer) =
  let amount_valid = transfer.amount >= config.min_transfer_amount && transfer.amount <= config.max_transfer_amount in
  let token_supported = List.mem transfer.token_type config.supported_tokens in
  let valid = amount_valid && token_supported in
  let message = 
    if not amount_valid then "Amount outside valid range"
    else if not token_supported then "Token not supported"
    else "Valid"
  in
  (valid, message)

(** Calculate bridge fee *)
let calculate_bridge_fee (config: bridge_config) (amount: int64) =
  Int64.div (Int64.mul amount (Int64.of_int config.fee_basis_points)) 10000L

(** Create bridge lock effect *)
let create_bridge_lock_effect (transfer: bridge_transfer) (domain_id: Identifiers.domain_id) () =
  let lock_id = Printf.sprintf "lock_%s" (Bytes.to_string transfer.transfer_id) |> Bytes.of_string in
  {
    id = lock_id;
    name = "BridgeLock";
    domain_id;
    resource_type = "bridge_lock";
    quantity = transfer.amount;
    timestamp = transfer.created_at;
  }

(** Create bridge unlock effect *)  
let create_bridge_unlock_effect (transfer: bridge_transfer) (domain_id: Identifiers.domain_id) () =
  let unlock_id = Printf.sprintf "unlock_%s" (Bytes.to_string transfer.transfer_id) |> Bytes.of_string in
  {
    id = unlock_id;
    name = "BridgeUnlock";
    domain_id;
    resource_type = "bridge_unlock";
    quantity = transfer.amount;
    timestamp = transfer.created_at;
  }

(** Create bridge verification proof *)
let create_bridge_verification_proof (transfer: bridge_transfer) =
  let proof_data = Printf.sprintf "proof_%s_%s_%Ld" 
    (Bytes.to_string transfer.source_account)
    (Bytes.to_string transfer.target_account)
    transfer.amount in
  (* Simplified proof - in practice would be a ZK proof *)
  let hash = Digest.string proof_data in
  Digest.to_hex hash 