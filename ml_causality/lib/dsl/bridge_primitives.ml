(* Purpose: Bridge primitives for cross-domain transfers using OCaml DSL *)

open Ml_causality_lib_types.Types

(*-----------------------------------------------------------------------------
 * Bridge Types
 *-----------------------------------------------------------------------------*)

(** Transfer status enumeration *)
type transfer_status =
  | Initiated
  | InProgress
  | Completed
  | Failed
  | TimedOut

(** Bridge configuration *)
type bridge_config = {
  bridge_id: entity_id;
  name: string;
  source_domain: domain_id;
  target_domain: domain_id;
  source_token: entity_id;
  target_token: entity_id;
  fee_basis_points: int; (* Fee in basis points (1/10000) *)
  min_transfer_amount: int64;
  max_transfer_amount: int64;
  timeout_seconds: int64;
}

(** Bridge transfer metadata *)
type bridge_transfer_metadata = {
  transfer_id: entity_id;
  bridge_id: entity_id;
  source_account: entity_id;
  target_account: entity_id;
  amount: int64;
  fee: int64;
  status: transfer_status;
  initiated_at: int64;
  completed_at: int64 option;
}

(*-----------------------------------------------------------------------------
 * Bridge Creation Functions
 *-----------------------------------------------------------------------------*)

(** Create a bridge resource *)
let create_bridge ~name ~source_domain ~target_domain ~source_token ~target_token 
    ~fee_basis_points ~min_transfer_amount ~max_transfer_amount ~timeout_seconds () =
  let bridge_id = Bytes.of_string (Printf.sprintf "bridge_%s" name) in
  let bridge_config = {
    bridge_id;
    name;
    source_domain;
    target_domain;
    source_token;
    target_token;
    fee_basis_points;
    min_transfer_amount;
    max_transfer_amount;
    timeout_seconds;
  } in
  let bridge_resource = {
    id = bridge_id;
    name = Printf.sprintf "Bridge: %s" name;
    domain_id = source_domain; (* Bridge is anchored to source domain *)
    resource_type = "bridge";
    quantity = 1L;
    timestamp = 0L;
  } in
  (bridge_config, bridge_resource)

(*-----------------------------------------------------------------------------
 * Bridge Utility Functions
 *-----------------------------------------------------------------------------*)

(** Calculate transfer fee *)
let calculate_fee ~amount ~fee_basis_points =
  Int64.div (Int64.mul amount (Int64.of_int fee_basis_points)) 10000L

(** Validate transfer amount *)
let validate_transfer_amount ~amount ~min_amount ~max_amount =
  amount >= min_amount && amount <= max_amount

(** Check if transfer has timed out *)
let is_transfer_timed_out ~initiated_at ~timeout_seconds ~current_time =
  Int64.sub current_time initiated_at > timeout_seconds

(*-----------------------------------------------------------------------------
 * Bridge Operations as Effects
 *-----------------------------------------------------------------------------*)

(** Create initiate transfer effect *)
let create_initiate_transfer_effect ~bridge_config ~source_account ~target_account ~amount ~domain_id () =
  let transfer_id = Bytes.of_string (Printf.sprintf "transfer_%s_%Ld" 
    (Bytes.to_string source_account) amount) in
  let fee = calculate_fee ~amount ~fee_basis_points:bridge_config.fee_basis_points in
  
  let source_domain = VerifiableDomain {
    domain_id = bridge_config.source_domain;
    zk_constraints = true;
    deterministic_only = true;
  } in

  let target_domain = VerifiableDomain {
    domain_id = bridge_config.target_domain;
    zk_constraints = true;
    deterministic_only = true;
  } in

  {
    id = transfer_id;
    name = "InitiateTransfer";
    domain_id;
    effect_type = "bridge_initiate_transfer";
    inputs = [{
      resource_type = "token_balance";
      quantity = Int64.add amount fee;
      domain_id = bridge_config.source_domain;
    }];
    outputs = [{
      resource_type = "bridge_transfer";
      quantity = 1L;
      domain_id;
    }];
    expression = Some (Bytes.of_string "bridge_initiate_transfer_logic");
    timestamp = 0L;
    resources = [];
    nullifiers = [];
    scoped_by = bridge_config.bridge_id;
    intent_id = None;
    source_typed_domain = source_domain;
    target_typed_domain = target_domain;
    originating_dataflow_instance = None;
  }

(** Create complete transfer effect *)
let create_complete_transfer_effect ~bridge_config ~transfer_id ~domain_id () =
  let complete_id = Bytes.of_string (Printf.sprintf "complete_%s" 
    (Bytes.to_string transfer_id)) in

  let target_domain = VerifiableDomain {
    domain_id = bridge_config.target_domain;
    zk_constraints = true;
    deterministic_only = true;
  } in

  {
    id = complete_id;
    name = "CompleteTransfer";
    domain_id;
    effect_type = "bridge_complete_transfer";
    inputs = [{
      resource_type = "bridge_transfer";
      quantity = 1L;
      domain_id;
    }];
    outputs = [{
      resource_type = "token_balance";
      quantity = 1L;
      domain_id = bridge_config.target_domain;
    }];
    expression = Some (Bytes.of_string "bridge_complete_transfer_logic");
    timestamp = 0L;
    resources = [];
    nullifiers = [];
    scoped_by = bridge_config.bridge_id;
    intent_id = None;
    source_typed_domain = target_domain;
    target_typed_domain = target_domain;
    originating_dataflow_instance = None;
  }

(** Create rollback transfer effect *)
let create_rollback_transfer_effect ~bridge_config ~transfer_id ~domain_id () =
  let rollback_id = Bytes.of_string (Printf.sprintf "rollback_%s" 
    (Bytes.to_string transfer_id)) in

  let source_domain = VerifiableDomain {
    domain_id = bridge_config.source_domain;
    zk_constraints = true;
    deterministic_only = true;
  } in

  {
    id = rollback_id;
    name = "RollbackTransfer";
    domain_id;
    effect_type = "bridge_rollback_transfer";
    inputs = [{
      resource_type = "bridge_transfer";
      quantity = 1L;
      domain_id;
    }];
    outputs = [{
      resource_type = "token_balance";
      quantity = 1L;
      domain_id = bridge_config.source_domain;
    }];
    expression = Some (Bytes.of_string "bridge_rollback_transfer_logic");
    timestamp = 0L;
    resources = [];
    nullifiers = [];
    scoped_by = bridge_config.bridge_id;
    intent_id = None;
    source_typed_domain = source_domain;
    target_typed_domain = source_domain;
    originating_dataflow_instance = None;
  } 