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
val create_bridge_config :
  name:string ->
  source_domain:Identifiers.domain_id ->
  target_domain:Identifiers.domain_id ->
  fee_basis_points:int ->
  min_transfer_amount:int64 ->
  max_transfer_amount:int64 ->
  supported_tokens:string list ->
  verification_type:string ->
  unit ->
  bridge_config

(** Create bridge state *)
val create_bridge_state :
  bridge_config_id:Identifiers.entity_id ->
  unit ->
  bridge_state

(** Create bridge transfer record *)
val create_bridge_transfer :
  bridge_config_id:Identifiers.entity_id ->
  source_account:Identifiers.entity_id ->
  target_account:Identifiers.entity_id ->
  token_type:string ->
  amount:int64 ->
  unit ->
  bridge_transfer

(** Validate bridge transfer parameters *)
val validate_bridge_transfer :
  bridge_config ->
  bridge_transfer ->
  (bool * string)

(** Calculate bridge fee *)
val calculate_bridge_fee :
  bridge_config ->
  int64 ->
  int64

(** Create bridge lock effect *)
val create_bridge_lock_effect :
  bridge_transfer ->
  Identifiers.domain_id ->
  unit ->
  resource

(** Create bridge unlock effect *)  
val create_bridge_unlock_effect :
  bridge_transfer ->
  Identifiers.domain_id ->
  unit ->
  resource

(** Create bridge verification proof *)
val create_bridge_verification_proof :
  bridge_transfer ->
  string 